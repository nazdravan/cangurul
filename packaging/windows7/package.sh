#!/usr/bin/env bash
# Construiește Cangurul pentru Windows 7 / 8 / 10 / 11 pe 32 de biți (merge și pe 64 de biți)
# dintr-un calculator cu Linux.
# Rezultatul: dist/release/cangurul-<versiune>-windows7-x86-portable.zip
#
# Necesită: curl, tar cu zstd, 7z, gcc, compilatorul MinGW pentru 32 de biți
# (Arch: pacman -S mingw-w64-gcc; Debian/Ubuntu: apt install gcc-mingw-w64-i686).
# Rust nightly este instalat automat în .cache/, fără a atinge Rust-ul din sistem.
set -euo pipefail

. "$(dirname "$0")/../env.sh"
cd "$ROOT"
CACHE=$ROOT/.cache/win7
NAME=$APP-$VERSION-windows7-x86-portable
DIST=$ROOT/dist/stage/$NAME
TARGET=i686-win7-windows-gnu
HERE=$ROOT/packaging/windows7

# Versiuni fixate, ca build-ul să fie reproductibil. Ținta win7 este „tier 3”:
# un nightly nou poate strica build-ul, de aceea se schimbă doar după un test.
NIGHTLY=nightly-2026-10-19
MSYS=https://repo.msys2.org/mingw/mingw32
MSYS_PKGS=(
    mingw-w64-i686-crt-git-13.0.0.r380.gb83511db8-1-any.pkg.tar.zst
    mingw-w64-i686-gcc-16.2.0-6-any.pkg.tar.zst
    mingw-w64-i686-winpthreads-git-12.0.0.r747.g1a99f8514-1-any.pkg.tar.zst
)
MESA_VERSION=26.2.4
MESA_URL=https://github.com/pal1000/mesa-dist-win/releases/download/$MESA_VERSION/mesa3d-$MESA_VERSION-release-msvc.7z

command -v i686-w64-mingw32-gcc >/dev/null || {
    echo "Lipsește i686-w64-mingw32-gcc (pachetul mingw-w64-gcc / gcc-mingw-w64-i686)." >&2
    exit 1
}
mkdir -p "$CACHE"

# 1. Rust nightly cu sursele bibliotecii standard (ținta win7 cere -Z build-std).
if ! command -v rustup >/dev/null; then
    export CARGO_HOME=$CACHE/cargo RUSTUP_HOME=$CACHE/rustup
    if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
        curl -sSf -o "$CACHE/rustup-init" https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init
        chmod +x "$CACHE/rustup-init"
        "$CACHE/rustup-init" -y --no-modify-path --profile minimal --default-toolchain none
    fi
    export PATH=$CARGO_HOME/bin:$PATH
fi
rustup toolchain install $NIGHTLY --profile minimal -c rust-src

# 2. Bibliotecile MinGW legate de msvcrt.dll (MinGW-ul din distribuții folosește UCRT,
#    care lipsește pe Windows 7 fără actualizări).
MSVCRT=$CACHE/msys32
if [ ! -d "$MSVCRT/mingw32/lib" ]; then
    mkdir -p "$MSVCRT"
    for p in "${MSYS_PKGS[@]}"; do
        curl -sSfL -o "$MSVCRT/$p" "$MSYS/$p"
        tar --zstd -xf "$MSVCRT/$p" -C "$MSVCRT" mingw32/lib
    done
fi
LIB=$MSVCRT/mingw32/lib
GCC_LIB=$(echo "$LIB"/gcc/i686-w64-mingw32/*)

# 3. Programul (fără accesskit, care cere Windows 8+).
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-B$LIB -C link-arg=-B$GCC_LIB -C link-arg=-L$LIB -C link-arg=-L$GCC_LIB"
cargo +$NIGHTLY build --release --locked -Z build-std --target $TARGET --no-default-features

# 4. Mesa (OpenGL software) pentru calculatoarele fără driver video.
if [ ! -f "$CACHE/mesa/x86/libgallium_wgl.dll" ]; then
    curl -sSfL -o "$CACHE/mesa.7z" "$MESA_URL"
    7z e -y -o"$CACHE/mesa/x86" "$CACHE/mesa.7z" x86/opengl32.dll x86/libgallium_wgl.dll >/dev/null
fi

# 5. Dosarul de distribuit.
rm -rf "$DIST" "$OUT/$NAME.zip"
mkdir -p "$DIST/soft-gl" "$DIST/Exemple"
cp "target/$TARGET/release/cangurul.exe" "$DIST/Cangurul.exe"
i686-w64-mingw32-gcc -shared -nostdlib -Os -s -o "$DIST/combase.dll" \
    "$HERE/combase/dllmain.c" "$HERE/combase/combase.def" -Wl,-e,_DllMainCRTStartup@12
cp "$CACHE"/mesa/x86/*.dll "$DIST/soft-gl/"
cp examples/*.cng "$DIST/Exemple/"
# Notepad-ul din Windows 7 recunoaște UTF-8 doar cu BOM și vrea CRLF.
{ printf '\xEF\xBB\xBF'; sed 's/$/\r/' "$HERE/CITESTE.txt"; } > "$DIST/CITESTE.txt"
{
    cat assets/license-header.txt assets/fonts/LICENSE.txt
    echo "soft-gl: Mesa $MESA_VERSION (https://mesa3d.org), licență MIT, build mesa-dist-win (https://github.com/pal1000/mesa-dist-win)."
} | sed 's/$/\r/' > "$DIST/LICENTE.txt"
sed 's/$/\r/' LICENSE > "$DIST/LICENSE.txt"
mkdir -p "$OUT"
(cd "$DIST/.." && 7z a -tzip -mx9 "$OUT/$NAME.zip" "$NAME" >/dev/null)

echo "Gata: $OUT/$NAME.zip"
