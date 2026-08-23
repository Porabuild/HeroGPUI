# Builds the workspace after making sure nothing still holds `gallery.exe`.
#
# `smoke.ps1` and `capture2.ps1` launch the gallery dozens of times, and Windows
# keeps the image locked for a moment after the process dies, so a build started
# right after one of them fails with `Access is denied. (os error 5)` -- and then
# the next capture silently screenshots the *previous* binary, which is worse
# than a failed build.
param([switch]$Quiet)

$exe = "E:\work\HeroGPUI\target\debug\gallery.exe"
$stale = "E:\work\HeroGPUI\target\debug\gallery.locked.exe"

Get-Process -Name gallery -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Opening the image for read/write can succeed while *deleting* it still fails --
# cargo needs DELETE access, and a closed process (or an antivirus scan) can hold
# exactly that. So do not probe: move the old image out of the way. Renaming
# works where deleting does not, and cargo then writes a fresh one.
Remove-Item $stale -Force -ErrorAction SilentlyContinue
if (Test-Path $exe) {
    for ($i = 0; $i -lt 20; $i++) {
        try { Rename-Item $exe $stale -Force -ErrorAction Stop; break }
        catch { Start-Sleep -Milliseconds 300 }
    }
    if (Test-Path $exe) {
        Write-Host "could not move gallery.exe aside; the build will likely fail" -ForegroundColor Yellow
    }
}

cargo build --workspace
$code = $LASTEXITCODE
Remove-Item $stale -Force -ErrorAction SilentlyContinue
if ($code -ne 0) {
    Write-Host "build failed ($code) -- do not trust screenshots taken after this" -ForegroundColor Red
    exit $code
}
if (-not $Quiet) { Write-Host "build ok" -ForegroundColor Green }
