<#
.SYNOPSIS
Checks Cargo.lock against the current RustSec advisory database. Does not run tests.
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$version = '0.22.2'
$checksum = '0a7316540862c13d954f648917ceacca593747baed6eec180fafa590be2710ab'
$tools = Join-Path $root '.tools/security'
$archive = Join-Path $tools "cargo-audit-$version.zip"
$destination = Join-Path $tools "cargo-audit-$version"
New-Item -ItemType Directory -Force -Path $tools | Out-Null
if (-not (Test-Path -LiteralPath $archive)) {
    $url = "https://github.com/rustsec/rustsec/releases/download/cargo-audit/v$version/cargo-audit-x86_64-pc-windows-msvc-v$version.zip"
    Invoke-WebRequest -Uri $url -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $checksum) {
    throw 'cargo-audit archive checksum mismatch. Remove the archive and retry.'
}
# Re-extract the verified archive so the cached executable is never trusted on its own.
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
$executable = Join-Path $destination "cargo-audit-x86_64-pc-windows-msvc-v$version/cargo-audit.exe"
& $executable audit --file (Join-Path $root 'Cargo.lock')
if ($LASTEXITCODE -ne 0) { throw "Dependency audit failed ($LASTEXITCODE)." }
