# Header refinement — 0.5.2

## Scope

The 0.5.1 header was 44 logical pixels high while the DWM caption controls
retained their native height. Aligning navigation with those controls did not
centre either group in the oversized header. The user's screenshot exposed
this defect after publication; the earlier visual assessment was insufficient.

This refinement keeps the native caption controls and the existing charcoal
appearance. The shared header height becomes 32 logical pixels, matching the
standard Windows title row. Navigation retains 40 × 32 px targets. Icons use
one logical pixel strokes; Settings has six teeth and a larger open centre.
The keyboard focus rectangle is inset by two logical pixels.

The single header height drives DWM frame extension, the draggable row and
WebView2's top bound. No profile, provider navigation, updater or audio behavior
is changed. No application dependency is added.

## Design review method

Impeccable's polish/craft guidance was used with the installed Windows UI skill:
preserve native affordances, remove disproportionate empty space, retain usable
targets, and check optical weight as well as alignment. The web-only detector
does not validate native Rust/GDI rendering and was not used as evidence.

## Verification boundaries

A normal-window candidate was observed before the user stopped desktop
automation. The subsequent request explicitly requires finishing without
desktop control; no more player launches, clicks, keyboard input or installer
execution are authorized for this task.

Formatting, Cargo check, Clippy, dependency audit, whitespace/secret inspection
and the tagged Windows build/installer workflow provide the static and
packaging evidence. No unit tests or functional-check script are run.

The final packaged version's maximized rendering, hover/focus behavior, multiple
DPIs, installer/update execution, audio, DRM and performance are not validated
by these checks. Successful compilation is not proof of visual quality.
