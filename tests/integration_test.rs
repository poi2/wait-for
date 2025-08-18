use std::net::TcpListener;
use std::process::Command;
use std::thread;

#[test]
fn test_tcp_success() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let _stream = listener.accept();
    });

    let output = Command::new("cargo")
        .args(["run", "--", &format!("127.0.0.1:{port}")])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_tcp_timeout() {
    let output = Command::new("cargo")
        .args(["run", "--", "-t", "1", "127.0.0.1:65432"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Timeout"));
}

#[test]
fn test_command_execution() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let _stream = listener.accept();
    });

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            &format!("127.0.0.1:{port}"),
            "--",
            "echo",
            "test_message",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test_message"));
}

#[test]
fn test_command_exit_code_propagation() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let _stream = listener.accept();
    });

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            &format!("127.0.0.1:{port}"),
            "--",
            "sh",
            "-c",
            "exit 42",
        ])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code().unwrap_or(0), 42);
}

#[test]
fn test_quiet_mode() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let _stream = listener.accept();
    });

    let output = Command::new("cargo")
        .args(["run", "--", "-q", &format!("127.0.0.1:{port}")])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // In quiet mode, there should be minimal output
    assert!(!stdout.contains("Waiting for"));
    assert!(!stderr.contains("Connection to"));
}

#[test]
fn test_invalid_target_format() {
    let output = Command::new("cargo")
        .args(["run", "--", "invalid-target"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Target must be in format"));
}

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wait-for"));
}

#[test]
fn test_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A simple CLI to wait for a service"));
    assert!(stdout.contains("--timeout"));
    assert!(stdout.contains("--quiet"));
}
