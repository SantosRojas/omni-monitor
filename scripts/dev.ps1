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
Write-Host "`n[1/2] Verificando herramientas..." -ForegroundColor Cyan

$hasWatch = Get-Command cargo-watch -ErrorAction SilentlyContinue
if (-not $hasWatch) {
    Write-Host "  -> Instalando cargo-watch..." -ForegroundColor Yellow
    cargo install cargo-watch
}

Write-Host "  -> OK" -ForegroundColor Green

# ---- Watch ----
Write-Host "`n[2/2] Iniciando cargo watch (vigilando src/)..." -ForegroundColor Cyan
Write-Host "  -> Los cambios en 'src/' reconstruirán el servidor automáticamente.`n" -ForegroundColor Yellow

cargo watch -w src/ -x "run --features ssr"
