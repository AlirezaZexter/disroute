# Public beta readiness

This is a verification ledger, not a claim that all Windows/network combinations work.

## Evidence and remaining gates (2026-10-11)

| Area | Evidence | Remaining check |
| --- | --- | --- |
| Voice and streaming | Maintainer reports successful testing with several people | Windows versions, network types, durations and build versions were not supplied; no universal performance claim |
| Profile persistence | Real Windows DPAPI save/load/replace, rejected-write preservation and legacy encrypted-document migration tests | Installed update/relaunch under a normal user account |
| Updater lifecycle | Regression tests for concurrent update/connect exclusion, fresh backend status before install and failure preservation | Actual older installed app → new signed release, relaunch and profile restore |
| Installer | [CI run 38097114322 passed](https://github.com/AlirezaZexter/disroute/actions/runs/38097114322): real NSIS install of pinned public 0.5.9, candidate upgrade, engine/bootstrapper hash verification and preserved/decryptable dummy profile | Its hosted image is NOT a clean OS; upstream driver/runtime installation and restart still need a clean Windows test |
| Selective routing | Exact Discord allowlist across all supported protocols, loopback-only inbound, no TUN/system proxy/DNS override; read-only active-config diagnostic | Compare OS routes/proxy/DNS before and after a real connection; confirm browser/game traffic stays direct |
| Public release | Public 0.5.9 has installer, signature, portable archive, checksum and updater metadata | Candidate changes are not released merely by editing this document |

## Clean Windows test

Use a disposable Windows 10/11 x64 machine or VM with no Windows Packet Filter / ProxiFyre installed. Do not uninstall drivers from the developer's working PC to simulate a clean system.

1. Record Windows build and DisRoute versions, without personal paths or network addresses.
2. Install the official setup. Check WebView2 setup when missing, UAC, and first-run prerequisite detection. The app must not report a connection while prerequisites are missing.
3. Accept the upstream prerequisite installer only after reading its terms. Test cancellation and retry, internet unavailable then restored, and reboot completion when requested. Check .NET requirements separately.
4. With a test configuration, confirm text, two-way voice and stream upload. Close/reopen Discord and DisRoute; verify configuration restore.
5. From an older installed version, use the built-in updater to install the signed candidate. Confirm app relaunch, same Windows-account profile, and saved appearance/source preferences. A downloaded file with a verified signature alone does not establish this installed workflow.
6. Confirm Windows routes, DNS and system proxy settings match their pre-connection baseline. With no other VPN enabled, check browser egress through a trusted endpoint before/after; privately compare addresses, never attach them to public issues. Check a game alongside a short call, both without streaming and with upload; bandwidth contention is not proof of system-wide routing.

No provisioned VM/hypervisor command was available in the current workspace. These manual gates remain open until real test results are recorded. No current driver, firewall or active tunnel was changed to make a test appear successful.

The maintainer confirmed that previous external installation tests used the portable ZIP with ProxiFyre installed separately. No tester has yet verified the current single-setup, in-app prerequisite installation flow on a machine without those prerequisites. Those earlier reports do not close the clean-install gate.

## Read-only active configuration check

In an Administrator PowerShell, while DisRoute is connected:

```powershell
./scripts/verify-selective-routing.ps1
```

If process paths cannot be inspected, supply `-RouterConfig` and `-TunnelConfig` with local application-owned files. The script prints booleans, not credentials. It validates the active generated configuration, not packet routing or session quality. An existing Windows proxy may belong to another application; the diagnostic does not disable it.

## Start small

Invite 10–20 willing testers first. Gather version, installation method, connection mode, symptom and reproduction steps using the issue form. Do not collect configuration links or enable telemetry. Resolve installation/update/data-loss failures before announcing widely. Community source uptime remains a third-party limitation, not a promised free service.

[Persian quick start](QUICK-START-FA.md) · [Announcement copy](ANNOUNCEMENTS-FA.md) · [Verification details](TESTING.md)
