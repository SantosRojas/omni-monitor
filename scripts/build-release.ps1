$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  BUILD PRODUCCIÓN - Monitor OMNI" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# ---- 1. Compilar frontend WASM (release) ----
Write-Host "[1/3] Compilando frontend WASM (release)..." -ForegroundColor Yellow
cargo build --no-default-features --features frontend --target wasm32-unknown-unknown --release
if ($LASTEXITCODE -ne 0) { throw "Error compilando frontend WASM" }
Write-Host "  -> OK`n" -ForegroundColor Green

# ---- 2. wasm-bindgen ----
Write-Host "[2/3] Ejecutando wasm-bindgen (release)..." -ForegroundColor Yellow
wasm-bindgen --out-dir static/pkg --target web ./target/wasm32-unknown-unknown/release/monitor.wasm
if ($LASTEXITCODE -ne 0) { throw "Error ejecutando wasm-bindgen" }
Write-Host "  -> OK`n" -ForegroundColor Green

# ---- 3. Compilar servidor (release) ----
Write-Host "[3/3] Compilando servidor (release)..." -ForegroundColor Yellow
cargo build --features ssr --release
if ($LASTEXITCODE -ne 0) { throw "Error compilando servidor" }
Write-Host "  -> OK`n" -ForegroundColor Green

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  BUILD COMPLETADO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Ejecutable: .\target\release\monitor-server.exe`n" -ForegroundColor White
