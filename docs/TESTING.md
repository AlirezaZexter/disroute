# Verification

Run `npm run check` for frontend tests, TypeScript/build checks and Rust unit tests.

## Actual engine integration

These tests use the pinned sing-box executable, with synthetic loopback-only credentials. They do not change Windows routes or firewall rules.

```powershell
npm run prepare:installer
$env:DISROUTE_TEST_ENGINE = (Resolve-Path src-tauri/resources/engine/sing-box.exe).Path
cargo test --manifest-path src-tauri/Cargo.toml isolated_engine_authentication_and_cleanup -- --ignored
```

The test checks successful proxied traffic, failed authentication, process cleanup and port release. CI runs it after preparing engines.

## Prerequisite setup verification (0.5.6)

The startup probe reads native x64 registry/file versions without MSI repair, service activation, route changes, or firewall changes. A registered driver with a compatible file version is not proof that the driver can route traffic; a real Discord connection remains a separate manual check.

After preparing resources, exercise the same installer hash verifier used before launching the official bootstrapper:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml verifies_actual_bundled_prerequisite_installer -- --ignored
```

Local verification on 2026-09-30:

- 22 frontend tests and 42 Rust tests passed, including explicit-consent setup, missing components, cancellation, restart, failure/retry, active-connection exclusion, installer size/tamper rejection, and bounded probe timeout.
- The official bundled bootstrapper passed production SHA256 verification. The actual isolated sing-box authentication/failure/cleanup integration test passed.
- The x64 Release executable was built with the production custom protocol. Its extracted embedded manifest contains `requireAdministrator`.
- The generated NSIS installer includes the fixed `resources/prerequisites/ProxiFyre-2.6.1-win-x64-setup.exe` payload and Microsoft's WebView2 bootstrapper.
- Persian setup states and the revised guide were visually inspected in the local browser preview. The plain-text Persian guide passed orthography checks.

Not verified on a clean Windows installation: accepting the upstream installer terms, actual driver/runtime download and installation, cancellation/rollback during an MSI change, restart completion, installed-app UAC launch, Discord voice/streaming, and an end-to-end update from 0.5.5. Do not treat mocked UI outcomes or a ready status on this already-provisioned PC as proof of those workflows. No driver was installed or removed on the user's machine for testing.

## Opt-in local checks

The public-source test downloads the first configured default feed, parses it with the production validator and performs actual certificate-verified Discord HTTPS requests through isolated proxy instances. It prints aggregate counts and latency, never credentials. It intentionally fails if no currently working endpoints are found. It is not a deterministic release gate.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml live_community_scan -- --ignored --nocapture
```

Firewall integration requires an elevated terminal. Set `DISROUTE_FIREWALL_TEST_PROGRAM` to the local ProxiFyre executable, then run `installed_firewall_roundtrip -- --ignored`. This updates only the application's named TCP/UDP rules and checks both create/update behavior with canonical verbatim Windows paths.

## 0.5.2 verification record (2026-09-25)

- 12 frontend tests and 32 Rust tests passed; TypeScript and production frontend build passed.
- The checksum-pinned sing-box authentication, failed-authentication and cleanup integration test passed.
- The bundled Vazirmatn files were emitted in the production build, and the connection and setup-guide views were inspected in the local desktop preview.
- Component extraction, shared-layout motion, saved-profile feedback, RTL typography and responsive presentation were covered without changing the network backend.
- No networking, routing, Xray lifecycle, Firewall, profile-storage format, Community source, or updater logic changed in this release.

The signed NSIS installer, updater signature, `latest.json`, and portable package are built by the tagged GitHub Actions release workflow.

## 0.5.1 verification record (2026-09-25)

- 12 frontend tests and 32 Rust tests passed; TypeScript and production frontend build passed.
- The isolated sing-box authentication and cleanup integration test passed with the checksum-pinned engine.
- Installer resources were downloaded from pinned upstream releases and passed checksum verification.
- The Persian setup guide passed the project writing checks, and the connection and guide views were inspected at the narrow desktop breakpoint.
- No networking, routing, Xray lifecycle, Firewall, profile-storage, Community source, or updater logic changed in this release.

The signed NSIS installer, updater signature, `latest.json`, and portable package are built by the tagged GitHub Actions release workflow.

## 0.5.0 verification record (2026-09-25)

- 12 frontend tests and 32 Rust tests passed; TypeScript and Release build passed.
- Real isolated-engine authentication/cleanup passed.
- Elevated Firewall create/update roundtrip passed.
- Au1rxx Netherlands live feed: 77 accepted configurations, 12 working Discord HTTPS paths; best observed connection time 386 ms. This is a point-in-time observation, not a performance promise.
- Radikal's feed yielded no working paths on the same network during an earlier scan; source availability varies.

The manual Discord account/voice/streaming session is not covered by these tests. UDP DNS reachability is reported separately and does not guarantee media quality. Cancellation is cooperative between bounded operations. Windows DPAPI tests may return early in non-interactive environments without a usable user token; manual installed-app save/restore should be checked under the actual Windows account.
