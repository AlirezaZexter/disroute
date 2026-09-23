<div align="center">
  <img src="assets/icon.svg" width="96" alt="DisRoute icon">

  # DisRoute

  **Discord-only proxy routing for Windows — without sending the rest of the system through a VPN.**

  [![CI](https://github.com/AlirezaZexter/disroute/actions/workflows/ci.yml/badge.svg)](https://github.com/AlirezaZexter/disroute/actions/workflows/ci.yml)
  [![Release](https://img.shields.io/github/v/release/AlirezaZexter/disroute?include_prereleases&sort=semver)](https://github.com/AlirezaZexter/disroute/releases)
  [![Windows x64](https://img.shields.io/badge/Windows-x64-0078D4?logo=windows11&logoColor=white)](https://github.com/AlirezaZexter/disroute/releases)
  [![License: AGPL-3.0](https://img.shields.io/github/license/AlirezaZexter/disroute)](LICENSE)

  [Download](https://github.com/AlirezaZexter/disroute/releases) · [راهنمای فارسی](docs/START-HERE-FA.txt) · [Troubleshooting](docs/TROUBLESHOOTING.md) · [Contributing](CONTRIBUTING.md)
</div>

> [!IMPORTANT]
> DisRoute is an unsigned preview. It improves routing convenience; it is not a privacy, anonymity, or leak-prevention product.

## Why DisRoute?

Some networks cannot reach Discord reliably, while routing the whole computer through a VPN adds latency to games, browsers, and downloads. DisRoute targets the Discord desktop processes only and keeps unrelated applications on the normal network path.

- Per-process TCP and UDP routing for Discord Stable, PTB, and Canary
- VLESS, VMess, Trojan, and Shadowsocks share links
- Reality, TLS, TCP, WebSocket, HTTP, HTTPUpgrade, and gRPC transports where supported by the protocol
- XUDP and `packetaddr` selection from compatible VLESS share links
- Remote DNS recovery for Discord and dynamic voice hosts
- Windows-protected profile storage with explicit save and delete controls
- Persian RTL interface, tray lifecycle, keyboard access, and reduced-motion support
- Portable, versioned Windows builds with pinned and SHA256-verified engines

## Quick start

1. Open [Releases](https://github.com/AlirezaZexter/disroute/releases) and download `DisRoute-<version>-windows-x64.zip` — not GitHub's source archive.
2. Extract the ZIP completely.
3. Run the included ProxiFyre prerequisite installer once.
4. Start `DisRoute.exe` as Administrator.
5. Paste your own VLESS, VMess, Trojan, or Shadowsocks share link and connect.

The package includes a Persian text guide. WebView2, the Microsoft Visual C++ x64 runtime, and Windows Packet Filter are required.

## How it works

```mermaid
flowchart LR
    D[Discord] <--> P[ProxiFyre]
    P <--> S[Local SOCKS5 bridge]
    S <--> V[sing-box]
    V <--> R[Your proxy server]
    O[Games, browsers, other apps] <--> I[Direct internet]
```

The Tauri/Rust controller owns the local engine lifecycle. ProxiFyre performs Windows per-process interception, the loopback bridge recovers validated Discord voice SNI without decrypting TLS, and sing-box carries the resulting TCP/UDP traffic through the selected proxy outbound.

## Voice and streaming

DisRoute does not cap upload bandwidth. Discord media uses UDP when available, and the UDP relay bypasses the TCP/SNI inspection path inside DisRoute. Actual stream quality still depends on:

- the proxy server's upstream capacity, distance, packet loss, and UDP support;
- whether the selected protocol and server support UDP (including XUDP or `packetaddr` for VLESS/VMess);
- TCP head-of-line blocking when UDP is encapsulated through a TCP-based transport;
- other applications competing for the same physical upload connection.

If calls connect but streams stall or pixelate, read [Voice and streaming troubleshooting](docs/TROUBLESHOOTING.md#voice-and-streaming). A successful HTTPS probe does not prove media quality.

## Supported links

DisRoute accepts the share-link formats below and validates required credentials, server, port, security mode, transport, and certificate settings before startup. Certificate verification cannot be disabled through the UI. Existing profiles saved by version 0.2.x are migrated automatically.

| Link | Supported values |
| --- | --- |
| VLESS | `vless://` with TLS/Reality, Vision, and UDP packet encoding |
| VMess | Base64 JSON `vmess://` links, including TLS and common transports |
| Trojan | `trojan://` links with TLS/Reality and common transports |
| Shadowsocks | SIP002 `ss://` links without external plugins |
| Transport | `tcp` / `raw`, `ws`, `http` / `h2`, `httpupgrade`, `grpc` |

Unknown transport, flow, security, and packet-encoding values fail closed with a readable error.

## Security boundaries

- Generated process rules match Discord executables and Discord's own updater path; games and browsers are not added.
- Saved profiles use Windows DPAPI and are bound to the current Windows account.
- The active sing-box configuration is temporarily written under the app's local AppData runtime directory and removed during normal shutdown.
- Releases are currently unsigned previews. Verify the release checksum before use.
- A local Administrator or malware running as the same user can still access runtime state and traffic.
- Routing separation cannot reserve bandwidth or prevent unrelated apps from saturating the connection.

See [SECURITY.md](SECURITY.md) for reporting guidance and [VOICE_ROUTING.md](VOICE_ROUTING.md) for the voice-host recovery design.

## Development

Requirements: Node.js 24+, Rust MSVC, Microsoft C++ Build Tools, and WebView2.

```powershell
npm ci
npm run check
npm run tauri dev
```

Build the production executable with Tauri's custom protocol enabled:

```powershell
npm run build:portable
```

Do not substitute a plain `cargo build --release`; that leaves the Tauri development URL in the executable. Engine payloads are intentionally not committed. See [the release checklist](docs/RELEASING.md) for packaging and verification.

## Repository guide

| Path | Purpose |
| --- | --- |
| `src/` | React/TypeScript desktop interface |
| `src-tauri/src/` | Rust controller, profile protection, routing configuration, and voice bridge |
| `scripts/` | Reproducible packaging and dependency verification |
| `.github/workflows/` | Windows CI and tagged prerelease builds |
| `docs/` | Setup, release, and troubleshooting documentation |

## Contributing

Issues and focused pull requests are welcome. Never attach a real proxy link, unredacted engine configuration, packet capture, or user log. Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the available issue templates.

## License

DisRoute is licensed under [AGPL-3.0-or-later](LICENSE). Bundled third-party components retain their own licenses and notices.

<div align="center"><sub>Created by Zexter · Independent of Discord Inc.</sub></div>
