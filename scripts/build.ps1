param([switch]$Installer)
$ErrorActionPreference='Stop'
$repoRoot=Split-Path -Parent $PSScriptRoot
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
    if($Installer){
        Push-Location (Join-Path $repoRoot 'apps\desktop')
        try { Invoke-Checked npm @('run','tauri','--','build','--config','../../packaging/tauri.windows.json') } finally { Pop-Location }
    }
    Write-Output "Artifacts: $destination"
} finally { Pop-Location }
