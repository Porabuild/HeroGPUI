# Walks every route and reports any page that panics.
#
# The gallery renders lazily, so a page can compile and still panic at runtime.
# Every path, including an isolated retry, requires a rendered-frame acknowledgement.
#
# It used to launch one process per page: about four seconds of startup each, so
# five minutes for a run that finds nothing. The app can be told which page to
# show while it runs (`HEROGPUI_CONTROL`, see `gallery/src/control.rs`), so this
# now walks the whole list in a single process and only relaunches where one
# actually dies -- seconds instead of minutes, which is the difference between
# running it after every change and running it "later".
#
# A panic takes the process with it, so the page being rendered when it died is
# the suspect. It is retried in a fresh process, alone, and only reported if it
# dies again: launching gpui windows back to back intermittently kills one during
# startup (exit -1, empty stderr, a different page each run), and reporting those
# made the gate unheedable.
#
# The window is parked off-screen rather than minimized. Minimizing is quieter
# but a minimized window may never present a frame, which would let this pass
# without having rendered anything -- the opposite of what it is for.
param(
    # Kept for callers that want the old one-process-per-page behaviour.
    [switch]$PerProcess,
    [int]$Width = 1200
)

. (Join-Path $PSScriptRoot "control.ps1")

$exe = "E:\work\HeroGPUI\target\debug\herogpui-gallery.exe"
if (-not (Test-Path $exe)) { throw "build the gallery first: cargo build --workspace" }

# Keep in sync with Page::title in gallery/src/pages/mod.rs.
$metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
$workspaceVersion = ($metadata.packages | Where-Object name -eq "herogpui-gallery").version
$currentVersionPage = "v$workspaceVersion"
$pages = @(
    "All Components", "Releases", $currentVersionPage,
    "Introduction", "Installation", "Theming", "Dark Mode", "Customization",
    "Styling", "Design Principles",
    "Button", "Button Group", "Close Button", "Toggle Button",
    "Dropdown", "List Box", "Tag Group",
    "Color Area", "Color Field", "Color Picker", "Color Slider", "Color Swatch",
    "Color Swatch Picker",
    "Slider", "Switch",
    "Badge", "Chip", "Table",
    "Calendar", "Date Field", "Date Picker", "Date Range Picker", "Range Calendar",
    "Time Field",
    "Alert", "Meter", "Progress Bar", "Progress Circle", "Skeleton", "Spinner",
    "Checkbox", "Checkbox Group", "Fieldset", "Label & Messages", "Form", "Input",
    "Input Group", "Input OTP", "Number Field", "Radio Group", "Search Field",
    "Text Area", "Text Field",
    "Card", "Separator", "Surface", "Toolbar",
    "Avatar",
    "Accordion", "Breadcrumbs", "Disclosure", "Link", "Pagination", "Tabs",
    "Alert Dialog", "Drawer", "Modal", "Popover", "Toast", "Tooltip",
    "Autocomplete", "Combo Box", "Select",
    "Kbd", "Typography",
    "Scroll Shadow"
)

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Smoke {
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hh, uint flags);
}
"@
$SWP_NOACTIVATE = 0x0010
$SWP_NOZORDER = 0x0004

Add-Type -AssemblyName System.Windows.Forms
$screenH = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Height

$control = Join-Path $env:TEMP "herogpui-smoke-$([Guid]::NewGuid().ToString('N')).txt"
$ack = [System.IO.Path]::ChangeExtension($control, ".ack")
$errorPath = [System.IO.Path]::ChangeExtension($control, ".error")

function Start-Gallery {
    param([string]$ControlPath = $control, [string]$Page = '')
    $lines = @('seq=0')
    if ($Page) { $lines += "page=$Page" }
    Write-GalleryControl -Path $ControlPath -Lines $lines
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.WorkingDirectory = "E:\work\HeroGPUI"
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardError = $true
    $psi.Environment['HEROGPUI_UNFOCUSED'] = '1'
    $psi.Environment['HEROGPUI_CONTROL'] = $ControlPath
    $psi.Environment['HEROGPUI_WINDOW_SIZE'] = "${Width}x${screenH}"
    foreach ($name in @('HEROGPUI_PAGE', 'HEROGPUI_SECTION')) {
        [void]$psi.Environment.Remove($name)
    }
    # `Process::Start(psi)` returns null in pwsh for a console-subsystem binary
    # launched with `CreateNoWindow`; constructing the object and calling
    # `Start()` on it always hands back something to poll.
    $p = New-Object System.Diagnostics.Process
    $p.StartInfo = $psi
    $started = $false
    try {
        [void]$p.Start()
        $started = $true
        $err = $p.StandardError.ReadToEndAsync()
        for ($try = 0; $try -lt 30; $try++) {
            Start-Sleep -Milliseconds 350
            if ($p.HasExited) { break }
            $p.Refresh()
            if ($p.MainWindowHandle -ne [IntPtr]::Zero) {
                [void][Smoke]::SetWindowPos($p.MainWindowHandle, [IntPtr]::Zero, -32000, -32000,
                    $Width, $screenH, $SWP_NOACTIVATE -bor $SWP_NOZORDER)
                return @{ proc = $p; err = $err }
            }
        }
        return @{ proc = $p; err = $err }
    } catch {
        if ($started) {
            if (-not $p.HasExited) { $p.Kill() }
            $p.WaitForExit()
        }
        $p.Dispose()
        throw
    }
}

