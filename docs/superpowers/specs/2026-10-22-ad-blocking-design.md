# YouTube-focused ad blocking

Status: approved by the user on 22 October 2026; implementation authorized.

## User intent

Add effective ad blocking to UltraMusic, prioritizing video/audio advertisements
on YouTube and YouTube Music. Preserve existing provider sessions, playback
controls and the lightweight native host. Existing authorization excludes unit
and functional tests; compatibility and effectiveness cannot be called validated
without changing that constraint.

## Proposed approach

Evaluate embedding a pinned official Chromium build of uBO Lite using WebView2's
browser-extension APIs. This is a candidate integration, not a claim that the
extension already works in UltraMusic. A small hand-maintained domain list is
insufficient for the requested YouTube-focused blocking.

- Bundle the reviewed extension in the installer, with its license, notices and
  corresponding source reference. Verify the upstream archive digest during
  packaging; do not download or execute arbitrary extensions supplied by pages.
- Initialize extension support before creating the WebView environment. Install
  or reconcile the bundled blocker before navigating to the selected service.
  Preserve the existing profile and generation-scoped asynchronous callbacks.
- Scope advanced filtering to YouTube and YouTube Music. Do not introduce
  unrelated changes to Apple Music or the service navigation allowlists.
- Add a remembered blocking switch to native settings and show whether the
  blocker is initializing, installed/enabled, disabled or unavailable. These
  states describe extension initialization, not proof that ads were removed.
- Provide a reversible disable-and-reload path if a provider fails to play.
  Failure to initialize blocking must leave the player usable and visibly report
  that protection is unavailable rather than claiming it is active.
- Deliver pinned blocker/rules updates through UltraMusic releases initially.
  Never add an unrestricted remote-code updater. The browser may run the
  blocker's bundled content scripts; this is a specific exception to the host's
  previous avoidance of persistent page scripts, requiring approval of this design.

## Feasibility checks before committing to the candidate

Inspect the chosen package's required extension APIs, permission and filtering
configuration, license obligations, and WebView2's extension UI limitations.
WebView2 supports loading extensions but does not provide ordinary browser
toolbar entry points; UltraMusic must expose the necessary controls itself.
If the required behavior cannot be integrated, report that limitation rather
than substituting a token blocker and calling it powerful.

No promise of permanent or complete YouTube ad removal is made. Site changes,
anti-adblock behavior and runtime compatibility may affect filtering. DRM and
paid account entitlements remain unchanged.

## Verification boundary

Source review, formatting, Clippy, dependency/packaging integrity checks and
Windows application/installer builds remain authorized. Do not launch the
player, test accounts, click playback or claim observed ad-blocking performance
under the current no-functional-test instruction. Installation success alone
does not prove effective YouTube filtering.

## Primary references

- [WebView2 extension APIs](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2profile7)
- [WebView2 extension design](https://github.com/MicrosoftEdge/WebView2Feedback/blob/main/specs/Extensions.md)
- [Official uBO Lite project](https://github.com/uBlockOrigin/uBOL-home)
