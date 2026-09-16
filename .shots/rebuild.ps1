# Builds the workspace after making sure nothing still holds `herogpui-gallery.exe`.
#
# `smoke.ps1` and `capture2.ps1` launch the gallery dozens of times, and Windows
# keeps the image locked for a moment after the process dies, so a build started
# right after one of them fails with `Access is denied. (os error 5)` -- and then
# the next capture silently screenshots the *previous* binary, which is worse
# than a failed build.
param([switch]$Quiet)

# Every capture script launches this fixed path, so it is the harness contract.
$launcher = "E:\work\HeroGPUI\target\debug\herogpui-gallery.exe"
# `CARGO_TARGET_DIR` redirects the build, so the image cargo replaces is not
# necessarily the one the capture scripts launch.
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "E:\work\HeroGPUI\target" }
$exe = Join-Path $targetDir "debug\herogpui-gallery.exe"
$stale = Join-Path $targetDir "debug\herogpui-gallery.locked.exe"

Get-Process -Name herogpui-gallery -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

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
        Write-Host "could not move herogpui-gallery.exe aside; the build will likely fail" -ForegroundColor Yellow
    }
}
# The launcher may be a hard link to the image just moved aside. Left in place it
# keeps serving the previous binary, which is the silent stale capture this
# script exists to prevent, so it is dropped and re-pointed after the build.
if ($exe -ne $launcher) { Remove-Item $launcher -Force -ErrorAction SilentlyContinue }

cargo build --workspace
$code = $LASTEXITCODE
Remove-Item $stale -Force -ErrorAction SilentlyContinue
if ($code -ne 0) {
    Write-Host "build failed ($code) -- do not trust screenshots taken after this" -ForegroundColor Red
    exit $code
}

if ($exe -ne $launcher) {
    try { New-Item -ItemType HardLink -Path $launcher -Target $exe -ErrorAction Stop | Out-Null }
    catch { Copy-Item $exe $launcher -Force }
}
if (-not (Test-Path $launcher)) {
    Write-Host "build ok but $launcher is missing -- captures would launch nothing" -ForegroundColor Red
    exit 1
}
$fresh = Get-Item $exe
$served = Get-Item $launcher
if ($served.Length -ne $fresh.Length -or $served.LastWriteTime -ne $fresh.LastWriteTime) {
    Write-Host "build ok but $launcher is not the image just built -- captures would screenshot the previous binary" -ForegroundColor Red
    exit 1
}
if (-not $Quiet) { Write-Host "build ok" -ForegroundColor Green }
