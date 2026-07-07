$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  BUILD PRODUCCIÓN - Monitor OMNI" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# ---- 1. Build frontend ----
Write-Host "[1/2] Compilando frontend..." -ForegroundColor Yellow
Push-Location frontend
npm ci
npm run build
if ($LASTEXITCODE -ne 0) { throw "Error compilando frontend" }
Pop-Location
Write-Host "  -> OK`n" -ForegroundColor Green

# ---- 2. Compilar servidor (release) ----
Write-Host "[2/2] Compilando servidor (release)..." -ForegroundColor Yellow
cargo build --features ssr --release
if ($LASTEXITCODE -ne 0) { throw "Error compilando servidor" }
Write-Host "  -> OK`n" -ForegroundColor Green

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  BUILD COMPLETADO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Ejecutable: .\target\release\monitor-server.exe`n" -ForegroundColor White
