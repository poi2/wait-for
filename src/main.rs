use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::Command;
use std::time::{Duration, Instant};
use std::{process, thread};
use termcolor::{Color, ColorChoice as TermColorChoice, ColorSpec, StandardStream, WriteColor};
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Service to wait for (host:port or URL)
    target: String,

    /// Timeout in seconds (0 for no timeout)
    #[arg(short, long, default_value = "15")]
    timeout: u64,

    /// Quiet mode - suppress output
    #[arg(short, long)]
    quiet: bool,

    /// Colored output control
    #[arg(long, value_enum, default_value = "never")]
    color: ColorChoice,

    /// Command to execute after successful wait
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

struct ColorOutput {
    stdout: StandardStream,
    stderr: StandardStream,
    use_color: bool,
}

impl ColorOutput {
    fn new(choice: &ColorChoice) -> Self {
        let use_color = should_use_color(choice);
        let color_choice = if use_color {
            TermColorChoice::Auto
        } else {
            TermColorChoice::Never
        };

        Self {
            stdout: StandardStream::stdout(color_choice),
            stderr: StandardStream::stderr(color_choice),
            use_color,
        }
    }

    fn print_info(&mut self, msg: &str) {
        if self.use_color {
            self.stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Blue)))
                .ok();
        }
        writeln!(&mut self.stdout, "{msg}").ok();
        if self.use_color {
            self.stdout.reset().ok();
        }
    }

    fn print_success(&mut self, msg: &str) {
        if self.use_color {
            self.stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                .ok();
        }
        writeln!(&mut self.stdout, "{msg}").ok();
        if self.use_color {
            self.stdout.reset().ok();
        }
    }

    fn print_error(&mut self, msg: &str) {
        if self.use_color {
            self.stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                .ok();
        }
        writeln!(&mut self.stderr, "{msg}").ok();
        if self.use_color {
            self.stderr.reset().ok();
        }
    }

    fn print_warning(&mut self, msg: &str) {
        if self.use_color {
            self.stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                .ok();
        }
        writeln!(&mut self.stderr, "{msg}").ok();
        if self.use_color {
            self.stderr.reset().ok();
        }
    }
}

fn should_use_color(choice: &ColorChoice) -> bool {
    match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => {
            if std::env::var("NO_COLOR").is_ok() {
                return false;
            }
            if std::env::var("FORCE_COLOR").is_ok() {
                return true;
            }
            is_terminal::IsTerminal::is_terminal(&std::io::stdout())
        }
    }
}

#[derive(Debug, Clone)]
enum Target {
    HostPort(String, u16),
    Url(String),
}

fn parse_target(target: &str) -> Result<Target> {
    if target.starts_with("http://") || target.starts_with("https://") {
        Url::parse(target).with_context(|| format!("Invalid URL format: {target}"))?;
        Ok(Target::Url(target.to_string()))
    } else if let Some(colon_pos) = target.rfind(':') {
        let host = &target[..colon_pos];
        let port_str = &target[colon_pos + 1..];
        let port = port_str
            .parse::<u16>()
            .with_context(|| format!("Invalid port number: {port_str}"))?;

        if host.is_empty() {
            bail!("Host cannot be empty");
        }

        Ok(Target::HostPort(host.to_string(), port))
    } else {
        bail!("Target must be in format 'host:port' or 'http(s)://...'")
    }
}

fn check_tcp(host: &str, port: u16, quiet: bool, output: &mut ColorOutput) -> Result<()> {
    let addr = format!("{host}:{port}");
    let socket_addrs: Vec<SocketAddr> = addr
        .to_socket_addrs()
        .with_context(|| format!("Failed to resolve address: {addr}"))?
        .collect();

    if socket_addrs.is_empty() {
        bail!("No addresses found for {addr}");
    }

    let connect_timeout = Duration::from_secs(1);

    for socket_addr in socket_addrs {
        match TcpStream::connect_timeout(&socket_addr, connect_timeout) {
            Ok(_) => {
                if !quiet {
                    output.print_success(&format!("Connection to {host}:{port} succeeded"));
                }
                return Ok(());
            }
            Err(e) => {
                if !quiet {
                    output.print_error(&format!("Connection to {socket_addr} failed: {e}"));
                }
            }
        }
    }

    bail!("Failed to connect to {host}:{port}")
}

fn check_http(url: &str, quiet: bool, output: &mut ColorOutput) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to send HTTP request to {url}"))?;

    if response.status().is_success() {
        if !quiet {
            output.print_success(&format!(
                "HTTP request to {} succeeded (status: {})",
                url,
                response.status()
            ));
        }
        Ok(())
    } else {
        bail!(
            "HTTP request to {url} failed with status: {}",
            response.status()
        )
    }
}

fn execute_command(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Ok(());
    }

    let program = &command[0];
    let args = &command[1..];

    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute command: {program}"))?;

    process::exit(status.code().unwrap_or(1));
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut output = ColorOutput::new(&cli.color);

    let target_str = cli.target.clone();
    let target = parse_target(&cli.target)
        .with_context(|| format!("Failed to parse target: {target_str}"))?;

    let timeout_duration = if cli.timeout == 0 {
        None
    } else {
        Some(Duration::from_secs(cli.timeout))
    };

    let start_time = Instant::now();

    if !cli.quiet {
        match &target {
            Target::HostPort(host, port) => {
                output.print_info(&format!("Waiting for {host}:{port} to become available..."));
            }
            Target::Url(url) => {
                output.print_info(&format!("Waiting for {url} to become available..."));
            }
        }
    }

    loop {
        if let Some(timeout) = timeout_duration {
            if start_time.elapsed() >= timeout {
                let timeout_secs = cli.timeout;
                bail!("Timeout waiting for service after {timeout_secs} seconds");
            }
        }

        let check_result = match &target {
            Target::HostPort(host, port) => check_tcp(host, *port, cli.quiet, &mut output),
            Target::Url(url) => check_http(url, cli.quiet, &mut output),
        };

        match check_result {
            Ok(()) => {
                if !cli.quiet {
                    output.print_success("Service is available!");
                }
                break;
            }
            Err(e) => {
                if !cli.quiet {
                    output.print_error(&format!("Check failed: {e}"));
                    output.print_warning("Retrying in 1 second...");
                }
            }
        }

        thread::sleep(Duration::from_secs(1));
    }

    execute_command(&cli.command)?;

    Ok(())
}
