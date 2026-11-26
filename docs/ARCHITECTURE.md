# Architecture notes

## CEF process model

CEF uses a multi-process architecture identical to Chromium:

- Browser process (main)
- Renderer process(es)
- GPU process
- Utility / network / etc.

The **same executable** is re-executed with different `--type=` arguments.  
Any real implementation must call `cef::execute_process` early and only continue with window creation in the browser process.

## Recommended next implementation steps

1. Follow the `cefsimple` example from tauri-apps/cef-rs closely.
2. Implement a `Client` + `LifeSpanHandler` + `LoadHandler` + `DisplayHandler`.
3. Create a single browser window that loads an external URL (e.g. https://archlinux.org).
4. Add a minimal native chrome (or Alloy style + custom HTML chrome) for address bar and navigation buttons.
5. Introduce tab management (multiple `CefBrowser` instances or CEF's own tab helpers if available).
6. Persist bookmarks and history with a simple SQLite or JSON store.
7. Package with `bundle-cef-app` or custom scripts that ship `libcef.so` + resources.

## Resources

- Upstream bindings: https://github.com/tauri-apps/cef-rs
- CEF documentation: https://bitbucket.org/chromiumembedded/cef/wiki/Home
- Official CEF binary builds: https://cef-builds.spotifycdn.com/index.html
