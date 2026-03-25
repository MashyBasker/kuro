#!/usr/bin/env bash
set -e

BINARY="kuro"
INSTALL_DIR="$HOME/.local/bin"

echo "Building release binary..."
cargo build --release

mkdir -p "$INSTALL_DIR"
cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"

echo "Installed $BINARY to $INSTALL_DIR/$BINARY"
