#!/usr/bin/env bash
# Pachetul .deb pentru Ubuntu / Debian / Mint: dist/release/cangurul-<versiune>-linux-x86_64.deb
# Se instalează cu `sudo apt install ./cangurul-*.deb`; apare în meniu, iar
# fișierele .cng se deschid cu dublu-click. Cere dpkg-deb (pachetul dpkg).
set -euo pipefail
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

cargo build --release --locked

name=$APP-$VERSION-linux-x86_64
root=$ROOT/dist/stage/deb
rm -rf "$root" "$OUT/$name.deb"
mkdir -p "$root/DEBIAN" "$root/usr/bin" "$root/usr/share/applications" \
    "$root/usr/share/icons/hicolor/256x256/apps" "$root/usr/share/mime/packages" \
    "$root/usr/share/doc/cangurul/exemple" "$OUT"
install -m 755 target/release/cangurul "$root/usr/bin/cangurul"
install -m 644 packaging/linux/cangurul.desktop "$root/usr/share/applications/cangurul.desktop"
install -m 644 assets/logo.png "$root/usr/share/icons/hicolor/256x256/apps/cangurul.png"
install -m 644 packaging/linux/cangurul-mime.xml "$root/usr/share/mime/packages/cangurul.xml"
install -m 644 examples/*.cng "$root/usr/share/doc/cangurul/exemple/"
install -m 644 README.md "$root/usr/share/doc/cangurul/README.md"
copy_docs "$root/usr/share/doc/cangurul"
# Formatul DEP-5. Notițele de copyright se citesc din NOTICE și din licențele fonturilor (prima linie),
# ca să nu rămână în urmă; fonturile egui și crate-urile sunt în THIRD-PARTY.txt.
font_copyright() { sed -n '1s/^Copyright //p' "assets/fonts/OFL-$1.txt"; }
{
    echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: cangurul"
    echo "Source: https://github.com/nazdravan/cangurul"
    echo "Comment: The Rust crates and egui's fonts built into usr/bin/cangurul, with the full text"
    echo " of each licence, are listed in /usr/share/doc/cangurul/THIRD-PARTY.txt."
    echo
    echo "Files: *"
    echo "Copyright: $(sed -n 's/^Copyright (C) //p' NOTICE | head -1)"
    echo "License: GPL-3.0-or-later"
    echo
    echo "Files: assets/fonts/NotoSans-*"
    echo "Copyright: $(font_copyright NotoSans)"
    echo "License: OFL-1.1"
    echo
    echo "Files: assets/fonts/FiraCode-*"
    echo "Copyright: $(font_copyright FiraCode)"
    echo "License: OFL-1.1"
    echo
    echo "License: GPL-3.0-or-later"
    echo " On Debian systems, the full text is in /usr/share/common-licenses/GPL-3."
    echo
    echo "License: OFL-1.1"
    sed '1,2d; s/^/ /; s/^ $/ ./' assets/fonts/OFL-NotoSans.txt
} >"$root/usr/share/doc/cangurul/copyright"

# Versiunile Debian folosesc ~ pentru pre-release-uri: 3.1.0-rc.1 -> 3.1.0~rc.1.
debver=${VERSION/-/\~}
size=$(du -sk "$root/usr" | cut -f1)
cat >"$root/DEBIAN/control" <<CONTROL
Package: cangurul
Version: $debver
Architecture: amd64
Maintainer: nazdravan <nazdravan@users.noreply.github.com>
Installed-Size: $size
Depends: libc6 (>= 2.35), libgl1, libxkbcommon0
Recommends: libxkbcommon-x11-0, libwayland-client0
Section: education
Priority: optional
Homepage: https://github.com/nazdravan/cangurul
Description: mediu de învățare a algoritmilor pentru elevi
 Elevul scrie un program cu comenzi simple (PAS, SALT, ROTIRE, REPETĂ,
 CÎT, DACĂ, proceduri), iar cangurul îl execută pe un câmp și desenează
 linii. Rescris în Rust după „Cangurul” versiunea 2; deschide fișierele
 .cng vechi. Interfața în română, rusă și engleză.
CONTROL

dpkg-deb --root-owner-group -Zxz --build "$root" "$OUT/$name.deb"
echo "Gata: $OUT/$name.deb"
