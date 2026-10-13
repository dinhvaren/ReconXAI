$ErrorActionPreference='Stop'
$repoRoot=Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    cargo build --locked -p reconx
    if($LASTEXITCODE -ne 0){throw 'CLI build failed'}
    Set-Location (Join-Path $repoRoot 'apps\desktop')
    if(-not(Test-Path -LiteralPath 'node_modules')){
        npm ci --registry https://registry.npmjs.org
        if($LASTEXITCODE -ne 0){throw 'Frontend dependency installation failed'}
    }
    npm run tauri -- dev
    if($LASTEXITCODE -ne 0){throw 'Desktop development process failed'}
} finally { Pop-Location }
