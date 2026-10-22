# Native Windows UI Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking. No subagents are authorized.

**Goal:** Replace the inconsistent 0.4.1 host UI with a coherent native Windows toolbar and settings window.

**Architecture:** Keep WS_OVERLAPPEDWINDOW and default nonclient processing. Use common-controls version 6, a native toolbar and settings controls, with shared owned system-font and icon resources. Keep WebView callbacks generation-scoped and delegate actions through the existing command dispatcher.

**Tech Stack:** Rust 2021, windows 0.62, Win32 common controls, WebView2, existing winresource build dependency.

**Spec:** `docs/superpowers/specs/2026-10-22-native-ui-refinement-design.md` (approved by the user on 2026-10-22).

## Global Constraints

- Windows 11 x64; Rust/Win32/WebView2; no new framework or dependency.
- Preserve provider sessions, blocker preferences, audio settings and update integrity.
- No unit or functional test runs, application launches or installer launches under the current user instruction. Use static inspection, formatting, Clippy and build checks. Visual acceptance remains pending unless the user explicitly authorizes a visual run.
- No subagents. Implement in the current checkout after plan review.
- Keep the standard Windows caption; README remains English and existing UI labels remain French.

## Review Focus

- DPI 100%, 150%, 200% and monitor transitions: system fonts/icons must be rebuilt without dangling handles or clipped controls. Static lifecycle and geometry review belongs to Task 1.
- Minimum width and longest provider label: toolbar actions must not overlap. Text-metric and rectangle review belongs to Task 2.
- Empty history, service switch and stale callbacks: Back must reflect the live controller only. Generation and callback review belongs to Task 2.
- Keyboard and high contrast: each action must retain a native name, focus and appropriate enabled/checked state. Control/dispatch review belongs to Task 3.
- Maximized/hidden windows and owned settings: opening settings must preserve maximized state and closing/minimizing must preserve tray behavior. Window-message review belongs to Task 3.

These are static review obligations, not assertions that runtime behavior was tested. The user's prohibition on test runs overrides the skill's default TDD steps.

---

### Task 1: Activate native styling and own UI resources

**Files:** Create `assets/ultramusic.manifest`, `src/native/ui.rs`; modify `build.rs`, `src/native/mod.rs`.

**Interfaces:** `ui::Resources::new(hwnd: HWND) -> windows::core::Result<Self>` owns a system message HFONT and a two-icon HIMAGELIST. `Resources::apply_font(&self, child: HWND)` sends WM_SETFONT. App stores `RefCell<Option<ui::Resources>>`; resources outlive their controls. DPI replacement installs new handles before dropping old resources.

- [x] Request Microsoft.Windows.Common-Controls 6.0 with an asInvoker manifest; preserve any effective existing DPI/compatibility declarations. Embed it using the existing winresource dependency and register rerun-if-changed.
- [x] Implement Resources using NONCLIENTMETRICS for the system font at the window DPI. Rasterize Back and Settings Segoe Fluent Icons glyphs at 16 logical pixels into an owned image list; preserve transparency and selected GDI objects. Use no external icon font downloads.
- [x] Add resource initialization, DPI refresh and cleanup to App. Review partial-creation failures and teardown order; no live control may retain a deleted font/image-list handle.
- [x] Run `scripts/dev.ps1 check` and `scripts/dev.ps1 clippy`; require exit code 0 and no warnings. Inspect the built manifest resource to confirm v6 activation, rather than assuming its presence.
- [x] Commit the styling/resource foundation with its manifest and relevant architecture note.

### Task 2: Replace plain buttons with a compact native toolbar

**Files:** Modify `src/native/chrome.rs`, `src/native/mod.rs`, `src/native/webview.rs`, `src/native/commands.rs`, `src/native/window.rs`.

**Interfaces:** Preserve `App::create_chrome() -> Result<()>`, `layout_chrome(&self)`, `keyboard(&self, message: &MSG) -> bool`, `service_menu(&self)`, `go_back(&self)`. Add `App::refresh_navigation(&self)` to read CanGoBack and enable BACK. Replace the three individual toolbar HWND fields with one toolbar HWND; retain command IDs BACK, PICKER and SETTINGS. Task 1 resources supply toolbar font and icons.

