#!/bin/sh
set -eu

# dotman installer for Linux and macOS
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/brilyyy/dotman/main/scripts/install.sh | sh
# Or locally:
#   sh scripts/install.sh

DOTMAN_REPO="${DOTMAN_REPO:-brilyyy/dotman}"
DOTMAN_VERSION="${DOTMAN_VERSION:-latest}"

# Determine default install directory
if [ "$(id -u)" -eq 0 ]; then
    DEFAULT_DIR="/usr/local/bin"
else
    DEFAULT_DIR="$HOME/.local/bin"
fi
INSTALL_DIR="${DOTMAN_INSTALL_DIR:-$DEFAULT_DIR}"

# Styling helpers (respects NO_COLOR and non-tty)
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD="\033[1m"
    GREEN="\033[32m"
    YELLOW="\033[33m"
    CYAN="\033[36m"
    DIM="\033[2m"
    NC="\033[0m"
else
    BOLD=""
    GREEN=""
    YELLOW=""
    CYAN=""
    DIM=""
    NC=""
fi

print_badge_ok() {
    printf "%b\n" "${GREEN}${BOLD}✓${NC} $1"
}

print_badge_info() {
    printf "%b\n" "${CYAN}${BOLD}•${NC} $1"
}

print_badge_warn() {
    printf "%b\n" "${YELLOW}${BOLD}!${NC} $1"
}

print_help() {
    cat << 'EOF'
dotman installer

Usage:
  install.sh [options]

Options:
  -d, --dir <DIR>        Installation directory (default: ~/.local/bin or /usr/local/bin for root)
  -v, --version <VER>    Specific dotman version to install (default: latest)
  -h, --help             Show this help message
EOF
}

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -v|--version)
            DOTMAN_VERSION="$2"
            shift 2
            ;;
        -h|--help)
            print_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            print_help
            exit 1
            ;;
    esac
done

printf "%b\n\n" "${BOLD}dotman installer${NC}"

# 1. Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux)
        OS_TARGET="unknown-linux-musl"
        ;;
    Darwin)
        OS_TARGET="apple-darwin"
        ;;
    *)
        echo "Error: Unsupported operating system '$OS'. dotman supports Linux and macOS." >&2
        exit 1
        ;;
esac

# 2. Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        ARCH_TARGET="x86_64"
        ;;
    aarch64|arm64)
        ARCH_TARGET="aarch64"
        ;;
    *)
        echo "Error: Unsupported CPU architecture '$ARCH'." >&2
        exit 1
        ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
print_badge_info "Detected platform: ${BOLD}${OS} ${ARCH}${NC} (${TARGET})"

# 3. Locate binary (Check local repo build first, fallback to remote download)
SCRIPT_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd || echo "")"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." 2>/dev/null && pwd || echo "")"

TEMP_DIR=""
CLEANUP_TMP=0

cleanup() {
    if [ "$CLEANUP_TMP" -eq 1 ] && [ -n "$TEMP_DIR" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

if [ -f "${REPO_DIR}/target/${TARGET}/release/dotman" ]; then
    SOURCE_BIN="${REPO_DIR}/target/${TARGET}/release/dotman"
    print_badge_info "Using local release binary: ${DIM}${SOURCE_BIN}${NC}"
elif [ -f "${REPO_DIR}/target/release/dotman" ]; then
    SOURCE_BIN="${REPO_DIR}/target/release/dotman"
    print_badge_info "Using local release binary: ${DIM}${SOURCE_BIN}${NC}"
else
    # Download precompiled binary
    print_badge_info "Downloading dotman (${DOTMAN_VERSION}) for ${TARGET}..."
    
    if command -v curl >/dev/null 2>&1; then
        DOWNLOADER="curl"
    elif command -v wget >/dev/null 2>&1; then
        DOWNLOADER="wget"
    else
        echo "Error: Neither curl nor wget was found. Please install one to download dotman." >&2
        exit 1
    fi

    TEMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t dotman.XXXXXX)"
    CLEANUP_TMP=1

    if [ "$DOTMAN_VERSION" = "latest" ]; then
        URL="https://github.com/${DOTMAN_REPO}/releases/latest/download/dotman-${TARGET}.tar.gz"
        URL_RAW="https://github.com/${DOTMAN_REPO}/releases/latest/download/dotman-${TARGET}"
    else
        URL="https://github.com/${DOTMAN_REPO}/releases/download/${DOTMAN_VERSION}/dotman-${TARGET}.tar.gz"
        URL_RAW="https://github.com/${DOTMAN_REPO}/releases/download/${DOTMAN_VERSION}/dotman-${TARGET}"
    fi

    # Try tar.gz or direct raw binary
    DOWNLOAD_SUCCESS=0
    if [ "$DOWNLOADER" = "curl" ]; then
        if curl -fsSL "$URL" -o "${TEMP_DIR}/dotman.tar.gz" 2>/dev/null; then
            tar -xzf "${TEMP_DIR}/dotman.tar.gz" -C "$TEMP_DIR"
            DOWNLOAD_SUCCESS=1
        elif curl -fsSL "$URL_RAW" -o "${TEMP_DIR}/dotman" 2>/dev/null; then
            DOWNLOAD_SUCCESS=1
        fi
    elif [ "$DOWNLOADER" = "wget" ]; then
        if wget -qO "${TEMP_DIR}/dotman.tar.gz" "$URL" 2>/dev/null; then
            tar -xzf "${TEMP_DIR}/dotman.tar.gz" -C "$TEMP_DIR"
            DOWNLOAD_SUCCESS=1
        elif wget -qO "${TEMP_DIR}/dotman" "$URL_RAW" 2>/dev/null; then
            DOWNLOAD_SUCCESS=1
        fi
    fi

    if [ "$DOWNLOAD_SUCCESS" -eq 0 ] || [ ! -f "${TEMP_DIR}/dotman" ]; then
        echo "Error: Failed to download prebuilt binary for ${TARGET} from GitHub releases." >&2
        echo "Please check https://github.com/${DOTMAN_REPO}/releases for available assets." >&2
        exit 1
    fi

    SOURCE_BIN="${TEMP_DIR}/dotman"
fi

# 4. Install binary
mkdir -p "$INSTALL_DIR"
cp "$SOURCE_BIN" "${INSTALL_DIR}/dotman"
chmod +x "${INSTALL_DIR}/dotman"

print_badge_ok "Installed dotman to ${BOLD}${INSTALL_DIR}/dotman${NC}"

# 5. Verify installation
INSTALLED_VER="$("${INSTALL_DIR}/dotman" --version 2>/dev/null || echo "dotman")"
print_badge_ok "Verified: ${DIM}${INSTALLED_VER}${NC}"

# 6. Check $PATH
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        IN_PATH=1
        ;;
    *)
        IN_PATH=0
        ;;
esac

if [ "$IN_PATH" -eq 0 ]; then
    echo ""
    print_badge_warn "${INSTALL_DIR} is not in your \$PATH."
    echo "  To use dotman directly, add it to your shell configuration:"
    echo ""
    
    CURRENT_SHELL="$(basename "${SHELL:-sh}")"
    case "$CURRENT_SHELL" in
        zsh)
            echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc"
            echo "    source ~/.zshrc"
            ;;
        fish)
            echo "    fish_add_path ${INSTALL_DIR}"
            ;;
        *)
            echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc"
            echo "    source ~/.bashrc"
            ;;
    esac
fi

printf "%b\n" "\nRun '${BOLD}dotman --help${NC}' to get started!"
