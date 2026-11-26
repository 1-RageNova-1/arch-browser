#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${VERSION:-$(awk -F '"' '/^version = / { print $2; exit }' "$ROOT/Cargo.toml")}"
ARCH="$(dpkg --print-architecture)"
BUILD_DIR="$ROOT/target/release"
CEF_DIR="$BUILD_DIR"
OUT_DIR="$ROOT/dist"
PACKAGE_DIR="$(mktemp -d)"
trap 'rm -rf "$PACKAGE_DIR"' EXIT

if [[ "$ARCH" != "amd64" ]]; then
    echo "This package script currently supports amd64 only (found: $ARCH)." >&2
    exit 1
fi

if [[ ! -x "$BUILD_DIR/arch-browser" ]]; then
    echo "Release executable missing: $BUILD_DIR/arch-browser" >&2
    exit 1
fi
if [[ ! -s "$CEF_DIR/libcef.so" ]]; then
    echo "CEF runtime was not copied beside the release executable." >&2
    exit 1
fi

STAGE="$PACKAGE_DIR/arch-browser"
LIB_DIR="$STAGE/usr/lib/arch-browser"
mkdir -p "$STAGE/DEBIAN" "$LIB_DIR" "$STAGE/usr/bin" \
    "$STAGE/usr/share/applications" \
    "$STAGE/usr/share/icons/hicolor/scalable/apps" \
    "$STAGE/usr/share/doc/arch-browser"

install -m 0755 "$BUILD_DIR/arch-browser" "$LIB_DIR/arch-browser"

# Bundle the matching CEF runtime, but not headers, CMake files or build tools.
shopt -s nullglob
for pattern in '*.so' '*.so.*' '*.dat' '*.bin' '*.json' '*.pak' chrome-sandbox; do
    for file in "$CEF_DIR"/$pattern; do
        [[ -e "$file" ]] || continue
        cp -a "$file" "$LIB_DIR/"
    done
done
for directory in locales resources Resources swiftshader; do
    if [[ -d "$CEF_DIR/$directory" ]]; then
        cp -a "$CEF_DIR/$directory" "$LIB_DIR/"
    fi
done

for required in libcef.so icudtl.dat chrome-sandbox; do
    if [[ ! -s "$LIB_DIR/$required" ]]; then
        echo "Required CEF runtime file is missing: $required" >&2
        exit 1
    fi
done
if [[ ! -d "$LIB_DIR/locales" ]]; then
    echo "CEF locales directory is missing." >&2
    exit 1
fi
if ! find "$LIB_DIR" -type f -name '*.pak' -print -quit | grep -q .; then
    echo "CEF resource .pak files are missing." >&2
    exit 1
fi

# Chromium's Linux sandbox helper must be root-owned and setuid in the package.
chmod 4755 "$LIB_DIR/chrome-sandbox"

cat > "$STAGE/usr/bin/arch-browser" <<'LAUNCHER'
#!/bin/sh
set -eu
APP_DIR="/usr/lib/arch-browser"
export LD_LIBRARY_PATH="$APP_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$APP_DIR/arch-browser" "$@"
LAUNCHER
chmod 0755 "$STAGE/usr/bin/arch-browser"

install -m 0644 "$ROOT/packaging/arch-browser.desktop" \
    "$STAGE/usr/share/applications/arch-browser.desktop"
install -m 0644 "$ROOT/packaging/arch-browser.svg" \
    "$STAGE/usr/share/icons/hicolor/scalable/apps/arch-browser.svg"

cat > "$STAGE/usr/share/doc/arch-browser/README.Debian" <<'DOC'
Arch Browser
============

Launch from the application menu or run: arch-browser

The initial page defaults to https://www.google.com/.
A startup URL can be supplied with: arch-browser --url=https://example.com/

This package includes the matching CEF runtime and locale resources.
DOC
gzip -n -9 "$STAGE/usr/share/doc/arch-browser/README.Debian"

for cef_doc in CREDITS.html LICENSE.txt LICENSE; do
    if [[ -f "$CEF_DIR/$cef_doc" ]]; then
        install -m 0644 "$CEF_DIR/$cef_doc" "$STAGE/usr/share/doc/arch-browser/cef-$cef_doc"
    fi
done

cat > "$STAGE/DEBIAN/control" <<CONTROL
Package: arch-browser
Version: $VERSION
Section: web
Priority: optional
Architecture: amd64
Maintainer: Arch Browser Contributors
Depends: libc6 (>= 2.39), libstdc++6, libgcc-s1, libgtk-3-0t64 | libgtk-3-0, libnss3, libnspr4, libatk1.0-0t64 | libatk1.0-0, libatk-bridge2.0-0t64 | libatk-bridge2.0-0, libcups2t64 | libcups2, libdrm2, libxkbcommon0, libgbm1, libasound2t64 | libasound2, libglib2.0-0t64 | libglib2.0-0, libdbus-1-3, libx11-6, libx11-xcb1, libxcb1, libxcb-dri3-0, libxcomposite1, libxdamage1, libxext6, libxfixes3, libxrandr2, libxrender1, libxtst6, libxi6, libxinerama1, libpango-1.0-0, libpangocairo-1.0-0, libfontconfig1, libfreetype6, libwayland-client0, libwayland-egl1, libwayland-cursor0
Description: Minimalist Chromium desktop browser powered by CEF and Rust
 Arch Browser opens a Chromium-based browser window using the bundled CEF
 runtime. This initial build provides a usable browsing window; further browser
 features remain under development.
CONTROL

mkdir -p "$OUT_DIR"
DEB="$OUT_DIR/arch-browser_${VERSION}_amd64.deb"
dpkg-deb --build --root-owner-group "$STAGE" "$DEB" >/dev/null

dpkg-deb --info "$DEB"
dpkg-deb --contents "$DEB" > "$PACKAGE_DIR/deb-contents.txt"
grep -E 'usr/lib/arch-browser/(libcef|chrome-sandbox|locales/)' "$PACKAGE_DIR/deb-contents.txt" | sed -n '1,12p'

echo "Built package: $DEB"
echo "Package size: $(du -h "$DEB" | cut -f1)"
