# Exercises the file protocol without a GUI or a gallery build.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'control.ps1')

function Assert-Control([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

$directory = Join-Path ([System.IO.Path]::GetTempPath()) "herogpui-control-test-$([Guid]::NewGuid().ToString('N'))"
[void][System.IO.Directory]::CreateDirectory($directory)
$control = Join-Path $directory 'control.txt'
$ack = [System.IO.Path]::ChangeExtension($control, '.ack')
$errorPath = [System.IO.Path]::ChangeExtension($control, '.error')
$process = [System.Diagnostics.Process]::GetCurrentProcess()
try {
    [System.IO.File]::WriteAllText($ack, 'old')
    [System.IO.File]::WriteAllText($errorPath, 'old error')
    Write-GalleryControl -Path $control -Lines @('seq=1', 'page=Button', 'section=État')
    Assert-Control ([System.IO.File]::ReadAllText($control) -eq "seq=1`npage=Button`nsection=État") 'publish must retain the complete UTF-8 request'
    $bytes = [System.IO.File]::ReadAllBytes($control)
    Assert-Control ($bytes[0] -eq [byte][char]'s') 'publish must not add a UTF-8 BOM'
    Assert-Control (-not (Test-Path $ack) -and -not (Test-Path $errorPath)) 'new requests must retire previous result files'
    Assert-Control (@(Get-ChildItem $directory -Filter '*.tmp').Count -eq 0) 'publish must leave no temporary files'

    [System.IO.File]::WriteAllText($ack, 'other')
    $result = Wait-GalleryControl -Process $process -Path $control -Sequence '1' -TimeoutMs 100
    Assert-Control (-not $result.ok) 'a different sequence cannot acknowledge this request'
    [System.IO.File]::WriteAllText($ack, '1')
    $result = Wait-GalleryControl -Process $process -Path $control -Sequence '1'
    Assert-Control $result.ok 'the matching sequence must acknowledge the request'
    [System.IO.File]::WriteAllText($errorPath, "seq=1`nerror=unknown page")
    $result = Wait-GalleryControl -Process $process -Path $control -Sequence '1'
    Assert-Control (-not $result.ok -and $result.error.Contains('unknown page')) 'a matching error must surface even if a stale acknowledgement exists'

    [System.IO.File]::WriteAllText($ack, 'RUN-A')
    [System.IO.File]::WriteAllText($errorPath, "seq=RUN-A`nerror=wrong case")
    $result = Wait-GalleryControl -Process $process -Path $control -Sequence 'run-a' -TimeoutMs 100
    Assert-Control (-not $result.ok -and -not $result.error.Contains('wrong case')) 'sequences must match case for acknowledgements and errors'

    # Exercise the real drivers' cleanup blocks after a publication failure.
    # Skip their Win32 bootstrap so this failure path also runs without a GUI.
    foreach ($driver in @('batch.ps1', 'smoke.ps1')) {
        $tokens = $null; $parseErrors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile(
            (Join-Path $PSScriptRoot $driver), [ref]$tokens, [ref]$parseErrors)
        Assert-Control ($parseErrors.Count -eq 0) "$driver must parse"
        $loop = if ($driver -eq 'batch.ps1') { 'foreach ($step in $Steps)' } else { 'if ($PerProcess)' }
        $lifecycle = $ast.Find({
            param($node)
            $node -is [System.Management.Automation.Language.TryStatementAst] -and
                $node.Body.Extent.Text.Contains($loop)
        }, $true)
        Assert-Control ($null -ne $lifecycle -and $null -ne $lifecycle.Finally) "$driver must own its loop in try/finally"
        $body = $lifecycle.Body.Extent.Text
        $entry = if ($driver -eq 'batch.ps1') { '$failed = @()' } else { 'if ($PerProcess)' }
        $offset = $body.IndexOf($entry)
        Assert-Control ($offset -gt 0) "$driver must contain the loop setup"
        $failurePath = [scriptblock]::Create('try {' + $body.Substring($offset) + ' finally ' + $lifecycle.Finally.Extent.Text)
        $caseDirectory = Join-Path $directory $driver
        [void][System.IO.Directory]::CreateDirectory($caseDirectory)
        & {
            $control = Join-Path $caseDirectory 'blocked.txt'
            $ack = [System.IO.Path]::ChangeExtension($control, '.ack')
            $errorPath = [System.IO.Path]::ChangeExtension($control, '.error')
            [void][System.IO.Directory]::CreateDirectory($control)
            $psi = [System.Diagnostics.ProcessStartInfo]::new((Join-Path $PSHOME $(if ($IsWindows) { 'pwsh.exe' } else { 'pwsh' })))
            $psi.UseShellExecute = $false
            foreach ($argument in @('-NoProfile', '-NonInteractive', '-Command', 'Start-Sleep -Seconds 30')) {
                [void]$psi.ArgumentList.Add($argument)
            }
            $p = [System.Diagnostics.Process]::new()
            $p.StartInfo = $psi
            [void]$p.Start()
            $observer = [System.Diagnostics.Process]::GetProcessById($p.Id)
            $started = $true
            $Steps = @(@{page='Button'})
            $pages = @('Button')
            $PerProcess = $false; $NoShot = $true; $Quiet = $true
            $session = $null
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            function Start-Gallery { return @{proc=$p; err=[System.Threading.Tasks.Task]::FromResult('')} }
            $priorControl = $env:HEROGPUI_CONTROL
            $rejected = $false
            try {
                try { & $failurePath }
                catch {
                    Assert-Control ($_.Exception.InnerException -is [System.IO.IOException]) "$driver must fail because publication was rejected: $_"
                    $rejected = $true
                }
                Assert-Control $rejected "$driver must surface publication failure"
                Assert-Control $observer.HasExited "$driver must stop the child after publication failure"
                Assert-Control (-not (Test-Path $control) -and -not (Test-Path $ack) -and -not (Test-Path $errorPath)) "$driver must remove control results after stopping the child"
                Assert-Control ($env:HEROGPUI_CONTROL -ceq $priorControl) "$driver must preserve its caller's environment"
            } finally {
                if (-not $observer.HasExited) { $observer.Kill(); $observer.WaitForExit() }
                $observer.Dispose()
                $p.Dispose()
            }
        }
    }

    if ($IsWindows) {
        $before = [System.IO.File]::ReadAllText($control)
        $locked = [System.IO.File]::Open($control, 'Open', 'Read', 'None')
        $rejected = $false
        try {
            try {
                Write-GalleryControl -Path $control -Lines @('seq=2', 'page=Select')
            } catch {
                $rejected = $true
            }
        } finally {
            $locked.Dispose()
        }
        Assert-Control $rejected 'a locked destination must reject publication'
        Assert-Control ([System.IO.File]::ReadAllText($control) -eq $before) 'failed publication must preserve the previous request'
        Assert-Control (@(Get-ChildItem $directory -Filter '*.tmp').Count -eq 0) 'failed publication must remove its temporary file'
    }
    Write-Host 'Control protocol tests passed.'
} finally {
    Remove-Item $directory -Recurse -Force
    $process.Dispose()
}
