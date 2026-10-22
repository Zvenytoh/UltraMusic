# UI verification — 22 October 2026

This record covers the rendering corrections in UltraMusic 0.5.1. The user
explicitly authorized targeted manual UI checks after supplying screenshots of
0.5.0. Earlier reports that excluded opening the player remain historical.

## Problems reproduced and corrected

- GDI painting into the extended DWM frame produced an additive gray title
  surface. Painting opaque pixels across the caption area also hid Windows'
  buttons. Buffered painting now makes the host opaque and exposes only the
  native caption area.
- The native toolbar's list/mixed-button layout retained approximately 24 px
  item widths. Its allocated width left a white strip at the end. The toolbar
  now uses icon-only uniform buttons, explicit width limits, and its measured
  final item rectangle. Labels remain available to accessibility and tooltips.
- The header is 44 logical pixels high, with 40 px wide, 32 px high navigation
  targets and 16 px vector icons. Their vertical centre follows the Windows
  caption bounds, including the clipped resize-frame inset when maximized.
- Font glyphs were replaced by supersampled vector strokes. Settings headings
  use a smaller, lighter Segoe font, and appearance choices use native radio
  controls with a segmented presentation.
- The volume track/thumb could disappear after appearance changes. Both are
  now painted from their native trackbar geometry on every prepaint.

## Manual observations

The local Windows GNU release build was opened and inspected with the desktop
automation tool. The final candidate displayed version 0.5.1 in Settings.

| Check | Observed result |
| --- | --- |
| Normal window, dark host | Uniform dark header, thin spaced icons, no trailing white strip; navigation and caption icons share their vertical axis. |
| Maximize, then Restore | Native buttons responded; the icon alignment remained consistent in both states after correcting the maximized inset. |
| Settings icon and Ctrl+, | Opened the settings window. Its own close button closed it. |
| Dark → Light → Dark | Host colors changed without restarting; the volume slider stayed visible. The music page kept its own appearance. |
| Appearance persistence | Light survived an earlier candidate restart; dark was restored for the final candidate. |
| Service icon | Opened Apple Music / YouTube Music / YouTube; Apple Music was checked. Escape dismissed the menu without switching providers. |
| Toolbar accessibility names | Windows exposed Back/Forward as disabled buttons and the current provider/Settings as named buttons, despite no visible labels. |
| Minimize and second launch | An earlier rendering candidate hid its window while retaining the process; another launch restored that same instance. |
| Main Close | An earlier rendering candidate hid to the tray and retained its process, as intended. |
| Settings layout | All sections and the update button fit the normal opening size without a vertical scrollbar. A small width reduction retained readable layout. |

The final header spacing change was rechecked for normal/maximized alignment,
Restore, the service menu, named controls, Settings, and Light/Dark switching.
The earlier tray checks were not repeated after that geometry-only change.

## Static checks

`scripts/dev.ps1 fmt`, `clippy`, `audit`, and `build` passed locally on the final
source. Clippy used `--all-targets -- -D warnings`; the release build used
`--locked`. The audit scanned 78 dependencies against 1,349 RustSec advisories
without reporting vulnerabilities. The pinned blocker archive SHA-256 was
verified while preparing the build. Staged whitespace checks and Gitleaks
completed without findings before publication. The tagged Windows workflow supplies separate
MSVC build and installer packaging evidence.

## Limits

No unit tests or full functional-check script were run. No installer or in-app
update installation was executed. These checks do not establish audio output,
DRM playback, ad-blocking effectiveness, performance, account/session
persistence, screen-reader conformance, high-contrast behavior, multiple
monitors or high-DPI rendering. Larger settings resize/scroll breakpoints were
not exercised. The provider page was signed out during the final checks; no
authentication was automated and no profile or cookies were deleted.
