#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo ""
echo "========================================"
echo "  BUILD PRODUCCIÓN - Monitor OMNI"
echo "========================================"
echo ""

echo "[1/3] Compilando frontend WASM (release)..."
cargo build --no-default-features --features frontend --target wasm32-unknown-unknown --release
echo "  -> OK"
echo ""

echo "[2/3] Ejecutando wasm-bindgen (release)..."
wasm-bindgen --out-dir static/pkg --target web ./target/wasm32-unknown-unknown/release/monitor.wasm
echo "  -> OK"
echo ""

echo "[3/3] Compilando servidor (release)..."
cargo build --features ssr --release
echo "  -> OK"
echo ""

echo "========================================"
echo "  BUILD COMPLETADO"
echo "========================================"
echo "Ejecutable: ./target/release/monitor-server"
echo ""
