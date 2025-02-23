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
{
    echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: cangurul"
    echo "Source: https://github.com/nazdravan/cangurul"
    echo
    echo "Files: *"
    echo "License: GPL-3.0-or-later"
    echo " On Debian systems, the full text is in /usr/share/common-licenses/GPL-3."
    echo
    echo "Files: fonts"
    echo "License: OFL-1.1"
    sed 's/^/ /; s/^ $/ ./' assets/fonts/LICENSE.txt
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
