# GitHub Actions CI/CD Pipeline

This repository includes automated CI/CD pipelines using GitHub Actions.

## Workflows

### 1. Main CI/CD Pipeline (`ci.yml`)
**Triggers**: Push to `main`/`master` branches, pull requests to these branches

**Jobs**:
- **Test Suite**: Runs on Ubuntu, Windows, and macOS
  - Code formatting check (`cargo fmt`)
  - Linting with Clippy (`cargo clippy`)
  - Unit tests (`cargo test`)
  - Debug and release builds
  
- **Security Audit**: Runs `cargo audit` to check for vulnerabilities

- **Build Artifacts**: Creates release binaries for multiple platforms
  - Linux (x86_64)
  - Windows (x86_64)
  - macOS (x86_64 and ARM64)

- **Release Creation**: Automatically creates GitHub releases with binaries
  - Uses version from `Cargo.toml`
  - Includes detailed release notes
  - Attaches platform-specific binaries

### 2. Development Workflow (`dev.yml`)
**Triggers**: Push to development branches, pull request updates

**Jobs**:
- **Quick Checks**: Fast feedback for development
  - Formatting and linting checks
  - Unit tests on Ubuntu
  
- **Cross-Platform Test**: Ensures compatibility across platforms

## Release Process

When you push to the `main` or `master` branch:

1. ✅ **All tests must pass** on all platforms (Linux, Windows, macOS)
2. ✅ **Security audit** must pass (no known vulnerabilities)
3. ✅ **Code quality** checks must pass (formatting, linting)
4. 🏗️ **Release binaries** are built for all supported platforms
5. 📦 **GitHub release** is created automatically with version from `Cargo.toml`
6. ⬆️ **Binaries are uploaded** as release assets

## Supported Platforms

The CI pipeline builds and tests on:
- **Linux**: Ubuntu Latest (x86_64-unknown-linux-gnu)
- **Windows**: Windows Latest (x86_64-pc-windows-msvc) 
- **macOS Intel**: macOS Latest (x86_64-apple-darwin)
- **macOS ARM**: macOS Latest (aarch64-apple-darwin)

## Artifacts

Each successful pipeline run produces:
- `clipboard-history-linux-x64` - Linux executable
- `clipboard-history-windows-x64.exe` - Windows executable  
- `clipboard-history-macos-x64` - macOS Intel executable
- `clipboard-history-macos-arm64` - macOS Apple Silicon executable

## Local Development

Before pushing, ensure your code passes local checks:

```bash
# Format code
cargo fmt

# Check linting
cargo clippy -- -D warnings  

# Run tests
cargo test

# Build release
cargo build --release
```

## Security

- **Dependency Scanning**: `cargo audit` runs on every push
- **Vulnerability Database**: Automatically updated
- **Security Alerts**: Pipeline fails if vulnerabilities are found

This ensures the clipboard manager maintains high security and code quality standards across all platforms.
