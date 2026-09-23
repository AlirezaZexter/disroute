# Changelog

Notable changes to DisRoute are recorded here. The project follows Semantic Versioning while it remains in preview.

## [Unreleased]

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

[Unreleased]: https://github.com/AlirezaZexter/disroute/compare/v0.2.6...HEAD
[0.2.6]: https://github.com/AlirezaZexter/disroute/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/AlirezaZexter/disroute/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/AlirezaZexter/disroute/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/AlirezaZexter/disroute/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/AlirezaZexter/disroute/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/AlirezaZexter/disroute/releases/tag/v0.2.1
