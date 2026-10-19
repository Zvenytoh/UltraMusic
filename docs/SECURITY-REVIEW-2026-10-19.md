# Security and code review — 19 October 2026

Scope: the native Rust host, WebView2 callbacks, media commands, update transport
and helper, installer, build scripts and GitHub Actions. Expected use is one
interactive Windows account and one player instance.

## Changes made

- Convert canonical Windows paths to ordinary drive paths for Inno Setup's
  `/DIR` and `/LOG` arguments. Canonical paths remain in the helper's file
  comparisons. Reject unsupported network paths before closing the player.
- Deny browser requests for camera, microphone, location, clipboard reading,
  sensors, MIDI, file access, fonts, notifications and window management.
  Cancel page downloads. Streaming and the browser sandbox remain active.
- Bound audio-preference reads to 4 KiB, even if the file changes during reading.
- Include the HTTPS origin and any non-default port in external-link confirmation.
- Skip Windows audio-session enumeration when playback stops. Volume is still
  applied when audio starts and when the user adjusts it. No polling was added.
- Pin CI actions to commit IDs, disable persisted checkout credentials, and
  audit locked dependencies in CI. The audit executable is pinned to version
  0.22.2 and its SHA-256 is verified before extraction.

## Static verification

- RustSec cargo-audit 0.22.2: 78 locked dependencies scanned; no known
  vulnerability reported. Advisory database revision:
  `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`.
- Gitleaks 8.30.1: the Git history (13 commits) and the tracked/new project
  files were scanned with secret redaction. No secrets were reported in that
  scope. Ignored toolchains, binaries and local runtime caches are not project
  publication inputs.
- Formatting, Cargo check, Clippy with warnings denied, and a GNU Windows
  release build passed locally. The MSVC build and installer are checked in CI.

## Trust boundaries and limits

Navigation uses explicit HTTPS origins. Native host objects and WebMessages
are disabled; the playback script is a fixed string, with no page-supplied
code interpolation. Updates require the expected repository asset, an allowed
HTTPS redirect host, bounded size and matching SHA-256 before execution.

The public GitHub repository and its release metadata are trusted update
sources. A matching hash is not an independent publisher signature. These
checks do not protect against compromise of the publishing account or code
already running as the same Windows user. The per-user installer does not
request administrator rights.

WebView2 and the music providers remain external runtime dependencies. Keeping
the Evergreen runtime current is necessary. Provider pages can still perform
their normal network, JavaScript and media work.

No unit or functional tests, application launch, installer execution, audio/DRM
check, penetration test or performance benchmark was performed. A successful
static review and build do not establish those runtime properties or guarantee
the absence of vulnerabilities.
