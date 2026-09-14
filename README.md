# DisRoute

DisRoute is a Persian-first Windows desktop utility that routes Discord through your own VLESS server. Other applications are not added to its routing rules. Built with Tauri, Rust, React and Motion; independent of Discord.

> Early development preview. Do not rely on this build for privacy or leak prevention yet.

## Releases / دانلود نسخه‌ها

Windows builds are available on the [GitHub Releases page](https://github.com/AlirezaZexter/disroute/releases). Download `DisRoute-<version>-windows-x64.zip`, not “Source code.zip”. Extract it fully and read [راهنمای فارسی](docs/START-HERE-FA.txt).

Windows x64, WebView2, Visual C++ runtime and Windows Packet Filter are required. The package includes the official prerequisite installer; setup remains user-controlled.

## New in 0.2

- Windows DPAPI-encrypted profile storage, automatic restore, opt-out and confirmed deletion.
- Shared-layout navigation, status crossfade, keyboard access and reduced-motion support.
- Close-to-tray, reopen from the tray icon and explicit disconnect-and-exit; hidden engine consoles.
- Runtime VLESS files separated from the shareable program folder.
- Versioned ZIP packaging, pinned engine checksums and GitHub release workflow.

## Architecture

```text
Discord.exe ── TCP/UDP ──► ProxiFyre ──► local SOCKS5 ──► sing-box ──► VLESS server
Everything else ─────────────────────────────────► Direct internet
```

The UI is built with Tauri 2, React, and TypeScript. The native controller is Rust. ProxiFyre performs per-process TCP/UDP routing, while sing-box translates the local SOCKS5 connection to VLESS.

## Development

Prerequisites: Node.js 24+, Rust MSVC, Microsoft C++ Build Tools, and WebView2.

```powershell
npm ci
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
- Private-IP LAN bypass is disabled for Discord because ISP DNS may return a block-page address. Fixed-name recovery plus a loopback SOCKS bridge recover dynamic voice SNI. See [voice routing limitations](VOICE_ROUTING.md). ECH and unrecognized voice TLS ports are not covered.
- Saved profiles are encrypted with Windows DPAPI in `%LOCALAPPDATA%/app.disroute.desktop/profile.dpapi`, bound to the current Windows account. No browser localStorage is used. This does not protect against malware acting as that user or Administrator.
- sing-box requires a temporary plaintext configuration under AppData/runtime. Normal shutdown removes it; a crash can leave it behind. ProxiFyre's non-secret routing config and logs remain beside its executable. Older 0.1.x builds wrote VLESS runtime configuration beside the engine; never share those used folders.
- TLS certificate validation is enabled when SOCKS5-over-TLS is selected. Insecure certificate bypass is not exposed.
- Packaging downloads version-pinned engines and verifies SHA256. Releases are unsigned previews; actual two-way voice testing and clean-machine prerequisite validation are required before a stable release.
- Routing separation does not eliminate competition for shared bandwidth.

## Release workflow

Run `npm run build:portable`, then `./scripts/package-portable.ps1`. For a custom Cargo target directory, pass `-Executable <absolute-exe-path>`. Only clean engine archive payloads are packaged; used runtime folders are never copied. Existing versioned archives are not overwritten.

CI tests main and pull requests. Pushing a matching `v<package-version>` tag builds and publishes a prerelease ZIP with checksum. Manual workflow runs upload Actions artifacts only. See [release checklist](docs/RELEASING.md).

## Roadmap

- Verified engine/downloader and prerequisite setup
- UAC relaunch and background service lifecycle
- Further runtime-secret hardening
- Connection test, Discord detection, logs with secret redaction
- Signed NSIS installer and stable releases
- English and Persian localization

## License

AGPL-3.0-or-later. Third-party components retain their respective licenses and notices.
