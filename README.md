# Arch Browser

**Minimalist Chromium-based desktop browser for Linux**, inspired by Arch Linux aesthetics (deep black + lavender).

> **Status (honest)**  
> This is an **early experimental skeleton**.  
> It uses the maintained Rust CEF bindings (`cef` crate from tauri-apps/cef-rs).  
> It is **not** yet a full-featured browser. Many requested features (tabs UI, bookmarks, history persistence, custom new-tab page, animations, packaging, etc.) are still to be implemented.  
> No binary has been fully tested as a production browser in this repository yet.

Repository: https://github.com/1-RageNova-1/arch-browser

## Architecture choice

- **Engine**: Chromium Embedded Framework (CEF)
- **Language**: Rust
- **Bindings**: Official maintained crate [`cef`](https://crates.io/crates/cef) (tauri-apps/cef-rs)
- **Why this stack**: CEF provides a real Chromium browser process model. The `cef` crate is actively maintained and supports Linux x86_64 / ARM64.

### Known major constraints

| Constraint | Impact |
|------------|--------|
| Binary size | Final package usually > 150 MB (libcef + resources) |
| Multi-process | The same binary must handle browser / renderer / GPU / utility processes |
| Memory | Higher idle memory than pure WebKitGTK solutions |
| Wayland | Support depends on CEF version and flags |
| Packaging | Requires shipping `libcef.so`, `.pak` files, locales, snapshots |

CEF covers natively: navigation, cookies, cache, downloads, request interception (useful for tracker blocking), permissions, pop-up control.  
UI chrome, tabs management, bookmarks, history, new-tab page, settings and packaging must be built on top.

## Design (target)

- Background: `#000000`
- Accent: `#A78BFA` (lavender)
- Text: `#E6DDF5`
- Minimal, elegant, low chrome
- Typography: JetBrains Mono for technical elements

## Current state of the code

- Cargo project using the `cef` crate
- Basic entry point following the CEF multi-process model
- Placeholder for future browser window / client handlers
- GitHub Actions workflow that attempts Linux builds

**What is NOT implemented yet** (explicit list):

- Real multi-tab UI
- Address bar + search integration
- Bookmarks / history persistence
- Downloads manager
- Private browsing mode UI
- Custom new-tab / home page
- Animations
- Tracker blocking rules
- `.deb` / AppImage / portable packaging that has been verified
- Automated tests covering the full feature set

## Building (local)

You need a machine with more RAM than a minimal container (CEF is heavy).

```bash
# 1. Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Clone
git clone https://github.com/1-RageNova-1/arch-browser.git
cd arch-browser

# 3. Export CEF binaries (recommended)
cargo install export-cef-dir
export-cef-dir --force $HOME/.local/share/cef
export CEF_PATH="$HOME/.local/share/cef"
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$CEF_PATH"

# 4. Build
cargo build --release

# 5. Run (after successful build)
./target/release/arch-browser
```

Alternative using the helper from cef-rs:

```bash
cargo run --bin bundle-cef-app -- . -o target/bundle --release
```

## GitHub Actions

A workflow is present under `.github/workflows/`.  
It tries to build on `ubuntu-latest`.  
Success of the workflow does **not** automatically mean the resulting binary is a complete, tested browser. Always verify locally.

## License

MIT OR Apache-2.0 (same as the upstream `cef` crate where applicable).

## Contributing / Next steps

Priority order recommended:

1. Make a minimal window that loads a real external URL
2. Implement basic navigation handlers (load start/end, title, URL changes)
3. Add a simple native or CEF-based chrome (address bar + back/forward/reload)
4. Multi-tab support
5. Persistence layer (bookmarks, history)
6. Packaging scripts (.deb, AppImage)
7. Tests and polish

Pull requests welcome. Please keep the README honest about what works and what does not.
