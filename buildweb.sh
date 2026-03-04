#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Setup builder cargo/rustup paths
export CARGO_HOME="$SCRIPT_DIR/builder/tools/cargo"
export RUSTUP_HOME="$SCRIPT_DIR/builder/tools/rustup"
export PATH="$CARGO_HOME/bin:$PATH"

echo "=== cokaclang WASM 빌드 ==="

# Check wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack 설치 중..."
    cargo install wasm-pack
fi

# Check wasm32 target
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "wasm32-unknown-unknown 타겟 추가 중..."
    rustup target add wasm32-unknown-unknown
fi

# Build WASM
echo "WASM 빌드 중..."
wasm-pack build --target web --features wasm --no-default-features

# Copy pkg/ to website/pkg/
echo "website/pkg/ 에 복사 중..."
rm -rf website/pkg
cp -r pkg website/pkg

# Bundle JS (CodeMirror + app)
echo "JS 번들링 중..."
cd website
npm install --silent
npx esbuild app.js --bundle --format=esm --outfile=app.bundle.js --minify --external:./pkg/*
cd ..

# Copy website to root for GitHub Pages
echo "루트에 복사 중..."
rm -rf assets/* 2>/dev/null || true
rmdir assets 2>/dev/null || true
rm -f index.html style.css app.bundle.js
rm -rf pkg 2>/dev/null || true
cp website/index.html .
cp website/style.css .
cp website/app.bundle.js .
cp -r website/pkg .
rm -f pkg/.gitignore

echo "=== 빌드 완료 ==="
echo "로컬 테스트: cd website && python3 -m http.server 8080"
