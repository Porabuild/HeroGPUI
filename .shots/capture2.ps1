param(
    [string]$PageList = "",
    [string]$OutDir = "E:\work\HeroGPUI\.shots",
    # The gallery's nav rail plus its content column come to about 1200px, so a
    # wider window only adds empty margin. Pages are long instead, so the
    # default is narrow and as tall as the monitor allows: most pages then fit
    # in one shot with no scrolling.
    #
    # Windows clamps a window to the display, so asking for more height than the
    # screen has silently gives less. `0` means "as tall as this monitor
    # permits", and the captured size is printed so a clamp is never a surprise.
    [int]$Width = 1200,
    [int]$Height = 0,
    [int]$Scroll = 0,
    [int]$HoverX = -1,
    [int]$HoverY = -1,
    [switch]$HoldPress,
    # Size the window to the monitor's work area instead of -Width/-Height.
    [switch]$Fullscreen,
    # Park the window off-screen so capturing never covers what you are doing.
    # Implied unless a hover/press is requested, which needs a real cursor.
    [switch]$Offscreen,
    [switch]$Theme
)

# Captures the *window*, not the monitor.
#
# This used to call Graphics.CopyFromScreen at the window's coordinates, which
# reads whatever pixels are physically on screen there. When Windows refused the
# foreground steal -- which it does whenever another app is active, and always
# for a fullscreen game -- the file ended up holding whatever was in front,
# including unrelated windows. PrintWindow asks the window to render itself into
# a bitmap, so the capture is correct whether or not the window is focused,
# visible, or even on screen.

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win2 {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hh, uint flags);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    // PW_RENDERFULLCONTENT (2) is what makes this work for a GPU-composited
    // window; flag 0 returns blank for anything drawing through
    // DirectComposition, which is how gpui presents.
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, int d, int extra);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    public struct RECT { public int Left, Top, Right, Bottom; }
}
"@

# SWP flags: move/resize without activating or reordering.
$SWP_NOACTIVATE = 0x0010
$SWP_NOZORDER   = 0x0004
$SWP_NOOWNERZORDER = 0x0200

$interactive = ($HoverX -ge 0) -or $HoldPress -or ($Scroll -gt 0)
if ($Offscreen -and $interactive) {
    Write-Error "-Offscreen cannot be combined with -Scroll/-HoverX/-HoldPress: those drive the real cursor, which needs the window on screen."
    exit 2
}
# Off-screen is the default for a plain capture; a hover/press needs it visible.
$park = $Offscreen -or (-not $interactive)

Add-Type -AssemblyName System.Windows.Forms
$wa = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
if ($Fullscreen) {
    $Width = $wa.Width
    $Height = $wa.Height
} elseif ($Height -le 0) {
    $Height = $wa.Height
}

# Far enough left that no monitor arrangement overlaps it, but still a real
# position: a minimized window does not render, an off-screen one does.
$parkX = -32000
$parkY = -32000

function Test-Blank([System.Drawing.Bitmap]$bmp) {
    # PrintWindow can hand back an untouched bitmap for a window that has not
    # presented a frame yet. Sample a grid rather than trusting it.
    $first = $null
    for ($x = 4; $x -lt $bmp.Width; $x += [Math]::Max(1, [int]($bmp.Width / 16))) {
        for ($y = 4; $y -lt $bmp.Height; $y += [Math]::Max(1, [int]($bmp.Height / 16))) {
            $c = $bmp.GetPixel($x, $y)
            if ($null -eq $first) { $first = $c; continue }
            if ($c.ToArgb() -ne $first.ToArgb()) { return $false }
        }
    }
    return $true
}

