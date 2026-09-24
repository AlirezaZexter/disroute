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

## Opt-in local checks

The public-source test downloads the first configured default feed, parses it with the production validator and performs actual certificate-verified Discord HTTPS requests through isolated proxy instances. It prints aggregate counts and latency, never credentials. It intentionally fails if no currently working endpoints are found. It is not a deterministic release gate.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml live_community_scan -- --ignored --nocapture
```

Firewall integration requires an elevated terminal. Set `DISROUTE_FIREWALL_TEST_PROGRAM` to the local ProxiFyre executable, then run `installed_firewall_roundtrip -- --ignored`. This updates only the application's named TCP/UDP rules and checks both create/update behavior with canonical verbatim Windows paths.

## 0.5.0 verification record (2026-09-25)

- 12 frontend tests and 32 Rust tests passed; TypeScript and Release build passed.
- Real isolated-engine authentication/cleanup passed.
- Elevated Firewall create/update roundtrip passed.
- Au1rxx Netherlands live feed: 77 accepted configurations, 12 working Discord HTTPS paths; best observed connection time 386 ms. This is a point-in-time observation, not a performance promise.
- Radikal's feed yielded no working paths on the same network during an earlier scan; source availability varies.

The manual Discord account/voice/streaming session is not covered by these tests. UDP DNS reachability is reported separately and does not guarantee media quality. Cancellation is cooperative between bounded operations. Windows DPAPI tests may return early in non-interactive environments without a usable user token; manual installed-app save/restore should be checked under the actual Windows account.
