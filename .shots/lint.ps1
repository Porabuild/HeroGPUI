# Thin Windows wrapper around the lint gate, which lives in `.shots/lint.sh`
# so it runs on macOS, Linux and CI without pwsh. Git for Windows ships the
# bash this calls. Every check, and its exit code, is the bash script's.
#
#   .shots/lint.ps1            # same as: bash .shots/lint.sh
#   .shots/lint.ps1 -Fix       # same as: bash .shots/lint.sh --fix
param([switch]$Fix)

$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot 'lint.sh'
$arguments = @($script)
if ($Fix) { $arguments += '--fix' }
& bash @arguments
exit $LASTEXITCODE
