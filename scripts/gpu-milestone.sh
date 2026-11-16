#!/usr/bin/env bash
# Run only on the coordinator's milestone instruction, never per landing.
# Recovered from the retired L4's batch3.sh; paths no longer depend on its orb.
set -eu -o pipefail
root=$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)
# shellcheck source=scripts/gpu-env.sh
source "$root/scripts/gpu-env.sh"
# shellcheck source=.agents/environment
source "$root/.agents/environment"
unset FORCE_ALL OG_GATE_LANE OG_GATE_THREADS OG_EVIDENCE_DIR CARGO_BUILD_JOBS GATE_LOGS OG_GFX_PIPELINE_CACHE OG_TARGET_DIR
export GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null
# shellcheck source=scripts/lock.sh
# shellcheck disable=SC1091
source "$root/scripts/lock.sh"
test -r "$VK_ICD_FILENAMES"
test -x "${CHROME:-}"
git -C "$root" fetch -q --no-filter origin main
head=$(git -C "$root" rev-parse origin/main)
run="$(date -u +%Y%m%dT%H%M%SZ)-${head:0:9}"
logs="$root/.og/gpu-milestone/$run"
art="$root/.amp/in/artifacts/review/$run"
mkdir -p "$logs" "$art"
summary="$logs/summary.log"
gate="$(dirname "$root")/l4-gate"
perf="$(dirname "$root")/l4-perf"
for tree in "$gate" "$perf"; do
  if [[ ! -e $tree ]]; then
    git -C "$root" worktree add --detach "$tree" "$head"
  else
    [[ $(git -C "$tree" rev-parse --path-format=absolute --git-common-dir) == "$(git -C "$root" rev-parse --path-format=absolute --git-common-dir)" ]] || {
      echo "Not a worktree of this repository: $tree" >&2; exit 1;
    }
    [[ -z $(git -C "$tree" status --porcelain) ]] || {
      echo "Worktree has local changes: $tree" >&2; exit 1;
    }
    git -C "$tree" checkout -q --detach "$head"
  fi
done
printf 'milestone head %s\nlogs %s\nartifacts %s\n' "$head" "$logs" "$art" | tee "$summary"
failed=0
record() {
  local label=$1 log=$2 start=$SECONDS status=0
  shift 2
  "$@" > "$log" 2>&1 || status=$?
  printf '%s exit %s (%ss)\n' "$label" "$status" "$((SECONDS-start))" | tee -a "$summary"
  [[ $status == 0 ]] || failed=1
  return "$status"
}
clean() {
  record "clean $1" "$logs/clean-$1.log" bash "$root/scripts/clean-targets.sh" || true
}

# 1. Every full-gate lane, including rust-gpu and every wasm variant.
clean gate
cd "$gate"
record 'full gate' "$logs/full.log" env OG_WASM_VARIANTS=all scripts/check.sh full --all || true
printf 'reference mismatches: %s\n' "$(grep -c 'REFERENCE MISMATCH' "$logs/full.log" || true)" >> "$summary"

# 2. Four-style screens and contrast counts on the same pinned head.
if record 'release og' "$logs/og-build.log" cargo build -q --release -p og-cli --bin og; then
  og_gpu_acquire 'milestone screens'
  for style in default fantasy scifi cartoon; do
    record "screens $style" "$logs/screens-$style.log" target/release/og ui review screens --style "$style" --locales game --out "$art/screens4/$style" || true
  done
  og_lock_release
  for style in default fantasy scifi cartoon; do
    record "style check $style" "$logs/style-$style.log" target/release/og style check "$style" || true
  done
fi

# 3. Dense-menu perf, final main against the last measured main.
clean uiperf
cd "$perf"
og_gpu_acquire_all 'milestone UI perf'
for rev in "$head" 2fb1c4819; do
  git checkout -q --detach "$rev"
  record "UI perf $rev" "$logs/uiperf-$rev.log" cargo test -q -p og-ui --release --no-fail-fast -- --ignored perf --nocapture --test-threads=1 || true
done
og_lock_release

# 4. Release native town bench twice, final main against the last benched rev.
clean town
for rev in "$head" 6cc20b167; do
  git checkout -q --detach "$rev"
  cd "$perf/packages/engine"
  if record "native build $rev" "$logs/native-$rev.log" env OG_RELEASE=1 sh scripts/build.sh --native; then
    og_gpu_acquire_all 'milestone town bench'
    for n in 1 2; do
      record "town $rev run $n" "$logs/town-$rev-$n.log" bun bench/hlod-town.ts --out "$art/hlod-profile/$rev-$n" || true
    done
    og_lock_release
  fi
  cd "$perf"
done
git checkout -q --detach "$head"
printf 'batch-done: failed=%s (frames still need visual review)\n' "$failed" | tee -a "$summary"
# Keep the supervised one-off service available for log inspection.
sleep infinity
