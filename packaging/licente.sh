#!/usr/bin/env bash
# dist/THIRD-PARTY.txt: licențele tuturor crate-urilor din Cangurul, cu textul fiecărei licențe,
# inclusiv fonturile egui. Îl cer toate scripturile de împachetare (copy_docs).
# Cere cargo-about: cargo install cargo-about --locked --features cli --version <CARGO_ABOUT_VERSION din env.sh>
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$ROOT"

if ! have=$(cargo about --version 2>/dev/null) || [ "$have" != "cargo-about $CARGO_ABOUT_VERSION" ]; then
    echo "Trebuie cargo-about $CARGO_ABOUT_VERSION (acum: ${have:-lipsește}):" >&2
    echo "  cargo install cargo-about --locked --features cli --version $CARGO_ABOUT_VERSION" >&2
    exit 1
fi
mkdir -p dist
# --fail: un crate cu o licență necunoscută sau neacceptată (about.toml) oprește generarea.
cargo about generate --locked --fail -c about.toml packaging/about.hbs -o dist/THIRD-PARTY.txt

# Fonturile egui vin din [epaint_default_fonts.clarify] (about.toml); fără ele textul lor lipsește.
for text in "UBUNTU FONT LICENCE" "BITSTREAM VERA LICENSE" "Copyright (c) 2014 John Slegers" \
    "SIL OPEN FONT LICENSE"; do
    if ! grep -qF "$text" dist/THIRD-PARTY.txt; then
        echo "dist/THIRD-PARTY.txt nu conține '$text': verifică [epaint_default_fonts.clarify] din about.toml" >&2
        exit 1
    fi
done
echo "Gata: dist/THIRD-PARTY.txt ($(grep -c '^  - ' dist/THIRD-PARTY.txt) crate-uri)"
