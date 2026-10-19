# Native header implementation plan

> For agentic workers: use superpowers:executing-plans inline. The user approved
> execution; AGENTS.md forbids subagents unless requested. Existing authorization
> covers main, pushes and releases. No additional approval handoff is needed.

**Goal:** Deliver the approved native service header and settings window.
**Architecture:** Win32 child buttons and event-driven GDI drawing live in
focused native modules. Service, audio and update commands are shared by tray,
header and settings. One existing WebView remains active.
**Tech stack:** Rust, Win32, DWM, GDI, WebView2; no new frontend framework.
**Spec:** ../specs/2026-10-19-native-header-design.md

## Global constraints

56 logical-pixel black header; three services; embedded outline icons; native
window behavior and existing security policy retained. No unit/functional tests
or subagents, as required by the project. Verification uses source review,
format/check/Clippy/audit and Windows builds instead of TDD.

## Review focus

DPI transitions and maximized edges; caption buttons and Snap hit testing;
keyboard focus and settings ownership; stale callbacks during service switches;
legacy service preferences, CLI flags and update restarts.

## Tasks

- [x] Add YouTube video service in config.rs; preserve `youtube` as YouTube Music,
  add `youtube-video`, labels and restart argument; update helper argument validation.
- [x] Move shared tray commands into native/commands.rs; use the same service
  command IDs and actions for all native entry points.
- [x] Add native/drawing.rs for scoped GDI rendering, native/chrome.rs for frame,
  child buttons and service menu, and native/settings.rs for the owned settings
  window. Connect window dispatch, WebView bounds, keyboard handling and cleanup.
- [x] Synchronize service, volume, mute, rendering compatibility and update status
  from existing actions; preserve preferences and close-to-tray behavior.
- [x] Review all callers and the five focus areas. Run fmt, check, Clippy, audit
  and release build. Update documentation/version. No runtime visual/audio claims.

Publication follows through the `v0.3.0` tag: push main and tag, inspect GitHub
Actions and verify the public installer digest before reporting completion.

## Execution record

- Ruling: execute inline in the existing checkout, following the user's go-ahead
  and established main/publish authorization. No new approval round or worktree.
- Ruling: use native vector outline icons instead of a React package; this keeps
  the approved appearance within the existing Rust/Win32 architecture.
- Verification: format, locked check, Clippy with warnings denied, RustSec audit
  (78 dependencies) and local Windows GNU release build succeeded. Gitleaks found
  no secrets in the staged changes. No unit or functional tests were run.
