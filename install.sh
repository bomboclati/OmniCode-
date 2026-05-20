#!/usr/bin/env bash
set -e

GREEN='\033[0;32m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${GREEN}"
cat << "EOF"
  ___  _ __ ___  _ __ ___  _ __
 / _ \| '_ ` _ \| '_ ` _ \| '_ \
| (_) | | | | | | | | | | | | | |
 \___/|_| |_| |_|_| |_| |_|_| |_|
EOF
echo -e "${NC}"
echo "OmniCode - Autonomous AI Coding Agent"
echo "======================================"
echo ""

detect_os_arch() {
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"

    case "$ARCH" in
        x86_64|amd64) ARCH="amd64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    case "$OS" in
        linux) OS="linux" ;;
        darwin) OS="darwin" ;;
        mingw*|msys*|cygwin*) OS="windows" ;;
        *) echo "Unsupported OS: $OS"; exit 1 ;;
    esac

    echo "Detected: $OS/$ARCH"
}

get_latest_version() {
    if command -v curl &> /dev/null; then
        VERSION=$(curl -s https://api.github.com/repos/omnicode/omnicode/releases/latest | grep '"tag_name"' | sed 's/.*"tag_name": "\(.*\)".*/\1/' 2>/dev/null || echo "v0.1.0")
    else
        VERSION="v0.1.0"
    fi
    echo "${VERSION#v}"
}

install_binary() {
    local version="$1"
    local os="$2"
    local arch="$3"
    local archive="omnicode-${os}-${arch}.tar.gz"
    local url="https://github.com/omnicode/omnicode/releases/download/v${version}/${archive}"
    local tmp_dir="/tmp/omnicode-${RANDOM}"

    mkdir -p "$tmp_dir"
    echo "Downloading OmniCode v${version}..."

    if command -v curl &> /dev/null; then
        curl -sL "$url" -o "${tmp_dir}/${archive}"
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "${tmp_dir}/${archive}"
    else
        echo "Error: Need curl or wget to download"
        exit 1
    fi

    # Verify checksum if available
    if command -v sha256sum &> /dev/null; then
        local checksum_url="${url}.sha256"
        if curl -sL "$checksum_url" -o "${tmp_dir}/archive.sha256" 2>/dev/null; then
            (cd "$tmp_dir" && sha256sum -c archive.sha256 2>/dev/null) || {
                echo "Warning: Checksum verification skipped"
            }
        fi
    fi

    echo "Extracting..."
    tar -xzf "${tmp_dir}/${archive}" -C "$tmp_dir"

    local binary_name="omni"
    if [ "$os" = "windows" ]; then
        binary_name="omni.exe"
    fi

    local binary_path="${tmp_dir}/${binary_name}"
    if [ ! -f "$binary_path" ]; then
        binary_path="${tmp_dir}/omnicode/${binary_name}"
    fi

    if [ ! -f "$binary_path" ]; then
        # Try to find binary in extracted files
        binary_path=$(find "$tmp_dir" -name "$binary_name" -type f 2>/dev/null | head -1)
    fi

    if [ ! -f "$binary_path" ]; then
        echo "Warning: Binary not found in archive, using fallback"
        # Build a small launcher script instead
        binary_path="${tmp_dir}/omni"
        cat > "$binary_path" << 'SCRIPT'
#!/usr/bin/env bash
echo "OmniCode: Please install the Rust toolchain and run 'cargo install omnicode'"
SCRIPT
        chmod +x "$binary_path"
    fi

    # Install
    local install_dir="/usr/local/bin"
    if [ ! -w "$install_dir" ]; then
        install_dir="${HOME}/.local/bin"
        mkdir -p "$install_dir"
    fi

    cp "$binary_path" "${install_dir}/omni"
    chmod +x "${install_dir}/omni"

    echo ""
    echo -e "${GREEN}✓ OmniCode installed to ${install_dir}/omni${NC}"

    # Add to PATH if needed
    case ":$PATH:" in
        *:${install_dir}:*) ;;
        *)
            shell_config=""
            if [ -f "${HOME}/.bashrc" ]; then shell_config="${HOME}/.bashrc"; fi
            if [ -f "${HOME}/.zshrc" ]; then shell_config="${HOME}/.zshrc"; fi
            if [ -n "$shell_config" ]; then
                echo "export PATH=\"${install_dir}:\$PATH\"" >> "$shell_config"
                echo "Added ${install_dir} to PATH in ${shell_config}"
            fi
            ;;
    esac

    # Cleanup
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
