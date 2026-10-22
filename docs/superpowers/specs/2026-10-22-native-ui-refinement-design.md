# Native Windows UI refinement — proposed design

Status: approved by the user on 2026-10-22; execution in the current checkout.

## Intent

Replace the inconsistent 0.4.1 presentation with a compact, coherent Windows interface. Keep the genuine Windows caption and its minimize, maximize/restore and close controls. The requested native appearance remains the design constraint; do not return to a simulated caption.

## Findings from static inspection

- `chrome.rs` creates plain BUTTONs without assigning the system UI font with WM_SETFONT. Their fixed widths are unrelated to text metrics.
- The main toolbar uses COLOR_BTNFACE while settings forces black DWM colors and draws its own labels, buttons and volume slider.
- The volume slider is painted artwork with pointer handling rather than an accessible native trackbar.
- The host does not explicitly request common-controls version 6 in build.rs. Verify the resulting executable's activation manifest before claiming that visual styles are active.
- Compilation of 0.4.1 did not verify its visual appearance. Do not equate static checks with visual acceptance.

## Proposed presentation

Keep the standard Windows title bar, title and application icon. Below it, use a genuine Win32 toolbar, 44 logical pixels high, with a 32 × 32 Back icon button, a provider dropdown with its current name, and a 32 × 32 Settings icon button aligned to the right. Use 8 logical pixels at the edges and between controls. These two icon buttons share icon size (16 logical pixels), button size, theme, focus and hover treatment. The provider needs a wider label area; size it from text metrics instead of clipping it to an arbitrary width.

Use monochrome Windows system glyphs for Back and Settings; use the provider's text name and a native dropdown arrow. Keep the red brand accent in the application identity, rather than painting toolbar controls red. Native menu entries show the current provider with a standard check mark.

Use the system message font and system palette throughout the host UI. Explicitly activate common-controls version 6. Do not force a black title bar or undocumented dark-mode APIs: the host follows supported Windows styling and high-contrast colors. The websites retain their own themes.

Rework settings as a modeless native window with three clearly spaced groups: Audio, YouTube protection, Application. Use standard labels, a real volume trackbar with a percentage label, native checkboxes for mute, rendering compatibility and blocker preference, and a native update button. Keep explanatory text beside its associated option. Keep the existing French UI labels; README remains English. Use 16 logical pixels of outer padding, 8 pixels between related controls and 16 pixels between groups. Size labels with the UI font; wrap explanations. Preserve keyboard navigation, accessible names, F6, Ctrl+, Escape and close/minimize to tray.

Back is disabled until the current WebView reports history; synchronize it through generation-scoped HistoryChanged callbacks, with no polling. Navigation allowlists continue to apply.

## Boundaries

- Windows 11 x64; Rust/Win32/WebView2; no new framework or dependency.
- Preserve provider sessions, blocker preferences, audio settings and update integrity.
- Keep changes in the owning native modules. Localize and document unsafe calls and release GDI/image-list resources after their controls.
- No unit or functional test runs, application launches or installer launches under the current user instruction. Use static inspection, formatting, Clippy and build checks. Visual acceptance remains pending unless the user explicitly authorizes a visual run.
- No subagents. Implement in the current checkout after plan review.
- Publish a new installer release only after implementation and successful static checks and CI packaging; retain the sole user download UltraMusic-Setup.exe.

## Acceptance

The host has one genuine Windows caption, one compact themed toolbar, consistently sized icon actions and readable system text. Settings uses actual Windows controls rather than a mixture of painted black widgets and themed gray buttons. DPI changes rebuild fonts and icons and relayout without overlap. No caption hit-testing overrides or page scripts are introduced. Report exactly which checks ran and explicitly distinguish visual acceptance from compilation.
