$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  DEV MODE - Monitor OMNI" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "[1/2] Iniciando frontend (http://localhost:5173)..." -ForegroundColor Yellow
$frontend = Start-Job -ScriptBlock {
    Set-Location "$using:ProjectRoot\frontend"
    npm run dev
}

Start-Sleep -Seconds 3

Write-Host "[2/2] Iniciando backend (http://localhost:9002)..." -ForegroundColor Yellow
Write-Host "Presiona Ctrl+C para detener ambos`n" -ForegroundColor Gray

try {
    cargo run
} finally {
    Write-Host "`nDeteniendo frontend..." -ForegroundColor Yellow
    Stop-Job $frontend -ErrorAction SilentlyContinue
    Remove-Job $frontend -ErrorAction SilentlyContinue
}
