<div align="center">
  <img src="assets/logo.png" width="112" height="112" alt="UltraMusic logo">
  <h1>UltraMusic</h1>
  <p><strong>Your music. A native Windows home.</strong></p>
  <p>Apple Music and YouTube Music in a focused desktop player.</p>
  <p>
    <a href="https://github.com/Zvenytoh/UltraMusic/releases/download/v0.2.2/UltraMusic-Setup.exe"><img src="https://img.shields.io/badge/Download_for_Windows-Install_UltraMusic-E32636?style=for-the-badge" alt="Download UltraMusic for Windows"></a>
  </p>
  <p>Windows 11 · x64 · Version 0.2.2</p>
  <p>
    <a href="https://github.com/Zvenytoh/UltraMusic/actions/workflows/windows.yml"><img src="https://github.com/Zvenytoh/UltraMusic/actions/workflows/windows.yml/badge.svg?branch=main" alt="Windows build"></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-native-DEA584?logo=rust&amp;logoColor=white" alt="Built with Rust"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-E32636" alt="MIT license"></a>
  </p>
  <p>
    <a href="#installation">Installation</a> ·
    <a href="#everyday-controls">Controls</a> ·
    <a href="#updates">Updates</a> ·
    <a href="CONTRIBUTING.md">Contributing</a> ·
    <a href="https://github.com/Zvenytoh/UltraMusic/issues">Support</a>
  </p>
</div>

---

## Music, with less in the way

UltraMusic brings the official Apple Music and YouTube Music websites into a
native Windows application. Sign in to your provider, open your library and
keep listening while the window is tucked away in the notification area.

The host is written in **Rust**, uses **Win32** for its window and tray menu,
and runs the music service through **Microsoft Edge WebView2**. It shares the
installed WebView2 runtime rather than shipping a separate browser engine.

**Keep the audio. Stop the unnecessary work.**

## Features

- **Native desktop experience** — a black title bar, red application identity,
  Start menu shortcut and standard Windows uninstall entry.
- **Background listening** — close or minimize the window to keep the player
  available in the notification area.
- **Controls within reach** — play/pause, application volume and mute from the
  tray menu, without bringing the player window forward.
- **Persistent sign-in** — the same local WebView2 profile is reused between sessions.
- **One active instance** — opening UltraMusic again restores the existing window.
- **In-app updates** — a startup check, explicit installation confirmation and
  verified downloads from this repository's releases.
- **Focused architecture** — an event-driven message loop, with no periodic song
  polling, animated overlay or custom frontend framework.

## Installation

