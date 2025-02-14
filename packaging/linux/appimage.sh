#!/usr/bin/env bash
# AppImage pentru Linux x86_64: dist/release/cangurul-<versiune>-linux-x86_64.AppImage
# Un singur fișier: se face executabil (chmod +x) și se pornește, fără instalare.
# appimagetool se descarcă în .cache/ la prima rulare.
set -euo pipefail
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

TOOL_URL=https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
tool=$ROOT/.cache/appimagetool-x86_64.AppImage
if [ ! -x "$tool" ]; then
    mkdir -p "$ROOT/.cache"
    curl -fsSL -o "$tool.part" "$TOOL_URL"
    chmod +x "$tool.part"
    mv "$tool.part" "$tool"
fi

cargo build --release --locked

name=$APP-$VERSION-linux-x86_64
appdir=$ROOT/dist/stage/Cangurul.AppDir
rm -rf "$appdir" "$OUT/$name.AppImage"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" \
    "$appdir/usr/share/icons/hicolor/256x256/apps" "$appdir/usr/share/doc/cangurul" "$OUT"
install -m 755 target/release/cangurul "$appdir/usr/bin/cangurul"
cp packaging/linux/cangurul.desktop "$appdir/usr/share/applications/"
cp assets/logo.png "$appdir/usr/share/icons/hicolor/256x256/apps/cangurul.png"
mkdir -p "$appdir/usr/share/mime/packages"
cp packaging/linux/cangurul-mime.xml "$appdir/usr/share/mime/packages/cangurul.xml"
cp README.md "$appdir/usr/share/doc/cangurul/"
cp LICENSE "$appdir/usr/share/doc/cangurul/LICENSE.txt"
{ cat assets/license-header.txt; cat assets/fonts/LICENSE.txt; } >"$appdir/usr/share/doc/cangurul/LICENTE.txt"
# appimagetool cere fișierul .desktop și iconița și în rădăcina AppDir.
ln -s usr/share/applications/cangurul.desktop "$appdir/cangurul.desktop"
ln -s usr/share/icons/hicolor/256x256/apps/cangurul.png "$appdir/cangurul.png"
ln -s cangurul.png "$appdir/.DirIcon"
ln -s usr/bin/cangurul "$appdir/AppRun"

# Fără FUSE (containere, CI): appimagetool se dezarhivează și rulează din dosar.
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION=$VERSION \
    "$tool" --no-appstream "$appdir" "$OUT/$name.AppImage"
echo "Gata: $OUT/$name.AppImage"
