param([switch]$Native, [switch]$ToolInstall)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
function Invoke-Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}
Push-Location $repoRoot
try {
    Invoke-Checked cargo @('fmt','--all','--','--check')
    Invoke-Checked cargo @('clippy','--locked','--workspace','--all-targets','--','-D','warnings')
    Invoke-Checked cargo @('test','--locked','--workspace')
    Push-Location (Join-Path $repoRoot 'apps\desktop')
    try {
        Invoke-Checked npm @('run','build')
        Invoke-Checked npm @('test')
    } finally { Pop-Location }
    if ($Native) {
        Invoke-Checked cargo @('build','--locked','-p','reconx','-p','reconx-desktop','--features','reconx-desktop/custom-protocol')
        $previousToolFlag = $env:RECONX_E2E_TOOL_INSTALL
        try {
            if ($ToolInstall) { $env:RECONX_E2E_TOOL_INSTALL='1' }
            Invoke-Checked node @('tests/e2e/native.mjs')
        } finally { $env:RECONX_E2E_TOOL_INSTALL=$previousToolFlag }
    }
} finally { Pop-Location }
