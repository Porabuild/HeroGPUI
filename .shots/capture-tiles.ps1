# Isolated catalog tiles: one Usage example, light and dark, no gallery chrome.
#
# Launches the gallery in `HEROGPUI_PREVIEW=component` so each card is the
# component itself, then walks every catalog page through the shared batch
# driver.
#
#   .\.shots\capture-tiles.ps1
#   .\.shots\capture-tiles.ps1 -PageList "Button,Switch"
param(
    [string]$PageList = "",
    [int]$Width = 640,
    [int]$Height = 400
)

$ErrorActionPreference = "Stop"
$repo = Split-Path $PSScriptRoot -Parent
$web = Join-Path $repo "web"
$catalogPath = Join-Path $web "src\data\catalog.json"
$examplesPath = Join-Path $web "src\data\rust-examples.json"
if (-not (Test-Path $catalogPath)) { throw "missing $catalogPath" }

$catalog = Get-Content $catalogPath -Raw | ConvertFrom-Json
$examples = $null
if (Test-Path $examplesPath) {
    $examples = Get-Content $examplesPath -Raw | ConvertFrom-Json
}

$wanted = @()
if ($PageList) {
    $wanted = $PageList.Split(",") | ForEach-Object { $_.Trim().ToLower() } | Where-Object { $_ }
}

$overlayPages = @(
    "Alert Dialog", "Drawer", "Modal", "Popover", "Toast", "Tooltip"
)

$steps = @()
foreach ($entry in $catalog.components.PSObject.Properties) {
    $component = $entry.Value
    if ($wanted.Count -gt 0 -and $wanted -notcontains $component.title.ToLower() -and $wanted -notcontains $component.slug) {
        continue
    }

    $section = "Usage"
    $ex = $null
    if ($examples) { $ex = $examples.($component.slug) }
    if ($ex) {
        $usage = @($ex | Where-Object { $_.heading -eq "Usage" } | Select-Object -First 1)
        if ($usage.Count -eq 0) { $section = $ex[0].heading }
    }

    $base = ($component.title.ToLower() -replace " ", "")
    $overlays = if ($overlayPages -contains $component.title) { "1" } else { $null }
    $steps += @{
        page     = $component.title
        section  = $section
        overlays = $overlays
        out      = (Join-Path $PSScriptRoot "$base-tile-v3.png")
    }
    $steps += @{
        page     = $component.title
        section  = $section
        theme    = "dark"
        overlays = $overlays
        out      = (Join-Path $PSScriptRoot "$base-tile-dark-v3.png")
    }
}

if ($steps.Count -eq 0) { throw "no catalog pages matched" }

$env:HEROGPUI_PREVIEW = "component"
try {
    & (Join-Path $PSScriptRoot "batch.ps1") -Width $Width -Height $Height -ClientOnly -Steps $steps
} finally {
    Remove-Item Env:HEROGPUI_PREVIEW -ErrorAction SilentlyContinue
}
