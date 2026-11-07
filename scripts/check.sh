#!/usr/bin/env bash
# quick: landing checks. full: functional checks with bindgen-only wasm and browsers.
# --all ignores previous passing input snapshots. --only STEP adds prerequisites.
set -eu -o pipefail
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT"
case "${1:-}" in --help|-h|--list) exec python3 scripts/gate/run.py "$@" ;; esac

og_temp_prepare "$OG_ROOT"
# Admission is shared, not a machine-wide work lock. Only this checkout's
# outputs are exclusive. GPU ownership is acquired by rendering children.
og_admission_acquire shared "gate $* ($PWD)"
og_lock_acquire "$OG_TARGET_DIR" "gate $* ($PWD)"
export OG_TARGET_DIR
export OG_GATE_TREE="$(og_worktree_tree)"
# Jobs per step follow the machine: its cores shared between the gates
# admitted now and each gate's concurrent steps, never fewer than 2. An idle
# machine gives one gate most of its cores; a busy one falls back to 2.
if [ -z "${OG_GATE_THREADS:-}" ]; then
  og_cores="$(sysctl -n hw.perflevel0.logicalcpu 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || nproc)"
  og_gates=0
  for og_gate in "$og_gpu_root/admission/active/"*; do
    if [ -d "$og_gate" ] && og_owner_alive "$og_gate"; then og_gates=$((og_gates + 1)); fi
  done
  [ "$og_gates" -gt 0 ] || og_gates=1
  OG_GATE_THREADS=$((og_cores / (og_gates * ${OG_GATE_JOBS:-2})))
  [ "$OG_GATE_THREADS" -ge 2 ] || OG_GATE_THREADS=2
fi
export OG_GATE_THREADS
export CARGO_BUILD_JOBS="$OG_GATE_THREADS"
export NEXTEST_TEST_THREADS="$OG_GATE_THREADS"
export RUST_TEST_THREADS="$OG_GATE_THREADS"
export RAYON_NUM_THREADS="$OG_GATE_THREADS"
python3 scripts/gate/run.py "$@" &
runner=$!
# Keep admission and target ownership until the scheduler drains its children.
stop() { kill -TERM "$runner" 2>/dev/null || true; wait "$runner" || true; exit "$1"; }
trap 'stop 130' INT
trap 'stop 143' TERM
wait "$runner"
