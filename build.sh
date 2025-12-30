#!/usr/bin/env bash
#
# Brushfire build script
#
# Handles building brush with proper wrapper binary management
# and state cleaning between development and release builds.

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
  dev               Build for development (no embedded wrappers)
  release           Build optimized release (no embedded wrappers)
  release-embedded  Build release with embedded wrappers (for distribution)
  clean             Clean all build artifacts
  help              Show this help message

Options:
  --webhook         Enable webhook support (policy-webhook feature)
  --verbose         Show full cargo output

Examples:
  $0 dev                    # Development build
  $0 release-embedded       # Distribution build with embedded wrappers
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

    print_info "Building for development (no embedded wrappers)..."

    # Build wrappers in debug mode
    print_info "Building wrapper binaries (debug mode)..."
    if [[ "$VERBOSE" == "1" ]]; then
        cargo build -p brushfire-wrappers
    else
        cargo build -p brushfire-wrappers --quiet
    fi

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
    print_info "Wrappers: target/debug/{cat,ls,grep,...}"

    # Show size
    local size=$(du -h target/debug/brush 2>/dev/null | cut -f1)
    print_info "Binary size: $size"
}

build_release() {
    local features="policy"
    if [[ "$ENABLE_WEBHOOK" == "1" ]]; then
        features="policy-webhook"
    fi

    print_info "Building optimized release (no embedded wrappers)..."

    # Build wrappers in release mode
    print_info "Building wrapper binaries (release mode)..."
    if [[ "$VERBOSE" == "1" ]]; then
        cargo build --release -p brushfire-wrappers
    else
        cargo build --release -p brushfire-wrappers --quiet
    fi

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
    print_info "Wrappers: target/release/{cat,ls,grep,...}"

    # Show size
    local size=$(du -h target/release/brush 2>/dev/null | cut -f1)
    print_info "Binary size: $size"
}

build_release_embedded() {
    local features="policy,embed-wrappers"
    if [[ "$ENABLE_WEBHOOK" == "1" ]]; then
        features="policy-webhook,embed-wrappers"
    fi

    print_info "Building release with embedded wrappers (for distribution)..."

    # Build wrappers in release mode FIRST (with webhook if enabled)
    print_info "Building wrapper binaries (release mode)..."
    local wrapper_features=""
    if [[ "$ENABLE_WEBHOOK" == "1" ]]; then
        wrapper_features="--features webhook"
    fi

    if [[ "$VERBOSE" == "1" ]]; then
        cargo build --release -p brushfire-wrappers $wrapper_features
    else
        cargo build --release -p brushfire-wrappers $wrapper_features --quiet
    fi

    print_success "Wrappers built"

    # Verify key wrappers exist (sample check)
    local missing_wrappers=()
    for util in cat ls grep rm cp; do
        if [[ ! -f "target/release/$util" ]]; then
            missing_wrappers+=("$util")
        fi
    done

    if [[ ${#missing_wrappers[@]} -gt 0 ]]; then
        print_error "Wrapper binaries not found: ${missing_wrappers[*]}"
        print_error "Cannot embed wrappers"
        exit 1
    fi

    # Build brush with embedding
    print_info "Building brush with embedded wrappers..."
    print_warning "This may take a while (embedding ~30 binaries)..."
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

    print_success "Release build with embedded wrappers complete"
    print_info "Binary: target/release/brush"
    print_info "Distribution: Single self-contained binary"

    # Show size
    local size=$(du -h target/release/brush 2>/dev/null | cut -f1)
    print_info "Binary size: $size (includes embedded wrappers)"

    # Verify embedding by checking size
    local size_mb=$(du -m target/release/brush 2>/dev/null | cut -f1)
    if [[ "$size_mb" -lt 10 ]]; then
        print_warning "Binary size is unusually small ($size)"
        print_warning "Wrappers may not be properly embedded"
    fi
}

# Parse arguments
TARGET=""
ENABLE_WEBHOOK="0"
VERBOSE="0"

while [[ $# -gt 0 ]]; do
    case $1 in
        dev|release|release-embedded|clean|help)
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
    release-embedded)
        build_release_embedded
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