1. **[Download UltraMusic Setup](https://github.com/Zvenytoh/UltraMusic/releases/download/v0.2.2/UltraMusic-Setup.exe)**.
2. Run `UltraMusic-Setup.exe` and follow the installation wizard.
3. Open **UltraMusic** from the Start menu.

Setup installs the application for your Windows account. A desktop shortcut is
optional. If Microsoft Edge WebView2 is missing, setup installs its Evergreen
runtime using Microsoft's bootstrapper. An Internet connection is required.

Remove the application through **Windows Settings → Apps → Installed apps**.
Your music service session data is kept separately from the program files.

[Release notes and downloads](https://github.com/Zvenytoh/UltraMusic/releases/tag/v0.2.2)

## Everyday controls

Right-click the UltraMusic icon in the Windows notification area:

| Control | Action |
| :--- | :--- |
| Open UltraMusic | Restore the existing player window |
| Apple Music / YouTube Music | Switch provider and remember the selection |
| Play / Pause | Toggle playback on the service's accessible media player |
| Mute | Mute or unmute the WebView's audio |
| Volume −10% / +10% | Adjust UltraMusic's audio sessions, rather than system master volume |
| Check for updates | Request an additional update check |
| Restart player | Recreate the WebView while retaining the local profile |
| Compatibility: keep rendering | Keep the WebView controller visible while its parent is hidden |
| Quit | Stop the player and close the application |

Double-click the tray icon to restore the window. UltraMusic must remain running
in the notification area for background audio and tray controls to work.
Playback commands depend on the media elements exposed by the provider's page;
mute and volume use WebView2 and Windows audio interfaces respectively.
Your volume and mute preferences are remembered between launches.

## Choose your service

Right-click the UltraMusic icon in the Windows notification area and choose
**Apple Music** or **YouTube Music**. The selected service has a check mark and
is remembered for future launches. Switching interrupts playback and opens the
chosen service in the player window; your sign-in sessions are preserved.

YouTube Music is used on first launch. Command-line options override the saved
choice for that launch; select a service in the tray menu to save a new default.

```powershell
# Apple Music
& "$env:LOCALAPPDATA\Programs\UltraMusic\UltraMusic.exe" --apple

# Keep rendering enabled when the window is hidden
& "$env:LOCALAPPDATA\Programs\UltraMusic\UltraMusic.exe" --keep-rendering
```

Quit the active instance before using a command-line override. A second launch
restores that instance instead of changing its provider. Sign-in, subscriptions, DRM
and available playback features are managed by the music service.

## Updates

UltraMusic checks published releases once when it starts. If a newer compatible
version is available, it asks before downloading and installing it. You can also
check manually from the tray menu.

Downloads are restricted to the official repository and GitHub's delivery hosts.
The installer size and SHA-256 digest are verified before it runs. After you
confirm, UltraMusic downloads the update, closes the player, runs setup and
restarts with the selected service and compatibility option. Playback is
interrupted during installation; the service profile is preserved.

There is no recurring background update timer. Temporary update files and the
previous executable are stored under `%LOCALAPPDATA%\UltraMusic\Updates`.

## Background behavior

When hidden, UltraMusic hides the native window and normally sets the WebView2
controller's visibility to false. The compatibility option keeps the controller
visible. The page is never suspended automatically, and hardware acceleration
keeps the runtime's default configuration.

The music provider still performs its own JavaScript, network and media work.
UltraMusic avoids adding continuous polling or rendering work in its host process;
actual resource usage depends on the provider and the runtime.

## Privacy

- Authentication takes place on the official service pages.
- The Rust host does not collect or handle your passwords.
- Cookies and site data stay in `%LOCALAPPDATA%\UltraMusic\WebView2`.
- The chosen service is saved locally in `%LOCALAPPDATA%\UltraMusic\service.txt`.
- No custom analytics or application log files are produced by the host.
- Remote pages have no Rust host objects or native message bridge.
- External HTTPS links opened by the user require confirmation.
- Update checks contact the public GitHub API; downloads use GitHub delivery hosts.

To reset a service session, quit UltraMusic and remove only its `WebView2`
profile folder. Music providers, WebView2 and the Microsoft runtime installer
operate under their own privacy policies.

## Build from source

Requirements: Rust stable, the MSVC C++ build tools, a Windows SDK and Windows 11
x64. Building the installer additionally requires **Inno Setup 6**.

```powershell
git clone https://github.com/Zvenytoh/UltraMusic.git
cd UltraMusic
cargo build --release --locked
.\target\release\ultramusic.exe

# Build the Windows installer
.\scripts\build-installer.ps1
```

The MSVC target statically links the C runtime. The application icon and WebView2
loader are embedded. GitHub Actions builds the app and installer, then publishes
tagged versions to GitHub Releases.

```text
src/
├── main.rs             Entry point and update-helper dispatch
├── config.rs           Service selection and navigation policy
├── native/             Win32 window, WebView2, tray, theme and audio controls
└── update/             Release discovery, HTTPS transport and setup coordination
installer/              Windows installer definition
assets/                 Application identity and icons
docs/                   Architecture and design references
scripts/                Development and packaging helpers
```

See the [architecture](docs/ARCHITECTURE.md), [visual identity](docs/BRANDING.md)
and [contribution guide](CONTRIBUTING.md) for implementation details.

## Contributing and support

Bug reports and focused contributions are welcome. Open an
[issue](https://github.com/Zvenytoh/UltraMusic/issues) with your Windows version,
music provider, UltraMusic version and the steps needed to reproduce the problem.
Never include passwords, cookies or a copied WebView2 profile.

## License

UltraMusic is released under the [MIT license](LICENSE). The original project
artwork is included under the same license.

UltraMusic is independent of Apple, Google, YouTube and Microsoft. Service names
and trademarks belong to their respective owners.
