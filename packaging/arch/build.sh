#!/usr/bin/env bash
# Build Arch package for Wangsap from the project tree.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARCH_DIR="$ROOT/packaging/arch"
export PATH="${HOME}/.cargo/bin:${PATH}"

cd "$ARCH_DIR"
# Clean previous artifacts
rm -rf pkg src *.pkg.tar* *.log 2>/dev/null || true

echo "==> makepkg wangsap"
# --holdver: no VCS; -f force rebuild; no root needed for package file
makepkg -f --noconfirm 2>&1

echo
echo "==> packages:"
ls -lh "$ARCH_DIR"/*.pkg.tar.* 2>/dev/null || ls -lh "$ARCH_DIR"/*.pkg.tar.zst 2>/dev/null

echo
echo "Install with:"
echo "  sudo pacman -U $ARCH_DIR/wangsap-*.pkg.tar.zst"
