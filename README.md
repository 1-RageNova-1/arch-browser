# Arch Browser

Arch Browser is a Rust desktop browser powered by Chromium Embedded Framework (CEF). Its visual direction is inspired by Arch Linux: deep black surfaces, lavender accents and a compact animated start page.

## Privacy protection

Arch Browser enables the CEF Chrome runtime and configures the official **uBlock Origin Lite** extension for automatic installation at first launch. It is the Manifest V3 version maintained by Raymond Hill. Its default rules include uBlock's built-in lists, EasyList, EasyPrivacy and Peter Lowe's ad/tracking server list. The extension is force-installed through a per-user Chromium policy that points to the official Chrome Web Store update service.

The existing [Brave adblock-rust](https://github.com/brave/adblock-rust) network filter remains as an additional request-level layer, using EasyList, EasyPrivacy and Arch Browser's extra Google tracker rules. This fallback does not replace uBlock Origin Lite and does not guarantee removal of every first-party telemetry request or provide complete fingerprinting protection.

## Browser extensions

- **uBlock Origin Lite** is configured to install automatically and remain managed by the browser.
- **1Password**, **Bitwarden**, and **Proton VPN** are optional. The start page links directly to their official Chrome Web Store listings; users choose which ones to install and confirm installation in the store.
- Proton VPN's browser extension protects browser traffic only, not the rest of the computer's applications.
- Extension policy and browser profile data are stored under the user's XDG configuration and data directories. No system-wide policy file is written.

The full Helium fork of uBlock Origin is not copied into this CEF build. Helium vendors and patches a Manifest V2 uBlock fork inside its own Chromium source tree, including component-loader and resource-build changes. Those patches cannot be applied as a drop-in extension to an unmodified CEF build. Arch Browser uses the official MV3 uBlock Origin Lite project instead; it is related, but it is not feature-identical to full uBlock Origin.

CEF's Chrome runtime is marked experimental upstream. CI compiles and packages the application but does not launch a graphical browser; extension installation, browser chrome and site compatibility still require a manual Ubuntu test.

## Start page

The first page now uses an Arch-themed animated start page with a search field. Google remains available as a search option, and DuckDuckGo is also provided. The animation respects the operating system's reduced-motion preference.

## Current state

Implemented:
- CEF browser window and Chromium page rendering.
- Animated start page with Google and DuckDuckGo search.
- Network-level filtering through adblock-rust.
- EasyList, EasyPrivacy and extra Google tracker filters.
- Persistent Rust state model for bookmarks/history (the browser UI for these is not yet connected).

Still under development:
- Browser tab strip and full address/navigation toolbar.
- Bookmark and history panels.
- Downloads UI and private profiles.
- Permission controls and a polished settings panel.
- Cosmetic filter injection and scriptlet-based protections.

## Install on Ubuntu

Download `arch-browser_0.1.2_amd64.deb` and install it with:

    sudo apt install ./arch-browser_0.1.2_amd64.deb

Launch Arch Browser from the applications menu or run `arch-browser` in a terminal.

The Debian package contains the matching CEF runtime, Chromium resources, locales, sandbox helper, and EasyList/EasyPrivacy snapshots. See [extension integration](docs/EXTENSIONS.md) and [release signing](docs/RELEASE_SIGNING.md) for details. It also installs the application launcher and desktop icon. The package targets Ubuntu 24.04 and newer x86_64 systems; it is not an ARM build.

## Development

Install the native dependencies and a recent stable Rust toolchain, then run:

    cargo test --all-targets
    cargo check --all-targets
    cargo build --release --bin arch-browser
    packaging/build-deb.sh

The CI workflow runs those tests and build steps and creates the .deb. It deliberately does not launch a graphical browser process; interactive startup and navigation are tested manually on Ubuntu.

CEF is multi-process. The executable calls `execute_process` before browser-process initialization and runs CEF's message loop. The package ships the matching CEF runtime and data packs beside the executable.

## License

MIT OR Apache-2.0. Bundled CEF and filter lists retain their respective upstream licenses; see `/usr/share/doc/arch-browser/` after installation.
