# Blocker and maximized caption review — 22 October 2026

Scope: UltraMusic 0.4.0, one Windows user and one active player. The user approved
the uBO Lite design and confirmed that caption clicks fail in a maximized window.
They kept the restriction against unit and functional tests.

## Source and packaging review

- Followed extension initialization from environment options through profile
  installation, enable/disable completion and initial navigation. Callbacks use
  the player generation; preference changes during initialization are serialized.
  Old controller extension references are released before a new controller starts.
- Checked saved-preference bounds, replacement writes and failure paths. Status
  never reports enabled unless the COM interface confirms it. Initialization
  failure leaves navigation available; extension status is not effectiveness proof.
- Apple navigation requests extension disablement before loading its website.
  No service allowlist, native host object or web message channel was added.
- The unmodified official package is pinned to 2026.1021.1354 and SHA-256 checked
  before extraction. Packaging verifies its manifest, license and core resources.
  Only two explicit build destinations beneath target/ may be replaced. Checked
  that the pinned upstream source tag exists; license/source notices ship with setup.
- Caption glyphs, accessible names and mouse hit testing refer to the actual
  native child rectangles. Root HTMAXBUTTON retains the Snap entry point; native
  BUTTON capture/release drives WM_SYSCOMMAND. Removed implicit DWM hit regions,
  which could disagree with the explicitly painted controls after maximization.

## Verification and limits

Local checks use fmt, locked Cargo check, Clippy with warnings denied, RustSec
audit, PowerShell parsing, release compilation and pinned package preparation.
RustSec covers the 78 locked Rust crates, not the entire upstream extension.
The upstream extension is trusted as the pinned dependency; its complete source
was not independently security-audited. No new Rust dependency was added.

GitHub Actions separately builds MSVC and the complete EXE installer. Publication
verification checks the release asset against its GitHub SHA-256 digest.

No player/installer launch, unit test, functional test, account login, caption
click, Snap interaction, observed ad removal or performance measurement was run.
Compilation cannot establish runtime compatibility or guarantee ad removal.
