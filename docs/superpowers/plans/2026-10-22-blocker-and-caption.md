# Blocker and caption controls implementation plan

> Execute inline with superpowers:executing-plans. No subagents or tests: the
> user's project instructions override those generic skill defaults. Existing
> authorization covers main, publishing and releases. No additional handoff.

**Goal:** Integrate the approved YouTube blocker and make maximized caption
controls use the same explicit drawing and input coordinates.
**Architecture:** A focused native/blocker.rs owns WebView2 extension setup,
preference and status. A pinned packaging script supplies the unmodified uBO
Lite package. Native BUTTON caption controls dispatch standard system commands.
**Stack:** Rust, Win32, WebView2, PowerShell, Inno Setup.
**Spec:** ../specs/2026-10-22-ad-blocking-design.md

## Constraints and review focus

No application or installer launch, unit tests or functional tests. Preserve
profiles, navigation restrictions and update integrity. One active WebView.
Review stale async callbacks, disabled/Apple behavior, startup failure paths,
packaged third-party provenance, and caption input at maximized/DPI boundaries.

## Tasks

- [x] Package uBO Lite 2026.1021.1354 Chromium archive, pinned SHA-256
  0ff92d3e33437e0fa389a16a3d4c3a486fbda08e33114e07859db1f58d789d0b.
  Keep upstream files unchanged, ship license/source notices, call preparation
  from development/installer builds, include files only in the EXE installer.
- [x] Add native/blocker.rs: bounded saved preference, honest status, extension
  initialization before navigation, enable only on Google services, disable and
  reload on user request. Keep async callbacks generation-scoped; fail open with
  an unavailable status, not a playback failure. Connect App/settings/commands.
- [x] Replace implicit DWM buttons with three accessible native BUTTON controls,
  Segoe caption glyphs and standard minimize/maximize/restore/close commands.
  Keep root HTMAXBUTTON for Snap, route mouse input to native button capture,
  reuse one rectangle layout for drawing/hit testing in all window states.
- [ ] Review connected callers and packaging. Run fmt/check/Clippy/audit/build,
  scan staged changes for secrets, update README/architecture/version, push and
  publish. Check MSVC/installer CI and the public installer's SHA-256 digest.

## Rulings

- The user confirmed the bug occurs in a maximized window, not YouTube's media
  fullscreen mode. Do not add unrelated media-fullscreen behavior.
- uBO Lite's shipped default is optimal filtering with broad host permissions.
  Keep its package unmodified and disable the extension before Apple navigation;
  do not add a second filter engine or pretend initialization proves ad removal.
- Use a stable installed extension folder so its unpacked extension ID remains
  stable across app releases. Reconcile before service navigation.
- Skill i-have-adhd installed from the requested upstream repository and read.
