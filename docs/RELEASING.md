# Release checklist

1. Match versions in package.json/package-lock.json, Cargo.toml/Cargo.lock, tauri.conf.json, UI, changelog, and release notes.
2. Run frontend tests/build and Rust tests on Windows. Verify DPAPI save/reload/replace/delete with a dummy profile, never a real profile in screenshots.
3. Test keyboard navigation, reduced motion, narrow/desktop layouts, the update panel, and the Windows tray lifecycle.
4. Run `scripts/prepare-installer.ps1` and inspect the generated resource inventory. Engine archives must match their pinned SHA256 values.
5. Build locally with `TAURI_SIGNING_PRIVATE_KEY` set, then confirm the NSIS installer and `.sig` updater signature are produced.
6. Test the installer on a clean Windows account. Confirm Discord text, two-way voice, stream upload, and a signed update from an older test version.
7. Review staged files and Git history for secrets. Never commit the private signing key, password, generated resources, outputs, work files, AppData, or runtime configs.
8. Push the release commit only after CI passes. Create and push the matching `v<version>` tag; the release workflow publishes the installer, `latest.json`, signatures, and portable fallback.
9. Confirm the GitHub release is public and `latest.json` references the NSIS asset. The updater endpoint ignores drafts and prereleases.
10. Keep a secure offline backup of the updater private key. Losing it prevents installed copies from trusting later updates.
