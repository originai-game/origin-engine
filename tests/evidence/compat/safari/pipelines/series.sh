#!/bin/bash
# Run after run.sh build. Each timing run holds all GPU slots.
set -eu -o pipefail
repo="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo"
mkdir -p .amp/in/sp
for m in sync async prewarm; do rm -rf .amp/in/sp/dist-$m; bun tests/evidence/compat/safari/pipelines/measure.ts --mode $m --build-only > .amp/in/sp/build-$m.log 2>&1 || { echo "EXPORT $m FAILED"; exit 1; }; done
one() { # one <browser> <mode> <tap>
  (
    source scripts/lock.sh
    og_gpu_acquire_all "webboot safari pipelines $1 $2 $3"
    bun tests/evidence/compat/safari/pipelines/measure.ts --browser "$1" --mode "$2" --tap "$3" 2>&1 | tail -1
  )
}
for rep in 1 2 3; do
  for tap in early late; do
    for m in sync async prewarm; do
      for b in safari chrome; do one $b $m $tap; done
    done
  done
done
echo DONE
