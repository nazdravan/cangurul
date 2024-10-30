# shellcheck shell=bash
# Variabile comune scripturilor de împachetare (se includ cu `. packaging/env.sh`).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP=cangurul
# Singurul loc al versiunii: [package] version din Cargo.toml.
VERSION="$(sed -n '/^\[package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' "$ROOT/Cargo.toml" | head -1)"
OUT="$ROOT/dist/release"
export ROOT APP VERSION OUT
