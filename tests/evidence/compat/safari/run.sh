#!/bin/sh
# One Safari compatibility run on macrunner, in one GPU slot (scripts/lock.sh
# og_gpu_acquire), with its JSON kept under tests/evidence/compat/safari/runs/.
#
#   sh tests/evidence/compat/safari/run.sh build [gfx]
#   sh tests/evidence/compat/safari/run.sh <run> [--chrome] [runner args...]
#
# build  og, og-tests-web's module and the web host (scripts/build.sh), no GPU;
#        `build gfx` skips the web host
# fx     fx-showcase stations          tests/web/reference.ts
# looks  fx-showcase looks             tests/web/reference.ts --looks
# e5     fx-showcase E5                tests/web/reference-e5.ts
# gfx    og-tests gfx scenes           tests/web/reference-gfx.ts (--scene a,b, --cpu-culling)
# ui     og-ui kit frames              tests/web/reference-ui.ts
# wire   the og-probe wire             tests/web/wire.ts
# tp     third-person's frames         tests/web/reference.ts --project templates/archetypes/third-person
# example <name>  an example's frames  tests/web/reference.ts --project examples/<name> --playtest
# probe  a scratch WebGPU probe script (bun <script>), for a minimal repro
#
# Without --chrome the run is Safari's (--safari, the webgpu-safari tier);
# with it, the same run in Chrome for Testing, the control a Safari frame is
# read beside. Compatibility work yields to development (the owner): before
# it queues for a slot it waits for every single waiter already queued in
# gpu-wait/, so it never goes ahead of them; then it takes one slot for the
# one run.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
engine="$repo/packages/engine"
run="${1:?run: build fx looks e5 gfx ui wire}"
shift

if [ "$run" = build ]; then
  # Built before a slot is taken, so no slot waits on cargo: og (dev, for
  # cooks: $OG_BIN below), og-tests-web's module where reference-gfx.ts
  # builds it, and unless `build gfx`, the web host (scripts/build.sh).
  cd "$repo"
  cargo build -p og-cli --bin og
  cargo build --profile wasm --target wasm32-unknown-unknown --target-dir "${CARGO_TARGET_DIR:-$repo/target}/wasm-gfx" -p og-tests-web --lib
  [ "${1:-}" = gfx ] && exit 0
  cd "$engine" && exec sh scripts/build.sh
fi
export OG_BIN="${OG_BIN:-${CARGO_TARGET_DIR:-$repo/target}/debug/og}"

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
  # The templates' playtests, as scripts/check.sh web-reference runs them.
  tp) set -- tests/web/reference.ts --project templates/archetypes/third-person \
    --script "1.0:run;1.1:warp:0;1.3:key_down:KeyF;3.5:key_up:KeyF;3.6:light:1;3.7:light:2;3.8:light:3" \
    --shots title@54,beacon@204,dawn@432 ;;
  example)
    name="${1:?example: a name under examples/}"
    run="example-$name"
    set -- tests/web/reference.ts --project "examples/$name" --playtest tests/frames.og.ts ;;
  probe) set -- "$(cd "$(dirname "${1:?probe: a script}")" && pwd)/$(basename "$1")" ;;
  *) echo "run.sh: no run $run" >&2; exit 2 ;;
esac

root="${OG_GPU_ROOT:-$HOME/.cache/og-gate}"
# The single waiters already queued go first: each is waited for until it
# leaves the queue (takes its slot) or ends; later ones queue behind us.
said=""
for w in "$root"/gpu-wait/*; do
  [ -e "$w" ] || continue
  while [ -e "$w" ] && kill -0 "${w##*/}" 2>/dev/null; do
    [ -n "$said" ] || { echo "run.sh: development is queued for the GPU; safari $run waits" >&2; said=1; }
    sleep 10
  done
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
  probe) bun "$@" || status=$? ;;
  ui) bun "$@" $flag || status=$? ;;
  *) bun "$@" $flag --out "$out" || status=$? ;;
esac
if [ "$run" = ui ]; then cp "$engine"/dist/test-reference/ui/reference-ui-*.json "$keep"/ 2>/dev/null || true
else find "$out" -maxdepth 2 -name '*.json' ! -name '*.cull.json' -exec cp {} "$keep"/ \;; fi
echo "run.sh: safari compat $run ($browser) exited $status in $(( $(date +%s) - started )) s; frames in $out, JSON in $keep" >&2
exit "$status"
