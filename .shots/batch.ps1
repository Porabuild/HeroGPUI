# Runs many checks against ONE gallery process, without taking the focus.
#
# `drive.ps1` launches, waits for the first frame, acts, captures and kills --
# about four seconds of startup per step, which is most of the wall-clock of a
# verification round and five minutes for a 73-page sweep. The app can be told
# which page and section to show while it runs (`HEROGPUI_CONTROL`, see
# `gallery/src/control.rs`), so one launch serves the whole batch: write the
# control file, wait for the app's ack, post the input, capture, repeat.
#
#   .\.shots\batch.ps1 -Steps @(
#       @{ page = 'Table';      section = 'Sorting'; do = 'click:353,387 key:enter' }
#       @{ page = 'Text Field'; section = 'Usage';   do = 'click:400,410 type:hello' }
#       @{ page = 'Calendar';   out = 'E:\work\HeroGPUI\.shots\calendar-v3.png' }
#   )
#
# Each step is a hashtable: page, section, do, out, theme ('dark'), overlays
# ('1'). Anything omitted resets to the default, so a step never inherits the
# previous one's section or theme by accident. `out` defaults to
# `.shots/~batch-<n>.png`.
#
# Coordinates are the ones you read off the PNG; the frame offset is measured
# from the window, as in `drive.ps1`.
param(
    [Parameter(Mandatory = $true)][object[]]$Steps,
    [int]$Width = 1200,
    [int]$Height = 0,
    # Skip the capture and only report whether the app is still alive after each
    # step -- what the smoke sweep needs.
    [switch]$NoShot,
    [switch]$Quiet
)

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Batch {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hh, uint flags);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    public struct RECT { public int Left, Top, Right, Bottom; }
    public struct POINT { public int X, Y; }
}
"@

$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_LBUTTONDBLCLK = 0x0203
$WM_MOUSEWHEEL = 0x020A
$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_CHAR = 0x0102
$MK_LBUTTON = 1

$VK = @{
    'tab' = 0x09; 'enter' = 0x0D; 'return' = 0x0D; 'escape' = 0x1B; 'esc' = 0x1B
    'space' = 0x20; 'pageup' = 0x21; 'pgup' = 0x21; 'pagedown' = 0x22; 'pgdn' = 0x22
    'end' = 0x23; 'home' = 0x24; 'left' = 0x25; 'up' = 0x26; 'right' = 0x27
    'down' = 0x28; 'delete' = 0x2E; 'del' = 0x2E; 'backspace' = 0x08; 'bs' = 0x08
    'shift' = 0x10; 'ctrl' = 0x11; 'control' = 0x11; 'alt' = 0x12
}

$script:offX = 0
$script:offY = 0

function Get-Vk([string]$name) {
    if ($VK.ContainsKey($name)) { return $VK[$name] }
    if ($name.Length -eq 1) {
        $c = $name.ToUpper()[0]
        if ((($c -ge 'A') -and ($c -le 'Z')) -or (($c -ge '0') -and ($c -le '9'))) {
            return [int][char]$c
        }
    }
    throw "unknown key '$name'"
}

function Get-Lparam([int]$x, [int]$y) {
    [IntPtr]((($y - $script:offY) -shl 16) -bor (($x - $script:offX) -band 0xFFFF))
}

