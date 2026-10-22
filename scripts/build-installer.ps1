<#
.SYNOPSIS
Builds the Windows setup executable from a completed MSVC release build.
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$compiler = Get-ChildItem -Path @(
    "${env:ProgramFiles(x86)}/Inno Setup */ISCC.exe",
    "$env:ProgramFiles/Inno Setup */ISCC.exe"
) -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1
if (-not $compiler) { throw 'Install Inno Setup 6 or later before building the installer.' }
$version = [regex]::Match((Get-Content (Join-Path $root 'Cargo.toml') -Raw), '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $version) { throw 'The Cargo package version is missing.' }
& (Join-Path $PSScriptRoot 'prepare-blocker.ps1')
$dependencies = Join-Path $root 'target/dependencies'
New-Item -ItemType Directory -Force $dependencies | Out-Null
$bootstrapper = Join-Path $dependencies 'MicrosoftEdgeWebview2Setup.exe'
Invoke-WebRequest 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $bootstrapper
$signature = Get-AuthenticodeSignature -LiteralPath $bootstrapper
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '(^|,\s*)O=Microsoft Corporation(,|$)') {
    throw 'The Microsoft WebView2 bootstrapper signature is invalid.'
}
& $compiler.FullName "/DAppVersion=$version" (Join-Path $root 'installer/UltraMusic.iss')
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup compilation failed.' }
