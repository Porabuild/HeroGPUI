param([string]$PageList = "", [string]$OutDir = "E:\work\HeroGPUI\.shots", [int]$Width = 1690, [int]$Height = 700, [int]$Scroll = 0, [int]$HoverX = -1, [int]$HoverY = -1, [switch]$HoldPress)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win2 {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool repaint);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, int d, int extra);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    public struct RECT { public int Left, Top, Right, Bottom; }
}
"@

$Pages = $PageList.Split(","); foreach ($pg in $Pages) {
    $env:HEROGPUI_PAGE = $pg
    $p = Start-Process -FilePath "E:\work\HeroGPUI\target\debug\gallery.exe" -PassThru -WorkingDirectory "E:\work\HeroGPUI"
    $ok = $false
    for ($try = 0; $try -lt 20 -and -not $ok; $try++) {
        Start-Sleep -Milliseconds 700
        if ($p.HasExited) { break }
        $p.Refresh()
        $h = $p.MainWindowHandle
        if ($h -ne [IntPtr]::Zero) {
            [Win2]::MoveWindow($h, 10, 10, $Width, $Height, $true) | Out-Null
            [Win2]::SetForegroundWindow($h) | Out-Null
            Start-Sleep -Milliseconds 900
            # Scroll the page body so sections below the fold can be captured.
            if ($Scroll -gt 0) {
                [Win2]::SetCursorPos(900, 500) | Out-Null
                for ($w = 0; $w -lt $Scroll; $w++) {
                    [Win2]::mouse_event(0x0800, 0, 0, -120, 0)
                    Start-Sleep -Milliseconds 60
                }
                Start-Sleep -Milliseconds 700
            }
            # Park the cursor on a control so hover-only surfaces (tooltips)
            # are visible in the capture.
            if ($HoverX -ge 0) {
                [Win2]::SetCursorPos(10 + $HoverX, 10 + $HoverY) | Out-Null
                Start-Sleep -Milliseconds 2200
                # Hold the left button so `:active` styling is on screen for the
                # capture; released after the shot.
                if ($HoldPress) {
                    [Win2]::mouse_event(0x0002, 0, 0, 0, 0)
                    Start-Sleep -Milliseconds 500
                }
            }
            $r = New-Object Win2+RECT
            [Win2]::GetWindowRect($h, [ref]$r) | Out-Null
            $w = $r.Right - $r.Left; $hh = $r.Bottom - $r.Top
            if ($w -gt 400 -and $hh -gt 300) {
                $bmp = New-Object System.Drawing.Bitmap($w, $hh)
                $g = [System.Drawing.Graphics]::FromImage($bmp)
                $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($w, $hh)))
                $g.Dispose()
                $tmp = Join-Path $OutDir "~tmp.png"
                $bmp.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
                $bmp.Dispose()
                if ($HoldPress) { [Win2]::mouse_event(0x0004, 0, 0, 0, 0) }
                $name2 = ($pg -replace ' ', '').ToLower() + "-v3.png"
                for ($sv = 0; $sv -lt 6; $sv++) {
                    try { Move-Item $tmp (Join-Path $OutDir $name2) -Force; $ok = $true; break } catch { Start-Sleep -Milliseconds 800 }
                }
            }
        }
    }
    if ($p.HasExited) { "$pg : CRASHED" }
    elseif (-not $ok) { "$pg : FAILED"; Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
    else { "$pg : captured"; Stop-Process -Id $p.Id -Force }
}
Remove-Item Env:HEROGPUI_PAGE -ErrorAction SilentlyContinue

