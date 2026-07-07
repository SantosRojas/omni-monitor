#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo ""
echo "========================================"
echo "  BUILD PRODUCCIÓN - Monitor OMNI"
echo "========================================"
echo ""

echo "[1/2] Compilando frontend..."
cd frontend && npm ci && npm run build && cd ..
echo "  -> OK"
echo ""

echo "[2/2] Compilando servidor (release)..."
cargo build --features ssr --release
echo "  -> OK"
echo ""

echo "========================================"
echo "  BUILD COMPLETADO"
echo "========================================"
echo "Ejecutable: ./target/release/monitor-server"
echo ""
