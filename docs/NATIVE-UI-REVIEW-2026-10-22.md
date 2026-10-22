# Native UI review — 2026-10-22

Version: 0.4.2. Scope: Windows host presentation, settings controls and navigation state.

## Changes

- Explicit common-controls v6 activation, asInvoker execution and PerMonitorV2 awareness in the embedded application manifest.
- Native flat toolbar: matching Back/Settings icon actions, text-measured provider dropdown, flexible spacing and system tooltips. The standard caption remains handled by Windows.
- System message fonts and premultiplied alpha system glyphs, with owned GDI/image-list resources and replacement before old resources are released.
- Native settings labels, groups, checkboxes and a keyboard-accessible volume trackbar. Settings keeps a separate font so it can follow its own monitor DPI.
- Generation-scoped HistoryChanged updates Back; no recurring polling. Opening settings preserves a maximized main window.

## Static review

Reviewed toolbar sizing at minimum window width and DPI scaling formulas; native caption/resize messages are not intercepted. Checked double-null toolbar string buffers, synchronous pointer lifetimes, controller generation checks, image/font replacement and partial initialization cleanup. Checked settings command routing, checkbox state restoration after errors, trackbar range and focus order. Confirmed the removed drawing/theme modules have no remaining callers.

The toolbar is one tab stop: F6 enters it, arrows select actions, Space activates, and Tab/Escape returns to the player. This uses the native toolbar interaction model instead of tabbing through the old separate buttons.

Review was performed by the implementer. No subagents were used, following project instructions.

## Checks performed locally

- Cargo formatting check: passed.
- Cargo Clippy (`--locked --all-targets -- -D warnings`): passed; this compiles targets without executing tests.
- Cargo release build (local Windows GNU toolchain): passed.
- Static RT_MANIFEST #1 extraction from the built executable, loaded only as a data file: common-controls v6, asInvoker and PerMonitorV2 confirmed.
- Bundled uBO Lite archive: existing pinned SHA-256 verified by the build script.
- Staged diff secret scan: no leaks reported by Gitleaks.

Windows MSVC compilation, audit, installer packaging and release publication are handled by the tagged GitHub Actions run. Its result is the publication record.

## Limits

No unit or functional tests, application launch or installer launch were performed. Caption interactions, toolbar rendering, keyboard behavior and DPI transitions were reviewed statically; they were not observed at runtime. Audio, DRM, blocker effectiveness and performance were not validated by this change.

## Platform references

- [Microsoft: enabling visual styles](https://learn.microsoft.com/en-us/windows/win32/controls/cookbook-overview).
- [Microsoft: native toolbar controls](https://learn.microsoft.com/en-us/windows/win32/controls/using-toolbar-controls).
- [Microsoft: toolbar button sizing](https://learn.microsoft.com/en-us/windows/win32/controls/tb-setbuttonsize).
