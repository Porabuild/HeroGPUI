# Builds the workspace after making sure nothing still holds `gallery.exe`.
#
# `smoke.ps1` and `capture2.ps1` launch the gallery dozens of times, and Windows
# keeps the image locked for a moment after the process dies, so a build started
# right after one of them fails with `Access is denied. (os error 5)` -- and then
# the next capture silently screenshots the *previous* binary, which is worse
# than a failed build. This waits for the lock to clear, renames the image out of
# the way if it does not, and only then builds.
param([switch]$Quiet)

$exe = "E:\work\HeroGPUI\target\debug\gallery.exe"

Get-Process -Name gallery -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

function Test-Locked([string]$path) {
    if (-not (Test-Path $path)) { return $false }
    try {
        $f = [System.IO.File]::Open($path, 'Open', 'ReadWrite', 'None')
        $f.Close()
        return $false
    } catch { return $true }
}

for ($i = 0; $i -lt 20 -and (Test-Locked $exe); $i++) { Start-Sleep -Milliseconds 300 }
if (Test-Locked $exe) {
    # A closed process can leave the image locked (antivirus, or a pending
    # delete). Renaming works where deleting does not, and cargo then writes a
    # fresh one.
    $stale = "E:\work\HeroGPUI\target\debug\gallery.locked.exe"
    Remove-Item $stale -Force -ErrorAction SilentlyContinue
    Rename-Item $exe $stale -Force
    Write-Host "gallery.exe was still locked; renamed it aside" -ForegroundColor Yellow
}

cargo build --workspace
$code = $LASTEXITCODE
Remove-Item "E:\work\HeroGPUI\target\debug\gallery.locked.exe" -Force -ErrorAction SilentlyContinue
if ($code -ne 0) {
    Write-Host "build failed ($code) -- do not trust screenshots taken after this" -ForegroundColor Red
    exit $code
}
if (-not $Quiet) { Write-Host "build ok" -ForegroundColor Green }
