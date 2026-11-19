#!/usr/bin/env bash
# The website's captures: the settled frame and the recording of the
# whole course.
#
#   tools/capture.sh [frame|recording|all]        (default: all)
#
# Both are one scripted run from tick 0. Every captured frame waits for
# og.motion's results due by its tick (natively), so a frame is a function
# of the script alone and the same GPU draws the same frames every time.
# Outputs land in og-out/captures/ with their sha256 in SHA256SUMS. The
# frames of record come from the L4 host; anything drawn on a software
# renderer is a preview.
#
# Environment:
#   OG     the og binary (default: this checkout's packaged one, else `og`)
#   SIZE   the frame size (default 1280x720)
#   FPS    the recording's frame rate (default 60)
#   FRAME  the settled frame's time, s (default 12.5: the runner waving on
#          the terrace, the porter cheering)
#   TO     the recording's end, s (default 13.5: the course from its start)
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
packaged="$here/../../packages/engine/native/og.linux-x64"
if [[ -z ${OG:-} ]]; then if [[ -x $packaged ]]; then OG="$packaged"; else OG=og; fi; fi
SIZE="${SIZE:-1280x720}"; FPS="${FPS:-60}"; FRAME="${FRAME:-12.5}"; TO="${TO:-13.5}"
out="$here/og-out/captures"
mkdir -p "$out"
# The pinned models and maps, checked against og-assets.json.
bun "$here/../../scripts/assets.ts" ensure "$here"
what="${1:-all}"

frame() {
  echo "capture: the settled frame at ${FRAME} s"
  rm -rf "$out/frame"
  "$OG" render "$here" --commands "$(awk -v t="$FRAME" 'BEGIN { print t + 1 }'):quit" --from "$FRAME" --to "$(awk -v t="$FRAME" 'BEGIN { print t + 0.5 }')" --fps 1 --size "$SIZE" --out "$out/frame"
  cp "$out/frame/frames/00000.png" "$out/motion-course.png"
}

recording() {
  echo "capture: the course, 0 to ${TO} s at ${FPS} fps"
  # The footing over the whole course (dump_motion), taken three frames
  # before the end: a command at the range's end, or on its last frame's
  # tick, never runs.
  local last
  last="$(awk -v t="$TO" -v f="$FPS" 'BEGIN { print t - 3 / f }')"
  "$OG" render "$here" --commands "$last:dump_motion:feet.json" --from 0 --to "$TO" --fps "$FPS" --size "$SIZE" --out "$out/motion-course.mp4"
}

case "$what" in
  frame) frame ;;
  recording) recording ;;
  all) frame; recording ;;
  *) echo "usage: tools/capture.sh [frame|recording|all]" >&2; exit 2 ;;
esac
(cd "$out" && find . -maxdepth 1 -type f \( -name '*.png' -o -name '*.mp4' \) -printf '%f\n' | sort | xargs -r sha256sum > SHA256SUMS)
echo "capture: $(grep -c . "$out/SHA256SUMS") files in $out"
