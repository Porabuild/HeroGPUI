# One-component native/WASM parity loop.
#
#   .shots/parity-loop.ps1 -Slug <component-slug> [-Message "..."]
#   .shots/parity-loop.ps1 -All
#
# Per slug the loop transplants composition-only example bodies (layout
# helpers, placeholders, common builder props) from the native gallery into
# the WASM migration source, rebuilds the real artifact, regenerates the
# parity manifest, runs the web gates, and commits ONLY the owned generated
# files. API-gap examples (missing builder methods, callbacks) are skipped
# with a report; they need a narrow component port first, not a transplant.
#
# -All iterates every drifted page smallest-first and commits after each,
# stopping on the first failure so a red step never hides behind a green one.

param(
  [string]$Slug = "",
  [switch]$All,
  [string]$Message = ""
)

$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$wasmSource = "D:/herogpui-wasm/gallery/src/pages/components.rs"

function Invoke-Step([string]$Command) {
  Write-Host "==> $Command" -ForegroundColor Cyan
  Invoke-Expression $Command
  if ($LASTEXITCODE -ne 0) { throw "step failed ($LASTEXITCODE): $Command" }
}

function Get-DriftSlugs {
  $json = Get-Content (Join-Path $root "web/src/data/wasm-parity.json") -Raw | ConvertFrom-Json
  $groups = @{}
  foreach ($key in $json.codeDrift) {
    $slug = $key.Split("/")[0]
    if (-not $groups.ContainsKey($slug)) { $groups[$slug] = 0 }
    $groups[$slug] += 1
  }
  return $groups.GetEnumerator() | Sort-Object Value | ForEach-Object { $_.Key }
}

function Invoke-Slug([string]$Name, [string]$CommitMessage) {
  Set-Location $root
  $before = (Get-Content web/src/data/wasm-parity.json -Raw | ConvertFrom-Json).codeDrift.Count

  Write-Host "--- plan: $Name ---" -ForegroundColor Green
  node web/scripts/sync-parity-component.mjs plan $Name
  if ($LASTEXITCODE -ne 0) { throw "plan failed for $Name" }

  $backup = Join-Path ([System.IO.Path]::GetTempPath()) "parity-components-backup.rs"
  Copy-Item $wasmSource $backup -Force
  $extracted = $false

  try {
    $syncOut = node web/scripts/sync-parity-component.mjs sync $Name --composition-only 2>&1
    if ($LASTEXITCODE -ne 0) { $syncOut; throw "sync failed for $Name" }
    $syncOut
    if ($syncOut -match "0 transplanted" -and -not ($syncOut -match "already synced")) {
      Write-Host "no composition-only examples on $Name; needs a component port first" -ForegroundColor Yellow
      return
    }

  Invoke-Step "node web/scripts/lift-wasm-descriptions.mjs '$wasmSource'"

  $env:CARGO_TARGET_DIR = "D:/herogpui-wasm-target"
  $env:CARGO_HOME = "D:/cargo-home"
  # An aborted build can leave fingerprints newer than the synced source, so
  # cargo wrongly reports "fresh" and the old binary ships under a new
  # manifest. Touching the source forces a real recompile every iteration.
  (Get-Item $wasmSource).LastWriteTime = Get-Date
  Push-Location "D:/herogpui-wasm"
  try {
    Invoke-Step "cargo build --target wasm32-unknown-unknown --profile wasm-release -p herogpui-web"
  } finally {
    Pop-Location
  }
  Invoke-Step "D:/cargo-home/bin/wasm-bindgen.exe D:/herogpui-wasm-target/wasm32-unknown-unknown/wasm-release/herogpui_web.wasm --out-dir '$root/web/public/gallery' --target web --no-typescript"
  Invoke-Step "node web/scripts/extract-wasm-sections.mjs --source '$wasmSource'"
  # The artifact is built from a separate checkout. Vendoring its baseline and
  # working diff in the same commit keeps the shipped binary reviewable.
  Invoke-Step "node web/scripts/vendor-wasm-source.mjs"
  $extracted = $true

  $after = (Get-Content web/src/data/wasm-parity.json -Raw | ConvertFrom-Json).codeDrift.Count
  Write-Host "drift: $before -> $after" -ForegroundColor Green

  Set-Location (Join-Path $root "web")
  Invoke-Step "pnpm run extract:check"
  Invoke-Step "pnpm run typecheck"
  Invoke-Step "pnpm run lint"
  Invoke-Step "pnpm run build"

  Set-Location $root
  git add web/public/gallery/herogpui_web.js web/public/gallery/herogpui_web_bg.wasm web/src/data/wasm-sections.json web/src/data/wasm-parity.json web/wasm-migration/source.json web/wasm-migration/wasm-migration.patch
  $staged = git diff --cached --name-only
  if (-not $staged) { throw "nothing staged after $Name; refusing empty commit" }
  $unexpected = $staged | Where-Object { $_ -notin @(
    "web/public/gallery/herogpui_web.js",
    "web/public/gallery/herogpui_web_bg.wasm",
    "web/src/data/wasm-sections.json",
    "web/src/data/wasm-parity.json",
    "web/wasm-migration/source.json",
    "web/wasm-migration/wasm-migration.patch"
  ) }
  if ($unexpected) { throw "unexpected staged files: $unexpected" }
  if ([string]::IsNullOrWhiteSpace($CommitMessage)) { $CommitMessage = "feat: sync $Name wasm examples with native" }
  git commit -m $CommitMessage
  if ($LASTEXITCODE -ne 0) { throw "commit failed for $Name" }
  Write-Host "committed $Name ($before -> $after)" -ForegroundColor Green
  } catch {
    if (-not $extracted) { Copy-Item $backup $wasmSource -Force }
    throw "loop failed for $Name (source restored: $(-not $extracted)): $_"
  }
}

if ($All) {
  foreach ($slug in Get-DriftSlugs) { Invoke-Slug $slug "" }
} elseif ($Slug) {
  Invoke-Slug $Slug $Message
} else {
  throw "pass -Slug <component> or -All"
}
