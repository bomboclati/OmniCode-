#!/usr/bin/env bash
set -e

GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}"
cat << "EOF"
  ___  _ __ ___  _ __ ___  _ __
 / _ \| '_ ` _ \| '_ ` _ \| '_ \
| (_) | | | | | | | | | | | | |
 \___/|_| |_| |_|_| |_| |_| |_|
EOF
echo -e "${NC}"
echo "OmniCode - Autonomous AI Coding Agent"
echo "======================================"
echo ""

detect_os_arch() {
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"

    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    case "$OS" in
        linux) OS="linux" ;;
        darwin) OS="macos" ;;
        mingw*|msys*|cygwin*) OS="windows" ;;
        *) echo "Unsupported OS: $OS"; exit 1 ;;
    esac

    echo "Detected: $OS/$ARCH"
}

get_latest_version() {
    if command -v curl &> /dev/null; then
        VERSION=$(curl -s https://api.github.com/repos/bomboclati/OmniCode-/releases/latest | grep '"tag_name"' | sed 's/.*"tag_name": "\(.*\)".*/\1/' 2>/dev/null || echo "v0.1.0")
    else
        VERSION="v0.1.0"
    fi
    echo "${VERSION#v}"
}

install_binary() {
    local version="$1"
    local os="$2"
    local arch="$3"

    local ext=""
    local binary_name="omni"
    local archive=""

    case "$os" in
        windows)
            ext="zip"
            binary_name="omni.exe"
            archive="omni-${version}-windows-x86_64.zip"
            ;;
        linux)
            ext="tar.gz"
            archive="omni-${version}-linux-x86_64.tar.gz"
            ;;
        macos)
            ext="tar.gz"
            if [ "$arch" = "aarch64" ]; then
                archive="omni-${version}-macos-aarch64.tar.gz"
            else
                archive="omni-${version}-macos-x86_64.tar.gz"
            fi
            ;;
        *)
            echo "Unsupported OS: $os"
            exit 1
            ;;
    esac

    local url="https://github.com/bomboclati/OmniCode-/releases/download/v${version}/${archive}"
    local tmp_dir="/tmp/omni-${RANDOM}"

    mkdir -p "$tmp_dir"
    echo "Downloading OmniCode v${version}..."

    if command -v curl &> /dev/null; then
        curl -sL "$url" -o "${tmp_dir}/${archive}" || {
            echo "Download failed. Trying cargo install..."
            if command -v cargo &> /dev/null; then
                cargo install omnicode
            else
                echo "Install Rust from https://rustup.rs and run: cargo install omnicode"
            fi
            rm -rf "$tmp_dir"
            return
        }
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "${tmp_dir}/${archive}" || {
            echo "Download failed. Trying cargo install..."
            if command -v cargo &> /dev/null; then
                cargo install omnicode
            else
                echo "Install Rust from https://rustup.rs and run: cargo install omnicode"
            fi
            rm -rf "$tmp_dir"
            return
        }
    else
        echo "Error: Need curl or wget to download"
        exit 1
    fi

    if [ "$ext" = "zip" ]; then
        unzip -q "${tmp_dir}/${archive}" -d "$tmp_dir"
    else
        tar -xzf "${tmp_dir}/${archive}" -C "$tmp_dir"
    fi

    local binary_path=$(find "$tmp_dir" -name "$binary_name" -type f 2>/dev/null | head -1)

    if [ ! -f "$binary_path" ]; then
        echo "Binary not found. Installing via cargo instead..."
        if command -v cargo &> /dev/null; then
            cargo install omnicode
        else
            echo "Install Rust from https://rustup.rs and run: cargo install omnicode"
        fi
        rm -rf "$tmp_dir"
        return
    fi

    local install_dir="/usr/local/bin"
    if [ ! -w "$install_dir" ]; then
        install_dir="${HOME}/.local/bin"
        mkdir -p "$install_dir"
    fi

    cp "$binary_path" "${install_dir}/omni"
    chmod +x "${install_dir}/omni"

    echo ""
    echo -e "${GREEN}✓ OmniCode installed to ${install_dir}/omni${NC}"

    case ":$PATH:" in
        *:${install_dir}:*) ;;
        *)
            shell_config=""
            if [ -f "${HOME}/.zshrc" ]; then shell_config="${HOME}/.zshrc"; fi
            if [ -f "${HOME}/.bashrc" ] && [ -z "$shell_config" ]; then shell_config="${HOME}/.bashrc"; fi
            if [ -n "$shell_config" ]; then
                echo "export PATH=\"${install_dir}:\$PATH\"" >> "$shell_config"
                echo "Added ${install_dir} to PATH in ${shell_config}"
            fi
            ;;
    esac

    rm -rf "$tmp_dir"
}

echo "Installing OmniCode..."
detect_os_arch
VERSION=$(get_latest_version)
install_binary "$VERSION" "$OS" "$ARCH"

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║       OmniCode installed successfully!   ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Quick start:"
echo "  omni                    # Launch TUI"
echo "  omni serve              # Start web server at http://localhost:9420"
echo '  omni "build my api"     # Run an agent task'
echo "  omni --help             # See all commands"
echo ""
echo "For more info: https://omnicode.ai"