# One page in a process of its own: the retry path, and what `-PerProcess` does
# for every page.
function Test-Page([string]$page) {
    $singleControl = Join-Path $env:TEMP "herogpui-smoke-single-$([Guid]::NewGuid().ToString('N')).txt"
    $singleAck = [System.IO.Path]::ChangeExtension($singleControl, '.ack')
    $singleError = [System.IO.Path]::ChangeExtension($singleControl, '.error')
    $session = $null
    try {
        $session = Start-Gallery -ControlPath $singleControl -Page $page
        $result = Wait-GalleryControl -Process $session.proc -Path $singleControl -Sequence '0'
    } finally {
        try {
            if ($session) {
                if (-not $session.proc.HasExited) { $session.proc.Kill() }
                $session.proc.WaitForExit()
            }
        } finally {
            if ($session) { $session.proc.Dispose() }
            Remove-Item $singleControl, $singleAck, $singleError -ErrorAction SilentlyContinue
        }
    }
    $text = if ($result.ok) { '' } else { "$($result.error)`n$($session.err.Result)".Trim() }
    return @{ ok = $result.ok; text = $text }
}

$sw = [System.Diagnostics.Stopwatch]::StartNew()
$failures = @()

$session = $null
try {
    if ($PerProcess) {
        foreach ($page in $pages) {
            $r = Test-Page $page
            if ($r.ok) { Write-Host "ok    $page" }
            else {
                $again = Test-Page $page
                if ($again.ok) { Write-Host "ok    $page  (first request failed; isolated frame acknowledged)" }
                else {
                    Write-Host "FAIL  $page" -ForegroundColor Red
                    if ($again.text) { Write-Host ($again.text.Trim()) -ForegroundColor DarkGray }
                    $failures += $page
                }
            }
        }
    } else {
        $session = Start-Gallery
        $seq = 0
        foreach ($page in $pages) {
            $seq++
            if ($session.proc.HasExited) { $session.proc.Dispose(); $session = $null; $session = Start-Gallery; $seq = 1 }
            if ($session.proc.HasExited) {
                Write-Host "FAIL  $page  (gallery will not start)" -ForegroundColor Red
                $failures += $page
                continue
            }
            Write-GalleryControl -Path $control -Lines @("seq=$seq", "page=$page")
            $result = Wait-GalleryControl -Process $session.proc -Path $control -Sequence "$seq"
            if ($result.ok) { Write-Host "ok    $page"; continue }

            # Either it died rendering this page or it never acknowledged. Both are
            # suspicions, not verdicts: retry the page alone.
            $text = $result.error
            if ($session.proc.HasExited) { $text += "`n$($session.err.Result)" }
            $again = Test-Page $page
            if ($again.ok) {
                Write-Host "ok    $page  (shared request failed; isolated frame acknowledged)"
            } else {
                Write-Host "FAIL  $page" -ForegroundColor Red
                $msg = if ($again.text) { $again.text } else { $text }
                if ($msg) { Write-Host ($msg.Trim()) -ForegroundColor DarkGray }
                $failures += $page
            }
            if ($session.proc.HasExited) { $session.proc.Dispose(); $session = $null; $session = Start-Gallery; $seq = 0 }
        }
    }

} finally {
    try {
        if ($session) {
            if (-not $session.proc.HasExited) { $session.proc.Kill() }
            $session.proc.WaitForExit()
        }
    } finally {
        if ($session) { $session.proc.Dispose() }
        Remove-Item $control, $ack, $errorPath -ErrorAction SilentlyContinue
        $sw.Stop()
    }
}

Write-Host ""
if ($failures.Count -eq 0) {
    Write-Host ("all {0} pages rendered  ({1:N1}s)" -f $pages.Count, $sw.Elapsed.TotalSeconds)
} else {
    Write-Host ("{0} of {1} pages failed: {2}" -f $failures.Count, $pages.Count,
        ($failures -join ", ")) -ForegroundColor Red
    exit 1
}
