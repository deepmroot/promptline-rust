# Deployment Guide

This document describes how to build, package, and distribute PromptLine across different platforms.

## Table of Contents

- [Building from Source](#building-from-source)
- [Release Process](#release-process)
- [Distribution Channels](#distribution-channels)
- [Platform-Specific Instructions](#platform-specific-instructions)
- [Configuration Management](#configuration-management)
- [Monitoring and Telemetry](#monitoring-and-telemetry)

## Building from Source

### Prerequisites

**Required:**
- Rust 1.70+ ([Install from rustup.rs](https://rustup.rs/))
- Git

**Optional:**
- Docker (for containerized builds)
- Cross-compilation tools (for multi-platform builds)

### Development Build

```bash
# Clone repository
git clone https://github.com/yourusername/promptline-rust.git
cd promptline-rust

# Build in debug mode (fast compilation, slower runtime)
cargo build

# Run
./target/debug/promptline --version
```

### Release Build

```bash
# Build with optimizations
cargo build --release

# Binary location
./target/release/promptline
```

### Build Configuration

**Cargo.toml optimizations:**
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization, slower compile
strip = true            # Remove debug symbols
panic = "abort"         # Smaller binary size
```

### Feature Flags

```bash
# Build without telemetry
cargo build --release --no-default-features

# Build with all features
cargo build --release --all-features

# Build with specific features
cargo build --release --features "local-llm,plugin-system"
```

**Available features:**
- `telemetry` - Anonymous usage analytics (opt-in)
- `local-llm` - Local model support (llama.cpp)
- `plugin-system` - Dynamic plugin loading
- `sandbox` - Command sandboxing

## Release Process

### Version Bumping

Follow [Semantic Versioning](https://semver.org/):

```bash
# Update version in Cargo.toml
# MAJOR.MINOR.PATCH
version = "0.2.0"

# Update CHANGELOG.md
## [0.2.0] - 2023-11-17
### Added
- Context management and memory
- REPL interactive mode

### Fixed
- Token estimation accuracy

# Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"

# Create tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push
git push origin main --tags
```

### Automated Release (GitHub Actions)

```.yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: promptline
            asset_name: promptline-linux-x86_64
          
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact_name: promptline
            asset_name: promptline-linux-aarch64
          
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: promptline
            asset_name: promptline-macos-x86_64
          
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: promptline
            asset_name: promptline-macos-aarch64
          
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: promptline.exe
            asset_name: promptline-windows-x86_64.exe
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Strip binary (Linux/macOS)
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
      
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.asset_name }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
  
  release:
    needs: build
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Download artifacts
        uses: actions/download-artifact@v3
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            promptline-*/promptline*
          body_path: CHANGELOG.md
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Manual Cross-Compilation

Using [cross](https://github.com/cross-rs/cross):

```bash
# Install cross
cargo install cross

# Build for Linux
cross build --release --target x86_64-unknown-linux-gnu

# Build for Windows
cross build --release --target x86_64-pc-windows-gnu

# Build for macOS (from macOS only)
cargo build --release --target x86_64-apple-darwin

# Build for ARM
cross build --release --target aarch64-unknown-linux-gnu
```

## Distribution Channels

### 1. Crates.io

```bash
# Login (first time only)
cargo login

# Publish
cargo publish

# Users install with:
# cargo install promptline
```

**Pre-publish checklist:**
- [ ] All tests passing
- [ ] Documentation complete
- [ ] README.md up to date
- [ ] CHANGELOG.md updated
- [ ] Version bumped
- [ ] License file present
- [ ] No sensitive data in repo

### 2. Homebrew (macOS/Linux)

Create formula in homebrew-promptline tap:

```ruby
# Formula/promptline.rb
class Promptline < Formula
  desc "AI-powered CLI assistant for developers"
  homepage "https://github.com/yourusername/promptline-rust"
  url "https://github.com/yourusername/promptline-rust/archive/v0.1.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "promptline", shell_output("#{bin}/promptline --version")
  end
end
```

Users install with:
```bash
brew tap yourusername/promptline
brew install promptline
```

### 3. Binary Releases (GitHub)

Download pre-built binaries from [Releases page](https://github.com/yourusername/promptline-rust/releases).

**Installation:**
```bash
# Linux/macOS
curl -L https://github.com/yourusername/promptline-rust/releases/download/v0.1.0/promptline-linux-x86_64 -o promptline
chmod +x promptline
sudo mv promptline /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/yourusername/promptline-rust/releases/download/v0.1.0/promptline-windows-x86_64.exe" -OutFile "promptline.exe"
# Move to PATH location
```

### 4. Docker Image

```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/promptline /usr/local/bin/

ENTRYPOINT ["promptline"]
```

Build and publish:
```bash
# Build
docker build -t promptline:latest .

# Tag
docker tag promptline:latest yourusername/promptline:0.1.0
docker tag promptline:latest yourusername/promptline:latest

# Push to Docker Hub
docker push yourusername/promptline:0.1.0
docker push yourusername/promptline:latest
```

Users run with:
```bash
docker run -it --rm \
  -v $(pwd):/workspace \
  -e OPENAI_API_KEY=$OPENAI_API_KEY \
  yourusername/promptline:latest \
  "list all rust files"
```

### 5. Snap (Linux)

```yaml
# snapcraft.yaml
name: promptline
version: '0.1.0'
summary: AI-powered CLI assistant
description: |
  PromptLine is an intelligent command-line interface that uses
  AI to help with coding tasks.

grade: stable
confinement: strict
base: core22

apps:
  promptline:
    command: bin/promptline
    plugs:
      - home
      - network

parts:
  promptline:
    plugin: rust
    source: .
```

Build and publish:
```bash
snapcraft
snapcraft upload --release=stable promptline_0.1.0_amd64.snap
```

## Platform-Specific Instructions

### Linux

**System Package:**
```bash
# Debian/Ubuntu (.deb)
cargo install cargo-deb
cargo deb

# Install
sudo dpkg -i target/debian/promptline_0.1.0_amd64.deb

# RPM (Fedora/RHEL)
cargo install cargo-rpm
cargo rpm build

# Install
sudo rpm -i target/release/rpmbuild/RPMS/x86_64/promptline-0.1.0-1.x86_64.rpm
```

**Shell Completions:**
```bash
# Generate completions
promptline --completions bash > /etc/bash_completion.d/promptline
promptline --completions zsh > /usr/share/zsh/site-functions/_promptline
promptline --completions fish > /usr/share/fish/completions/promptline.fish
```

### macOS

**Code Signing:**
```bash
# Sign binary (requires Apple Developer ID)
codesign --sign "Developer ID Application: Your Name" \
  --options runtime \
  --entitlements promptline.entitlements \
  target/release/promptline

# Verify
codesign --verify --verbose target/release/promptline
```

**Notarization:**
```bash
# Create ZIP
ditto -c -k --keepParent target/release/promptline promptline.zip

# Submit for notarization
xcrun notarytool submit promptline.zip \
  --apple-id "your@email.com" \
  --password "@keychain:NOTARIZATION_PASSWORD" \
  --team-id "TEAMID"

# Check status
xcrun notarytool log <submission-id>

# Staple ticket
xcrun stapler staple target/release/promptline
```

### Windows

**Code Signing:**
```powershell
# Sign with certificate
signtool sign /f certificate.pfx /p password /tr http://timestamp.digicert.com /td sha256 /fd sha256 target\release\promptline.exe

# Verify
signtool verify /pa target\release\promptline.exe
```

**Installer (WiX):**
```xml
<!-- promptline.wxs -->
<?xml version="1.0"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="PromptLine" Version="0.1.0" 
           Manufacturer="YourName" Language="1033">
    <Package InstallerVersion="200" Compressed="yes" />
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLDIR" Name="PromptLine">
          <Component Id="MainExecutable">
            <File Source="target\release\promptline.exe" />
          </Component>
        </Directory>
      </Directory>
    </Directory>
    
    <Feature Id="Complete" Level="1">
      <ComponentRef Id="MainExecutable" />
    </Feature>
  </Product>
</Wix>
```

Build installer:
```powershell
cargo install cargo-wix
cargo wix
```

## Configuration Management

### Default Config Location

- **Linux**: `~/.config/promptline/config.yaml`
- **macOS**: `~/Library/Application Support/PromptLine/config.yaml`
- **Windows**: `%APPDATA%\PromptLine\config.yaml`

### First-Run Setup

```bash
# Initialize configuration
promptline init

# This creates:
# - Config file with defaults
# - Log directory
# - Backup directory
# - Prompts user for API keys
```

### Environment Variables

```bash
# API Keys
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Configuration override
export PROMPTLINE_CONFIG="/path/to/config.yaml"

# Disable telemetry
export PROMPTLINE_NO_TELEMETRY=1

# Debug mode
export PROMPTLINE_LOG_LEVEL=debug
```

## Monitoring and Telemetry

### Opt-In Telemetry

If telemetry feature is enabled and user opts in:

```rust
// Collected (anonymous):
- OS and architecture
- PromptLine version
- Model provider used (not API key)
- Tool usage counts
- Error types and frequency
- Performance metrics

// Never collected:
- Source code
- File paths
- User prompts
- Command outputs
- API keys
- Personally identifiable information
```

### Logging

```yaml
# config.yaml
logging:
  level: info  # debug, info, warn, error
  file: ~/.promptline/logs/promptline.log
  max_size_mb: 10
  backup_count: 3
```

**Log files:**
- Session logs: `~/.promptline/logs/session_20231117_143022.log`
- Error logs: `~/.promptline/logs/error.log`

### Health Check

```bash
# Check installation
promptline --doctor

# Output:
# ✓ Binary version: 0.1.0
# ✓ Config file found
# ✓ OpenAI API key configured
# ✗ Local model not configured
# ✓ All tools available
# ✓ Backup directory writable
```

## Update Mechanism

### Self-Update (Future)

```bash
# Check for updates
promptline --check-update

# Upgrade to latest
promptline --self-update
```

### Version Checking

```rust
impl Updater {
    pub async fn check_for_updates(&self) -> Result<Option<Version>> {
        let latest = reqwest::get("https://api.github.com/repos/yourusername/promptline-rust/releases/latest")
            .await?
            .json::<Release>()
            .await?;
        
        let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let latest_version = Version::parse(&latest.tag_name.trim_start_matches('v'))?;
        
        if latest_version > current {
            Ok(Some(latest_version))
        } else {
            Ok(None)
        }
    }
}
```

## Rollback Strategy

If a release has critical issues:

1. **Immediate:** Unpublish from crates.io (if possible)
2. **Quick Fix:** Release patch version (e.g., 0.1.1)
3. **Communication:** Update release notes, notify users
4. **Rollback Binary:** Update "latest" tag to previous version

## Deployment Checklist

Before releasing:

- [ ] All tests passing on all platforms
- [ ] Benchmarks show acceptable performance
- [ ] Security audit clean (`cargo audit`)
- [ ] Documentation updated
- [ ] CHANGELOG.md complete
- [ ] Version bumped
- [ ] Git tag created
- [ ] Binaries built for all platforms
- [ ] Binaries tested on target platforms
- [ ] Release notes written
- [ ] Published to crates.io
- [ ] GitHub release created
- [ ] Docker image published
- [ ] Homebrew formula updated
- [ ] Announcement posted

---

**See Also:**
- [Contributing](CONTRIBUTING.md) - Development workflow
- [Testing](TESTING.md) - Test requirements before release
- [Roadmap](ROADMAP.md) - Upcoming releases
