#!/bin/bash
# Process CPU time (user+sys, every thread: the renderer and the Metal driver's)
# per frame, from the difference of two runs: --frames LO and --frames HI.
# usage: cputime.sh ROUNDS LO HI OUTFILE
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
R=$1; LO=$2; HI=$3; OUT=$4
B=target/evidence/culling/og-gfx-before; A=target/release/og-gfx
: > "$OUT"
cpu() { # variant bin scene frames -> user+sys seconds
  local v=$1 bin=$2 s=$3 n=$4 env=""
  [ "$v" = cpu ] && env="OG_GFX_CULLING=cpu"
  /usr/bin/time -p env $env $bin $s --frames $n 2>&1 >/dev/null | awk '/^user|^sys/{t+=$2} END{print t}'
}
for r in $(seq 1 $R); do
  for s in city-street city-aerial; do
    for v in before gpu cpu; do
      bin=$A; [ $v = before ] && bin=$B
      lo=$(cpu $v $bin $s $LO); hi=$(cpu $v $bin $s $HI)
      echo "round=$r scene=$s variant=$v lo=$lo hi=$hi per_frame_ms=$(echo "($hi-$lo)*1000/($HI-$LO)" | bc -l | cut -c1-6)" >> "$OUT"
    done
  done
done
