# Legacy updater compatibility — 19 October 2026

## Observed failure

The failed update used helper version 0.2.3 to launch installer version 0.3.0.
The retained installer log records a `/DIR` value with the Windows extended
local-drive prefix (`\\?\C:\…`), followed by the invalid-folder-character error
shown by the user. The `/LOG` file was successfully created.

The Rust helper has converted those paths since 0.2.4. That fix cannot run until
an older installation successfully updates; shipping it only in the application
left existing 0.2.3 helpers unable to install the new executable.

## Change

Installer 0.3.1 normalizes this legacy directory input in `InitializeWizard`,
before the directory page is validated, including in silent mode. It removes
only the exact extended prefix on a letter-based absolute local drive path.
Ordinary paths are unchanged. UNC, device and malformed paths retain normal
Setup validation. The installer retains its AppId, destination, permissions,
profile preservation and existing installation flow.

Reference: Inno Setup 6.7.1 source, `Setup.MainFunc.pas` (`InitializeWizard`
before `ClickThroughPages`) and `Setup.WizardForm.pas` (`DirEdit.Text` read and
validated during page processing):
[upstream source](https://github.com/jrsoftware/issrc/tree/is-6_7_1/Projects/Src).

## Verification scope

Review the incoming directory conversion and installer event order. Compile
the installer and run formatting, locked Cargo checks, Clippy and release
compilation. Verify release CI and the public installer's digest after publishing.
No unit or functional tests, installer execution or installed-app replacement
are part of this verification, following the existing project instruction.
