#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${VERSION:-$(awk -F '"' '/^version = / { print $2; exit }' "$ROOT/Cargo.toml")}"
ARCH="$(dpkg --print-architecture)"
BUILD_DIR="$ROOT/target/release"
OUT_DIR="$ROOT/dist"

CEF_DIR="$(
    find "$ROOT/target/release/build" -type f -name output -print0 \
        | xargs -0 -r grep -hE '^cargo::?metadata=CEF_DIR=' \
        | sed -E 's/^cargo::?metadata=CEF_DIR=//' \
        | tail -n 1
)"
if [[ -z "$CEF_DIR" || ! -d "$CEF_DIR" || ! -s "$CEF_DIR/libcef.so" ]]; then
    CEF_FILE="$(find -L "$ROOT/target" -name libcef.so -print -quit)"
    if [[ -n "$CEF_FILE" ]]; then
        CEF_DIR="$(dirname "$CEF_FILE")"
    fi
fi
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
if [[ -z "$CEF_DIR" || ! -s "$CEF_DIR/libcef.so" ]]; then
    echo "Could not locate the CEF runtime selected by cef-dll-sys." >&2
    exit 1
fi

FILTER_SOURCE="$PACKAGE_DIR/filter-source"
mkdir -p "$FILTER_SOURCE"
curl --fail --location --silent --show-error --retry 3 --connect-timeout 20 --max-time 180 \
    https://easylist.to/easylist/easylist.txt -o "$FILTER_SOURCE/easylist.txt"
curl --fail --location --silent --show-error --retry 3 --connect-timeout 20 --max-time 180 \
    https://easylist.to/easylist/easyprivacy.txt -o "$FILTER_SOURCE/easyprivacy.txt"

for list in easylist.txt easyprivacy.txt; do
    if [[ ! -s "$FILTER_SOURCE/$list" ]] || ! grep -q '^! Title:' "$FILTER_SOURCE/$list"; then
        echo "Downloaded filter list $list is empty or has an invalid header." >&2
        exit 1
    fi
done

STAGE="$PACKAGE_DIR/arch-browser"
LIB_DIR="$STAGE/usr/lib/arch-browser"
FILTER_DIR="$STAGE/usr/share/arch-browser/filters"
mkdir -p "$STAGE/DEBIAN" "$LIB_DIR" "$STAGE/usr/bin" \
    "$STAGE/usr/share/applications" \
    "$STAGE/usr/share/icons/hicolor/scalable/apps" \
    "$STAGE/usr/share/doc/arch-browser" "$FILTER_DIR"

install -m 0755 "$BUILD_DIR/arch-browser" "$LIB_DIR/arch-browser"

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

strip --strip-debug "$LIB_DIR/libcef.so"

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

install -m 0644 "$FILTER_SOURCE/easylist.txt" "$FILTER_DIR/easylist.txt"
install -m 0644 "$FILTER_SOURCE/easyprivacy.txt" "$FILTER_DIR/easyprivacy.txt"
chmod 4755 "$LIB_DIR/chrome-sandbox"

cat > "$STAGE/usr/bin/arch-browser" <<'LAUNCHER'
#!/bin/sh
set -eu
APP_DIR="/usr/lib/arch-browser"
export LD_LIBRARY_PATH="$APP_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$APP_DIR/arch-browser" --enable-chrome-runtime "$@"
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

The initial page is the animated Arch Browser start page. Search is available
through Google or DuckDuckGo. Network requests are checked by the bundled
tracker-blocking engine using the installed EasyList and EasyPrivacy snapshots
plus Arch Browser's Google tracker rules.

The tracker filters operate at the network-request level. They do not remove
Chromium/CEF code or promise complete protection against fingerprinting,
first-party telemetry, or every tracking technique.
DOC
gzip -n -9 "$STAGE/usr/share/doc/arch-browser/README.Debian"

cat > "$STAGE/usr/share/doc/arch-browser/FILTER-LISTS.txt" <<'DOC'
Bundled filter lists
====================

EasyList
  https://easylist.to/easylist/easylist.txt

EasyPrivacy
  https://easylist.to/easylist/easyprivacy.txt

The EasyList and EasyPrivacy lists are maintained by their respective community
authors. They are distributed under the terms stated by the EasyList project:
GPL-3.0-or-later or Creative Commons Attribution-ShareAlike 3.0 Unported.
Attribution: The EasyList authors.

Arch Browser includes additional Google advertising and analytics network
rules in its own source tree under src/privacy/google-trackers.txt.
DOC
gzip -n -9 "$STAGE/usr/share/doc/arch-browser/FILTER-LISTS.txt"

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
Description: Privacy-focused Chromium desktop browser powered by CEF and Rust
 Arch Browser opens a Chromium-based browser window with an animated start page
 and network-level ad and tracker filtering.
CONTROL

mkdir -p "$OUT_DIR"
DEB="$OUT_DIR/arch-browser_${VERSION}_amd64.deb"
dpkg-deb --build --root-owner-group "$STAGE" "$DEB" >/dev/null

dpkg-deb --info "$DEB"
dpkg-deb --contents "$DEB" > "$PACKAGE_DIR/deb-contents.txt"
grep -E 'usr/lib/arch-browser/(libcef|chrome-sandbox|locales/)|usr/share/arch-browser/filters/' \
    "$PACKAGE_DIR/deb-contents.txt" | sed -n '1,18p'

echo "Built package: $DEB"
echo "Package size: $(du -h "$DEB" | cut -f1)"
