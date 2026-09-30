# Changelog

Notable changes to DisRoute are recorded here. The project follows Semantic Versioning while it remains in preview.

## [Unreleased]

## [0.5.7] - 2026-09-30

### Fixed

- Unclipped, non-shrinking vector header logo; explicitly set the native window/taskbar icon from the current bundled logo at startup.
- Community freshness now distinguishes fresh sources from cached fallback instead of labelling a mixed list entirely old. Enabled-source failures are visible outside collapsed source settings.
- Sanitized source fetch errors distinguish HTTP status, timeout and connection failure without disclosing subscription URLs.
- Release descriptions use real headings/lists, stable Persian direction, one keyboard-accessible scroll area and clear installation implications. Routing and updater trust/download logic are unchanged.

## [0.5.6] - 2026-09-30

### Added

- In-app prerequisite detection and an explicit, interactive installation action for missing Windows Packet Filter and Visual C++ x64 components.
- Bundled checksum-pinned official ProxiFyre prerequisite bootstrapper in both setup and portable packages. Normal users no longer need a separate portable download.
- Clear setup progress, cancellation, failure/retry, and restart feedback; connection/test operations cannot overlap prerequisite installation.
- Persian What's new view and a dismissible once-per-version notice. Offline in-app notes and the published release body share one version-validated source.

### Fixed

- Removed the dark matte and noisy colored edges around the accepted D logo. Rebuilt desktop/tray icons from a transparent vector master and explicitly configured installer/uninstaller icons.
- Production-feature Rust tests no longer attempt to run an empty executable test harness requiring UAC elevation; the library test suite remains enabled.

### Changed

- Production executables automatically request Administrator permission through Windows UAC.
- Embed the WebView2 bootstrapper in the Windows installer; runtime downloads still require internet.
- Updated the in-app guide, Persian text guide, and installation/troubleshooting documentation. Discord-only routing is unchanged. Windows Authenticode signing remains unavailable.

## [0.5.5] - 2026-09-27

### Changed

- Removed the ambiguous traffic-direction diagram from the connection status card.

## [0.5.4] - 2026-09-26

### Fixed

- Show the status route as `Discord → Proxy → Internet` with two right-facing arrows instead of bidirectional arrowheads.

## [0.5.3] - 2026-09-26

### Changed

- Applied the new DisRoute logo across the desktop UI, executable, tray, installer, and project page.
- Improved Community source refresh, source fairness, local result reuse, early healthy-shortlist completion, and updater progress reporting.

## [0.5.2] - 2026-09-25

### Changed

- Split the desktop interface into focused view and controller components without changing the connection backend.
- Added restrained shared-layout and cross-fade transitions with reduced-motion support.
- Bundled Vazirmatn locally and improved the type scale, line height, mixed-direction text, and small-label readability.
- Refined the connection status, profile persistence feedback, route summary, Community scan progress, and setup guide presentation.

### Fixed

- Keep saved-profile state visible after loading and make form state changes clearer in the Persian interface.

## [0.5.1] - 2026-09-25

### Changed

- Refined the desktop layout and synchronized the Persian setup guide with the installer, updater, Community mode, tray lifecycle, and voice requirements.

## [0.5.0] - 2026-09-25

### Fixed

- Normalize Tauri verbatim resource paths before passing executable paths to Windows Firewall, and quote netsh arguments correctly. Existing DisRoute rules are updated instead of deleted before replacement.
- Prepare pinned engine resources for portable builds as well as installers.

### Added

- Optional Persian Community Quick Connect: refresh sources, test through isolated sing-box instances, rank measured Discord HTTPS connectivity, and connect with one click.
- Public Radikal subscription and CDN mirror, explicit first-use warning, configurable sources, encrypted cached fallback and local history.
- Rustls HTTPS checks with mandatory SOCKS routing, UDP reporting, preflight before switching, and opt-in failover.
- Regression coverage for installed Firewall paths, mixed subscriptions, one-click UI flow and actual isolated engine authentication/cleanup.

## [0.4.1] - 2026-09-24

### Added

- Signed in-app update checks, download progress, and passive Windows installation.
- A reproducible NSIS installer containing checksum-pinned ProxiFyre and sing-box resources.
- Automatic GitHub release metadata and updater signatures for tagged versions.

### Changed

- Stops the active proxy engines before the Windows updater exits the application.
- Keeps the portable ZIP as a fallback while making the installer the recommended first download.

## [0.3.0] - 2026-09-23

### Added

- VMess, Trojan, and Shadowsocks share-link support alongside VLESS.
- Live protocol detection and inline unsupported-link feedback in the connection form.

### Changed

- Renamed the encrypted profile field from VLESS-specific `vlessLink` to `configLink` while preserving automatic migration for 0.2.x profiles.
- Generalized connection checks, status copy, documentation, and sing-box outbound tags for every supported protocol.

## [0.2.6] - 2026-09-23

### Added

- Support for `xudp`, `packetaddr`, and disabled UDP packet encoding values supplied by VLESS share links.
- Focused voice and streaming troubleshooting documentation.
- Structured GitHub issue and pull-request templates.

### Changed

- Replaced ambiguous one-way route arrows with an explicit bidirectional network path.
- Rebuilt the project README around current behavior, architecture, security boundaries, and contributor onboarding.
- Reduced ProxiFyre runtime logging from Info to Warning to avoid unnecessary background I/O.

## [0.2.5] - 2026-09-16

- Prevented redirects and edge/WAF responses from incorrectly shutting down a valid VLESS tunnel.
- Kept successful SOCKS connections active when the optional Windows HTTPS check is inconclusive.

## [0.2.4] - 2026-09-15

- Added in-app Discord restart and single-instance handling.

## [0.2.3] - 2026-09-15

- Repaired mixed Persian/English layout in the Windows text guide.

## [0.2.2] - 2026-09-15

- Refined Persian interface copy and added the portable setup guide.

## [0.2.1] - 2026-09-15

- Shipped a GUI-only executable with an explicit notification-area icon.

[Unreleased]: https://github.com/AlirezaZexter/disroute/compare/v0.5.7...HEAD
[0.5.7]: https://github.com/AlirezaZexter/disroute/compare/v0.5.6...v0.5.7
[0.5.6]: https://github.com/AlirezaZexter/disroute/compare/v0.5.5...v0.5.6
[0.5.5]: https://github.com/AlirezaZexter/disroute/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/AlirezaZexter/disroute/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/AlirezaZexter/disroute/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/AlirezaZexter/disroute/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/AlirezaZexter/disroute/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/AlirezaZexter/disroute/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/AlirezaZexter/disroute/compare/v0.3.0...v0.4.1
[0.3.0]: https://github.com/AlirezaZexter/disroute/compare/v0.2.6...v0.3.0
[0.2.6]: https://github.com/AlirezaZexter/disroute/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/AlirezaZexter/disroute/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/AlirezaZexter/disroute/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/AlirezaZexter/disroute/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/AlirezaZexter/disroute/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/AlirezaZexter/disroute/releases/tag/v0.2.1
