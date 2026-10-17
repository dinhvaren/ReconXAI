param([switch]$Native, [switch]$ToolInstall, [switch]$Visual, [switch]$Inputs, [switch]$Jobs, [switch]$Integrations, [switch]$Outputs, [switch]$LocalInventory)
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
    if ($Native -or $Visual -or $Inputs -or $Jobs -or $Integrations -or $Outputs -or $LocalInventory) {
        Invoke-Checked cargo @('build','--locked','-p','reconx','-p','reconx-desktop','--features','reconx-desktop/custom-protocol')
        $previousToolFlag = $env:RECONX_E2E_TOOL_INSTALL
        $previousVisualFlag = $env:RECONX_E2E_VISUAL
        try {
            if ($ToolInstall) { $env:RECONX_E2E_TOOL_INSTALL='1' }
            if ($Visual) { $env:RECONX_E2E_VISUAL='1' }
            if($Native -or $Visual){Invoke-Checked node @('tests/e2e/native.mjs')}
            if($Inputs){Invoke-Checked node @('tests/e2e/inputs.mjs')}
            if($Jobs){Invoke-Checked node @('tests/e2e/jobs.mjs')}
            if($Integrations){Invoke-Checked node @('tests/e2e/integrations.mjs')}
            if($Outputs){Invoke-Checked node @('tests/e2e/outputs.mjs')}
            if($LocalInventory){Invoke-Checked node @('tests/e2e/recon-local.mjs')}
        } finally {
            $env:RECONX_E2E_TOOL_INSTALL=$previousToolFlag
            $env:RECONX_E2E_VISUAL=$previousVisualFlag
        }
    }
} finally { Pop-Location }
