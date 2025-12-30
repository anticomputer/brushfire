# Release Process

## Quick Start (Recommended)

Use the provided build script for simplified building:

```bash
# Development build
./build.sh dev

# Release build for distribution (with embedded wrappers)
./build.sh release-embedded

# Release with webhook support
./build.sh release-embedded --webhook

# Clean all build artifacts
./build.sh clean
```

The build script handles wrapper building, state management, and proper feature flags automatically.

## Manual Build Process

If you prefer manual control or need to customize the build:

### 1. Build Wrapper Binaries

First, build all the wrapper binaries in release mode:

```bash
cargo build --release -p brushfire-wrappers
```

This creates wrapper binaries in `target/release/` for all supported utilities (cat, rm, cp, mv, ls, grep, etc.).

### 2. Build brush with Embedded Wrappers

Build brush with the `embed-wrappers` feature to embed the wrapper binaries:

```bash
# With policy support only
cargo build --release --features policy,embed-wrappers

# With webhook support
cargo build --release --features policy-webhook,embed-wrappers
```

The resulting `target/release/brush` binary contains all wrapper binaries embedded within it.

### 3. Distribution

The release binary is self-contained:

```bash
# Single binary - includes all wrappers
target/release/brush --version
```

When `--wrap-coreutils` is used, brush extracts embedded wrappers to a temporary directory automatically.

## Release Checklist

Using the build script:
- [ ] Clean build: `./build.sh clean`
- [ ] Build release: `./build.sh release-embedded --webhook`
- [ ] Verify binary size (should be >10MB with embedded wrappers)
- [ ] Test basic policy: `target/release/brush --profile test.profile -c 'ls'`
- [ ] Test wrapper extraction: `target/release/brush --profile test.profile --wrap-coreutils -c 'cat file.txt'`
- [ ] Test without wrapper files present (move them away temporarily to verify true embedding)

## Development vs Release

### Development Mode (default)

```bash
# Build without embedding (using build script)
./build.sh dev

# Or manually
cargo build --features policy

# Wrappers copied from target/debug/ at runtime
./target/debug/brush --profile test.profile --wrap-coreutils -c 'commands'
```

Wrappers must exist in the same directory as the brush binary.

### Release Mode (with embedding)

```bash
# Build with embedding (using build script)
./build.sh release-embedded --webhook

# Or manually
cargo build --release --features policy-webhook,embed-wrappers

# Wrappers extracted from embedded data
./target/release/brush --profile test.profile --wrap-coreutils -c 'commands'
```

Single self-contained binary, no external files needed.

## Platform-Specific Builds

### Linux

```bash
cargo build --release --target x86_64-unknown-linux-gnu --features policy-webhook,embed-wrappers
```

### macOS

```bash
cargo build --release --target x86_64-apple-darwin --features policy-webhook,embed-wrappers
cargo build --release --target aarch64-apple-darwin --features policy-webhook,embed-wrappers
```

### Cross-Compilation

For cross-compilation, build wrappers for the target platform first, then build brush with embedding.

## Distribution Methods

### Binary Release

Package `target/release/brush` as a single executable:

```bash
tar -czf brushfire-vX.Y.Z-platform.tar.gz -C target/release brush
```

### Cargo Install

Users can install from source:

```bash
cargo install --git https://github.com/anticomputer/brushfire --features policy-webhook,embed-wrappers
```

### Package Managers

For OS package managers, the `embed-wrappers` feature ensures a single binary with no runtime dependencies on wrapper files.