$Pages = $PageList.Split(",")
foreach ($pg in $Pages) {
    $env:HEROGPUI_PAGE = $pg
    if ($Theme) { $env:HEROGPUI_THEME = "dark" }
    # Hide the *console*, not the app. `gallery.exe` is a console-subsystem
    # binary, so launching it pops a console window that takes focus.
    # `CreateNoWindow` is the CREATE_NO_WINDOW creation flag: it suppresses that
    # console and nothing else, so the gpui window is still created and still
    # reports a handle. (`Start-Process -WindowStyle Hidden` hides both, which
    # leaves nothing to capture.) `HEROGPUI_UNFOCUSED` then keeps the gpui
    # window from taking focus.
    if (-not $interactive) { $env:HEROGPUI_UNFOCUSED = "1" }
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = "E:\work\HeroGPUI\target\debug\gallery.exe"
    $psi.WorkingDirectory = "E:\work\HeroGPUI"
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    $ok = $false
    $why = "no window"
    $shotW = 0; $shotH = 0
    for ($try = 0; $try -lt 20 -and -not $ok; $try++) {
        Start-Sleep -Milliseconds 700
        if ($p.HasExited) { break }
        $p.Refresh()
        $h = $p.MainWindowHandle
        if ($h -eq [IntPtr]::Zero) { continue }

        # Size it, and move it out of the way, without taking focus.
        $x = if ($park) { $parkX } else { 10 }
        $y = if ($park) { $parkY } else { 10 }
        [Win2]::SetWindowPos($h, [IntPtr]::Zero, $x, $y, $Width, $Height,
            $SWP_NOACTIVATE -bor $SWP_NOZORDER -bor $SWP_NOOWNERZORDER) | Out-Null
        Start-Sleep -Milliseconds 900

        if ($interactive) {
            # A real cursor needs the window under it and accepting input.
            [Win2]::SetForegroundWindow($h) | Out-Null
            Start-Sleep -Milliseconds 400
            if ($Scroll -gt 0) {
                [Win2]::SetCursorPos(900, 500) | Out-Null
                for ($w = 0; $w -lt $Scroll; $w++) {
                    [Win2]::mouse_event(0x0800, 0, 0, -120, 0)
                    Start-Sleep -Milliseconds 60
                }
                Start-Sleep -Milliseconds 700
            }
            if ($HoverX -ge 0) {
                [Win2]::SetCursorPos(10 + $HoverX, 10 + $HoverY) | Out-Null
                Start-Sleep -Milliseconds 2200
                if ($HoldPress) {
                    [Win2]::mouse_event(0x0002, 0, 0, 0, 0)
                    Start-Sleep -Milliseconds 500
                }
            }
        }

        if ([Win2]::IsIconic($h)) { $why = "window minimized"; continue }
        $r = New-Object Win2+RECT
        [Win2]::GetWindowRect($h, [ref]$r) | Out-Null
        $w = $r.Right - $r.Left; $hh = $r.Bottom - $r.Top
        if ($w -lt 400 -or $hh -lt 300) { $why = "window too small ($w x $hh)"; continue }
        $shotW = $w; $shotH = $hh

        $bmp = New-Object System.Drawing.Bitmap($w, $hh)
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $hdc = $g.GetHdc()
        $printed = [Win2]::PrintWindow($h, $hdc, 2)
        $g.ReleaseHdc($hdc)
        $g.Dispose()
        if ($HoldPress) { [Win2]::mouse_event(0x0004, 0, 0, 0, 0) }

        if (-not $printed) { $why = "PrintWindow failed"; $bmp.Dispose(); continue }
        if (Test-Blank $bmp) { $why = "PrintWindow returned a blank frame"; $bmp.Dispose(); continue }

        $tmp = Join-Path $OutDir "~tmp.png"
        $bmp.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
        $name2 = ($pg -replace ' ', '').ToLower() + "-v3.png"
        for ($sv = 0; $sv -lt 6; $sv++) {
            try { Move-Item $tmp (Join-Path $OutDir $name2) -Force; $ok = $true; break }
            catch { Start-Sleep -Milliseconds 800 }
        }
    }
    if ($p.HasExited) { "$pg : CRASHED" }
    elseif (-not $ok) { "$pg : FAILED ($why)"; Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
    else { "$pg : captured ($shotW x $shotH)"; Stop-Process -Id $p.Id -Force }
}
Remove-Item Env:HEROGPUI_PAGE -ErrorAction SilentlyContinue
Remove-Item Env:HEROGPUI_THEME -ErrorAction SilentlyContinue
Remove-Item Env:HEROGPUI_UNFOCUSED -ErrorAction SilentlyContinue
