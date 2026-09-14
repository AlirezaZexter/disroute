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

For local engine testing, place the ProxiFyre payload, `sing-box.exe`, and required dependencies in `%LOCALAPPDATA%\app.disroute.desktop\engine`, or set `DISROUTE_ENGINE_DIR` to that directory. Windows Packet Filter must also be installed. Run DisRoute as Administrator when starting the network engine.

## Security posture

- Only `Discord.exe`, `DiscordCanary.exe`, and `DiscordPTB.exe` are included in generated routing rules.
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
