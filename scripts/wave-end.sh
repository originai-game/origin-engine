#!/usr/bin/env bash
# Wave-end measurements on the current host, with no machine-specific lanes.
# Usage: scripts/wave-end.sh [--dry-run] [--stages LIST]
# Stages: gate,perf,web,conformance,release (all by default).
# Every stage runs even after a failure; nothing is published.
# shellcheck disable=SC2329 # stage functions are dispatched by name below
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

ALL_STAGES="gate perf web conformance release"
stages="$ALL_STAGES"
dry=0
usage() { sed -n '2,5p' "$0" | sed 's/^# \{0,1\}//'; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run) dry=1 ;;
    --stages) stages="$(printf '%s' "${2:?--stages LIST}" | tr ',' ' ')"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) og_err "wave-end: unknown argument '$1'"; usage >&2; exit 2 ;;
  esac
  shift
done
for s in $stages; do
  case " $ALL_STAGES " in *" $s "*) ;; *) og_err "wave-end: unknown stage '$s' (stages: $ALL_STAGES)"; exit 2 ;; esac
done

OUT="$OG_EVIDENCE_DIR/wave-end"
mkdir -p "$OUT"
LOG="$OUT/wave-end.log"
[ "$dry" = 1 ] || : > "$LOG"
say() { og_say "$*"; [ "$dry" = 1 ] || printf '%s\n' "$*" >> "$LOG"; }

run() {
  say "+ $(printf '%q ' "$@")"
  [ "$dry" = 1 ] && return 0
  "$@" 2>&1 | tee -a "$LOG"
  return "${PIPESTATUS[0]}"
}

stage_gate() { run scripts/check.sh full --all; }
stage_perf() { run scripts/perf.sh; }
stage_web() { run scripts/check.sh web-reference; }

stage_conformance() { run scripts/tsrs-diff.sh; }
stage_release() { run scripts/release.sh pack && run scripts/release.sh verify; }

if [ "$dry" != 1 ]; then
  og_admission_acquire exclusive "scripts/wave-end.sh ($PWD)" || exit 1
  og_lock_acquire "$OG_TARGET_DIR" "scripts/wave-end.sh" || exit 1
fi

note=""
[ "$dry" != 1 ] || note="$note (dry run: nothing runs)"
say "wave-end: $(git rev-parse --short HEAD 2>/dev/null || echo no-git), stages: $(printf '%s' "$stages" | og_words)$note"
summary=""
failed=0
t_all="$(date +%s)"
for s in $ALL_STAGES; do
  case " $(printf '%s' "$stages" | og_words) " in *" $s "*) ;; *) continue ;; esac
  say ""
  say "==> $s"
  t0="$(date +%s)"
  "stage_$s"; rc=$?
  dt=$(( $(date +%s) - t0 ))
  if [ "$dry" = 1 ]; then line="DRY $s"
  elif [ "$rc" -eq 0 ]; then line="PASS $s (${dt}s)"
  else line="FAIL $s (exit $rc, ${dt}s)"; failed=1; fi
  say "$line"
  summary="$summary$line"$'\n'
done
say ""
if [ "$dry" = 1 ]; then verdict="dry run"; elif [ "$failed" = 0 ]; then verdict=PASSED; else verdict=FAILED; fi
say "wave-end $verdict ($(( $(date +%s) - t_all ))s); logs in ${OUT#"$OG_ROOT"/}/"
printf '%s' "$summary" | while IFS= read -r l; do say "$l"; done
[ "$dry" = 1 ] || printf '%s' "$summary" > "$OUT/summary.txt"
exit "$failed"
