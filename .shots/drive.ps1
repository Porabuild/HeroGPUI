# Drives the gallery *without* taking the focus, and captures the result.
#
# `capture2.ps1` injects real input, which needs the window foreground: it
# raises the gallery, steals the focus and interrupts whatever the user is
# doing, once per page. This script posts the input messages straight to the
# window instead, so the window can stay parked off-screen and unfocused the
# whole time. `PrintWindow` never needed the window on screen either.
#
#   .\.shots\drive.ps1 -Page "Table" -Section "Sorting" -Do "click:120,180 key:enter"
#   .\.shots\drive.ps1 -Page "Text Field" -Do "click:90,60 type:hello key:tab"
#
# What a posted message *cannot* do is carry a modifier: Windows keeps the
# shift/ctrl state for real input and gpui asks it, so a capital, a shifted
# symbol and a chord (`ctrl+a`) still need `capture2.ps1`, which injects real
# input and therefore needs the foreground.
#
# Coordinates are *client* coordinates -- the same pixels as the saved PNG,
# since the capture is the window's own bitmap.
#
# `-Section` is the reason this is fast: a page is longer than any window, and
# scrolling to a section by wheel notches to photograph it is slow and fragile.
# Naming the section renders only that one, at the top of an otherwise empty
# page.
param(
    [string]$Page = "",
    # Comma-separated section titles (substring, case-insensitive). Empty renders
    # the whole page.
    [string]$Section = "",
    # Whitespace-separated steps, in order:
    #   click:X,Y        press and release at a client point
    #   dblclick:X,Y     two presses inside the double-click time
    #   drag:X,Y>X2,Y2   press, twelve moves, release
    #   key:tab          a key by name; `key:ctrl+a`, `key:shift+pageup` chord
    #   key:down*15      the same key fifteen times
    #   type:hello       characters, one WM_CHAR each (use _ for a space)
    #   wait:400         milliseconds
    [string]$Do = "",
    [string]$Out = "",
    [int]$Width = 1200,
    # The window is created at this size rather than resized into it: Windows
    # clamps a *resize* of a visible window to the monitor, but a window created
    # oversized keeps its height, and a taller window is more of the page per
    # capture.
    [int]$Height = 2000,
    [switch]$Dark,
    [switch]$Overlays,
    [switch]$ReduceMotion
)

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Drive {
    [DllImport("user32.dll")] public static extern uint MapVirtualKey(uint code, uint kind);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern short VkKeyScan(char ch);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hh, uint flags);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    public struct RECT { public int Left, Top, Right, Bottom; }
    public struct POINT { public int X, Y; }
}
"@

$WM_MOUSEMOVE   = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP   = 0x0202
$WM_LBUTTONDBLCLK = 0x0203
$WM_MOUSEWHEEL  = 0x020A
$WM_KEYDOWN     = 0x0100
$WM_KEYUP       = 0x0101
$MK_LBUTTON     = 1

# NB: PowerShell variable names are case-insensitive, so a local `$vk` would be
# this very table -- one keystroke worked and the next threw "unknown key".
$VK = @{
    'tab' = 0x09; 'enter' = 0x0D; 'return' = 0x0D; 'escape' = 0x1B; 'esc' = 0x1B
    'space' = 0x20; 'pageup' = 0x21; 'pgup' = 0x21; 'pagedown' = 0x22; 'pgdn' = 0x22
    'end' = 0x23; 'home' = 0x24; 'left' = 0x25; 'up' = 0x26; 'right' = 0x27
    'down' = 0x28; 'delete' = 0x2E; 'del' = 0x2E; 'backspace' = 0x08; 'bs' = 0x08
    'shift' = 0x10; 'ctrl' = 0x11; 'control' = 0x11; 'alt' = 0x12
}

function Get-Vk([string]$name) {
    if ($VK.ContainsKey($name)) { return $VK[$name] }
    if ($name.Length -eq 1) {
        $c = $name.ToUpper()[0]
        if (($c -ge 'A') -and ($c -le 'Z')) { return [int][char]$c }
        if (($c -ge '0') -and ($c -le '9')) { return [int][char]$c }
    }
    throw "unknown key '$name'"
}

function Post-Key($h, [uint32]$message, [int]$code) {
    [long]$flags = 1 -bor ([Drive]::MapVirtualKey($code, 0) -shl 16)
    if ($code -ge 0x21 -and $code -le 0x2E) { $flags = $flags -bor 0x01000000 }
    if ($message -eq $WM_KEYUP) { $flags = $flags -bor [long]3221225472 }
    [void][Drive]::PostMessage($h, $message, [IntPtr]$code, [IntPtr]$flags)
}

