# Native header and service selection

Status: approved by the user on 19 October 2026; implementation authorized.

## Intent

Make service selection and settings visible in the main window. The user asked
for a taller top bar, a workspace/service switcher with Lucide-like icons,
Apple Music and YouTube Music choices, consideration of ordinary YouTube, and
a polished settings entry point.

## Proposed appearance

- One black top bar, 56 logical pixels tall, scaled with Windows DPI.
- Left: the red UltraMusic logo and application name.
- Next: a rounded service button showing a 20-pixel outlined icon, current
  service name and downward chevron. Target height: 36 logical pixels.
- Right: a settings gear button, followed by window controls.
- Use Windows UI typography, comfortable padding, restrained red accents,
  visible keyboard focus and subtle hover/pressed states.
- Integrate the header with the window frame. Avoid a second application-title
  row above the service selector. Keep dragging, resizing, maximize/restore,
  Windows Snap behavior and close-to-tray behavior.

These dimensions and layout are implementation recommendations, not user-given
measurements. The implementation should remain native Rust/Win32, with embedded
outline icons similar to Lucide; the user allowed similar icons. React is not
required for this native host.

## Service switcher

The proposed menu has three choices: Apple Music, YouTube Music, and YouTube.
The current choice is marked. Adding ordinary YouTube in this revision is a
proposal based on the user's request to consider it.

Keep one active WebView. Switching stops the previous service, opens the chosen
service and remembers the selection. Browser sign-in data remains in the
existing profile. Repeated selection of the active service does not reload it.
Preserve existing service preferences and command-line options when adding the
third service. Keep the tray selector synchronized with the header.

Ordinary YouTube uses the official website. Extend only its explicit HTTPS
navigation/authentication allowlist; retain blocked page downloads, sensitive
permission restrictions, and external-link confirmation.

## Settings

The gear opens a compact native settings window owned by UltraMusic. Expose
the existing controls with clear labels:

- Playback volume and mute.
- Compatibility option to keep rendering while the player is hidden.
- Installed version and a check-for-updates button with operation status.

Audio changes apply immediately and remain saved. Closing settings does not
restart the player. Service accounts and subscriptions stay on provider pages.
Buttons and menus support keyboard navigation, accessible names, and tooltips
for icon-only actions.

## Implementation boundaries

Own the header and settings in focused native modules. Reuse current service,
audio and update actions from both the tray and main window. Resize WebView2
below the header so remote pages cannot cover or control native buttons.
Draw only in response to Windows paint/input events; add no rendering timer,
extra web frontend or background WebViews.

Review DPI changes, minimum window dimensions, stale WebView callbacks, menu
focus, and modal update dialogs. Follow the project's no-unit/no-functional-test
policy unless the user changes it. Use formatting, static analysis, dependency
audit, source review and Windows builds as the allowed checks; do not report
runtime visual or playback behavior as verified by compilation.
