# DisRoute desktop interface — silver utility

This page supersedes the older visual tokens in MASTER.md. It is a presentation-only redesign; routing, proxy lifecycle, community sources and updater contracts remain unchanged.

## Direction

An Apple-inspired desktop utility, with Devigner's LiquidGlass as a reference for restrained translucency and shared-layout selection. No copied component package, fake macOS window controls, decorative blobs or invented connection measurements.

- Light canvas #f2f4f8, solid white settings surfaces, text #1d1d1f, secondary #606575, action #0066d6.
- Optional graphite appearance; use semantic tokens for every state, including warnings and errors.
- Persian: bundled Vazirmatn, 15px body, at least 13px supporting copy, 28px workspace title. Latin: system UI stack. Isolate mixed-script values.
- 72px toolbar with text wordmark only. Right-side navigation at large desktop widths, horizontal navigation at compact widths. No broken header image.
- Real connect control stays bound to the existing form. Keep configuration visible before live-health history. Routing explanation stays beside settings.
- Two frosted surfaces maximum: toolbar and navigation. Remaining content is opaque for legibility and GPU cost.
- Blue shared-layout selection, short crossfades and existing reduced-motion support. No continuous decorative animation.
- Keyboard-visible focus, clear pressed states, minimum 40px controls, explicit community warning, no color-only status.

## Verification

Review personal, community acknowledgement, source manager, guide, release notes, update notice and both appearances. Check 1080px and 760px desktop widths, small-screen fallback, unsaved input preservation, appearance storage failure and existing UI regressions. Do not interrupt the user's live Discord connection to preview the layout.

### Verified locally, 2026-10-10

- 43 frontend tests and two release-note tests pass; 52 Rust tests pass, six opt-in live/elevated checks remain intentionally skipped.
- TypeScript/Vite production build and Windows x64 NSIS Release build succeed.
- Browser inspection: personal form, explicit community warning, source management, guide, release notes, current-version update notice, light/dark appearance. No horizontal document overflow at 1080, 760 or 375px.
- No browser console errors in the inspected states. Appearance changes preserve unsaved input; blocked preference storage fails closed to light mode.
- The preview does not operate the Windows network engine. No new live connection or elevated prerequisite installation was attempted for this visual-only change.
- This is a local 0.5.9 design preview, not a published GitHub release. It retains the existing updater public key and source catalog.
