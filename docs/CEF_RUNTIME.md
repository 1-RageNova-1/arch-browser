# CEF runtime and Debian packaging

Arch Browser starts CEF through its multi-process entry point, initializes the browser process, creates a CEF browser view and loads an initial URL. The default is Google; pass --url=https://example.com/ to choose another page at startup.

The GitHub Actions workflow intentionally performs compile-time validation only: Rust tests, cargo check, release build and Debian package inspection. It does not attempt to open a graphical window in CI. Interactive startup and navigation are left for manual testing on Ubuntu.

The .deb installs the app under /usr/lib/arch-browser, includes the matching CEF runtime libraries, resource packs, locales and sandbox helper, and adds a desktop launcher and icon. The launcher sets LD_LIBRARY_PATH to the private CEF directory. The sandbox helper is packaged root-owned with mode 4755; the browser is not built with Chromium sandboxing disabled.

This is an early functional CEF browser build, not yet a full feature-complete browser. Tabs, custom navigation chrome, history/bookmarks UI, downloads, private profiles, permissions and tracker filtering remain future work.
