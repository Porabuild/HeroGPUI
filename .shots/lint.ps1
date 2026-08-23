# The lint gate the workspace manifest points at.
#
# `[workspace.lints]` only applies to a crate that says `lints.workspace = true`,
# so a new crate that forgets the line silently opts out of the whole policy.
# This checks that first, then runs clippy with warnings denied so a warning
# fails rather than scrolls past.
#
#   .shots/lint.ps1            # check + clippy -D warnings
#   .shots/lint.ps1 -Fix       # apply the machine-applicable fixes first
param([switch]$Fix)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    # 1. Every member crate must opt in, or the policy is not what it looks like.
    $members = Get-ChildItem -Path (Join-Path $root 'crates') -Directory |
        ForEach-Object { Join-Path $_.FullName 'Cargo.toml' }
    $members += (Join-Path $root 'gallery/Cargo.toml')

    $missing = @()
    foreach ($m in $members) {
        if (-not (Test-Path $m)) { continue }
        $text = Get-Content $m -Raw
        if ($text -notmatch '(?m)^\s*workspace\s*=\s*true\s*$') {
            # `[lints]` with `workspace = true` under it is the only accepted form.
            $missing += (Resolve-Path -Relative $m)
        }
    }
    if ($missing.Count -gt 0) {
        Write-Host "these crates do not inherit [workspace.lints]:" -ForegroundColor Red
        $missing | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
        Write-Host "add:`n[lints]`nworkspace = true" -ForegroundColor Yellow
        exit 1
    }
    Write-Host ("all {0} crates inherit [workspace.lints]" -f $members.Count)

    if ($Fix) {
        # rustc's own lints (elided lifetimes, unused qualifications) are
        # machine-applicable; clippy's stylistic ones mostly are too.
        cargo fix --workspace --all-targets --allow-dirty
        cargo clippy --workspace --all-targets --fix --allow-dirty
    }

    # 2. Warnings are failures here. `--all-targets` covers tests, which is
    #    where float comparisons and unused imports usually hide.
    cargo clippy --workspace --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        Write-Host "clippy failed" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "clippy clean (warnings denied)" -ForegroundColor Green
}
finally {
    Pop-Location
}
