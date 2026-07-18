#!/usr/bin/env bash

set -euo pipefail

workspace_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$workspace_root"

mkdir -p frontend/public/wasm
mkdir -p frontend/src/lib/generated

wasm-pack build crates/inkform-wasm \
  --target web \
  --out-dir ../../frontend/public/wasm \
  --out-name inkform_wasm

# Let Next bundle the JS wrapper instead of asking the browser to import a raw
# public asset. The WebAssembly binary remains in public/wasm for streaming.
cp frontend/public/wasm/inkform_wasm.js frontend/src/lib/generated/inkform_wasm.js
cp frontend/public/wasm/inkform_wasm.d.ts frontend/src/lib/generated/inkform_wasm.d.ts

# Avoid Turbopack resolving wasm-pack's unused relative fallback as a bundled
# asset. Inkform initializes this wrapper with the public binary URL instead.
sed -i "s|new URL('inkform_wasm_bg.wasm', import.meta.url)|'/wasm/inkform_wasm_bg.wasm'|" \
  frontend/src/lib/generated/inkform_wasm.js
