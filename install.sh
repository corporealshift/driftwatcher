#!/bin/bash
set -e

REPO="corporealshift/driftwatcher"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="drifty"

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)  OS="linux" ;;
    Darwin*) OS="darwin" ;;
    *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)  ARCH="x86_64" ;;
    amd64)   ARCH="x86_64" ;;
    arm64)   ARCH="aarch64" ;;
    aarch64) ARCH="aarch64" ;;
    *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Construct download URL
PLATFORM="${OS}-${ARCH}"
LATEST_URL="https://github.com/${REPO}/releases/latest/download/drifty-${PLATFORM}.tar.gz"

echo "Installing drifty for ${PLATFORM}..."

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download and extract
echo "Downloading from ${LATEST_URL}..."
curl -fsSL "$LATEST_URL" -o "$TMP_DIR/drifty.tar.gz"
tar -xzf "$TMP_DIR/drifty.tar.gz" -C "$TMP_DIR"

# Install binary
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/drifty" "$INSTALL_DIR/$BINARY_NAME"
else
    echo "Installing to $INSTALL_DIR (requires sudo)..."
    sudo mv "$TMP_DIR/drifty" "$INSTALL_DIR/$BINARY_NAME"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "Successfully installed drifty to $INSTALL_DIR/$BINARY_NAME"
echo "Run 'drifty help' to get started."
