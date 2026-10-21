#!/usr/bin/env bash
# The perf suite (docs/ci.md §What runs where; `make perf`): og-tests' frame
# times (crates/og-tests/perf/<tier>-<adapter>/) and og-runtime's
# extraction cost (crates/og-runtime/perf/<cpu>/), release profile, each
# against this machine's committed baseline with a 15 % tolerance.
#
#   scripts/perf.sh             compare with this machine's baselines
#   OG_PERF_RECORD=1 scripts/perf.sh
#                               record this machine's baselines
#   scripts/perf.sh --measure   measure and compare nothing: the numbers go to
#                               $OG_EVIDENCE_DIR/perf/ and the tree is left as
#                               it was (CI's nightly: a shared runner has no
#                               committed baseline, and its lavapipe frame
#                               times measure the VM, not the renderer)
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

measure=0
case "${1:-}" in
  --measure) measure=1 ;;
  "") ;;
  -h|--help) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
  *) og_err "usage: scripts/perf.sh [--measure]"; exit 2 ;;
esac

dirs="crates/og-tests/perf crates/og-runtime/perf"
out="$OG_EVIDENCE_DIR/perf"
mkdir -p "$out"
if [ "$measure" = 1 ]; then
  # shellcheck disable=SC2086
  og_say "perf: --measure, so no frame time is compared with a baseline (committed: $(find $dirs -mindepth 1 -maxdepth 1 -type d | sort | og_words)); each is recorded and copied to $out"
  export OG_PERF_RECORD=1
fi

t0="$(date +%s)"
cargo test -p og-tests -p og-runtime --release --no-fail-fast -- --ignored perf --nocapture 2>&1 | tee "$out/perf.log"
rc="${PIPESTATUS[0]}"
og_say "perf: cargo test exited $rc after $(( $(date +%s) - t0 )) s"

if [ "$measure" = 1 ]; then
  # What the run recorded, new or rewritten, into the evidence; then the
  # tree as it was.
  # shellcheck disable=SC2086
  git status --porcelain --untracked-files=all -- $dirs | while read -r _ f; do
    mkdir -p "$out/$(dirname "$f")" && cp "$f" "$out/$f"
  done
  # shellcheck disable=SC2086
  git checkout -q -- $dirs
  # shellcheck disable=SC2086
  git clean -q -fd -- $dirs
  og_say "perf: the measured frame times (each recorded file is in $out/crates/):"
  grep -E '^perf ' "$out/perf.log" || og_say "(none printed)"
fi
exit "$rc"
