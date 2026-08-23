# Re-captures every page's reference screenshot, in one process.
#
# `capture2.ps1` relaunches the gallery per page, so a full refresh -- which any
# change to a shared metric needs -- took the better part of ten minutes. This
# drives one process through all 73 pages instead (see `batch.ps1`), which is
# under a minute, and writes the same `<page>-v3.png` names.
#
#   .\.shots\refresh.ps1                 # every page
#   .\.shots\refresh.ps1 -Pages Table,Tabs
#   .\.shots\refresh.ps1 -Overlays       # with every overlay demo open
param(
    [string[]]$Pages = @(),
    [switch]$Overlays,
    [switch]$Dark,
    [string]$OutDir = "E:\work\HeroGPUI\.shots"
)

$all = @(
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

$wanted = if ($Pages.Count -gt 0) { $Pages } else { $all }
$steps = foreach ($page in $wanted) {
    $slug = ($page -replace ' ', '').ToLower()
    $step = @{ page = $page; out = (Join-Path $OutDir "$slug-v3.png") }
    if ($Overlays) { $step.overlays = '1' }
    if ($Dark) { $step.theme = 'dark' }
    $step
}

& "E:\work\HeroGPUI\.shots\batch.ps1" -Steps $steps
