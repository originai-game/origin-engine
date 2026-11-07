#!/usr/bin/env bash
# Release preparation and size check, also called by release.sh pack's build.
# --development prepares bindgen-only modules for standalone development.
set -u -o pipefail
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1
og_admission_acquire shared "release wasm ($PWD)"
og_lock_acquire "$OG_TARGET_DIR" "release wasm ($PWD)"
FORCE_ALL=0 SKIP=3 RAN=" wasm-build "
WASM_TRIPLE=wasm32-unknown-unknown WASM_PROFILE_DEFAULT=wasm
export OG_GATE_THREADS="${OG_GATE_THREADS:-4}" OG_GATE_JOBS="${OG_GATE_JOBS:-2}"
case "$OG_GATE_THREADS:$OG_GATE_JOBS" in *[!0-9:]*|:*|*:) og_err 'invalid wasm job budget'; exit 2 ;; esac
[ "$OG_GATE_THREADS" -gt 0 ] && [ "$OG_GATE_JOBS" -gt 0 ] || exit 2
skip_unchanged() { return 1; }
rule_wasm() { echo crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo scripts; }
. "$OG_ROOT/scripts/gate/commands.sh"
step_wasm_build && step_wasm_bindgen || exit 1
[ "${1:-}" != --development ] || exit 0
step_wasm_opt && step_size_gate
