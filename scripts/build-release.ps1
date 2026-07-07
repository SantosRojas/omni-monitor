$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  BUILD PRODUCCIÓN - Monitor OMNI" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# ---- 1. Compilar servidor (release) ----
Write-Host "[1/1] Compilando servidor (release)..." -ForegroundColor Yellow
cargo build --features ssr --release
if ($LASTEXITCODE -ne 0) { throw "Error compilando servidor" }
Write-Host "  -> OK`n" -ForegroundColor Green

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  BUILD COMPLETADO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Ejecutable: .\target\release\monitor-server.exe`n" -ForegroundColor White
