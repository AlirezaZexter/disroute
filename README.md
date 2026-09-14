# DisRoute

DisRoute is a Windows desktop utility that routes only Discord traffic through a user-provided SOCKS5 proxy. Games, browsers, launchers, and other applications remain on the normal connection.

> Early development preview. Do not rely on this build for privacy or leak prevention yet.

## Architecture

```text
Discord.exe ── TCP/UDP ──► ProxiFyre ──► local SOCKS5 ──► sing-box ──► VLESS server
Everything else ─────────────────────────────────► Direct internet
```

The UI is built with Tauri 2, React, and TypeScript. The native controller is Rust. ProxiFyre performs per-process TCP/UDP routing, while sing-box translates the local SOCKS5 connection to VLESS.

## Development

Prerequisites: Node.js 24+, Rust MSVC, Microsoft C++ Build Tools, and WebView2.

```powershell
npm install
npm run test
npm run tauri dev
```

Build portable production executable (never use a plain `cargo build --release`,
which leaves Tauri's development URL enabled):

```powershell
npm run build:portable
```

For a portable build, place the ProxiFyre payload, `sing-box.exe`, and required dependencies in an `engine` directory beside `DisRoute.exe`. For local engine testing, the fallback path is `%LOCALAPPDATA%\app.disroute.desktop\engine`, or set `DISROUTE_ENGINE_DIR` explicitly. Windows Packet Filter must also be installed. Run DisRoute as Administrator when starting the network engine.

## Security posture

- Only `Discord.exe`, `DiscordCanary.exe`, and `DiscordPTB.exe` are included in generated routing rules.
- Discord's own `Update.exe` is matched by its `\\Discord\\Update.exe` path fragment; unrelated updater processes remain direct.
- On connect, two inbound Windows Firewall rules scoped to the bundled `ProxiFyre.exe` are created or updated for TCP and UDP.
- Startup verifies HTTPS 200 responses from the Discord Gateway API and update manifest over VLESS. A running tunnel does not prove voice connectivity.
- Private-IP LAN bypass is disabled for the Discord-only process rules because ISP DNS may return a private block-page address. TLS/HTTP sniffing restores a fixed list of Discord hostnames before sending them through VLESS. Unknown names and encrypted ClientHello are not covered by this recovery.
- VLESS links and UUIDs are never written to DisRoute's saved profile. The engine currently requires a temporary plaintext runtime configuration; this is tracked for hardening before a stable release.
- TLS certificate validation is enabled when SOCKS5-over-TLS is selected. Insecure certificate bypass is not exposed.
- Release automation and engine checksum pinning are required before public distribution.

## Roadmap

- Verified engine/downloader and prerequisite setup
- UAC relaunch and background service lifecycle
- Windows Credential Manager integration
- Connection test, Discord detection, logs with secret redaction
- Signed NSIS installer and GitHub Releases
- English and Persian localization

## License

AGPL-3.0-or-later. Third-party components retain their respective licenses and notices.
