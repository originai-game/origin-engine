#!/bin/bash
# The cull bench beside a background load: $1 busy loops (bash `while :`)
# started first, then the bench at OG_LANES 2, 4, 8 and 16 (THREADS), ITERS
# rounds with a 300 µs gap, the counts in NS (the bench's default
# otherwise), each run's output in out/${TAG}load$1-t<threads>.txt and the
# load averages before and after in out/load$1-uptime.txt.
n=$1; pids=()
for i in $(seq $n); do (while :; do :; done) & pids+=($!); done
sleep 3; uptime > out/load$n-uptime.txt
for t in ${THREADS:-2 4 8 16}; do OG_LANES=$t ./target/release/cullbench ${ITERS:-200} 300 ${NS:-} > out/${TAG:-}load$n-t$t.txt; done
uptime >> out/load$n-uptime.txt
kill ${pids[@]}
