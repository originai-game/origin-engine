#!/usr/bin/env bash
set -euo pipefail
out="$1"; shift
mkdir -p "$out"
. scripts/lock.sh
og_lock_acquire "${OG_GATE_MACHINE_DIR:-$HOME/.cache/og-gate}" "r4 final idle measurement"
og_lock_acquire "$PWD/target" "r4 final idle measurement"
load() { uptime | sed -E 's/.*load averages?: *//' | awk '{print $1}'; }
since=-1
while :; do
  n="$(load)"
  printf '%s\t%s\n' "$(date -u +%FT%TZ)" "$n" >> "$out/idle-before.tsv"
  if awk -v n="$n" 'BEGIN {exit !(n<8)}'; then
    if [ "$since" = -1 ]; then since=$SECONDS; fi
    if [ $((SECONDS-since)) -ge 60 ]; then break; fi
  else since=-1
  fi
  sleep 5
done
git rev-parse HEAD > "$out/head.txt"
printf '%q ' "$@" > "$out/command.txt"
(
  while :; do printf '%s\t%s\n' "$(date -u +%FT%TZ)" "$(load)"; sleep 1; done
) > "$out/load.tsv" &
monitor=$!
status=0
"$@" > "$out/stdout.txt" 2> "$out/stderr.txt" || status=$?
kill "$monitor"
wait "$monitor" 2>/dev/null || true
printf 'command_exit=%s\n' "$status" > "$out/status.txt"
if awk '$2 >= 8 {bad=1} END {exit !bad}' "$out/load.tsv"; then
  echo idle=false >> "$out/status.txt"
  exit 1
fi
echo idle=true >> "$out/status.txt"
exit "$status"
