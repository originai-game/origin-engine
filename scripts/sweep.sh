#!/usr/bin/env bash
# Prunes what target/ keeps and never reads again: incremental compilation
# sessions no build touched for DAYS days (default 2). Every test binary and
# feature set leaves one (3,450 sessions, 41 GB, on 2026-10-21); a pruned
# session only costs that crate one non-incremental build. Never the deps,
# never a worktree's whole target/ (AGENTS.md §Build and gate discipline).
#
#   scripts/sweep.sh [DAYS]
set -eu
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
days="${1:-2}"
inc="$OG_TARGET_DIR/debug/incremental"
[ -d "$inc" ] || { og_say "sweep: no $inc"; exit 0; }
before="$(du -sk "$inc" | cut -f1)"
n=0
while IFS= read -r d; do
  rm -rf "$d"; n=$((n + 1))
done < <(find "$inc" -mindepth 1 -maxdepth 1 -type d -mtime +"$days")
after="$(du -sk "$inc" | cut -f1)"
og_say "sweep: removed $n incremental session(s) untouched for $days day(s): $((before / 1024)) MB -> $((after / 1024)) MB"