function Send-Key($h, [string]$spec) {
    # `ctrl+a`, `shift+pageup`, or a bare key name.
    $parts = $spec.Split('+')
    $key = $parts[-1]
    $mods = @()
    foreach ($m in $parts[0..([Math]::Max(0, $parts.Count - 2))]) {
        if ($m -ne $key -and $m -ne "") { $mods += (Get-Vk $m) }
    }
    foreach ($m in $mods) {
        Post-Key $h $WM_KEYDOWN $m
    }
    $code = Get-Vk $key
    Post-Key $h $WM_KEYDOWN $code
    Post-Key $h $WM_KEYUP $code
    foreach ($m in $mods) {
        Post-Key $h $WM_KEYUP $m
    }
    Start-Sleep -Milliseconds 90
}

function Send-Text($h, [string]$text) {
    # Key-down is translated into WM_CHAR by the platform. Posting another
    # WM_CHAR would insert twice in fields that register an input handler.
    # Posted keys cannot change Windows modifier state; shifted characters
    # still need capture2.ps1 and explicit foreground permission.
    foreach ($ch in $text.ToCharArray()) {
        $mapped = [Drive]::VkKeyScan($ch)
        if ($mapped -eq -1) { throw "character has no key on the current keyboard layout: $ch" }
        $code = $mapped -band 0xFF
        Post-Key $h $WM_KEYDOWN $code
        Post-Key $h $WM_KEYUP $code
        Start-Sleep -Milliseconds 45
    }
}

# Bitmap coordinates (what you read off the PNG) are window-relative; a posted
# mouse message wants *client* coordinates. The difference is the frame, measured
# once from the window itself rather than assumed.
$script:offX = 0
$script:offY = 0
function Get-Lparam([int]$x, [int]$y) {
    $cx = $x - $script:offX
    $cy = $y - $script:offY
    [IntPtr](($cy -shl 16) -bor ($cx -band 0xFFFF))
}

function Send-Click($h, [int]$x, [int]$y, [int]$count) {
    [void][Drive]::PostMessage($h, $WM_MOUSEMOVE, [IntPtr]0, (Get-Lparam $x $y))
    for ($i = 0; $i -lt $count; $i++) {
        $msg = if ($i -gt 0) { $WM_LBUTTONDBLCLK } else { $WM_LBUTTONDOWN }
        [void][Drive]::PostMessage($h, $msg, [IntPtr]$MK_LBUTTON, (Get-Lparam $x $y))
        Start-Sleep -Milliseconds 30
        [void][Drive]::PostMessage($h, $WM_LBUTTONUP, [IntPtr]0, (Get-Lparam $x $y))
        Start-Sleep -Milliseconds 30
    }
    Start-Sleep -Milliseconds 260
}

function Send-Drag($h, [int]$x1, [int]$y1, [int]$x2, [int]$y2) {
    [void][Drive]::PostMessage($h, $WM_MOUSEMOVE, [IntPtr]0, (Get-Lparam $x1 $y1))
    [void][Drive]::PostMessage($h, $WM_LBUTTONDOWN, [IntPtr]$MK_LBUTTON, (Get-Lparam $x1 $y1))
    Start-Sleep -Milliseconds 60
    for ($s = 1; $s -le 12; $s++) {
        $ix = $x1 + [int](($x2 - $x1) * $s / 12)
        $iy = $y1 + [int](($y2 - $y1) * $s / 12)
        [void][Drive]::PostMessage($h, $WM_MOUSEMOVE, [IntPtr]$MK_LBUTTON, (Get-Lparam $ix $iy))
        Start-Sleep -Milliseconds 30
    }
    [void][Drive]::PostMessage($h, $WM_LBUTTONUP, [IntPtr]0, (Get-Lparam $x2 $y2))
    Start-Sleep -Milliseconds 260
}

