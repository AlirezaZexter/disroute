# Contributing to DisRoute

Thank you for helping improve DisRoute. Small, reviewable changes are preferred over broad rewrites.

## Before opening an issue

- Search existing issues and read [Troubleshooting](docs/TROUBLESHOOTING.md).
- Reproduce the problem with the newest release in a fresh directory.
- Remove every host, UUID, key, short ID, IP address, account identifier, and local path from diagnostic material.
- Use the appropriate issue template. Security-sensitive reports do not belong in public issues.

## Development setup

Requirements: Windows, Node.js 24+, Rust MSVC, Microsoft C++ Build Tools, and WebView2.

```powershell
npm ci
npm run check
npm run tauri dev
```

Engine binaries are not stored in Git. For local engine testing, put the required payload in `%LOCALAPPDATA%\app.disroute.desktop\engine` or set `DISROUTE_ENGINE_DIR` to a dedicated test directory.

## Pull requests

1. Create a focused branch from `main`.
2. Keep networking, UI, and documentation changes in separate commits when practical.
3. Add or update tests for routing, parsing, persistence, and interaction behavior.
4. Run `npm run check` and `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
5. Explain user-visible behavior, test coverage, and any unverified Windows/network cases.
6. Do not commit engine binaries, generated archives, runtime files, logs, packet captures, or credentials.

Network changes must preserve the central invariant: only the documented Discord processes are added to generated proxy rules. Any change that broadens routing scope must be explicit, tested, and documented.

## Style

- Rust: standard `rustfmt`, clear error messages, no secrets in logs.
- React/TypeScript: semantic controls, keyboard support, visible focus, and reduced-motion behavior.
- Persian copy: keep English commands and filenames in isolated LTR elements or lines.
- Documentation: describe verified behavior and limitations without marketing claims.

Contributions are accepted under AGPL-3.0-or-later.
