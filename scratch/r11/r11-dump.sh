#!/bin/bash
# usage: r11-dump.sh <name>   (run from the worktree root)
S=/tmp/claude-0/-home-user-origin-engine/f72358d0-7dfc-5ecf-bd16-bcbd4721652e/scratchpad
RUSTC_WRAPPER= RUSTFLAGS="-Zthreads=4" cargo test -p og-ui --test tests --no-run 2>&1 | grep -E "^error|warning: unused" -A8 | head -40
BIN=$(ls -t target/debug/build/og-ui/*/out/tests-* 2>/dev/null | grep -v '\.d$' | head -1)
[ -z "$BIN" ] && BIN=$(ls -t target/debug/deps/tests-* | grep -v '\.d$' | head -1)
echo "bin $BIN"
mkdir -p $S/r11/$1
cd crates/og-ui
for c in 0 1 2 3 4 5; do
  R11_CASE=$c R11_DUMP=$S/r11/$1/d "$OLDPWD/$BIN" r11_dump > $S/r11/$1/log.$c 2>&1 &
done
wait
ls -la $S/r11/$1/
