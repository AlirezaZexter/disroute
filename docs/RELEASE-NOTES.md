# DisRoute 0.4.0 — Windows preview

- Added signed in-app update checks, download progress, and passive installation.
- Added a Windows installer that bundles the pinned networking engines.
- Added a tagged GitHub release pipeline that publishes `latest.json`, the NSIS installer, updater signatures, and the portable fallback.
- Stops the active Discord route cleanly after download and before Windows installs an update.

Users of 0.3.0 and earlier must install 0.4.0 manually once. Updates after 0.4.0 can be installed from inside DisRoute. The updater signature protects the update channel, but the Windows installer is not yet Authenticode-signed and may still trigger SmartScreen.
