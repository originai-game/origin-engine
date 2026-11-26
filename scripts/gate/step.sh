#!/usr/bin/env bash
# A real child process: $$ must identify the resource holder, not its parent.
set -u -o pipefail
. "$(dirname "$0")/../lib.sh"
cd "$OG_ROOT" || exit 1
SKIP=3
SKIP_NOTE="" PASS_NOTE="" RAN="${OG_GATE_RAN:- }"
WASM_TRIPLE=wasm32-unknown-unknown
WASM_PROFILE_DEFAULT=wasm-test
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

if [ -n "${GATE_LOGS:-}" ]; then
  python3 -c 'import time; print(time.monotonic())' > "$GATE_LOGS/$1.resource-start"
fi
case "$1" in
  # nextest schedules the tests within this reservation by adapter memory.
  rust-gpu) og_gpu_acquire_all "gate $1 ($PWD)" || exit 1 ;;
  bun-test|fixtures|guide-recipes|showcase|looks|browser|web-reference|safari)
    og_gpu_acquire "gate $1 ($PWD)" || exit 1 ;;
esac
if [ -n "${GATE_LOGS:-}" ]; then
  python3 -c 'import time; print(time.monotonic())' > "$GATE_LOGS/$1.command-start"
fi
case "$1" in
  rust-build) python3 scripts/gate/native.py build; exit $? ;;
  rust-cpu) python3 scripts/gate/native.py cpu; exit $? ;;
  rust-gpu) python3 scripts/gate/native.py gpu; exit $? ;;
  doc-tests) cargo test --workspace --doc; rc=$? ;;
  bun-quick)
    # The lock, landing and guard tests read scripts/ alone (a minute).
    if [ "$FORCE_ALL" != 1 ] && og_unchanged bun-quick.scripts scripts; then
      echo "SKIPPED script tests: scripts/ unchanged since $OG_GREEN"; rc=0
    else
      python3 scripts/test-locks.py && python3 scripts/test-landing.py && scripts/test-gate-guards.sh &&
        og_green_set bun-quick.scripts
      rc=$?
    fi
    [ "$rc" != 0 ] || {
      # An explicit preflight skip must not also silently skip the toolchain test.
      script_tests=()
      while IFS= read -r file; do
        if [ "$file" = scripts/toolchain.test.ts ] && [[ "$RAN" == *' preflight '* ]]; then continue; fi
        script_tests+=("./$file")
      done < <(find scripts -name '*.test.ts' | sort)
      bun test "${script_tests[@]}" && bun scripts/gate/bun.ts quick
      rc=$?
    } ;;
  browser)
    require_web_ready browser || exit 1
    # packages/engine's test:browser is the one list of the browser tests;
    # each runs even when one before it failed.
    tests="$(bun -e 'console.log(require("./packages/engine/package.json").scripts["test:browser"])' | grep -o 'tests/web/[a-z0-9-]*\.ts')"
    [ -n "$tests" ] && rc=0 || { og_err "browser: packages/engine's test:browser names no tests/web file"; rc=1; }
    key="$(continuation_key browser)" || key=""
    for test in $tests; do
      name="${test##*/}"; name="${name%.ts}"
      if [ -n "$key" ] && python3 scripts/gate/continuation.py reuse browser "$name" "$key"; then continue; fi
      if (cd packages/engine && bun "$test"); then
        if [ -n "$key" ]; then
          python3 scripts/gate/continuation.py pass browser "$name" "$key" || og_err "browser: could not write $name continuation receipt"
        fi
      else
        rc=1
      fi
    done ;;
  *) "step_$(gate_key "$1")"; rc=$? ;;
esac
if [ "$rc" = 0 ]; then og_green_set "$1"; elif [ "$rc" != 3 ] && [ "$rc" != 4 ]; then og_green_clear "$1"; fi
[ -z "$SKIP_NOTE" ] || echo "$SKIP_NOTE"
[ -z "$PASS_NOTE" ] || echo "$PASS_NOTE"
exit "$rc"
