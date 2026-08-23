# Launches the gallery once per page and reports any that panic.
#
# The gallery renders lazily, so a page can compile and still panic at runtime
# (gpui asserts on things like a second `.hover()` call). This walks every route
# so those surface in one pass.
param([int]$SecondsPerPage = 4)

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

$failed = @()
foreach ($pg in $pages) {
  $env:HEROGPUI_PAGE = $pg
  $out = [System.IO.Path]::GetTempFileName()
  $p = Start-Process -FilePath $exe -PassThru -WorkingDirectory "E:\work\HeroGPUI" `
        -RedirectStandardError $out -WindowStyle Minimized
  Start-Sleep -Seconds $SecondsPerPage
  if ($p.HasExited) {
    $err = (Get-Content $out -Raw)
    $failed += [pscustomobject]@{ Page = $pg; Exit = $p.ExitCode; Error = $err }
    Write-Host ("FAIL  {0}  (exit {1})" -f $pg, $p.ExitCode) -ForegroundColor Red
    if ($err) { Write-Host ($err.Trim()) -ForegroundColor DarkRed }
  } else {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    Write-Host ("ok    {0}" -f $pg)
  }
  Remove-Item $out -Force -ErrorAction SilentlyContinue
}

Remove-Item Env:\HEROGPUI_PAGE -ErrorAction SilentlyContinue
Write-Host ""
if ($failed.Count -eq 0) {
  Write-Host ("all {0} pages rendered" -f $pages.Count) -ForegroundColor Green
} else {
  Write-Host ("{0} of {1} pages failed" -f $failed.Count, $pages.Count) -ForegroundColor Red
  exit 1
}
