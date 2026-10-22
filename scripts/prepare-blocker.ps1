<#
.SYNOPSIS
Prepares the pinned, unmodified uBO Lite package inside the workspace target folder.
#>
param([string]$Destination = '')
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$targetRoot = [IO.Path]::GetFullPath((Join-Path $root 'target')) + [IO.Path]::DirectorySeparatorChar
if (-not $Destination) { $Destination = Join-Path $targetRoot 'dependencies/blocker' }
$destinationPath = [IO.Path]::GetFullPath($Destination)
if (-not $destinationPath.StartsWith($targetRoot, [StringComparison]::OrdinalIgnoreCase) -or
    -not (@((Join-Path $targetRoot 'dependencies/blocker'), (Join-Path $targetRoot 'release/blocker')) -contains $destinationPath)) {
    throw 'Use target/dependencies/blocker or target/release/blocker as the blocker destination.'
}
$version = '2026.1021.1354'
$digest = '0ff92d3e33437e0fa389a16a3d4c3a486fbda08e33114e07859db1f58d789d0b'
$cache = Join-Path $targetRoot 'dependencies'
New-Item -ItemType Directory -Path $cache -Force | Out-Null
$archive = Join-Path $cache "uBOLite_$version.chromium.zip"
if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest "https://github.com/uBlockOrigin/uBOL-home/releases/download/$version/uBOLite_$version.chromium.zip" -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $digest) {
    throw 'The uBO Lite archive SHA-256 does not match the pinned version.'
}
$staging = [IO.Path]::GetFullPath((Join-Path $cache ('blocker-stage-' + [Guid]::NewGuid().ToString('N'))))
if (-not $staging.StartsWith($targetRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Invalid staging path.' }
try {
    Expand-Archive -LiteralPath $archive -DestinationPath $staging
    $manifest = Get-Content -LiteralPath (Join-Path $staging 'manifest.json') -Raw | ConvertFrom-Json
    if ($manifest.version -ne $version -or $manifest.manifest_version -ne 3) { throw 'Unexpected blocker manifest.' }
    foreach ($file in @('LICENSE.txt', 'js/background.js', 'rulesets/main/ublock-filters.json')) {
        if (-not (Test-Path -LiteralPath (Join-Path $staging $file) -PathType Leaf)) { throw "Missing blocker file: $file" }
    }
    # Both resolved targets were checked against the workspace target directory above.
    if (Test-Path -LiteralPath $destinationPath) { Remove-Item -LiteralPath $destinationPath -Recurse -Force }
    New-Item -ItemType Directory -Path (Split-Path $destinationPath -Parent) -Force | Out-Null
    Move-Item -LiteralPath $staging -Destination $destinationPath
} finally {
    if (Test-Path -LiteralPath $staging) { Remove-Item -LiteralPath $staging -Recurse -Force }
}
Write-Output "uBO Lite $version prepared; pinned SHA-256 verified."
