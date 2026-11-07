#!/usr/bin/env bash
# Release preparation and size check, also called by release.sh pack's build.
# --development prepares bindgen-only modules for standalone development.
set -u -o pipefail
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1
og_admission_acquire shared "release wasm ($PWD)"
og_lock_acquire "$OG_TARGET_DIR" "release wasm ($PWD)"
SKIP=3
WASM_TRIPLE=wasm32-unknown-unknown WASM_PROFILE_DEFAULT=wasm
export OG_WASM_PROFILE=wasm
export OG_WASM_REFERENCE_ONLY=0
[ "${1:-}" != --development ] || export OG_WASM_PROFILE=wasm-test
skip_unchanged() { return 1; }
rule_wasm() { echo crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo scripts; }
. "$OG_ROOT/scripts/gate/commands.sh"
step_wasm_build && step_wasm_bindgen || exit 1
[ "${1:-}" != --development ] || exit 0
step_size_gate
