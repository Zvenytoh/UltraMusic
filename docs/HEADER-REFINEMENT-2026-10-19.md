# Compact header refinement — 19 October 2026

The user reported missing minimize/maximize/close buttons and asked for a smaller,
more refined Apple-inspired toolbar. They explicitly chose Windows buttons on
the right and declined a visual/functional check; static verification only.
This supersedes the 56-pixel sizing in the original 0.3.0 header design.

## Changes

- The DWM frame now extends through the full header height instead of one pixel.
  DWM receives caption messages before application dispatch, including mouse and
  nonclient painting messages, rather than only hit testing. Native caption
  drawing, click handling and Snap hit testing remain Windows responsibilities.
- Header height: 40 logical pixels. Service selector: 188 × 28. Settings: 28 × 28.
  A smaller 20-pixel brand mark replaces the repeated brand text in the toolbar.
- Fine, antialiased Segoe Fluent Icons replace the hand-drawn GDI polygons.
  These use the Windows 11 system font; no Apple font or new dependency is shipped.
- The service menu uses 36-pixel rows, consistent icon alignment, quieter hover
  colors and restrained borders. Focus outlines and accessible names are retained.
- Minimum window width is 500 logical pixels. Browser bounds and custom hit
  testing share the same six-pixel resize edge and 40-pixel toolbar boundary.

## References and checks

Reviewed the frame/messaging changes against Microsoft's
[custom frame guidance](https://learn.microsoft.com/en-us/windows/win32/dwm/customframe)
and [Snap guidance](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-snap-layout-menu).
Glyph code points come from the official
[Segoe Fluent Icons reference](https://learn.microsoft.com/en-us/windows/apps/design/iconography/segoe-fluent-icons-font).

Verification is limited to source and caller review, formatting, locked Cargo
check, Clippy, dependency audit and Windows compilation/installer builds.
No application launch, button clicks, screenshot, unit test or functional test
is authorized for this revision. Static checks cannot establish the actual
appearance or behavior of caption controls on the user's display.
