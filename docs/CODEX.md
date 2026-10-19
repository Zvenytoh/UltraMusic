# Codex setup

UltraMusic keeps portable project instructions in [`AGENTS.md`](../AGENTS.md).
Codex reads them when opening the repository. Detailed Rust, build and release
conventions remain in the [contribution guide](../CONTRIBUTING.md) and
[architecture notes](ARCHITECTURE.md).

## Repository tools

- **Graphify** is installed as a project skill under `.codex/skills/graphify`.
  Use it for broad architecture questions or when tracing relationships across
  modules. Its generated `graphify-out/` data stays local and is ignored by Git.
- **RTK** is installed in the developer's Codex profile to shorten supported
  terminal output. Its Codex hook applies to Bash calls; in PowerShell, invoke
  `rtk` explicitly for supported commands when it is available.
- The project does not run unit or functional tests unless the user explicitly
  asks. Format, lint and build checks remain appropriate when the change calls
  for them.

## Optional user-level skills and plugins

The following are installed for this developer's Codex profile and are not
required by other contributors:

- **Ponytail** and **Ponytail Review** for reducing unnecessary code and
  reviewing changes.
- **Superpowers** for structured planning and debugging. Its general workflow
  follows the repository-specific rules in `AGENTS.md`, including the explicit
  test policy and the instruction not to use subagents unless requested.
- **GitHub** from the Codex plugin catalog for repository, issue and CI work;
  Codex may request account connection on first use.

Codex Desktop may need a restart to load newly installed skills and plugins.
