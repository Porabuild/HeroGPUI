# HeroGPUI: make `.vendor/` the patched GPUI sources, before anything runs cargo.
#
# The PowerShell half of `.shots/materialize.sh`; that file carries the full
# reasoning. In short: the workspace's `[patch.crates-io]` table points at five
# gitignored trees, cargo resolves those paths at manifest load -- before a
# build script could run and where a cargo alias cannot reach -- so every
# script here that shells out to cargo has to rebuild them first. Both halves
# call the same `.shots/gpui_patches.py`, so the work has one implementation.
#
#   & (Join-Path $PSScriptRoot 'materialize.ps1')
#
# Throws on failure. Every caller is a gate or a build, and all of them would
# otherwise fail a few seconds later with cargo's unexplanatory
# `failed to load source for dependency`.

# The probes below read $LASTEXITCODE deliberately, and PowerShell 7.4 turns a
# non-zero native exit code into a terminating error when the preference is
# 'Stop'. This runs in its own scope (callers invoke it with `&`), so neither
# assignment escapes back to the gate that called it.
$ErrorActionPreference = 'Continue'
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}

$root = Split-Path -Parent $PSScriptRoot
$materializer = Join-Path $root '.shots/gpui_patches.py'
# A revision from before the forks became patches has nothing to materialize.
if (-not (Test-Path $materializer)) { return }

$python = $null
foreach ($candidate in @('python3', 'python', 'py')) {
    if (-not (Get-Command $candidate -ErrorAction SilentlyContinue)) { continue }
    # Probed, not trusted by name: a bare `python` on Windows is often the
    # Microsoft Store install stub rather than an interpreter, and
    # `gpui_patches.py` imports `tomllib`, which arrives in Python 3.11.
    & $candidate -c 'import sys; raise SystemExit(sys.version_info < (3, 11))' 2>$null
    if ($LASTEXITCODE -eq 0) { $python = $candidate; break }
}
if (-not $python) {
    throw "no Python 3.11+ on PATH; .vendor/ cannot be rebuilt, and cargo will stop at 'failed to load source for dependency'"
}

# `--quiet` reports only the packages this run actually rebuilt, so the warm
# path adds no noise to the gate output it is standing in front of. Hook setup
# is deliberately left on: a developer who only ever runs the gates still ends
# up with the git hooks that make this automatic, and `gpui_patches.py` skips
# it when `CI` is set, so a runner checkout is never configured for nothing.
& $python $materializer --materialize --quiet
if ($LASTEXITCODE -ne 0) {
    throw "could not rebuild the patched GPUI sources under .vendor/ (exit $LASTEXITCODE)"
}
