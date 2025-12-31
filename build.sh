#!/usr/bin/env bash
#
# Brushfire build script
#
# Simplified build script for brush with policy enforcement

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust: https://rustup.rs/" >&2
    exit 1
fi

# Detect if output is a TTY for color support
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

print_info() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}==>${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}==>${NC} $1"
}

print_error() {
    echo -e "${RED}==>${NC} $1"
}

print_usage() {
    cat << EOF
Usage: $0 <target> [options]

Targets:
  dev               Build for development
  release           Build optimized release
  clean             Clean all build artifacts
  help              Show this help message

Options:
  --webhook         Enable webhook support (policy-webhook feature)
  --verbose         Show full cargo output

Examples:
  $0 dev                    # Development build
  $0 release                # Release build
  $0 release --webhook      # Release build with webhook support
  $0 clean                  # Clean everything

EOF
}

clean_build() {
    print_info "Cleaning build artifacts..."
    cargo clean
    print_success "Build artifacts cleaned"
}

build_dev() {
    local features="policy"
    if [[ "$ENABLE_WEBHOOK" == "1" ]]; then
        features="policy-webhook"
    fi

    print_info "Building for development..."

    # Build brush
    print_info "Building brush (debug mode)..."
    if [[ "$VERBOSE" == "1" ]]; then
        cargo build -p brush-shell --features "$features"
    else
        cargo build -p brush-shell --features "$features" --quiet
    fi

    # Verify binary was built
    if [[ ! -f target/debug/brush ]]; then
        print_error "Build failed: target/debug/brush not found"
        exit 1
    fi

    print_success "Development build complete"
    print_info "Binary: target/debug/brush"

    # Show size
    local size=$(du -h target/debug/brush 2>/dev/null | cut -f1)
    print_info "Binary size: $size"
}

build_release() {
    local features="policy"
    if [[ "$ENABLE_WEBHOOK" == "1" ]]; then
        features="policy-webhook"
    fi

    print_info "Building optimized release..."

    # Build brush
    print_info "Building brush (release mode)..."
    if [[ "$VERBOSE" == "1" ]]; then
        cargo build --release -p brush-shell --features "$features"
    else
        cargo build --release -p brush-shell --features "$features" --quiet
    fi

    # Verify binary was built
    if [[ ! -f target/release/brush ]]; then
        print_error "Build failed: target/release/brush not found"
        exit 1
    fi

    print_success "Release build complete"
    print_info "Binary: target/release/brush"

    # Show size
    local size=$(du -h target/release/brush 2>/dev/null | cut -f1)
    print_info "Binary size: $size"
}

# Parse arguments
TARGET=""
ENABLE_WEBHOOK="0"
VERBOSE="0"

while [[ $# -gt 0 ]]; do
    case $1 in
        dev|release|clean|help)
            TARGET="$1"
            shift
            ;;
        --webhook)
            ENABLE_WEBHOOK="1"
            shift
            ;;
        --verbose)
            VERBOSE="1"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate target
if [[ -z "$TARGET" ]]; then
    print_error "No target specified"
    print_usage
    exit 1
fi

# Execute build
case $TARGET in
    dev)
        build_dev
        ;;
    release)
        build_release
        ;;
    clean)
        clean_build
        ;;
    help)
        print_usage
        ;;
    *)
        print_error "Unknown target: $TARGET"
        print_usage
        exit 1
        ;;
esac
