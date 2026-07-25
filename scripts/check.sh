#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_root"

for required_command in cargo wasm-pack npm; do
    if ! command -v "$required_command" >/dev/null 2>&1; then
        echo "required command not found: $required_command" >&2
        exit 1
    fi
done

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo run --quiet --package raster-nights -- validate-content
wasm-pack build apps/web \
    --target web \
    --no-pack \
    --out-dir ../../website/public/wasm
wasm-pack test --headless --firefox crates/raster-games
npm --prefix website run build
