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
export OG_WASM_VARIANTS=all
[ "${1:-}" != --development ] || export OG_WASM_PROFILE=wasm-test
# The release wasm is the same bytes on any host (Linux or macOS, any user
# or checkout), so the frame archives pinned by hash rebuild anywhere:
# og-sccache-wrap gives each wasm32 unit a crate id of its own identity
# instead of cargo's host-dependent one, and remaps the cargo and rustup
# homes, build-std's sources (to /rustc/<commit>, as rustc's own std) and
# the checkout (crates/og-sccache-wrap, `reproducible`).
if [ "${1:-}" != --development ]; then
  [ "${RUSTC_WRAPPER-og-sccache-wrap}" = og-sccache-wrap ] || { og_err "release wasm: RUSTC_WRAPPER=${RUSTC_WRAPPER} bypasses og-sccache-wrap, which makes the release wasm reproducible"; exit 1; }
  sysroot="$(rustc --print sysroot)" commit="$(rustc -vV | sed -n 's/^commit-hash: //p')"
  OG_WASM_REPRODUCIBLE="${RUSTUP_HOME:-$HOME/.rustup}=/rustup
${CARGO_HOME:-$HOME/.cargo}=/cargo
$sysroot/lib/rustlib/src/rust=/rustc/$commit
$OG_ROOT=/origin-engine
$OG_TARGET_DIR=/origin-engine/target"
  export OG_WASM_REPRODUCIBLE CARGO_CACHE_RUSTC_INFO=0
  OG_WASM_REPRODUCIBLE=1 og-sccache-wrap rustc -vV 2>/dev/null | grep -qx 'og-wasm-reproducible: 1' \
    || { og_err "release wasm: the installed og-sccache-wrap predates reproducible builds; run scripts/install-tools.sh"; exit 1; }
fi
skip_unchanged() { return 1; }
rule_wasm() { echo crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo scripts; }
. "$OG_ROOT/scripts/gate/commands.sh"
step_wasm_build && step_wasm_bindgen || exit 1
[ "${1:-}" != --development ] || exit 0
step_size_gate
