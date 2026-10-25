#!/usr/bin/env bash
set -euo pipefail
out=tests/evidence/r4/measure/capture-profile
export OG_NATIVE_ADDON="$PWD/target/r4-profile.node"
export OG_CAPTURE_PROFILE=1
for mode in default fast-up fast-none store-none; do
  unset OG_CAPTURE_LEVEL OG_CAPTURE_NO_FILTER
  case "$mode" in
    fast-up) export OG_CAPTURE_LEVEL=1 ;;
    fast-none) export OG_CAPTURE_LEVEL=1 OG_CAPTURE_NO_FILTER=1 ;;
    store-none) export OG_CAPTURE_LEVEL=0 OG_CAPTURE_NO_FILTER=1 ;;
  esac
  for game in game meridian; do
    project=templates/game
    if [ "$game" = meridian ]; then project=examples/meridian-loop; fi
    dir="$out/$mode/$game"
    mkdir -p "$dir"
    bun tests/evidence/r4/measure/capture-native.mjs "$project" "$dir" > "$dir/stdout.txt" 2> "$dir/stderr.txt"
  done
done
