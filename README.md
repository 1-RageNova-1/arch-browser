# Arch Browser

Arch Browser is a Linux desktop browser written in Rust and powered by Chromium Embedded Framework (CEF). Its visual direction is inspired by Arch Linux: black surfaces, lavender accents and compact controls.

## Current state

The application initializes CEF, handles Chromium's child-process relaunch, creates a CEF browser view and opens a real web page. The default page is Google; a startup URL can be supplied with --url=https://example.com/.

This is an early browser build, not a feature-complete replacement for Chrome or Firefox. Tabs and a polished address/navigation toolbar, bookmarks/history UI, downloads UI, private profiles, permission controls and tracker blocking are not yet implemented. The Rust state module already contains bookmark/history persistence, but it is not yet connected to the CEF window.

## Install on Ubuntu

Download arch-browser_0.1.0_amd64.deb from the GitHub Actions artifact and install it with:

    sudo apt install ./arch-browser_0.1.0_amd64.deb

Launch Arch Browser from the applications menu or run arch-browser in a terminal.

The Debian package contains the matching CEF shared library, Chromium resources, locales and sandbox helper. It also installs the application launcher and desktop icon. The package targets Ubuntu 24.04 and newer x86_64 systems; it is not an ARM build.

## Development

Install the native dependencies and a recent stable Rust toolchain, then run:

    cargo test --all-targets
    cargo check --all-targets
    cargo build --release --bin arch-browser

The CI workflow runs these compile checks and builds the .deb. It does not launch a graphical browser process; interactive startup and navigation are left for manual testing on Ubuntu.

CEF is multi-process. The executable calls execute_process before browser-process initialization and runs CEF's message loop. The package ships the matching CEF runtime, data packs and locales beside the executable. The chrome-sandbox helper is installed root-owned with the required setuid mode.

## License

MIT OR Apache-2.0.
