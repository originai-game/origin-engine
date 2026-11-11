#!/bin/bash
# usage: r11-verify.sh <name> [full]   (run from the worktree root)
S=/tmp/claude-0/-home-user-origin-engine/f72358d0-7dfc-5ecf-bd16-bcbd4721652e/scratchpad
$S/r11-dump.sh $1 > $S/r11/$1.log 2>&1
for c in 0 1 2 3 4 5; do
  if cmp -s $S/r11/base/d.$c $S/r11/$1/d.$c; then echo "case $c identical"; else echo "case $c DIFFERS"; fi
done
if [ "$2" = full ]; then
  export RUSTC_WRAPPER= RUSTFLAGS="-Zthreads=4"
  cargo test -p og-ui --test tests 2>&1 | grep -E "^test result|FAILED|failed|panicked" | head -20
  cargo test -p og-ui --lib 2>&1 | grep -E "^test result|FAILED|failed|panicked" | head -20
  cargo clippy -p og-ui --all-targets -- -D warnings 2>&1 | grep -E "^(error|warning)" -A6 | head -30
  cargo fmt --all -- --check 2>&1 | head -10
  cargo check -p og-runtime --all-targets 2>&1 | grep -E "^(error|warning)" -A6 | head -20
  cargo test -p og-runtime --test tests -- ui_ 2>&1 | grep -E "^test result|FAILED|failed|panicked" | head -20
fi
echo done
