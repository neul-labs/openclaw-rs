#!/usr/bin/env bash
#
# OpenClaw Rust Core Installer
#
# This script installs openclaw using the following methods (in order):
#   1. GitHub Releases (pre-built binaries)
#   2. Cargo install from crates.io
#   3. Clone and build from source
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/openclaw/openclaw-rs/main/install.sh | bash
#
#   Or with options:
#   ./install.sh --method source    # Force source build
#   ./install.sh --prefix ~/.local  # Custom install prefix
#   ./install.sh --help             # Show help

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

REPO_OWNER="openclaw"
REPO_NAME="openclaw-rs"
BINARY_NAME="openclaw"
CRATE_NAME="openclaw-cli"
GITHUB_REPO="https://github.com/${REPO_OWNER}/${REPO_NAME}"

# Default install location
DEFAULT_PREFIX="${HOME}/.local"
INSTALL_PREFIX="${INSTALL_PREFIX:-$DEFAULT_PREFIX}"
BIN_DIR="${INSTALL_PREFIX}/bin"

# Colors (disabled if not a terminal)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

# ============================================================================
# Helper Functions
# ============================================================================

info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

die() {
    error "$@"
    exit 1
}

# Check if a command exists
has_command() {
    command -v "$1" &>/dev/null
}

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)          die "Unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        armv7l)         arch="armv7" ;;
        *)              die "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Get the latest release version from GitHub
get_latest_version() {
    local version

    if has_command curl; then
        version=$(curl -fsSL "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest" 2>/dev/null \
            | grep '"tag_name"' \
            | sed -E 's/.*"([^"]+)".*/\1/')
    elif has_command wget; then
        version=$(wget -qO- "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest" 2>/dev/null \
            | grep '"tag_name"' \
            | sed -E 's/.*"([^"]+)".*/\1/')
    fi

    echo "$version"
}

# Download a file
download() {
    local url="$1"
    local dest="$2"

    if has_command curl; then
        curl -fsSL "$url" -o "$dest"
    elif has_command wget; then
        wget -q "$url" -O "$dest"
    else
        die "Neither curl nor wget found. Please install one of them."
    fi
}

# Verify the binary works
verify_binary() {
    local binary="$1"

    if [[ -x "$binary" ]] && "$binary" --version &>/dev/null; then
        return 0
    fi
    return 1
}

# ============================================================================
# Installation Methods
# ============================================================================

# Method 1: Install from GitHub Releases
install_from_release() {
    info "Attempting to install from GitHub Releases..."

    local platform version download_url archive_name tmp_dir

    platform=$(detect_platform)
    version=$(get_latest_version)

    if [[ -z "$version" ]]; then
        warn "Could not determine latest release version"
        return 1
    fi

    info "Latest version: ${version}"
    info "Platform: ${platform}"

    # Construct the download URL
    # Expected format: openclaw-{version}-{os}-{arch}.tar.gz (or .zip for Windows)
    local ext="tar.gz"
    [[ "$platform" == windows-* ]] && ext="zip"

    archive_name="${BINARY_NAME}-${version}-${platform}.${ext}"
    download_url="${GITHUB_REPO}/releases/download/${version}/${archive_name}"

    info "Downloading from: ${download_url}"

    # Create temp directory
    tmp_dir=$(mktemp -d)
    trap "rm -rf '$tmp_dir'" EXIT

    # Download the archive
    if ! download "$download_url" "${tmp_dir}/${archive_name}"; then
        warn "Failed to download release archive"
        return 1
    fi

    # Extract the archive
    info "Extracting archive..."
    cd "$tmp_dir"

    if [[ "$ext" == "tar.gz" ]]; then
        tar -xzf "$archive_name"
    else
        unzip -q "$archive_name"
    fi

    # Find and install the binary
    local binary_file
    binary_file=$(find . -name "$BINARY_NAME" -type f -perm -u+x 2>/dev/null | head -1)

    if [[ -z "$binary_file" ]]; then
        # Try without execute permission (Windows)
        binary_file=$(find . -name "${BINARY_NAME}*" -type f 2>/dev/null | head -1)
    fi

    if [[ -z "$binary_file" ]]; then
        warn "Binary not found in archive"
        return 1
    fi

    # Install the binary
    mkdir -p "$BIN_DIR"
    cp "$binary_file" "${BIN_DIR}/${BINARY_NAME}"
    chmod +x "${BIN_DIR}/${BINARY_NAME}"

    if verify_binary "${BIN_DIR}/${BINARY_NAME}"; then
        success "Installed ${BINARY_NAME} from release to ${BIN_DIR}/${BINARY_NAME}"
        return 0
    else
        warn "Binary verification failed"
        rm -f "${BIN_DIR}/${BINARY_NAME}"
        return 1
    fi
}

