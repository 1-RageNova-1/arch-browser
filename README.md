# Arch Browser

Arch Browser is a Linux desktop browser project written in Rust and based on Chromium Embedded Framework (CEF). Its visual direction is inspired by Arch Linux: pure black, lavender accents and compact controls.

> **Development status — 26 November 2026:** this repository is under active development. The browser UI and CEF window integration are not yet complete. The current executable is still a placeholder and must not be presented as a usable browser. No release binaries have been validated or published.

## Architecture decision: CEF + Rust

The project uses the maintained `cef` crate from [tauri-apps/cef-rs](https://github.com/tauri-apps/cef-rs), rather than an abandoned binding. The crate provides safe Rust wrappers over CEF's C API and targets Linux x86_64 and ARM64. The matching CEF runtime is acquired by `cef-dll-sys` during the build or supplied through `CEF_PATH`.

CEF is feasible, but materially more complex than WebKitGTK:

- CEF is multi-process. The executable must call `execute_process` before normal application initialization, initialize CEF only in the browser process, run the CEF message loop, and shut it down cleanly.
- A distributable needs the CEF runtime and data files, including `libcef.so`, resource `.pak` files, locale data and other version-specific files. A Rust executable alone is not a portable browser.
- Expect a large download and package footprint (often well above 150 MB, depending on CEF build and compression).
- Linux display support and Wayland behavior depend on the exact CEF build, Chromium flags and window integration. XWayland is a fallback, not proof of native Wayland support.
- CEF normally uses multiple child processes. We will not force a single renderer process: that is unsafe and can break isolation. Process count and background services will instead be kept to the minimum supported by Chromium.

### Feature ownership

| Feature | CEF capability | Arch Browser work |
|---|---|---|
| Web navigation and rendering | Native | Browser window, navigation UI and lifecycle |
| Cookies and cache | Native request-context/storage APIs | Profile management and clear-data UI |
| Downloads | Native download handler callbacks | Download list, destination and cancellation UI |
| Private browsing | Separate in-memory request context | Private profile lifecycle and no-persistence guarantees |
| Permissions | Permission callbacks | Per-origin decisions and persistence |
| Pop-ups | Lifespan/popup callbacks | Policy and tab/window routing |
| History and bookmarks | Not a browser UI feature | Rust persistence layer |
| Tabs and session restore | Browser instances are supported | Tab model, UI and session persistence |
| Tracker blocking | Request interception is available | Not enabled yet; requires maintained filter lists, update policy and tests |

Tracker blocking is explicitly **not implemented yet**. A setting flag is not a blocker; it will only be exposed as functional after request interception and a real rule set are integrated and tested.

## Current implementation

Implemented in this branch:
- Rust state model for bookmarks, history and browser settings.
- JSON state persistence with atomic replacement.
- Explicit errors for malformed JSON and unsupported state versions.
- Bookmark JSON import/export and duplicate handling.
- Automated unit tests for defaults, bookmarks, import/export, history and corrupted state files.
- CI no longer ignores a failed release build.

Not implemented yet:
- CEF initialization and a visible browser window.
- Address/search bar, navigation controls and multi-tab UI.
- Homepage, wallpapers, logo assets and animations.
- Downloads, private contexts, cookie/cache controls, permission UI and pop-up policy.
- Actual tracker filtering.
- Verified .deb, AppImage and portable archive packaging.
- End-to-end tests on a graphical Linux desktop.

## Build prerequisites (Ubuntu/Debian)

Install a recent stable Rust toolchain and the CEF native dependencies:

```bash
sudo apt update
sudo apt install -y build-essential clang pkg-config libgtk-3-dev \
  libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libcups2 \
  libdrm2 libxkbcommon0 libgbm1 libasound2t64 \
  libx11-xcb1 libxcb-dri3-0 libxcomposite1 libxdamage1 \
  libxfixes3 libxrandr2 libxrender1 libxtst6
```

Build and run the current development binary:

```bash
cargo test --all-targets
cargo build --release
./target/release/arch-browser
```

The current binary is a development placeholder; successful compilation does not mean that web browsing is available.

## Repository CI

GitHub Actions runs unit tests, checks all targets and performs a release build on Ubuntu 24.04. Build failures fail the job. A successful CI run proves compilation only; it does not prove GUI startup, external navigation, installation, or runtime CEF resource discovery.

## Release policy

A `v0.1.0` release must not be created until a real CEF browser window can load external HTTPS pages and the .deb, AppImage and portable archive have been built and smoke-tested. Never download or attach an unverified binary as a release asset.

## License

MIT OR Apache-2.0.
