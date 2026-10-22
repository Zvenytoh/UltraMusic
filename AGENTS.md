# UltraMusic — instructions for Codex

- Read the README and the relevant part of `docs/ARCHITECTURE.md` before editing.
- Keep changes small, idiomatic Rust, and within the module that owns the behavior.
- Keep `unsafe` localized and document its invariants. Avoid polling, permanent page scripts, and dependencies without a clear need.
- Preserve Windows, WebView2, navigation allowlists, update integrity checks, and existing user data.
- Prefer scoped terminal commands and RTK's compact command forms where available; request full output only when diagnosing a failure.
- The project has no unit or functional test runs unless the user explicitly requests them. Use requested formatting, lint, and build checks; report exactly what ran.
- Never claim runtime, audio, DRM, or performance validation unless it was performed.
- Use the project Graphify skill for architecture and relationship questions when a graph exists; keep generated graph output local.
- Do not use subagents unless the user asks for them.

- Manual UI checks were previously authorized on 2026-10-22. The latest instruction requires finishing without desktop control: do not launch the player or use UI automation unless explicitly reauthorized. Static checks and CI remain permitted. Never infer visual correctness from build success. Unit tests remain opt-in.
