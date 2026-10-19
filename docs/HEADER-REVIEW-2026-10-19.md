# Native header review — 19 October 2026

Scope: UltraMusic 0.3.0, one user and one active WebView2 controller.

## Source review

- Traced service selection from the header and tray through the shared command
  dispatcher, preference write, generation invalidation and controller recreation.
  Selecting the active service restores it without recreating the player.
- Preserved `youtube` / `--youtube` for YouTube Music; followed the new YouTube
  flag through configuration, update coordination and the helper's explicit whitelist.
- Reviewed the new exact-host HTTPS navigation policy alongside the existing
  blocked downloads, denied sensitive permissions and external-link confirmation.
  Remote pages receive no new native bridge or access to header commands.
- Checked header/WebView bounds, DPI conversions, maximized frame insets,
  minimum width and DWM-first caption hit testing in the source. The browser
  leaves the resize edges exposed when the window is not maximized.
- Reviewed settings ownership, child-window cleanup, GDI object restoration,
  tooltip buffer lifetime, accessible button names and keyboard message routing.
  WebView keyboard callbacks defer UI work through generation-scoped messages.
- Kept one active player, event-driven drawing and the existing update integrity
  checks. No frontend runtime or additional crate dependency was introduced.

## Verification

Local Windows GNU checks: format, `cargo check --locked`, Clippy with warnings
denied, dependency audit and release compilation. The allowed commands do not
launch the player or installer. GitHub Actions separately builds the MSVC
application and Windows installer; its results are recorded with the release.

No unit tests or functional tests were run. This review does not establish
visual correctness at runtime, Snap behavior on a real display, provider sign-in,
audio, DRM, update installation or performance. Earlier validation records remain
historical and do not validate this version's interface.
