#!/usr/bin/env bash
# A real child process: $$ must identify the resource holder, not its parent.
set -u -o pipefail
. "$(dirname "$0")/../lib.sh"
cd "$OG_ROOT" || exit 1
SKIP=3
SKIP_NOTE="" PASS_NOTE="" RAN="${OG_GATE_RAN:- }"
WASM_TRIPLE=wasm32-unknown-unknown
WASM_PROFILE_DEFAULT=wasm
export OG_GATE=1
export OG_BUN_TEST_TIMEOUT="${OG_BUN_TEST_TIMEOUT:-30000}"

gate_key() { printf '%s' "$1" | tr '-' '_'; }
rule_wasm() { python3 scripts/gate/inputs.py crates og-host-web og-render; }
rule_drift() { echo .; }
rule_web() { echo .; }
rule_conformance() { echo .; }
rule_fixture() { python3 scripts/gate/inputs.py fixture "$1"; }
skip_unchanged() {
  [ "$FORCE_ALL" != 1 ] || return 1
  local key="$1" what="$2"
  shift 2
  # Unknown files force execution, even if none of the known inputs moved.
  python3 scripts/gate/inputs.py unchanged "$key" "$@" || return 1
  SKIP_NOTE="unchanged dependency inputs"
  echo "SKIPPED $what: $SKIP_NOTE"
}
. "$OG_ROOT/scripts/gate/commands.sh"

case "$1" in
  rust-build) python3 scripts/gate/native.py build; exit $? ;;
  rust-cpu) python3 scripts/gate/native.py cpu; exit $? ;;
  rust-gpu)
    og_gpu_acquire "gate Rust rendering ($PWD)" || exit 1
    python3 scripts/gate/native.py gpu; exit $? ;;
  bun-test|fixtures|browser|web-reference)
    og_gpu_acquire "gate $1 ($PWD)" || exit 1 ;;
esac
case "$1" in
  doc-tests) cargo test --workspace --doc; rc=$? ;;
  bun-quick)
    python3 scripts/test-locks.py && python3 scripts/test-landing.py && scripts/test-gate-guards.sh &&
      bun test scripts/gate/imports.test.ts scripts/changelog.test.ts scripts/release-dist.test.ts && bun scripts/gate/bun.ts quick
    rc=$? ;;
  browser)
    rc=0
    for test in wire player export unsupported; do
      (cd packages/engine && bun "tests/web/$test.ts") || rc=1
    done ;;
  *) "step_$(gate_key "$1")"; rc=$? ;;
esac
if [ "$rc" = 0 ]; then og_green_set "$1"; elif [ "$rc" != 3 ]; then og_green_clear "$1"; fi
[ -z "$SKIP_NOTE" ] || echo "$SKIP_NOTE"
[ -z "$PASS_NOTE" ] || echo "$PASS_NOTE"
exit "$rc"
