# Arch Browser

Arch Browser is a Rust desktop browser powered by Chromium Embedded Framework (CEF). Its visual direction is inspired by Arch Linux: deep black surfaces, lavender accents and a compact animated start page.

## Privacy protection

Network requests are checked by [Brave's adblock-rust engine](https://github.com/brave/adblock-rust), using the EasyList and EasyPrivacy filter lists plus Arch Browser's dedicated Google advertising and analytics rules. Known third-party ads and trackers are cancelled before they are fetched. The bundled filter lists are refreshed when a new Debian package is built.

The protection works at the network-request level. It blocks many known Google trackers and common third-party trackers, but it does not remove Chromium/CEF source code, guarantee removal of every first-party Google telemetry request, or provide complete fingerprinting protection. Cosmetic element hiding and scriptlet injection are not yet integrated.

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

Download `arch-browser_0.1.1_amd64.deb` and install it with:

    sudo apt install ./arch-browser_0.1.1_amd64.deb

Launch Arch Browser from the applications menu or run `arch-browser` in a terminal.

The Debian package contains the matching CEF runtime, Chromium resources, locales, sandbox helper, and EasyList/EasyPrivacy snapshots. It also installs the application launcher and desktop icon. The package targets Ubuntu 24.04 and newer x86_64 systems; it is not an ARM build.

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
