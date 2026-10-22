# Review corrections and native UI implementation plan

> Execute inline with `superpowers:executing-plans`. No subagents or runtime tests.

**Goal:** Address R1–R8 from the dated application review and refine the Windows UI direction approved in chat.

**Architecture:** Retain Rust/Win32, common-controls v6 and one WebView2 player. Separate measured settings geometry from behavior. Keep update preparation and retention on temporary workers.

**Tech stack:** Existing Rust dependencies, Win32 controls, GitHub Actions, Inno Setup.

**Spec:** `docs/APPLICATION-REVIEW-2026-10-22.md` and the approved Windows 11 direction: standard caption buttons, compact toolbar, uniform settings background, clear sections and adaptive scrolling.

## Constraints

- Preserve navigation allowlists, update verification and user data.
- No unit/functional tests, player launches or installer launches.
- Preserve the existing Win32 theme model; no WinUI 3 or automatic dark-mode migration.
- Apply relevant `winui-app` design guidance without new application dependencies.

## Tasks

- [x] R1: generation-scoped failure when a required blocker disable cannot be confirmed.
- [x] R2/R8: cache audio sessions within a gesture, save once at its end, report write errors.
- [x] R3: measure settings text, clamp to the monitor, add resizing/scrolling/focus visibility.
- [x] R4: helper verifies/pins Setup and backs up the player before acknowledging readiness.
- [x] R5/R6/R7: enforce tag/version equality, protect active update files during retention, distinguish update phases.
- [x] UI: replace group boxes with headings, unify surfaces, improve typography/spacing and narrow-window layout.
- [x] Complete diff/documentation review and formatting/lint/build/audit/secret checks.
- [ ] Publish 0.4.3, inspect Windows CI, compare remote commit and installer digest.

## Review focus

- Stale callbacks must not stop a replacement controller.
- COM/GDI owners must outlive their users without RefCell borrows spanning native calls.
- Preparation failures must leave the original player open.
- Cleanup must preserve active operations, unknown contents and reparse points.
- Long labels, high DPI and short viewports must retain reachable settings.

Static checks cannot establish visual quality, runtime responsiveness, audio/DRM behavior or accessibility conformance.
