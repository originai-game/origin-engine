#!/bin/sh
# One Safari compatibility run on macrunner, in one GPU slot (scripts/lock.sh
# og_gpu_acquire), with its JSON kept under tests/evidence/compat/safari/runs/.
#
#   sh tests/evidence/compat/safari/run.sh build
#   sh tests/evidence/compat/safari/run.sh <run> [--chrome] [runner args...]
#
# build  the engine's web host and og (packages/engine scripts/build.sh), no GPU
# fx     fx-showcase stations          tests/web/reference.ts
# looks  fx-showcase looks             tests/web/reference.ts --looks
# e5     fx-showcase E5                tests/web/reference-e5.ts
# gfx    og-tests gfx scenes           tests/web/reference-gfx.ts (--scene a,b, --cpu-culling)
# ui     og-ui kit frames              tests/web/reference-ui.ts
# wire   the og-probe wire             tests/web/wire.ts
#
# Without --chrome the run is Safari's (--safari, the webgpu-safari tier);
# with it, the same run in Chrome for Testing, the control a Safari frame is
# read beside. Compatibility work yields to development (the owner): before
# it queues for a slot it waits while every slot is taken exclusively
# (gpu-all.lock) or other single waiters are queued in gpu-wait/, so it
# never goes ahead of them; then it takes one slot for the one run.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
engine="$repo/packages/engine"
run="${1:?run: build fx looks e5 gfx ui wire}"
shift

if [ "$run" = build ]; then
  cd "$engine" && exec sh scripts/build.sh
fi

browser=safari
flag=--safari
if [ "${1:-}" = --chrome ]; then
  browser=chrome
  flag=""
  shift
fi
case "$run" in
  fx) set -- tests/web/reference.ts "$@" ;;
  looks) set -- tests/web/reference.ts --looks "$@" ;;
  e5) set -- tests/web/reference-e5.ts "$@" ;;
  gfx) set -- tests/web/reference-gfx.ts "$@" ;;
  ui) set -- tests/web/reference-ui.ts "$@" ;;
  wire) set -- tests/web/wire.ts "$@" ;;
  *) echo "run.sh: no run $run" >&2; exit 2 ;;
esac

root="${OG_GPU_ROOT:-$HOME/.cache/og-gate}"
waiting() {
  [ -d "$root/gpu-all.lock" ] && return 0
  for w in "$root"/gpu-wait/*; do
    [ -e "$w" ] || continue
    [ "${w##*/}" = "$$" ] && continue
    kill -0 "${w##*/}" 2>/dev/null && return 0
  done
  return 1
}
said=""
while waiting; do
  [ -n "$said" ] || { echo "run.sh: development is queued for the GPU; safari $run waits" >&2; said=1; }
  sleep 30
done
# shellcheck source=scripts/lock.sh
. "$repo/scripts/lock.sh"
og_gpu_acquire "safari compat $run ($browser)"

# Frames stay in dist/ (scratch); only the run's JSON is kept as evidence.
out="$engine/dist/compat/$browser-$run"
keep="$here/runs/$browser-$run"
rm -rf "$out"
mkdir -p "$out" "$keep"
cd "$engine"
started="$(date +%s)"
status=0
case "$run" in
  wire) bun "$@" $flag --out "$out/transcript.json" || status=$? ;;
  ui) bun "$@" $flag || status=$? ;;
  *) bun "$@" $flag --out "$out" || status=$? ;;
esac
if [ "$run" = ui ]; then cp "$engine"/dist/test-reference/ui/reference-ui-*.json "$keep"/ 2>/dev/null || true
else find "$out" -maxdepth 2 -name '*.json' -exec cp {} "$keep"/ \;; fi
echo "run.sh: safari compat $run ($browser) exited $status in $(( $(date +%s) - started )) s; frames in $out, JSON in $keep" >&2
exit "$status"
