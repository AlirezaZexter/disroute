# 0.5.3 validation notes

The approved mint D artwork is used in the desktop window, native Windows executable, tray and README. Existing layout and Discord-only routing rules are unchanged.

## Community connection

- Fresh cache avoids a redundant refresh on each click.
- Source fetches run four at a time; only due sources refresh on the background timer.
- Regional/stability subscriptions extend the existing authorized registry, with one-time migration preserving disabled and removed feeds.
- Source interleaving prevents a large feed from excluding smaller regional feeds.
- Three stable UDP results can end scheduling early; final ranking remains local and measured.
- Health failures do not count as dropped user sessions. Recent local timings order subsequent scans.
- HTTPS requires a successful status, at least two samples, certificate verification and mandatory SOCKS; no direct fallback.
- No system proxy, Firewall or routing changes are introduced by this update.

## Updater

Tauri downloads the complete signed installer, not a delta. A read-only measurement of the existing 0.5.2 release asset on 2026-09-26 returned 24,287,164 bytes in 8.24 seconds, with a 1.30-second time to first byte. This does not reproduce every user's network conditions or measure installer duration.

Progress callbacks previously rendered once per network chunk. They now coalesce to at most four periodic UI reports per second, plus start/finish, with byte totals and average download speed. A three-minute download timeout prevents an indefinite wait. Signature verification remains mandatory, and the active connection is stopped only after download succeeds. These changes reduce UI overhead and improve diagnosis; they do not claim to accelerate GitHub or implement resumable/delta downloads.

Long-duration voice/stream stability and a signed end-to-end update from an installed previous version still require validation after release. Public-source performance can change minute to minute.
