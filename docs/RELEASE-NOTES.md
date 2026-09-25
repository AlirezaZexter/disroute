# DisRoute 0.5.0 — Windows preview

## Highlights

- Restored installed-app connections after Tauri packaging and in-app updates by normalizing Windows executable paths before creating Firewall rules.
- Added optional Community Quick Connect with configurable public subscription, GitHub, HTTPS, and local-file sources.
- Added strict source validation, deduplication, Windows-protected caching, source attribution, stale-cache fallback, cancellation, and bounded parallel testing.
- Added real end-to-end health checks through isolated temporary sing-box instances instead of relying on ICMP ping.
- Added latency, jitter, failure-rate, recent-history, and UDP-aware ranking with bounded failover.
- Preserved Discord-only TCP and UDP process routing in personal and Community modes.
- Expanded support for VLESS, VMess, Trojan, and Shadowsocks links and compatible transports.
- Kept signed in-app updates, download progress, passive installation, and the portable fallback.

## Installation and updates

Download `DisRoute-0.5.0-windows-x64-setup.exe` from the GitHub Release and run DisRoute as Administrator. If Windows Packet Filter is not already installed, extract the portable ZIP and run `ProxiFyre-2.6.1-win-x64-setup.exe` once.

Users on `0.4.1` or newer can select **بررسی آپدیت** inside DisRoute. Versions older than `0.4.1` must install a current setup package manually once. The in-app updater verifies release signatures; the installer itself is not yet Authenticode-signed and may still trigger Windows SmartScreen.

## Community limitation

Community endpoints are operated by third parties. A successful health check is only a current connectivity result and does not guarantee safety, continued availability, privacy, Discord Voice quality, or streaming performance.