# Method 2: Install using cargo from crates.io
install_from_cargo() {
    info "Attempting to install using cargo..."

    if ! has_command cargo; then
        warn "Cargo not found. Please install Rust: https://rustup.rs"
        return 1
    fi

    # Check Rust version
    local rust_version
    rust_version=$(rustc --version | grep -oE '[0-9]+\.[0-9]+' | head -1)
    local required_version="1.85"

    if [[ "$(printf '%s\n' "$required_version" "$rust_version" | sort -V | head -n1)" != "$required_version" ]]; then
        warn "Rust version ${rust_version} is older than required ${required_version}"
        warn "Please update Rust: rustup update"
        return 1
    fi

    info "Using Rust version: ${rust_version}"

    # Try to install from crates.io
    if cargo install "$CRATE_NAME" 2>/dev/null; then
        local cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin/${BINARY_NAME}"

        if verify_binary "$cargo_bin"; then
            success "Installed ${BINARY_NAME} from crates.io"
            return 0
        fi
    fi

    warn "Failed to install from crates.io (package may not be published yet)"
    return 1
}

# Method 3: Build from local source (if running from repo directory)
install_from_local() {
    info "Attempting to install from local source..."

    if ! has_command cargo; then
        warn "Cargo not found. Please install Rust: https://rustup.rs"
        return 1
    fi

    # Check if we're in the repo directory
    local script_dir repo_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    if [[ -f "${script_dir}/Cargo.toml" ]] && grep -q "openclaw-cli" "${script_dir}/Cargo.toml" 2>/dev/null; then
        repo_dir="$script_dir"
    elif [[ -f "./Cargo.toml" ]] && grep -q "openclaw-cli" "./Cargo.toml" 2>/dev/null; then
        repo_dir="$(pwd)"
    else
        return 1  # Not in repo, skip this method
    fi

    info "Found local repository at: ${repo_dir}"

    # Check Rust version
    local rust_version
    rust_version=$(rustc --version | grep -oE '[0-9]+\.[0-9]+' | head -1)
    local required_version="1.85"

    if [[ "$(printf '%s\n' "$required_version" "$rust_version" | sort -V | head -n1)" != "$required_version" ]]; then
        warn "Rust version ${rust_version} is older than required ${required_version}"
        warn "Please update Rust: rustup update"
        return 1
    fi

    info "Using Rust version: ${rust_version}"

    cd "$repo_dir"

    info "Building from local source (this may take a few minutes)..."
    if ! cargo build --release -p "$CRATE_NAME"; then
        warn "Build failed"
        return 1
    fi

    # Install the binary
    local built_binary="target/release/${BINARY_NAME}"

    if [[ ! -f "$built_binary" ]]; then
        warn "Built binary not found at: ${built_binary}"
        return 1
    fi

    mkdir -p "$BIN_DIR"
    cp "$built_binary" "${BIN_DIR}/${BINARY_NAME}"
    chmod +x "${BIN_DIR}/${BINARY_NAME}"

    if verify_binary "${BIN_DIR}/${BINARY_NAME}"; then
        success "Installed ${BINARY_NAME} from local source to ${BIN_DIR}/${BINARY_NAME}"
        return 0
    else
        warn "Binary verification failed"
        rm -f "${BIN_DIR}/${BINARY_NAME}"
        return 1
    fi
}

# Method 4: Clone and build from source
install_from_source() {
    info "Attempting to clone and install from source..."

    if ! has_command cargo; then
        warn "Cargo not found. Please install Rust: https://rustup.rs"
        return 1
    fi

    if ! has_command git; then
        warn "Git not found. Please install git."
        return 1
    fi

    # Check Rust version
    local rust_version
    rust_version=$(rustc --version | grep -oE '[0-9]+\.[0-9]+' | head -1)
    local required_version="1.85"

    if [[ "$(printf '%s\n' "$required_version" "$rust_version" | sort -V | head -n1)" != "$required_version" ]]; then
        warn "Rust version ${rust_version} is older than required ${required_version}"
        warn "Please update Rust: rustup update"
        return 1
    fi

    info "Using Rust version: ${rust_version}"

    # Create temp directory for cloning
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf '$tmp_dir'" EXIT

    info "Cloning repository..."
    if ! git clone --depth 1 "$GITHUB_REPO" "$tmp_dir/repo" 2>/dev/null; then
        warn "Failed to clone repository (may not exist yet)"
        return 1
    fi

    cd "$tmp_dir/repo"

    info "Building from source (this may take a few minutes)..."
    if ! cargo build --release -p "$CRATE_NAME"; then
        warn "Build failed"
        return 1
    fi

    # Install the binary
    local built_binary="target/release/${BINARY_NAME}"

    if [[ ! -f "$built_binary" ]]; then
        warn "Built binary not found at: ${built_binary}"
        return 1
    fi

    mkdir -p "$BIN_DIR"
    cp "$built_binary" "${BIN_DIR}/${BINARY_NAME}"
    chmod +x "${BIN_DIR}/${BINARY_NAME}"

    if verify_binary "${BIN_DIR}/${BINARY_NAME}"; then
        success "Installed ${BINARY_NAME} from source to ${BIN_DIR}/${BINARY_NAME}"
        return 0
    else
        warn "Binary verification failed"
        rm -f "${BIN_DIR}/${BINARY_NAME}"
        return 1
    fi
}

# ============================================================================
# Post-Installation
# ============================================================================

