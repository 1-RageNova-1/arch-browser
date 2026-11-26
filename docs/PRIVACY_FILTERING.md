# Network privacy filtering

Arch Browser evaluates each non-navigation CEF resource request through Brave's
`adblock-rust` engine on a dedicated worker thread. CEF's IO callback sends
only the request URL, initiator URL, resource type and HTTP method to that
worker; it returns `RV_CANCEL` for a matched blocking rule and
`RV_CONTINUE` otherwise.

The Debian package installs current-at-build snapshots of EasyList and
EasyPrivacy in `/usr/share/arch-browser/filters`. The app also compiles its
small additional Google analytics and advertising list into the binary.
Setting `ARCH_BROWSER_FILTERS_DIR` can override the runtime filter-list
directory for development.

This is a network filter, not an ungoogled Chromium fork. It cannot guarantee
that every first-party Google telemetry call or every tracking technique is
removed, and it does not currently inject cosmetic CSS filters or scriptlets.
The EasyList/EasyPrivacy snapshots should be refreshed through future package
updates.
