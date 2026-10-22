# Review corrections — UltraMusic 0.4.3

The [application review](APPLICATION-REVIEW-2026-10-22.md) records the 0.4.2 baseline. Its findings remain dated evidence; this document records the implementation changes.

| Finding | Change |
| --- | --- |
| R1 | A required blocker disable failure posts a generation-scoped error and stops the controller before navigation. Activation failures retain the explicit unavailable state. |
| R2 | Unchanged values are skipped. Audio sessions are cached within a slider gesture, with one rediscovery on a stale-session failure; preferences are saved at gesture end. |
| R3 | Settings text is measured at its monitor DPI. The resizable window is clamped to the work area and supports scrolling, wrapping and keyboard focus visibility. |
| R4 | A worker waits for helper readiness. The helper pins/verifies Setup and creates a backup before acknowledging readiness. The player then authorizes installation and closes. |
| R5 | Tagged CI requires the exact `v{Cargo package version}` before building/publishing. |
| R6 | Before downloading, retain the two newest previous operations plus active/unknown operations. Cleanup only considers known direct files, refuses reparse points and uses file-sharing guards. Failed partial downloads are removed where possible. |
| R7 | Checking, downloading and preparation have distinct labels. Service/rendering changes are disabled while preparation captures restart options. |
| R8 | Audio preference errors are reported. The current in-memory choice is retained, with a later save retry on a subsequent audio action. |

## Windows UI refinement

The user approved a Windows 11 direction in chat. The implementation retains real Windows caption buttons, system menus, a native toolbar and common-controls v6. Settings use a single system-colored surface, semibold headings, readable body text and spacing instead of group boxes. Help/version text uses the system secondary text color. Toolbar spacers do not draw separators; the service selector is bounded by available width. The main window can shrink to 360 × 300 logical pixels; the provider owns its remote page layout.

`settings_layout.rs` owns geometry/scrolling, `settings.rs` owns controls/state, `ui.rs` owns fonts/icons, and `update/retention.rs` owns file retention. No application dependency was added. The installed official `winui-app` skill supplied applicable layout, typography, control and accessibility guidance; this release remains Rust/Win32, not WinUI 3.

## Verification scope

Only static checks and packaging are permitted. See the release commit and GitHub Actions for build evidence. No unit/functional test, player launch, installer launch, visual inspection of the modified application, screen-reader exercise or CPU/RAM/audio/DRM measurement was performed. The UI is implemented but its final appearance and interaction are not runtime-validated.

Local checks on 22 October 2026: `scripts/dev.ps1 fmt`, `check`, `clippy` and
`build` passed. The release build used the Windows GNU toolchain and verified
the pinned blocker SHA-256. `audit` scanned 78 dependencies against 1,349 RustSec
advisories without reporting vulnerabilities. All five PowerShell scripts parsed
successfully; the functional-check script was not executed. Gitleaks inspected
the staged patch without reporting secrets, and `git diff --cached --check` passed.
Windows MSVC compilation, installer packaging and publication are delegated to
the tagged GitHub Actions run and must be checked separately.
