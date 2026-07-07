#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo ""
echo "========================================"
echo "  DEV MODE - Monitor OMNI"
echo "========================================"
echo ""

echo "[1/2] Starting frontend (http://localhost:5173)..."
(cd frontend && npm run dev) &
FRONTEND_PID=$!

sleep 3

cleanup() {
    echo ""
    echo "Stopping frontend..."
    kill $FRONTEND_PID 2>/dev/null || true
    wait $FRONTEND_PID 2>/dev/null || true
}
trap cleanup EXIT

echo "[2/2] Starting backend (http://localhost:9002)..."
echo "Press Ctrl+C to stop both"
echo ""
cargo run
