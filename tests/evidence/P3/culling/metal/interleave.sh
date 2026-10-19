#!/bin/bash
# Interleaved native Metal runs: before (pre-culling og-gfx), after Auto (GPU culling), after CPU.
# usage: interleave.sh ROUNDS FRAMES OUTFILE
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
R=${1:-5}; F=${2:-300}; OUT=$3
B=target/evidence/culling/og-gfx-before; A=target/release/og-gfx
: > "$OUT"
for r in $(seq 1 $R); do
  for s in city-street city-aerial; do
    echo "round=$r variant=before $($B $s --frames $F 2>&1 | grep "^$s:")" >> "$OUT"
    echo "round=$r variant=gpu $($A $s --frames $F 2>&1 | grep "^$s:")" >> "$OUT"
    echo "round=$r variant=cpu $(OG_GFX_CULLING=cpu $A $s --frames $F 2>&1 | grep "^$s:")" >> "$OUT"
  done
done