function Invoke-Step($h, [string]$do) {
    foreach ($token in ($do -split '\s+')) {
        if ($token -eq "") { continue }
        $kind, $arg = $token.Split(':', 2)
        switch ($kind) {
            'click' {
                $xy = $arg.Split(',')
                [void][Batch]::PostMessage($h, $WM_MOUSEMOVE, [IntPtr]0, (Get-Lparam ([int]$xy[0]) ([int]$xy[1])))
                [void][Batch]::PostMessage($h, $WM_LBUTTONDOWN, [IntPtr]$MK_LBUTTON, (Get-Lparam ([int]$xy[0]) ([int]$xy[1])))
                Start-Sleep -Milliseconds 30
                [void][Batch]::PostMessage($h, $WM_LBUTTONUP, [IntPtr]0, (Get-Lparam ([int]$xy[0]) ([int]$xy[1])))
                Start-Sleep -Milliseconds 200
            }
            'dblclick' {
                $xy = $arg.Split(',')
                foreach ($msg in @($WM_LBUTTONDOWN, $WM_LBUTTONDBLCLK)) {
                    [void][Batch]::PostMessage($h, $msg, [IntPtr]$MK_LBUTTON, (Get-Lparam ([int]$xy[0]) ([int]$xy[1])))
                    Start-Sleep -Milliseconds 25
                    [void][Batch]::PostMessage($h, $WM_LBUTTONUP, [IntPtr]0, (Get-Lparam ([int]$xy[0]) ([int]$xy[1])))
                    Start-Sleep -Milliseconds 25
                }
                Start-Sleep -Milliseconds 200
            }
            'drag' {
                $ends = $arg.Split('>')
                $a = $ends[0].Split(','); $b = $ends[1].Split(',')
                $x1 = [int]$a[0]; $y1 = [int]$a[1]; $x2 = [int]$b[0]; $y2 = [int]$b[1]
                [void][Batch]::PostMessage($h, $WM_MOUSEMOVE, [IntPtr]0, (Get-Lparam $x1 $y1))
                [void][Batch]::PostMessage($h, $WM_LBUTTONDOWN, [IntPtr]$MK_LBUTTON, (Get-Lparam $x1 $y1))
                for ($s = 1; $s -le 12; $s++) {
                    $ix = $x1 + [int](($x2 - $x1) * $s / 12)
                    $iy = $y1 + [int](($y2 - $y1) * $s / 12)
                    [void][Batch]::PostMessage($h, $WM_MOUSEMOVE, [IntPtr]$MK_LBUTTON, (Get-Lparam $ix $iy))
                    Start-Sleep -Milliseconds 25
                }
                [void][Batch]::PostMessage($h, $WM_LBUTTONUP, [IntPtr]0, (Get-Lparam $x2 $y2))
                Start-Sleep -Milliseconds 200
            }
            'key' {
                $spec, $times = $arg.Split('*', 2)
                $n = if ($times) { [int]$times } else { 1 }
                $parts = $spec.Split('+')
                $key = $parts[-1]
                $mods = @()
                if ($parts.Count -gt 1) { foreach ($m in $parts[0..($parts.Count - 2)]) { $mods += (Get-Vk $m) } }
                for ($i = 0; $i -lt $n; $i++) {
                    foreach ($m in $mods) { [void][Batch]::PostMessage($h, $WM_KEYDOWN, [IntPtr]$m, [IntPtr]0) }
                    $vk = Get-Vk $key
                    [void][Batch]::PostMessage($h, $WM_KEYDOWN, [IntPtr]$vk, [IntPtr]0)
                    [void][Batch]::PostMessage($h, $WM_KEYUP, [IntPtr]$vk, [IntPtr]0)
                    foreach ($m in $mods) { [void][Batch]::PostMessage($h, $WM_KEYUP, [IntPtr]$m, [IntPtr]0) }
                    Start-Sleep -Milliseconds 70
                }
            }
            'type' {
                foreach ($ch in ($arg -replace '_', ' ').ToCharArray()) {
                    $vk = [int][char]([string]$ch).ToUpper()
                    [void][Batch]::PostMessage($h, $WM_KEYDOWN, [IntPtr]$vk, [IntPtr]0)
                    [void][Batch]::PostMessage($h, $WM_CHAR, [IntPtr][int][char]$ch, [IntPtr]0)
                    [void][Batch]::PostMessage($h, $WM_KEYUP, [IntPtr]$vk, [IntPtr]0)
                    Start-Sleep -Milliseconds 35
                }
            }
            'wheel' {
                for ($i = 0; $i -lt [int]$arg; $i++) {
                    [void][Batch]::PostMessage($h, $WM_MOUSEWHEEL, [IntPtr](-120 -shl 16), (Get-Lparam 600 400))
                    Start-Sleep -Milliseconds 35
                }
            }
            'wait' { Start-Sleep -Milliseconds ([int]$arg) }
            default { throw "unknown step '$token'" }
        }
    }
}

# -- one process for the whole batch ----------------------------------------
$control = Join-Path $env:TEMP "herogpui-control.txt"
$ack = [System.IO.Path]::ChangeExtension($control, ".ack")
Remove-Item $control, $ack -ErrorAction SilentlyContinue
Set-Content $control "seq=0`npage=Introduction"

Add-Type -AssemblyName System.Windows.Forms
if ($Height -le 0) {
    $Height = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Height
}

$env:HEROGPUI_UNFOCUSED = "1"
$env:HEROGPUI_CONTROL = $control
$env:HEROGPUI_WINDOW_SIZE = "${Width}x${Height}"
$env:HEROGPUI_PAGE = $null
$env:HEROGPUI_SECTION = $null
$env:HEROGPUI_THEME = $null
$env:HEROGPUI_OPEN_OVERLAYS = $null

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "E:\work\HeroGPUI\target\debug\gallery.exe"
$psi.WorkingDirectory = "E:\work\HeroGPUI"
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true
# `Process::Start(psi)` returns null in pwsh for a console-subsystem binary
# launched with `CreateNoWindow`; constructing the object and calling `Start()`
# on it always hands back something to poll.
$p = New-Object System.Diagnostics.Process
$p.StartInfo = $psi
[void]$p.Start()
if ($null -eq $p) { throw "could not start $($psi.FileName)" }

