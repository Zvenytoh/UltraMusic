<#
.SYNOPSIS
Runs static checks or a release build with optional workspace-local tools.
.DESCRIPTION
Does not run tests. Restores the caller's environment after completion.
#>
param([ValidateSet('check', 'build', 'fmt', 'clippy')][string]$Task = 'build')
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$savedEnvironment = @{}
foreach ($name in @('RUSTUP_HOME', 'CARGO_HOME', 'PATH', 'RUSTFLAGS', 'CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER')) {
    $savedEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
}
$localToolchain = Test-Path "$root\.tools\cargo\bin\cargo.exe"
Push-Location $root
try {
    if ($localToolchain) {
        if (-not (Test-Path "$root\.tools\mingw64\bin\gcc.exe")) {
            throw 'The local toolchain requires .tools/mingw64/bin/gcc.exe.'
        }
        $env:RUSTUP_HOME = "$root\.tools\rustup"
        $env:CARGO_HOME = "$root\.tools\cargo"
        # GCC 16 manifest specs require a toolchain path without spaces.
        $mingw = (cmd /c 'for %I in (".tools\mingw64") do @echo %~sI').Trim()
        if ($mingw.Contains(' ')) { throw 'The local GNU build requires Windows short-path support.' }
        $env:PATH = "$root\.tools\cargo\bin;$mingw\bin;$env:PATH"
        $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "$mingw\bin\gcc.exe"
        $specs = Join-Path (Split-Path $mingw -Parent) 'rust-link.specs'
        "*endfile:`ncrtend.o%s`n" | Set-Content -LiteralPath $specs -Encoding ascii
        $env:RUSTFLAGS = "$env:RUSTFLAGS -C link-arg=-specs=$($specs.Replace('\', '/'))".Trim()
    }
    switch ($Task) {
        build { cargo build --release --locked }
        fmt { cargo fmt --all -- --check }
        clippy { cargo clippy --locked --all-targets -- -D warnings }
        check { cargo check --locked }
    }
    if ($LASTEXITCODE -ne 0) { throw "Cargo failed ($LASTEXITCODE)." }
    if ($Task -eq 'build' -and $localToolchain) {
        $loader = Get-ChildItem "$root\target\release\build\webview2-com-sys-*\out\x64\WebView2Loader.dll" |
            Select-Object -First 1
        if (-not $loader) { throw 'The GNU build is missing WebView2Loader.dll.' }
        Copy-Item -LiteralPath $loader.FullName -Destination "$root\target\release\WebView2Loader.dll" -Force
    }
} finally {
    foreach ($name in $savedEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($name, $savedEnvironment[$name], 'Process')
    }
    Pop-Location
}
