param([switch]$Installer)
$ErrorActionPreference='Stop'
$repoRoot=Split-Path -Parent $PSScriptRoot
$previousIncremental=$env:CARGO_INCREMENTAL
$env:CARGO_INCREMENTAL='0'
function Invoke-Checked([string]$Program,[string[]]$Arguments) {
    & $Program @Arguments
    if($LASTEXITCODE -ne 0){throw "$Program failed with exit code $LASTEXITCODE"}
}
Push-Location $repoRoot
try {
    Push-Location (Join-Path $repoRoot 'apps\desktop')
    try { Invoke-Checked npm @('run','build') } finally { Pop-Location }
    Invoke-Checked cargo @('build','--locked','--release','-p','reconx','-p','reconx-desktop','--features','reconx-desktop/custom-protocol')
    $destination=Join-Path $repoRoot 'artifacts\windows-x64'
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    foreach($binary in @('reconx.exe','reconx-desktop.exe')){
        Copy-Item -LiteralPath (Join-Path $repoRoot "target\release\$binary") -Destination (Join-Path $destination $binary)
    }
    $lines=foreach($binary in @('reconx.exe','reconx-desktop.exe')){
        $digest=(Get-FileHash -LiteralPath (Join-Path $destination $binary) -Algorithm SHA256).Hash.ToLowerInvariant()
        "$digest  $binary"
    }
    [System.IO.File]::WriteAllLines((Join-Path $destination 'SHA256SUMS.txt'),$lines,[System.Text.UTF8Encoding]::new($false))
    $fontRoot=Join-Path $repoRoot 'apps\desktop\src\assets\fonts'
    $fontNotices="ReconX AI bundled fonts`n`nSource Sans 3`n`n" + [System.IO.File]::ReadAllText((Join-Path $fontRoot 'sourcesans3\LICENSE.md')) + "`n`nDM Sans (wordmark only)`n`n" + [System.IO.File]::ReadAllText((Join-Path $fontRoot 'dmsans\OFL.txt')) + "`n`nIBM Plex Mono`n`n" + [System.IO.File]::ReadAllText((Join-Path $fontRoot 'ibmplexmono\OFL.txt'))
    [System.IO.File]::WriteAllText((Join-Path $destination 'THIRD_PARTY_NOTICES.txt'),$fontNotices,[System.Text.UTF8Encoding]::new($false))
    if($Installer){
        Push-Location (Join-Path $repoRoot 'apps\desktop')
        try { Invoke-Checked npm @('run','tauri','--','build','--config','../../packaging/tauri.windows.json') } finally { Pop-Location }
    }
    Write-Output "Artifacts: $destination"
} finally { Pop-Location; $env:CARGO_INCREMENTAL=$previousIncremental }
