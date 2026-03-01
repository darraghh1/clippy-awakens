#!/usr/bin/env bash
# Build script for Clippy Awakens
# Prepares the frontend directory and runs cargo tauri build.
#
# Usage:  ./build.sh          (production build)
#         ./build.sh --dev    (dev mode)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Preparing frontend ==="

# On Windows, git clones symlinks as plain text files.
# Replace ui/build and ui/agents with actual file copies
# so Tauri's frontend bundler includes them.

# Clean stale symlink text files or old copies
rm -rf ui/build ui/agents

# Copy the real directories into ui/
cp -r build ui/build
cp -r agents ui/agents

echo "  Copied build/ -> ui/build/ ($(ls ui/build/ | wc -l) files)"
echo "  Copied agents/ -> ui/agents/ ($(ls ui/agents/ | wc -l) agents)"

# Run tests first
echo ""
echo "=== Running tests ==="
cd src-tauri
cargo test
cd ..

if [[ "${1:-}" == "--dev" ]]; then
    echo ""
    echo "=== Starting dev mode ==="
    RUST_LOG=debug cargo tauri dev
else
    echo ""
    echo "=== Building production installer ==="
    cargo tauri build

    echo ""
    echo "=== Build complete ==="
    echo "Installers:"
    ls -lh src-tauri/target/release/bundle/nsis/*.exe 2>/dev/null || true
    ls -lh src-tauri/target/release/bundle/msi/*.msi 2>/dev/null || true
fi
