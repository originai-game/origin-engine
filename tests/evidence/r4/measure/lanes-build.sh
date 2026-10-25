#!/usr/bin/env bash
# Diagnostic, non-publish lanes module: retain the whole-world probe dump.
set -euo pipefail
. scripts/lock.sh
og_lock_acquire "${OG_GATE_MACHINE_DIR:-$HOME/.cache/og-gate}" "r4 diagnostic lanes build"
og_lock_acquire "$PWD/target" "r4 diagnostic lanes build"
scripts/wasm-lanes.sh --profile wasm -p og-host-web --lib
mkdir -p target/r4-lanes-dev
wasm-bindgen --target web --out-dir target/r4-lanes-dev --out-name origin-engine-lanes target/wasm-mt/wasm32-unknown-unknown/wasm/og_host_web.wasm
sh packages/engine/scripts/rayon-worker.sh target/r4-lanes-dev