check_path() {
    local bin_dir="$1"

    if [[ ":$PATH:" != *":${bin_dir}:"* ]]; then
        echo ""
        warn "${bin_dir} is not in your PATH"
        echo ""
        echo "Add it to your shell configuration:"
        echo ""

        if [[ -f "$HOME/.bashrc" ]]; then
            echo "  echo 'export PATH=\"${bin_dir}:\$PATH\"' >> ~/.bashrc"
            echo "  source ~/.bashrc"
        fi

        if [[ -f "$HOME/.zshrc" ]]; then
            echo ""
            echo "  # Or for zsh:"
            echo "  echo 'export PATH=\"${bin_dir}:\$PATH\"' >> ~/.zshrc"
            echo "  source ~/.zshrc"
        fi

        echo ""
    fi
}

install_system_deps() {
    info "Checking system dependencies for sandboxing..."

    case "$(uname -s)" in
        Linux*)
            if ! has_command bwrap; then
                echo ""
                warn "bubblewrap (bwrap) is not installed"
                echo ""
                echo "Install it for sandboxing support:"
                echo ""
                if has_command apt-get; then
                    echo "  sudo apt-get install bubblewrap"
                elif has_command dnf; then
                    echo "  sudo dnf install bubblewrap"
                elif has_command pacman; then
                    echo "  sudo pacman -S bubblewrap"
                else
                    echo "  Please install bubblewrap using your package manager"
                fi
                echo ""
            else
                success "bubblewrap is installed"
            fi
            ;;
        Darwin*)
            success "macOS sandbox-exec is available (built-in)"
            ;;
        *)
            info "Windows uses Job Objects (no additional dependencies)"
            ;;
    esac
}

print_next_steps() {
    echo ""
    echo -e "${BOLD}${GREEN}Installation complete!${NC}"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. Run the setup wizard:"
    echo "     ${BINARY_NAME} onboard"
    echo ""
    echo "  2. Start the gateway:"
    echo "     ${BINARY_NAME} gateway run"
    echo ""
    echo "  3. Check status:"
    echo "     ${BINARY_NAME} status"
    echo ""
    echo "For help, run: ${BINARY_NAME} --help"
    echo ""
}

# ============================================================================
# Main
# ============================================================================

show_help() {
    cat << EOF
${BOLD}OpenClaw Installer${NC}

Usage: $0 [OPTIONS]

Options:
    -m, --method METHOD    Force installation method:
                           - release: GitHub releases only
                           - cargo:   cargo install from crates.io only
                           - local:   Build from local source (if in repo)
                           - source:  Clone and build from source
                           - auto:    Try all methods in order (default)

    -p, --prefix PATH      Installation prefix (default: ~/.local)
                           Binary will be installed to PREFIX/bin

    -h, --help             Show this help message

Examples:
    $0                           # Auto-detect best method
    $0 --method local            # Build from local repo
    $0 --method source           # Clone and build from source
    $0 --prefix /usr/local       # Install to /usr/local/bin

EOF
}

main() {
    local method="auto"
    local success_install=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -m|--method)
                method="$2"
                shift 2
                ;;
            -p|--prefix)
                INSTALL_PREFIX="$2"
                BIN_DIR="${INSTALL_PREFIX}/bin"
                shift 2
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                die "Unknown option: $1. Use --help for usage."
                ;;
        esac
    done

    echo ""
    echo -e "${BOLD}OpenClaw Installer${NC}"
    echo "=================="
    echo ""
    info "Install prefix: ${INSTALL_PREFIX}"
    info "Binary directory: ${BIN_DIR}"
    echo ""

    case "$method" in
        release)
            install_from_release && success_install=true
            ;;
        cargo)
            install_from_cargo && success_install=true
            ;;
        local)
            install_from_local && success_install=true
            ;;
        source)
            install_from_source && success_install=true
            ;;
        auto)
            # Try methods in order of preference
            if install_from_release; then
                success_install=true
            elif install_from_cargo; then
                success_install=true
            elif install_from_local; then
                success_install=true
            elif install_from_source; then
                success_install=true
            fi
            ;;
        *)
            die "Unknown method: $method. Valid options: release, cargo, local, source, auto"
            ;;
    esac

    if [[ "$success_install" == "true" ]]; then
        echo ""
        check_path "$BIN_DIR"
        install_system_deps
        print_next_steps
        exit 0
    else
        echo ""
        error "All installation methods failed."
        echo ""
        echo "Please try installing manually:"
        echo ""
        echo "  Option 1 - If you have the source locally:"
        echo "    cd /path/to/openclaw-rs"
        echo "    cargo build --release -p ${CRATE_NAME}"
        echo "    cp target/release/${BINARY_NAME} ~/.local/bin/"
        echo ""
        echo "  Option 2 - Clone and build:"
        echo "    1. Install Rust 1.85+: https://rustup.rs"
        echo "    2. git clone ${GITHUB_REPO}"
        echo "    3. cd ${REPO_NAME}"
        echo "    4. cargo install --path crates/${CRATE_NAME}"
        echo ""
        exit 1
    fi
}

main "$@"
