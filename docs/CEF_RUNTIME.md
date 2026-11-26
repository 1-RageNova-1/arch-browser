# CEF runtime and first graphical milestone

The application starts through the cef-rs process lifecycle and uses CEF's native Linux browser-window path. It opens the default page or a URL supplied with `--url=https://example.com`.

CEF is a multi-process runtime. The application must be launched from a directory containing the matching CEF shared library, resource packs, subprocess support files and locales. The build workflow copies the complete runtime into a portable archive rather than publishing the application executable alone.

The Linux CI now runs a graphical smoke test under Xvfb. It starts the browser, waits for CEF's remote-debugging endpoint, and verifies that the expected page target is created. This validates browser startup in a virtual X11 display; it does not replace testing on a physical Ubuntu/GNOME/Wayland desktop, and it does not certify every web page or GPU configuration.

Browser chrome (tabs, address bar, navigation controls), persistent profile management, downloads, private windows, permissions, pop-up policy and tracker filtering are separate follow-up work. Do not describe those features as implemented.
