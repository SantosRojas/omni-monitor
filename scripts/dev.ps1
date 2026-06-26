$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

# ---- Matar instancia previa del servidor si existe ----
$prev = Get-Process monitor-server -ErrorAction SilentlyContinue
if ($prev) {
    Write-Host "  -> Deteniendo servidor previo..." -ForegroundColor Yellow
    $prev | Stop-Process -Force
    Start-Sleep -Seconds 1
}

# ---- Verificar herramientas ----
Write-Host "`n[1/4] Verificando herramientas..." -ForegroundColor Cyan

$hasWatch = Get-Command cargo-watch -ErrorAction SilentlyContinue
if (-not $hasWatch) {
    Write-Host "  -> Instalando cargo-watch..." -ForegroundColor Yellow
    cargo install cargo-watch
}

$hasWasmTarget = rustup target list --installed | Select-String "wasm32-unknown-unknown"
if (-not $hasWasmTarget) {
    Write-Host "  -> Agregando target wasm32-unknown-unknown..." -ForegroundColor Yellow
    rustup target add wasm32-unknown-unknown
}

Write-Host "  -> OK" -ForegroundColor Green

# ---- Build WASM inicial ----
Write-Host "`n[2/4] Compilando frontend WASM (debug)..." -ForegroundColor Cyan
cargo build --no-default-features --features frontend --target wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) { throw "Error compilando frontend WASM" }

# ---- wasm-bindgen inicial ----
Write-Host "`n[3/4] Ejecutando wasm-bindgen..." -ForegroundColor Cyan
wasm-bindgen --out-dir static/pkg --target web ./target/wasm32-unknown-unknown/debug/monitor.wasm
if ($LASTEXITCODE -ne 0) { throw "Error ejecutando wasm-bindgen" }

Write-Host "  -> OK" -ForegroundColor Green

# ---- Watch ----
Write-Host "`n[4/4] Iniciando cargo watch (vigilando src/)..." -ForegroundColor Cyan
Write-Host "  -> Los cambios en 'src/' reconstruirán WASM + servidor automáticamente.`n" -ForegroundColor Yellow

cargo watch `
    -w src/ `
    -x "build --no-default-features --features frontend --target wasm32-unknown-unknown" `
    -s "wasm-bindgen --out-dir static/pkg --target web ./target/wasm32-unknown-unknown/debug/monitor.wasm" `
    -x "run --features ssr"
