# 🚀 GitHub Actions CI/CD Pipeline

This repository includes comprehensive automated CI/CD pipelines using GitHub Actions with advanced build versioning and release management.

## 📋 Workflows

### 1. Main CI/CD Pipeline (`ci.yml`)
**🔧 Triggers**: Push to `master` branch, pull requests to `master`

**📊 Jobs**:
- **🧪 Test Suite**: Runs on Ubuntu, Windows, and macOS
  - Code formatting check (`cargo fmt --check`)
  - Static analysis with Clippy (`cargo clippy -- -D warnings`)  
  - Unit tests (`cargo test --verbose`)
  - Build verification (debug and release)
  - Dependency caching for faster builds
  
- **🛡️ Security Audit**: Comprehensive security scanning
  - `cargo audit` for vulnerability detection
  - Dependency security analysis
  - Blocks releases if vulnerabilities found

- **🏗️ Build Artifacts**: Creates optimized release binaries
  - **Linux**: `x86_64-unknown-linux-gnu`
  - **Windows**: `x86_64-pc-windows-msvc` 
  - **macOS Intel**: `x86_64-apple-darwin`
  - **macOS Apple Silicon**: `aarch64-apple-darwin`

- **📦 Release Creation**: Automated GitHub releases with enhanced versioning
  - **Smart Versioning**: `v{version}-build.{build_number}.{commit_sha}`
  - **Example**: `v0.1.0-build.42.abc1234`
  - **Changelog Generation**: Automatic commit-based changelog
  - **Multiple Download Options**: Standard and versioned binaries
  - **Rich Release Notes**: Detailed feature descriptions and installation guides

### 2. Development Workflow (`dev.yml`) 
**🔧 Triggers**: Push to development branches (`develop`, `feature/*`, `fix/*`), pull requests

**📊 Jobs**:
- **⚡ Quick Checks**: Fast feedback for development
  - Formatting and linting validation
  - Build verification (`cargo check`)
  - Unit tests on Ubuntu (fastest runner)
  - Dependency caching for speed
  
- **🌐 Cross-Platform Test**: Compatibility validation
  - Full build and test on Ubuntu, Windows, macOS
  - Only runs for pull requests (resource optimization)
  - Parallel execution with fail-fast disabled

- **🔍 Security Check**: Development security validation  
  - Quick vulnerability scan
  - Pull request security validation

## 🔄 Release Process

When you push to the `master` branch:

1. ✅ **Quality Gates**: All checks must pass
   - 🧪 **Unit Tests**: Complete test suite on all platforms (Linux, Windows, macOS)
   - 🛡️ **Security Audit**: Vulnerability scan with `cargo audit` 
   - 🎯 **Code Quality**: Formatting (`rustfmt`) and linting (`clippy`) validation
   - 🏗️ **Build Verification**: Successful compilation on all target platforms

2. 🏭 **Build Pipeline**: Multi-platform binary generation
   - **Parallel Builds**: All platforms build simultaneously
   - **Optimized Releases**: `--release` flag for production binaries
   - **Target-Specific Compilation**: Native optimization for each platform

3. 📝 **Version Generation**: Smart versioning system
   - **Base Version**: From `Cargo.toml` (e.g., `0.1.0`)
   - **Build Number**: GitHub Actions run number (incremental)
   - **Commit SHA**: First 7 characters for traceability  
   - **Full Version**: `v0.1.0-build.42.abc1234`

4. 📦 **Release Creation**: Automated GitHub release
   - **Rich Release Notes**: Features, downloads, installation guides
   - **Dual Downloads**: Standard names + versioned names
   - **Build Metadata**: Complete build information file
   - **Automatic Changelog**: Generated from commit messages

5. 🎯 **Asset Management**: Multiple download options
   - **Standard Names**: `clipboard-history-linux-x64`
   - **Versioned Names**: `clipboard-history-linux-x64-42` 
   - **Platform Coverage**: All supported architectures
   - **Build Info**: Detailed metadata for troubleshooting

## 🎯 Supported Platforms

The CI pipeline builds and tests on multiple platforms with comprehensive coverage:

### 🏗️ Build Targets
- **🐧 Linux**: Ubuntu Latest (`x86_64-unknown-linux-gnu`)
  - System dependencies: `libxcb1-dev`, `libxcb-render0-dev`, `libxcb-shape0-dev`, `libxcb-xfixes0-dev`
  - Native clipboard integration via XCB
  
- **🪟 Windows**: Windows Latest (`x86_64-pc-windows-msvc`)
  - MSVC toolchain for optimal Windows performance
  - Windows API clipboard integration
  
