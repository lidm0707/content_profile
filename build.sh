#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "=> Building tailwind CSS..."
cd content_ui
npx @tailwindcss/cli -i ./assets/input.css -o ./assets/tailwind.css --minify
cd ..

echo "=> Building content_ui (dx build --release --web)..."
dx build --release --web --package content_ui

echo "=> Building content_proxy (cargo build --release)..."
cargo build --release -p content_proxy

echo "=> Copying SSL certs for proxy image..."
cp /etc/ssl/certs/ca-certificates.crt content_proxy/ca-certificates.crt

echo "=> Building docker images..."
docker compose build

echo "=> Done. Run: docker compose up"
