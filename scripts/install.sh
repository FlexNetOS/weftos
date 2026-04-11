#!/bin/sh
# WeftOS Universal Installer / Updater
#
# Usage:
#   curl -fsSL https://weftos.weavelogic.ai/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/weave-logic-ai/weftos/master/scripts/install.sh | sh
#
# Installs or updates: weft, weaver, weftos
# Detects platform automatically.
# Idempotent — safe to run multiple times.

set -eu

REPO="weave-logic-ai/weftos"
INSTALL_DIR="${WEFTOS_INSTALL_DIR:-/usr/local/bin}"
BINS="clawft-cli clawft-weave weftos"
BIN_NAMES="weft weaver weftos"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { printf "${CYAN}→${NC} %s\n" "$1"; }
ok()   { printf "${GREEN}✓${NC} %s\n" "$1"; }
warn() { printf "${YELLOW}!${NC} %s\n" "$1"; }
err()  { printf "${RED}✗${NC} %s\n" "$1" >&2; exit 1; }

# Detect platform
detect_triple() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            case "$ARCH" in
                x86_64|amd64)
                    # Check musl vs glibc
                    if ldd --version 2>&1 | grep -qi musl; then
                        echo "x86_64-unknown-linux-musl"
                    else
                        echo "x86_64-unknown-linux-gnu"
                    fi
                    ;;
                aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
                *) err "Unsupported architecture: $ARCH" ;;
            esac
            ;;
        darwin)
            case "$ARCH" in
                x86_64|amd64) echo "x86_64-apple-darwin" ;;
                aarch64|arm64) echo "aarch64-apple-darwin" ;;
                *) err "Unsupported architecture: $ARCH" ;;
            esac
            ;;
        *) err "Unsupported OS: $OS" ;;
    esac
}

# Get latest version from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"v\([^"]*\)".*/\1/'
}

# Get current installed version
get_current_version() {
    if command -v weaver >/dev/null 2>&1; then
        weaver version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1
    elif command -v weft >/dev/null 2>&1; then
        weft --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1
    else
        echo "none"
    fi
}

main() {
    echo ""
    printf "${CYAN}WeftOS Installer${NC}\n"
    echo "════════════════"
    echo ""

    TRIPLE=$(detect_triple)
    info "Platform: $TRIPLE"

    LATEST=$(get_latest_version)
    if [ -z "$LATEST" ]; then
        err "Failed to fetch latest version from GitHub"
    fi
    info "Latest version: v$LATEST"

    CURRENT=$(get_current_version)
    if [ "$CURRENT" = "$LATEST" ]; then
        ok "Already up to date (v$LATEST)"
        echo ""
        return 0
    elif [ "$CURRENT" = "none" ]; then
        info "Fresh install"
    else
        info "Updating: v$CURRENT → v$LATEST"
    fi

    # Stop kernel if running
    if command -v weaver >/dev/null 2>&1; then
        if weaver kernel status >/dev/null 2>&1; then
            info "Stopping running kernel..."
            weaver kernel stop 2>/dev/null || true
            RESTART_KERNEL=1
        fi
    fi

    echo ""

    # Download and install each binary
    set -- clawft-cli clawft-weave weftos
    set_names="weft weaver weftos"
    i=1
    for asset_prefix in "$@"; do
        bin_name=$(echo "$set_names" | cut -d' ' -f$i)
        asset="${asset_prefix}-${TRIPLE}.tar.gz"
        url="https://github.com/$REPO/releases/download/v${LATEST}/${asset}"

        info "Downloading $bin_name..."
        tmpdir=$(mktemp -d)
        if curl -fsSL -o "$tmpdir/$asset" "$url" 2>/dev/null; then
            tar xzf "$tmpdir/$asset" --strip-components=1 -C "$tmpdir"
            if [ -f "$tmpdir/$bin_name" ]; then
                chmod +x "$tmpdir/$bin_name"
                if cp "$tmpdir/$bin_name" "$INSTALL_DIR/$bin_name" 2>/dev/null; then
                    ok "$bin_name installed to $INSTALL_DIR/$bin_name"
                else
                    warn "Permission denied, trying sudo..."
                    sudo cp "$tmpdir/$bin_name" "$INSTALL_DIR/$bin_name"
                    ok "$bin_name installed to $INSTALL_DIR/$bin_name"
                fi
            else
                warn "$bin_name not found in archive, skipping"
            fi
        else
            warn "$asset not available for this platform, skipping"
        fi
        rm -rf "$tmpdir"
        i=$((i + 1))
    done

    echo ""

    # Restart kernel if it was running
    if [ "${RESTART_KERNEL:-0}" = "1" ]; then
        info "Restarting kernel..."
        weaver kernel start 2>/dev/null || true
    fi

    # Verify
    echo ""
    if command -v weaver >/dev/null 2>&1; then
        ok "$(weaver version 2>/dev/null || echo "weaver v$LATEST")"
    fi
    if command -v weft >/dev/null 2>&1; then
        ok "$(weft --version 2>/dev/null || echo "weft v$LATEST")"
    fi

    echo ""
    ok "WeftOS v$LATEST installed successfully"
    echo ""
    echo "  Getting started:"
    echo "    weaver kernel start    # Start the kernel"
    echo "    weft assess init       # Initialize a project"
    echo "    weft assess            # Run an assessment"
    echo ""
    echo "  Update anytime:"
    echo "    weaver update          # or re-run this script"
    echo ""
}

main
