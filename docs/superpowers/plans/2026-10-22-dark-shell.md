# Dark native shell implementation plan

**Approved design:** Codex-inspired charcoal surfaces, a single compact navigation/title row with Back, Forward, service picker and Settings, native caption buttons, Segoe typography, persistent Dark/Light/System selection with Dark as the default. Music providers retain their independent page settings.

**Implementation:** Keep Rust/Win32 and WebView2. Add focused theme and frame modules; retain native input controls and customize only their painting. DWM owns caption buttons. No undocumented dark-mode exports or page scripts.

- [x] Centralize palettes, preference persistence, high-contrast fallback and DWM colors.
- [x] Unify navigation and title frame, preserve resize/drag/native caption hit testing, add forward history.
- [x] Refine settings typography and controls; expose theme selection without restarting playback.
- [x] Review lifetimes, keyboard access, DPI handling and update compatibility.
- [ ] Run formatting, lint, build, audit and secret checks; publish an installer through Windows CI.

No subagents, unit/functional tests, application launches or installer launches. Screenshots supplied by the user are baseline evidence, not runtime validation of these changes.

## Local verification record

22 October 2026, version 0.5.0: formatting, Clippy (`--all-targets -- -D warnings`)
and the local Windows GNU release build passed. RustSec scanned 78 dependencies
against 1,349 advisories without reporting a vulnerability. The blocker package
SHA-256 was checked by the build helper. All five PowerShell scripts parsed;
`smoke.ps1` was not executed. No new Cargo package was added: existing `windows`
features were enabled for system appearance and high-contrast APIs.

Caption handling follows Microsoft's DWM custom-frame guidance; palettes and
native custom-draw controls were reviewed statically. Runtime rendering,
maximized caption interactions, keyboard/screen-reader behavior and performance
remain unvalidated. Tagged MSVC/installer CI and release integrity are checked
after publication; this plan does not substitute for their actual results.