- **🍎 macOS Intel**: macOS Latest (`x86_64-apple-darwin`)  
  - Intel Mac compatibility
  - Native macOS clipboard APIs
  
- **🍎 macOS Apple Silicon**: macOS Latest (`aarch64-apple-darwin`)
  - ARM64 optimization for M1/M2/M3 Macs
  - Native performance on Apple Silicon

### 🧪 Testing Matrix
- **Full Test Suite**: All platforms run complete test suites
- **Cross-Platform Validation**: Ensures consistent behavior
- **Performance Testing**: Platform-specific optimizations validated

## 📦 Release Artifacts

Each successful pipeline run produces multiple download options:

### 📥 Standard Downloads (latest)
- `clipboard-history-linux-x64` - Linux x86_64 executable
- `clipboard-history-windows-x64.exe` - Windows x86_64 executable  
- `clipboard-history-macos-x64` - macOS Intel executable
- `clipboard-history-macos-arm64` - macOS Apple Silicon executable

### 🔢 Versioned Downloads (build-specific) 
- `clipboard-history-linux-x64-{build_number}` - Linux with build ID
- `clipboard-history-windows-x64-{build_number}.exe` - Windows with build ID
- `clipboard-history-macos-x64-{build_number}` - macOS Intel with build ID  
- `clipboard-history-macos-arm64-{build_number}` - macOS ARM with build ID

### 📋 Additional Files
- `BUILD_INFO.txt` - Complete build metadata
  - Version information
  - Build number and date
  - Commit SHA for traceability  
  - Platform-specific details
  - Binary checksums

## 💻 Local Development

Before pushing, ensure your code passes local checks:

```bash
# 🎨 Format code (must pass for CI)
cargo fmt

# 🔍 Check linting (zero warnings required)
cargo clippy -- -D warnings  

# 🧪 Run complete test suite
cargo test

# ⚡ Quick build verification  
cargo check

# 🏗️ Full release build
cargo build --release

# 🛡️ Security audit (optional but recommended)
cargo install cargo-audit
cargo audit
```

### 🚀 Development Workflow
```bash
# 1. Create feature branch
git checkout -b feature/amazing-feature

# 2. Make changes and verify locally
cargo fmt && cargo clippy -- -D warnings && cargo test

# 3. Commit and push (triggers dev.yml)
git commit -m "Add amazing feature"
git push origin feature/amazing-feature

# 4. Create PR (triggers cross-platform tests)
# 5. Merge to master (triggers full CI/CD)
```

### 📊 Local Testing Strategy
- **Quick Checks**: `cargo fmt && cargo clippy && cargo check`
- **Full Validation**: `cargo test && cargo build --release`
- **Security**: `cargo audit` (install once, run periodically)

## 🛡️ Security & Quality Assurance

### 🔒 Security Measures
- **🔍 Dependency Scanning**: `cargo audit` on every push
- **📊 Vulnerability Database**: Automatically updated RustSec database
- **🚨 Security Alerts**: Pipeline fails immediately if vulnerabilities found
- **🔐 Supply Chain Security**: Dependency integrity verification

### 🎯 Code Quality Standards
- **📝 Formatting**: `rustfmt` with standard Rust style
- **🔍 Linting**: `clippy` with warnings-as-errors policy
- **🧪 Test Coverage**: Comprehensive unit and integration tests
- **🏗️ Build Verification**: Multi-platform compilation validation
- **📦 Release Optimization**: `--release` builds for production

### 📈 Quality Gates
- **Zero Warnings**: All `clippy` warnings must be resolved
- **Test Passing**: 100% test pass rate required
- **Security Clear**: No known vulnerabilities permitted
- **Cross-Platform**: All platforms must build and test successfully

## 🎖️ Best Practices

### 🔄 Version Management
- **Semantic Versioning**: Follow `MAJOR.MINOR.PATCH` in `Cargo.toml`
- **Build Tracking**: Automatic build numbering for traceability
- **Commit Linking**: SHA integration for source tracking
- **Tag Uniqueness**: Prevents duplicate releases

### 🚀 Deployment Strategy  
- **Automated Releases**: Zero-touch deployment on `master`
- **Quality First**: All gates must pass before release
- **Multi-Format**: Both standard and versioned downloads
- **Rich Documentation**: Comprehensive release notes

### 🧪 Testing Philosophy
- **Development Speed**: Fast feedback with `dev.yml`
- **Production Safety**: Comprehensive validation with `ci.yml`
- **Platform Parity**: Consistent behavior across all platforms

This ensures the clipboard manager maintains the highest standards for security, quality, and reliability across all supported platforms! 🌟
