#!/usr/bin/env bash
# quick: landing checks. full: all functional checks, including wasm and browsers.
# --all ignores previous passing input snapshots. --only STEP adds prerequisites.
set -eu -o pipefail
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT"
case "${1:-}" in --help|-h|--list) exec python3 scripts/gate/run.py "$@" ;; esac

# Admission is shared, not a machine-wide work lock. Only this checkout's
# outputs are exclusive. GPU ownership is acquired by rendering children.
og_admission_acquire shared "gate $* ($PWD)"
og_lock_acquire "$OG_TARGET_DIR" "gate $* ($PWD)"
export OG_TARGET_DIR
export OG_GATE_TREE="$(og_worktree_tree)"
export OG_GATE_THREADS="${OG_GATE_THREADS:-2}"
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
