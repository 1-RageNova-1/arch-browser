# Browser extensions

## Why the Helium patch is not copied verbatim

Helium does not install its blocker as a normal third-party extension. Its repository vendors a soft-fork of full uBlock Origin and patches Chromium's GN/resource build, component loader and extension allowlists so that the extension is registered as a browser component.

References:
- Helium's uBlock source integration: https://github.com/imputnet/helium/blob/main/patches/helium/core/ublock-setup-sources.patch
- Helium's component installation patch: https://github.com/imputnet/helium/blob/main/patches/helium/core/ublock-install-as-component.patch
- Helium's uBlock fork: https://github.com/imputnet/uBlock
- Full uBlock Origin upstream: https://github.com/gorhill/uBlock

The Helium fork is GPL-3.0 and is designed for Helium's own Chromium source tree. Its patches are not a drop-in CEF extension. CEF 156 uses the Chrome runtime for Chromium extension support, but its API still lacks programmatic extension management; its upstream extension-management issue is open.

## Default blocker

Arch Browser's Debian launcher adds `--enable-chrome-runtime`; the app sets an application-specific Chrome policy directory under the current user's XDG configuration directory. The managed policy force-installs the official uBlock Origin Lite extension from the Chrome Web Store:

- Extension ID: `ddkjiahejlhfcafbddmgiahcphecmpfh`
- Store listing: https://chromewebstore.google.com/detail/ublock-origin-lite/ddkjiahejlhfcafbddmgiahcphecmpfh
- Update service: https://clients2.google.com/service/update2/crx
- Source: https://github.com/uBlockOrigin/uBOL-home
- License: GPL-3.0

uBlock Origin Lite is Raymond Hill's Manifest V3 product. Its default rules include uBlock's built-in lists, EasyList, EasyPrivacy and Peter Lowe's ad/tracking server list. It is not feature-identical to full Manifest V2 uBlock Origin.

The extension is force-pinned in the toolbar so its status and controls are visible by default. It is fetched and updated from the official Chrome Web Store; its binary is not repackaged or modified by Arch Browser.

## Optional extensions

Users may choose to install these from the start page. Each link points to the official publisher's Chrome Web Store listing:

| Extension | Store ID | Publisher |
| --- | --- | --- |
| 1Password | `aeblfdkhhhdcdjpifhhbdiojplfjncoa` | AgileBits Inc. |
| Bitwarden | `nngceckbapebfimnlniiiahkandclblb` | Bitwarden Inc. |
| Proton VPN | `jplgfhpmjnbigmhklmmbgecoobifkmpa` | Proton AG |

Installation of optional extensions is user-initiated. Proton VPN's browser extension routes browser traffic only; it is not a system-wide VPN.

## Manual verification

After installing and launching Arch Browser with an internet connection:

1. Open `chrome://policy` and confirm that the `ExtensionSettings` policy is loaded.
2. Open `chrome://extensions` and confirm that uBlock Origin Lite appears as installed and enabled.
3. Open its extension details and verify that it is managed by Arch Browser.
4. Open a test page and use the extension popup to check its filtering status.

The project CI compiles and packages the application but does not launch a graphical CEF process. These runtime checks must be performed on a desktop session.
