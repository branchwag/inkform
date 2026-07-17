#!/usr/bin/env bash

set -euo pipefail

workspace_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$workspace_root"

mkdir -p frontend/public/wasm

wasm-pack build crates/inkform-wasm \
  --target web \
  --out-dir ../../frontend/public/wasm \
  --out-name inkform_wasm
