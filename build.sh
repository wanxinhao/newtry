#!/bin/bash
set -e

echo "=== 万物共生 - 生命演化模拟 构建脚本 ==="
echo ""

echo "[1/2] 编译 Rust WASM..."
cargo build --target wasm32-unknown-unknown --release

echo "[2/2] 生成 JS 绑定..."
mkdir -p pkg
wasm-bindgen target/wasm32-unknown-unknown/release/ecosystem_sim.wasm --out-dir pkg --web

echo ""
echo "构建完成！"
echo "启动服务器: python3 -m http.server 8080"
echo "然后打开: http://localhost:8080"
