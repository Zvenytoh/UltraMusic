<#
.SYNOPSIS
Packages a release build with the executable at the archive root.
.PARAMETER IncludeLoader
Includes the WebView2 loader required by the local GNU build. MSVC builds embed it.
#>
param([switch]$IncludeLoader)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$manifest = Get-Content -LiteralPath (Join-Path $root 'Cargo.toml') -Raw
$versionMatch = [regex]::Match($manifest, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $versionMatch.Success) { throw 'The package version is missing from Cargo.toml.' }
$version = $versionMatch.Groups[1].Value
$targetRoot = [IO.Path]::GetFullPath((Join-Path $root 'target'))
$staging = [IO.Path]::GetFullPath((Join-Path $targetRoot 'portable-package'))
$archive = Join-Path $targetRoot 'release/UltraMusic-windows-x64.zip'
$executable = Join-Path $targetRoot 'release/ultramusic.exe'
if (-not (Test-Path -LiteralPath $executable)) { throw 'Build the release executable first.' }

# Only this project's package staging directory can be cleared.
$prefix = $targetRoot + [IO.Path]::DirectorySeparatorChar
if (-not $staging.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'The staging directory must remain inside this project target directory.'
}
if (Test-Path -LiteralPath $staging) { Remove-Item -LiteralPath $staging -Recurse -Force }
New-Item -ItemType Directory -Path $staging | Out-Null
Copy-Item -LiteralPath $executable -Destination (Join-Path $staging 'UltraMusic.exe')
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $staging
if ($IncludeLoader) {
    Copy-Item -LiteralPath (Join-Path $targetRoot 'release/WebView2Loader.dll') -Destination $staging
}
@'
UltraMusic @VERSION@ — Prototype Windows 11 x64

1. Extraire tous les fichiers du ZIP dans un dossier.
2. Double-cliquer sur UltraMusic.exe.

Le runtime Microsoft Edge WebView2 Evergreen doit être installé :
https://developer.microsoft.com/microsoft-edge/webview2/

La croix masque le lecteur dans la zone de notification.
Double-cliquer sur son icône pour le rouvrir.
Clic droit sur son icône > Quitter pour fermer complètement.

YouTube Music est ouvert par défaut. Pour Apple Music, lancer :
UltraMusic.exe --apple

Cette version est un prototype : connexion, DRM et lecture audio prolongée
restent à valider. Aucun gain CPU/GPU n'est annoncé comme mesuré.

Documentation et assistance : https://github.com/Zvenytoh/UltraMusic
'@ | ForEach-Object { $_.Replace('@VERSION@', $version) } |
    Set-Content -LiteralPath (Join-Path $staging 'LIRE-MOI.txt') -Encoding utf8
Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $archive -Force
Write-Output $archive