- [x] Create TOOLBARCLASSNAMEW with native flat/list toolbar styles. Set Back and Settings to 32 × 32 logical pixels, glyphs to 16, toolbar height to 44 and margins/gaps to 8. Use native dropdown notification for the provider and measure its text with the shared font. Use a flexible separator to align Settings right. Preserve WS_OVERLAPPEDWINDOW and default caption handling.
- [x] Route toolbar commands and dropdown notifications to existing commands/service_menu. Replace Unicode text arrows with native glyph/dropdown presentation. Supply native tooltip/accessibility text and preserve F6, Tab, Space and Escape behavior for the toolbar control.
- [x] Implement refresh_navigation and generation-scoped HistoryChanged notification. Disable Back before controller creation and during service restart; update it on history changes and keep CanGoBack protection in go_back. Do not hold RefCell borrows across COM/Win32 calls.
- [x] Update WebView bounds to the 44-pixel toolbar height and remove the obsolete client resize-edge padding now that the resize border is genuinely nonclient. Review geometry at 500-pixel minimum outer width and DPI scales 100/150/200%, including the longest provider name; enlarge minimum width only if measured text requires it.
- [x] Run `scripts/dev.ps1 fmt`, `check`, `clippy` and `build`; require exit code 0. Statically trace all toolbar handles/commands and obsolete helper callers before deletion. Commit the toolbar and Back-state changes.

### Task 3: Make settings coherent and publish only verified artifacts

**Files:** Modify `src/native/settings.rs`, `src/native/theme.rs`, `src/native/drawing.rs`, `src/native/window.rs`, `src/native/mod.rs`, `README.md`, `docs/ARCHITECTURE.md`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/windows.yml`.

**Interfaces:** Preserve `App::open_settings(&self)` and `refresh_settings(&self)`. Replace painted settings buttons/slider with native STATIC, BUTTON checkbox, TRACKBAR controls. WM_HSCROLL uses the existing `App::set_volume(f32)`; checkbox commands reuse MUTE, COMPAT, BLOCKER. Font resources from Task 1 outlive settings controls. Remove theme/drawing helpers only after checking every caller.

- [x] Build the three groups defined in the spec, with 16-pixel outer/group spacing and 8-pixel related-control spacing. Use wrapped native explanation labels, a 0–100 volume trackbar and percentage label, checked states for the three preferences, installed version and update button. Apply the system font to all children and size text from its metrics.
- [x] Remove forced black settings caption colors and painted control interaction. Let system styling/high contrast apply to both windows. Keep the modeless ownership, native keyboard focus order and existing command/error paths.
- [x] In restore, select SW_RESTORE only for IsIconic; otherwise show without dropping maximized state. Review maximize, hide, tray restore and owned-settings destruction by tracing messages. Confirm the toolbar cannot intercept caption/resize hit testing.
- [x] Run fmt/check/Clippy/release build and `git diff --check`. Review resource cleanup, DPI handling, keyboard dispatch and generation invalidation against all five Review Focus items. Do not claim visual acceptance. If visual checking remains prohibited, record that explicitly rather than treating these checks as visual QA.
- [ ] After code review, bump the next available patch version, update English README/download links and release notes, and commit. Push main without force, tag that verified commit, follow GitHub Actions through installer/release success, verify remote SHAs and download the sole installer asset to compare its SHA-256 with release metadata. Do not run the installer. Report release link, commit and the remaining visual-validation limitation.

## Self-review

Spec coverage: native caption, coherent toolbar/settings, geometry, fonts, accessibility, history, constraints and publication each have an owning task. Interfaces retain existing entry points and define the shared resource/history methods once. Review Focus items are assigned to static inspections because unit/functional tests remain prohibited. No application code or release is changed by this plan itself.

Execution note: source tasks were integrated in one buildable commit (`ab3f908`), because replacing App fields affects both toolbar and settings callers. Local checks and the author’s static review are recorded in `docs/NATIVE-UI-REVIEW-2026-10-22.md`. The final publication step is tracked by the tagged GitHub Actions run.
