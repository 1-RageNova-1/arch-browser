# CEF runtime and first graphical milestone

The application now uses the upstream cef-rs process lifecycle and CEF Views API to create a native Chromium browser window on Linux. It launches the default start page or a URL supplied with --url=https://example.com.

CEF remains a multi-process runtime. The executable must be distributed with the matching CEF shared library, resource packs, locales and subprocess support files. A successful Cargo build alone does not prove that these runtime assets are present.

This milestone is limited to a real CEF window and page rendering. Browser chrome (tabs, address bar, navigation buttons), persistent profile management, downloads, private windows, permissions, pop-up policy and tracker filtering are separate follow-up work. Do not describe those features as implemented.

The Linux CI build validates compilation and unit tests. GUI startup and external HTTPS navigation still require a graphical Linux session and are not claimed as tested by headless CI.
