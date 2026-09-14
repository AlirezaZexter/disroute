# Release checklist

1. Match versions in package.json/package-lock.json, Cargo.toml/Cargo.lock, tauri.conf.json, UI and release notes.
2. Run frontend tests/build and Rust tests on Windows. Verify DPAPI save/reload/replace/delete with a dummy profile, never a real profile in screenshots.
3. Test keyboard navigation, reduced motion, narrow/desktop layouts, and the Windows tray lifecycle.
4. Test actual connection and two-way voice on a clean machine with documented prerequisites. Record unverified cases honestly.
5. Review staged files AND Git history for secrets. Never stage outputs/, work/, AppData or runtime configs.
6. Build with npm run build:portable; package with scripts/package-portable.ps1. Inspect ZIP inventory and hash.
7. Commit, push main, wait for CI, then create/push a matching version tag. The workflow publishes an unsigned prerelease. Do not tag while required checks fail.
8. Verify the public Releases page and downloadable ZIP/checksum. Signing, clean-machine QA, full source/license review and private vulnerability reporting are prerequisites for a stable release.
