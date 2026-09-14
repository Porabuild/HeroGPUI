# Shared file protocol for batch and smoke drivers. Requires PowerShell 7.
# Publish complete UTF-8 requests atomically; never expose a truncated command
# which contains a new sequence number and the previous page's defaults.
function Write-GalleryControl {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string[]]$Lines
    )
    $ErrorActionPreference = 'Stop'
    $temporary = "$Path.$([Guid]::NewGuid().ToString('N')).tmp"
    $ack = [System.IO.Path]::ChangeExtension($Path, '.ack')
    $errorPath = [System.IO.Path]::ChangeExtension($Path, '.error')
    try {
        [System.IO.File]::WriteAllText(
            $temporary,
            ($Lines -join "`n"),
            [System.Text.UTF8Encoding]::new($false)
        )
        Remove-Item $ack, $errorPath -ErrorAction SilentlyContinue
        [System.IO.File]::Move($temporary, $Path, $true)
    } finally {
        Remove-Item $temporary -ErrorAction SilentlyContinue
    }
}

function Wait-GalleryControl {
    param(
        [Parameter(Mandatory)][System.Diagnostics.Process]$Process,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Sequence,
        [int]$TimeoutMs = 3600
    )
    $ack = [System.IO.Path]::ChangeExtension($Path, '.ack')
    $errorPath = [System.IO.Path]::ChangeExtension($Path, '.error')
    $elapsed = [System.Diagnostics.Stopwatch]::StartNew()
    while ($elapsed.ElapsedMilliseconds -lt $TimeoutMs) {
        if ($Process.HasExited) {
            return @{ ok = $false; error = 'gallery exited before acknowledging' }
        }
        $reported = @(Get-Content $errorPath -ErrorAction SilentlyContinue)
        if ($reported.Count -gt 0 -and $reported[0] -ceq "seq=$Sequence") {
            return @{ ok = $false; error = ($reported -join ' ') }
        }
        $seen = Get-Content $ack -Raw -ErrorAction SilentlyContinue
        if ($seen -and $seen.Trim() -ceq $Sequence) {
            return @{ ok = $true; error = '' }
        }
        Start-Sleep -Milliseconds 50
    }
    return @{ ok = $false; error = "no rendered-frame acknowledgement for $Sequence" }
}
