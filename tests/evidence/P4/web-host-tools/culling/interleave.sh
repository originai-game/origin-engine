#!/bin/bash
# GPU against CPU culling across instance counts, native Metal, interleaved:
# each round runs every (side, scene) with OG_GFX_CULLING=cpu then gpu, in an
# order that alternates by round, and logs the load average first.
# usage: interleave.sh ROUNDS FRAMES OUTFILE [SIDES...]
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
R=${1:-6}; F=${2:-300}; OUT=$3; shift 3
SIDES=${*:-16 32 46 64 90 128 180 256}
A=target/release/og-gfx
: > "$OUT"
for r in $(seq 1 "$R"); do
  echo "round=$r load=$(uptime | sed 's/.*load averages*: //')" >> "$OUT"
  for side in $SIDES; do
    for s in city-street city-aerial; do
      if [ $((r % 2)) -eq 1 ]; then order="cpu gpu"; else order="gpu cpu"; fi
      for c in $order; do
        echo "round=$r side=$side variant=$c $(OG_GFX_CITY_SIDE=$side OG_GFX_CULLING=$c $A $s --frames "$F" 2>&1 | grep "^$s:")" >> "$OUT"
      done
    done
  done
done
