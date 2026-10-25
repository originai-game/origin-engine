#!/usr/bin/env bash
# From repo root: bash tests/evidence/r4/measure/idle-check.sh OUT COMMAND ...
set -euo pipefail
out="$1"
shift
expected=1bb341e51b22bdcdaf69a3eb60aac5265366196f
actual="$(git rev-parse HEAD)"
if ! git merge-base --is-ancestor "$expected" "$actual"; then
  echo "wrong measurement revision: $actual, expected $expected" >&2
  exit 1
fi
if ! git diff --quiet "$expected" -- crates packages templates examples tools scripts; then
  echo "measurement inputs differ from the agreed revision $expected" >&2
  exit 1
fi
mkdir -p "$out"
. scripts/lock.sh
og_lock_acquire "${OG_GATE_MACHINE_DIR:-$HOME/.cache/og-gate}" "r4 idle measurement"
og_lock_acquire "$PWD/target" "r4 idle measurement"
load() { uptime | sed -E 's/.*load averages?: *//' | awk '{print $1}'; }
idle_since=-1
while :; do
  current="$(load)"
  printf '%s\t%s\n' "$(date -u +%FT%TZ)" "$current" >> "$out/idle-before.tsv"
  if awk -v n="$current" 'BEGIN { exit !(n < 8) }'; then
    if [ "$idle_since" = -1 ]; then idle_since=$SECONDS; fi
    if [ $((SECONDS - idle_since)) -ge 60 ]; then break; fi
  else
    idle_since=-1
  fi
  sleep 5
done
printf '%q ' "$@" > "$out/command.txt"
printf '\nbase=%s\nhead=%s\n' "$expected" "$actual" >> "$out/command.txt"
(
  while :; do printf '%s\t%s\n' "$(date -u +%FT%TZ)" "$(load)"; sleep 1; done
) > "$out/load.tsv" &
monitor=$!
status=0
"$@" > "$out/stdout.txt" 2> "$out/stderr.txt" || status=$?
kill "$monitor"
wait "$monitor" 2>/dev/null || true
printf '%s\t%s\n' "$(date -u +%FT%TZ)" "$(load)" >> "$out/load.tsv"
printf 'command_exit=%s\n' "$status" > "$out/status.txt"
if awk '$2 >= 8 { bad=1 } END { exit !bad }' "$out/load.tsv"; then
  echo 'idle=false' >> "$out/status.txt"
  exit 1
fi
echo 'idle=true' >> "$out/status.txt"
exit "$status"
