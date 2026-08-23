# Launches the gallery once per page and reports any that panic.
#
# The gallery renders lazily, so a page can compile and still panic at runtime
# (gpui asserts on things like a second `.hover()` call). This walks every route
# so those surface in one pass.
param([int]$SecondsPerPage = 4)

# A page that exits early counts as a failure only if it does so *twice*.
# Launching 71 gpui windows back to back intermittently kills one during
# startup: exit -1, empty stderr, a different page every run, and never a panic
# message. Reporting those made the gate unheedable. A real panic reproduces on
# the retry and prints to stderr, which is what the retry distinguishes.

# The window is parked off-screen rather than minimized. Minimizing is quieter
# but a minimized window may never present a frame, which would make this pass
# without having rendered anything -- the opposite of what it is for. Off-screen
# renders normally and still never covers what you are doing.
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Smoke {
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hh, uint flags);
}
"@
$SWP_NOACTIVATE = 0x0010
$SWP_NOZORDER = 0x0004

$exe = "E:\work\HeroGPUI\target\debug\gallery.exe"
if (-not (Test-Path $exe)) { throw "build the gallery first: cargo build --workspace" }

# Keep in sync with Page::title in gallery/src/pages/mod.rs.
$pages = @(
  "Introduction", "Installation", "Theming", "Dark Mode", "Customization",
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

$env:HEROGPUI_UNFOCUSED = "1"
# Launches one page and returns $null if it was still running at the deadline,
# or the (exit code, stderr) pair if it had already gone.
function Invoke-Page($pg, $exe, $SecondsPerPage, $SWP_NOACTIVATE, $SWP_NOZORDER) {
  $env:HEROGPUI_PAGE = $pg
  # Hide the *console*, not the app. `gallery.exe` is a console-subsystem
  # binary, so 71 launches would pop 71 console windows and take focus 71 times.
  # `CreateNoWindow` is the CREATE_NO_WINDOW creation flag: it suppresses that
  # console only, leaving the gpui window created and reporting a handle.
  # (`-WindowStyle Hidden` hides both, and a hidden window never renders — which
  # would make this pass without having rendered anything.)
  $psi = New-Object System.Diagnostics.ProcessStartInfo
  $psi.FileName = $exe
  $psi.WorkingDirectory = "E:\work\HeroGPUI"
  $psi.UseShellExecute = $false
  $psi.CreateNoWindow = $true
  # stderr is read only once the process has exited, so the panic message is
  # still reported and a live process cannot block on a full pipe.
  $psi.RedirectStandardError = $true
  $p = [System.Diagnostics.Process]::Start($psi)
  # Move it out of sight as soon as it has a window, without taking focus.
  for ($t = 0; $t -lt 12; $t++) {
    Start-Sleep -Milliseconds 250
    if ($p.HasExited) { break }
    $p.Refresh()
    if ($p.MainWindowHandle -ne [IntPtr]::Zero) {
      [Smoke]::SetWindowPos($p.MainWindowHandle, [IntPtr]::Zero, -32000, -32000, 1690, 900,
        $SWP_NOACTIVATE -bor $SWP_NOZORDER) | Out-Null
      break
    }
  }
  Start-Sleep -Seconds $SecondsPerPage
  if ($p.HasExited) {
    return [pscustomobject]@{ Exit = $p.ExitCode; Error = $p.StandardError.ReadToEnd() }
  }
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  return $null
}

$failed = @()
foreach ($pg in $pages) {
  $r = Invoke-Page $pg $exe $SecondsPerPage $SWP_NOACTIVATE $SWP_NOZORDER
  if ($null -ne $r) {
    # Second chance: see the note on $SecondsPerPage above.
    Start-Sleep -Milliseconds 500
    $r2 = Invoke-Page $pg $exe $SecondsPerPage $SWP_NOACTIVATE $SWP_NOZORDER
    if ($null -eq $r2) {
      Write-Host ("ok    {0}  (first launch died with exit {1}; retry rendered)" -f $pg, $r.Exit) -ForegroundColor DarkYellow
      continue
    }
    $failed += [pscustomobject]@{ Page = $pg; Exit = $r2.Exit; Error = $r2.Error }
    Write-Host ("FAIL  {0}  (exit {1}, twice)" -f $pg, $r2.Exit) -ForegroundColor Red
    if ($r2.Error) { Write-Host ($r2.Error.Trim()) -ForegroundColor DarkRed }
  } else {
    Write-Host ("ok    {0}" -f $pg)
  }
}

Remove-Item Env:\HEROGPUI_PAGE -ErrorAction SilentlyContinue
Remove-Item Env:\HEROGPUI_UNFOCUSED -ErrorAction SilentlyContinue
Write-Host ""
if ($failed.Count -eq 0) {
  Write-Host ("all {0} pages rendered" -f $pages.Count) -ForegroundColor Green
} else {
  Write-Host ("{0} of {1} pages failed" -f $failed.Count, $pages.Count) -ForegroundColor Red
  exit 1
}
