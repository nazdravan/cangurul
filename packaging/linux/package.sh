#!/usr/bin/env bash
# Arhiva pentru Linux x86_64: dist/release/cangurul-<versiune>-linux-x86_64.tar.gz
set -euo pipefail
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

cargo build --release --locked

name=$APP-$VERSION-linux-x86_64
stage=$ROOT/dist/stage/$name
rm -rf "$stage" "$OUT/$name.tar.gz"
mkdir -p "$stage/exemple" "$OUT"
install -m 755 target/release/cangurul "$stage/cangurul"
cp examples/*.cng "$stage/exemple/"
cp README.md "$stage/"
{ cat assets/license-header.txt; cat assets/fonts/LICENSE.txt; } > "$stage/LICENTE.txt"
cp LICENSE "$stage/LICENSE.txt"
tar -C "$ROOT/dist/stage" -czf "$OUT/$name.tar.gz" "$name"
echo "Gata: $OUT/$name.tar.gz"
