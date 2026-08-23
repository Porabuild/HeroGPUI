param([string[]]$Pages = @(), [string]$OutDir = "E:\work\HeroGPUI\.shots")

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool repaint);
    public struct RECT { public int Left, Top, Right, Bottom; }
}
"@

foreach ($pg in $Pages) {
    $env:HEROGPUI_PAGE = $pg
    $p = Start-Process -FilePath "E:\work\HeroGPUI\target\debug\gallery.exe" -PassThru -WorkingDirectory "E:\work\HeroGPUI"
    $ok = $false
    for ($try = 0; $try -lt 20 -and -not $ok; $try++) {
        Start-Sleep -Milliseconds 700
        if ($p.HasExited) { break }
        $p.Refresh()
        $h = $p.MainWindowHandle
        if ($h -ne [IntPtr]::Zero) {
            # Force a large window regardless of DPI virtualization quirks.
            [Win]::MoveWindow($h, 10, 10, 1690, 700, $true) | Out-Null
            [Win]::SetForegroundWindow($h) | Out-Null
            Start-Sleep -Milliseconds 900
            $r = New-Object Win+RECT
            [Win]::GetWindowRect($h, [ref]$r) | Out-Null
            $w = $r.Right - $r.Left; $hh = $r.Bottom - $r.Top
            if ($w -gt 400 -and $hh -gt 300) {
                $bmp = New-Object System.Drawing.Bitmap($w, $hh)
                $g = [System.Drawing.Graphics]::FromImage($bmp)
                $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($w, $hh)))
                $g.Dispose()
                $name = ($pg -replace ' ', '').ToLower()
                $bmp.Save("$OutDir\$name-v2.png", [System.Drawing.Imaging.ImageFormat]::Png)
                $bmp.Dispose()
                $ok = $true
            }
        }
    }
    if ($p.HasExited) { "$pg : CRASHED" }
    elseif (-not $ok) { "$pg : NO WINDOW" ; Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
    else { "$pg : captured"; Stop-Process -Id $p.Id -Force }
}
Remove-Item Env:HEROGPUI_PAGE -ErrorAction SilentlyContinue

