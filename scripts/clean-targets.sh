#!/usr/bin/env bash
# Daily target/ hygiene for a machine with many worktrees (the owner,
# 2026-11-05, after /Volumes/app filled: the main checkout's
# target/debug/incremental alone had grown to 265 GB).
#
#   scripts/clean-targets.sh             clean every worktree of this repository now
#   scripts/clean-targets.sh --install   run it daily at 05:00 (launchd on macOS)
#
# For each worktree with a target/ that nothing is building in (no cargo,
# rustc or gate process mentions it, and no gate lock is held there):
#   1. target/*/incremental is removed (regenerable; never the whole target/);
#   2. `cargo sweep --time $OG_SWEEP_DAYS` (default 1) removes artifacts no
#      build used in that many days, so warm builds stay warm.
# A busy worktree is skipped and named. Lanes are never removed.
set -eu -o pipefail
main="$(git -C "$(dirname "$0")/.." worktree list --porcelain | awk '/^worktree / { print substr($0, 10); exit }')"
days="${OG_SWEEP_DAYS:-1}"

if [ "${1:-}" = --install ]; then
  [ "$(uname)" = Darwin ] || { echo "clean-targets: --install is for macOS (launchd)" >&2; exit 1; }
  plist="$HOME/Library/LaunchAgents/ai.origingame.clean-targets.plist"
  mkdir -p "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"
  cat > "$plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>ai.origingame.clean-targets</string>
  <key>ProgramArguments</key><array><string>/bin/bash</string><string>$main/scripts/clean-targets.sh</string></array>
  <key>EnvironmentVariables</key><dict><key>PATH</key><string>$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string></dict>
  <key>StartCalendarInterval</key><dict><key>Hour</key><integer>5</integer><key>Minute</key><integer>0</integer></dict>
  <key>StandardOutPath</key><string>$HOME/Library/Logs/og-clean-targets.log</string>
  <key>StandardErrorPath</key><string>$HOME/Library/Logs/og-clean-targets.log</string>
</dict></plist>
EOF
  launchctl unload "$plist" 2>/dev/null || true
  launchctl load "$plist"
  echo "clean-targets: installed $plist (daily 05:00, log ~/Library/Logs/og-clean-targets.log)"
  exit 0
fi

command -v cargo-sweep >/dev/null || echo "clean-targets: cargo-sweep missing (cargo install cargo-sweep); only incremental dirs are removed" >&2
echo "clean-targets: $(date) free before: $(df -h "$main" | awk 'NR==2 { print $4 }')"
procs="$(ps -axo command= 2>/dev/null || true)"
git -C "$main" worktree list --porcelain | awk '/^worktree / { print substr($0, 10) }' | while read -r d; do
  t="$d/target"
  [ -d "$t" ] || continue
  if [ -d "$t/og-gate.lock" ] || printf '%s\n' "$procs" | grep -E 'cargo|rustc|check\.sh|build\.sh|nextest' | grep -qF "$d/"; then
    echo "  skip $d (building)"; continue
  fi
  for inc in "$t"/*/incremental "$t"/*/*/incremental; do
    [ -d "$inc" ] && rm -rf "$inc"
  done
  if command -v cargo-sweep >/dev/null; then
    (cd "$d" && cargo sweep --time "$days" >/dev/null 2>&1) || echo "  sweep failed in $d" >&2
  fi
  echo "  cleaned $d ($(du -sh "$t" 2>/dev/null | cut -f1) left)"
done
echo "clean-targets: free after: $(df -h "$main" | awk 'NR==2 { print $4 }')"
