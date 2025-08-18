# wait-for

[![CI](https://github.com/poi2/wait-for/workflows/CI/badge.svg)](https://github.com/poi2/wait-for/actions)
[![Crates.io](https://img.shields.io/crates/v/wait-for.svg)](https://crates.io/crates/wait-for)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](https://github.com/poi2/wait-for)

A simple CLI to wait for a service to be available before executing a command.

`wait-for` is a Rust rewrite of the popular [eficode/wait-for](https://github.com/eficode/wait-for) shell script. It provides a reliable way to wait for network services to become available, which is especially useful in Docker Compose setups and CI/CD pipelines.

## Features

- **Self-contained**: No external dependencies like `netcat` or `wget` required
- **Cross-platform**: Single binary for Linux, macOS, and Windows
- **Fast**: Written in Rust for optimal performance
- **Docker-friendly**: Perfect for container orchestration scenarios
- **CI/CD ready**: Seamlessly integrates with any build pipeline

## Installation

### Using Cargo

```bash
cargo install wait-for
```

### Download Binary

Download pre-compiled binaries from the [GitHub Releases](https://github.com/poi2/wait-for/releases) page.

### Docker Usage

```dockerfile
# Copy the binary into your Docker image
COPY wait-for /usr/local/bin/wait-for
RUN chmod +x /usr/local/bin/wait-for

# Use it in your entrypoint
ENTRYPOINT ["wait-for", "db:5432", "--", "./start-app.sh"]
```

## Usage

### Basic Examples

Wait for a TCP service:
```bash
wait-for db:5432
```

Wait for an HTTP service:
```bash
wait-for https://api.example.com/health
```

Execute a command after service is ready:
```bash
wait-for redis:6379 -- ./run-tests.sh
```

With custom timeout:
```bash
wait-for -t 30 postgres:5432 -- python manage.py migrate
```

Silent mode:
```bash
wait-for -q api:8080 -- curl http://api:8080/status
```

### Docker Compose Example

```yaml
version: '3.8'
services:
  db:
    image: postgres:13
    environment:
      POSTGRES_PASSWORD: password

  web:
    image: myapp:latest
    depends_on:
      - db
    command: ["wait-for", "db:5432", "--", "python", "manage.py", "runserver"]
```

### CI/CD Pipeline Examples

#### GitHub Actions
```yaml
- name: Wait for test database
  run: wait-for localhost:5432 -- npm run test
```

#### GitLab CI
```yaml
test:
  script:
    - wait-for redis:6379 -- pytest tests/
```

## Command Line Reference

```
A simple CLI to wait for a service to be available before executing a command

Usage: wait-for [OPTIONS] <TARGET> [COMMAND]...

Arguments:
  <TARGET>      Service to wait for (host:port or URL)
  [COMMAND]...  Command to execute after successful wait

Options:
  -t, --timeout <TIMEOUT>  Timeout in seconds (0 for no timeout) [default: 15]
  -q, --quiet              Quiet mode - suppress output
  -h, --help               Print help
  -V, --version            Print version
```

### Target Formats

- **TCP**: `host:port` (e.g., `localhost:3306`, `redis:6379`)
- **HTTP/HTTPS**: Full URL (e.g., `https://api.example.com/health`, `http://localhost:8080/ready`)

### Exit Codes

- `0`: Service is available (and command executed successfully if provided)
- `1`: Timeout reached or connection failed
- Other: Exit code of the executed command (if provided)

## Examples by Use Case

### Database Initialization
```bash
# Wait for PostgreSQL and run migrations
wait-for postgres:5432 -- python manage.py migrate

# Wait for MySQL and seed data
wait-for mysql:3306 -- npm run db:seed
```

### Microservices
```bash
# Wait for API service before running tests
wait-for api:8080 -- npm run integration-tests

# Chain multiple service dependencies
wait-for redis:6379 && wait-for postgres:5432 -- ./start-worker.sh
```

### Health Checks
```bash
# Wait for HTTP health endpoint
wait-for https://api.example.com/health -- curl -f https://api.example.com/data

# Custom timeout for slow services
wait-for -t 60 elasticsearch:9200 -- python index_data.py
```

## Building from Source

```bash
git clone https://github.com/poi2/wait-for.git
cd wait-for
cargo build --release
```

The binary will be available at `target/release/wait-for`.

## Comparison with Original wait-for

| Feature | wait-for (shell) | wait-for (Rust) |
|---------|------------------|---------------------|
| Dependencies | netcat/wget required | None (self-contained) |
| Platform support | Unix-like only | Linux/macOS/Windows |
| Binary size | N/A (script) | ~2MB static binary |
| Performance | Depends on external tools | Native compiled speed |
| Error messages | Basic | Detailed with context |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

- Inspired by [eficode/wait-for](https://github.com/eficode/wait-for)
- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- HTTP client powered by [reqwest](https://github.com/seanmonstar/reqwest)