$h = [IntPtr]::Zero
for ($try = 0; $try -lt 30; $try++) {
    Start-Sleep -Milliseconds 400
    if ($p.HasExited) { Write-Host "gallery exited during startup"; exit 1 }
    $p.Refresh()
    $h = $p.MainWindowHandle
    if ($h -ne [IntPtr]::Zero) { break }
}
if ($h -eq [IntPtr]::Zero) { Write-Host "no window"; $p.Kill(); exit 1 }

$SWP_NOACTIVATE = 0x0010
$SWP_NOZORDER = 0x0004
[void][Batch]::SetWindowPos($h, [IntPtr]::Zero, -32000, -32000, $Width, $Height,
    $SWP_NOACTIVATE -bor $SWP_NOZORDER)
Start-Sleep -Milliseconds 700

$wr = New-Object Batch+RECT
[void][Batch]::GetWindowRect($h, [ref]$wr)
$origin = New-Object Batch+POINT
[void][Batch]::ClientToScreen($h, [ref]$origin)
$script:offX = $origin.X - $wr.Left
$script:offY = $origin.Y - $wr.Top

$failed = @()
$n = 0
$sw = [System.Diagnostics.Stopwatch]::StartNew()
foreach ($step in $Steps) {
    $n++
    if ($p.HasExited) {
        # A panic takes the process with it; report every remaining step as
        # unreached rather than pretending they passed.
        $failed += "$($step.page) (process already dead)"
        continue
    }
    $lines = @("seq=$n")
    if ($step.page) { $lines += "page=$($step.page)" }
    if ($step.section) { $lines += "section=$($step.section)" }
    if ($step.theme) { $lines += "theme=$($step.theme)" }
    if ($step.overlays) { $lines += "overlays=$($step.overlays)" }
    Set-Content $control ($lines -join "`n")

    # Wait for the app to acknowledge *this* sequence number: a fixed sleep
    # either wastes time or photographs the previous page.
    $ok = $false
    for ($w = 0; $w -lt 60; $w++) {
        Start-Sleep -Milliseconds 60
        if ($p.HasExited) { break }
        # The ack can be observed mid-write, so read defensively rather than
        # letting a null land in `.Trim()`.
        $seen = (Get-Content $ack -Raw -ErrorAction SilentlyContinue)
        if ($seen -and $seen.Trim() -eq "$n") { $ok = $true; break }
    }
    if ($p.HasExited) {
        $failed += "$($step.page) $($step.section) (died rendering)"
        continue
    }
    if (-not $ok) {
        $failed += "$($step.page) $($step.section) (no ack)"
        continue
    }

    if ($step.do) { Invoke-Step $h $step.do }

    if (-not $NoShot) {
        $out = if ($step.out) { $step.out } else { "E:\work\HeroGPUI\.shots\~batch-$n.png" }
        $r = New-Object Batch+RECT
        [void][Batch]::GetWindowRect($h, [ref]$r)
        $w2 = $r.Right - $r.Left
        $h2 = $r.Bottom - $r.Top
        $bmp = New-Object System.Drawing.Bitmap($w2, $h2)
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $hdc = $g.GetHdc()
        [void][Batch]::PrintWindow($h, $hdc, 2)
        $g.ReleaseHdc($hdc)
        $g.Dispose()
        $bmp.Save($out)
        $bmp.Dispose()
        if (-not $Quiet) { Write-Host ("{0,-22} {1,-24} -> {2}" -f $step.page, $step.section, $out) }
    } elseif (-not $Quiet) {
        Write-Host ("ok    {0} {1}" -f $step.page, $step.section)
    }
}
$sw.Stop()

if (-not $p.HasExited) { $p.Kill() }
Remove-Item $control, $ack -ErrorAction SilentlyContinue
Write-Host ("{0} steps in {1:N1}s ({2:N2}s each)" -f $Steps.Count, $sw.Elapsed.TotalSeconds,
    ($sw.Elapsed.TotalSeconds / [Math]::Max(1, $Steps.Count)))
if ($failed.Count -gt 0) {
    Write-Host "FAILED:" -ForegroundColor Red
    $failed | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}
