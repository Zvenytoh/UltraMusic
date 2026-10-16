# Exercises only the freshly built UltraMusic instance; no account login/audio test.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
if (Get-Process ultramusic -ErrorAction SilentlyContinue) { throw 'Quit UltraMusic before running this test.' }
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class UltraMusicSmoke {
    [DllImport("user32.dll", CharSet=CharSet.Unicode)]
    public static extern IntPtr FindWindow(string cls, string name);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wp, IntPtr lp);
}
'@
function Wait-Condition([scriptblock]$Condition, [string]$Failure, [int]$Seconds = 15) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    while (-not (& $Condition)) {
        if ($watch.Elapsed.TotalSeconds -gt $Seconds) { throw $Failure }
        Start-Sleep -Milliseconds 100
    }
}
$exe = Join-Path $root 'target\release\ultramusic.exe'
$process = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
$window = [IntPtr]::Zero
$engineIds = @()
try {
    Wait-Condition { [UltraMusicSmoke]::FindWindow('UltraMusic.Native.1', 'UltraMusic') -ne [IntPtr]::Zero } 'Window missing'
    $window = [UltraMusicSmoke]::FindWindow('UltraMusic.Native.1', 'UltraMusic')
    # Start-Process requested a hidden startup; explicitly activate the test window.
    [void][UltraMusicSmoke]::PostMessage($window, 0x8002, [IntPtr]::Zero, [IntPtr]::Zero)
    Wait-Condition { [UltraMusicSmoke]::IsWindowVisible($window) } 'Window not visible'
    Wait-Condition { @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$($process.Id) AND Name='msedgewebview2.exe'").Count -gt 0 } 'WebView2 process missing' 30
    # Collect the process tree, without exposing command lines or profile data.
    $all = Get-CimInstance Win32_Process
    $ids = @($process.Id)
    do {
        $new = @($all | Where-Object { $_.ParentProcessId -in $ids -and $_.ProcessId -notin $ids } | ForEach-Object ProcessId)
        $ids += $new
    } while ($new.Count -gt 0)
    $engineIds = @($ids | Where-Object { $_ -ne $process.Id })
    [void][UltraMusicSmoke]::PostMessage($window, 0x10, [IntPtr]::Zero, [IntPtr]::Zero)
    Wait-Condition { -not [UltraMusicSmoke]::IsWindowVisible($window) } 'Close did not hide the window'
    if ($process.HasExited) { throw 'Close terminated the process' }
    $second = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
    if (-not $second.WaitForExit(10000)) { throw 'Second instance stayed alive' }
    Wait-Condition { [UltraMusicSmoke]::IsWindowVisible($window) } 'Second launch did not restore the window'
    if (@(Get-Process ultramusic).Count -ne 1) { throw 'Multiple instances' }
    # Simulate the session-end path, which uses the same cleanup as Quit.
    [void][UltraMusicSmoke]::PostMessage($window, 0x16, [IntPtr]1, [IntPtr]::Zero)
    if (-not $process.WaitForExit(10000)) { throw 'Shutdown failed' }
    Wait-Condition { @($engineIds | Where-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue }).Count -eq 0 } 'WebView2 descendants still alive' 30
    Write-Output 'PASS: startup, WebView2 process, close-to-tray, single instance, restore, shutdown and descendant cleanup.'
} finally {
    if (-not $process.HasExited -and $window -ne [IntPtr]::Zero) {
        [void][UltraMusicSmoke]::PostMessage($window, 0x16, [IntPtr]1, [IntPtr]::Zero)
    }
}
