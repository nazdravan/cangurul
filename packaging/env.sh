# shellcheck shell=bash
# Variabile comune scripturilor de împachetare (se includ cu `. packaging/env.sh`).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP=cangurul
# Singurul loc al versiunii: [package] version din Cargo.toml.
VERSION="$(sed -n '/^\[package\]/,/^\[/{s/^version = "\(.*\)"/\1/p;}' "$ROOT/Cargo.toml" | head -1)"
OUT="$ROOT/dist/release"
export ROOT APP VERSION OUT

# Versiunea cargo-about cu care packaging/licente.sh generează dist/THIRD-PARTY.txt (și în CI).
export CARGO_ABOUT_VERSION=0.9.3

# copy_docs DEST [txt]: licențele care intră în fiecare pachet, în dosarul DEST. Toate scripturile
# de împachetare le copiază doar prin funcția asta (packaging/windows/package.ps1 are geamănul ei,
# Copy-Docs). Cu `txt`, LICENSE și NOTICE primesc extensia .txt (Windows, macOS).
# Mesa (licenses/mesa) e doar în pachetul pentru Windows 7, care o copiază singur.
copy_docs() {
    local dest=$1 ext=${2:+.txt}
    if [ ! -f "$ROOT/dist/THIRD-PARTY.txt" ]; then
        echo "Lipsește dist/THIRD-PARTY.txt: rulează packaging/licente.sh" >&2
        return 1
    fi
    mkdir -p "$dest"
    cp "$ROOT/LICENSE" "$dest/LICENSE$ext"
    cp "$ROOT/NOTICE" "$dest/NOTICE$ext"
    cp "$ROOT/dist/THIRD-PARTY.txt" "$ROOT"/assets/fonts/OFL-*.txt "$dest/"
}