# -- launch, parked off-screen and unfocused ---------------------------------
$env:HEROGPUI_PAGE = $Page
$env:HEROGPUI_SECTION = $Section
$env:HEROGPUI_UNFOCUSED = "1"
$env:HEROGPUI_WINDOW_SIZE = "${Width}x${Height}"
if ($Dark) { $env:HEROGPUI_THEME = "dark" } else { $env:HEROGPUI_THEME = $null }
if ($Overlays) { $env:HEROGPUI_OPEN_OVERLAYS = "1" } else { $env:HEROGPUI_OPEN_OVERLAYS = $null }
if ($ReduceMotion) { $env:HEROGPUI_REDUCE_MOTION = "1" } else { $env:HEROGPUI_REDUCE_MOTION = $null }

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "E:\work\HeroGPUI\target\debug\herogpui-gallery.exe"
$psi.WorkingDirectory = "E:\work\HeroGPUI"
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true
# `Process::Start(psi)` returns null in pwsh for a console-subsystem binary
# launched with `CreateNoWindow`; constructing the object and calling `Start()`
# on it always hands back something to poll.
$p = New-Object System.Diagnostics.Process
$p.StartInfo = $psi
[void]$p.Start()

$h = [IntPtr]::Zero
for ($try = 0; $try -lt 20; $try++) {
    Start-Sleep -Milliseconds 500
    if ($p.HasExited) { Write-Host "gallery exited early"; exit 1 }
    $p.Refresh()
    $h = $p.MainWindowHandle
    if ($h -ne [IntPtr]::Zero) { break }
}
if ($h -eq [IntPtr]::Zero) { Write-Host "no window"; $p.Kill(); exit 1 }

# Park it far off-screen without activating. Off-screen still renders; only
# minimizing stops that.
$SWP_NOACTIVATE = 0x0010
$SWP_NOZORDER = 0x0004
[void][Drive]::SetWindowPos($h, [IntPtr]::Zero, -32000, -32000, $Width, $Height,
    $SWP_NOACTIVATE -bor $SWP_NOZORDER)
Start-Sleep -Milliseconds 900

$wr = New-Object Drive+RECT
[void][Drive]::GetWindowRect($h, [ref]$wr)
$origin = New-Object Drive+POINT
[void][Drive]::ClientToScreen($h, [ref]$origin)
$script:offX = $origin.X - $wr.Left
$script:offY = $origin.Y - $wr.Top

# -- the steps ---------------------------------------------------------------
foreach ($step in ($Do -split '\s+')) {
    if ($step -eq "") { continue }
    $kind, $arg = $step.Split(':', 2)
    switch ($kind) {
        'click' { $xy = $arg.Split(','); Send-Click $h ([int]$xy[0]) ([int]$xy[1]) 1 }
        'dblclick' { $xy = $arg.Split(','); Send-Click $h ([int]$xy[0]) ([int]$xy[1]) 2 }
        'tripleclick' { $xy = $arg.Split(','); Send-Click $h ([int]$xy[0]) ([int]$xy[1]) 3 }
        'drag' {
            $ends = $arg.Split('>')
            $a = $ends[0].Split(','); $b = $ends[1].Split(',')
            Send-Drag $h ([int]$a[0]) ([int]$a[1]) ([int]$b[0]) ([int]$b[1])
        }
        'key' {
            $spec, $times = $arg.Split('*', 2)
            $n = if ($times) { [int]$times } else { 1 }
            for ($i = 0; $i -lt $n; $i++) { Send-Key $h $spec }
        }
        'type' { Send-Text $h ($arg -replace '_', ' ') }
        'wheel' {
            # Wheel messages carry screen coordinates; Get-Lparam handles the capture inset.
            for ($i = 0; $i -lt [int]$arg; $i++) {
                [void][Drive]::PostMessage($h, $WM_MOUSEWHEEL, [IntPtr](-120 -shl 16), (Get-Lparam ($origin.X + 600) ($origin.Y + 400)))
                Start-Sleep -Milliseconds 40
            }
        }
        'wait' { Start-Sleep -Milliseconds ([int]$arg) }
        default { throw "unknown step '$step'" }
    }
}
Start-Sleep -Milliseconds 500

# -- capture -----------------------------------------------------------------
$r = New-Object Drive+RECT
[void][Drive]::GetWindowRect($h, [ref]$r)
$w = $r.Right - $r.Left
$hh = $r.Bottom - $r.Top
$bmp = New-Object System.Drawing.Bitmap($w, $hh)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
$okShot = [Drive]::PrintWindow($h, $hdc, 2)
$g.ReleaseHdc($hdc)
$g.Dispose()

if ($Out -eq "") {
    $slug = ($Page -replace '[^A-Za-z0-9]', '').ToLower()
    if ($slug -eq "") { $slug = "page" }
    $Out = "E:\work\HeroGPUI\.shots\~drive-$slug.png"
}
$bmp.Save($Out)
$bmp.Dispose()
$p.Kill()
Write-Host "$Page$(if ($Section) { " / $Section" }) : $w x $hh -> $Out (printed=$okShot)"
