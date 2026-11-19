#!/usr/bin/env bash
# The website's captures, in one command: every weather in every season at
# night and at dawn (40 frames), and one recording of the switching.
#
#   tools/capture.sh [frames|recording|all]        (default: all)
#
# Each season is one scripted run from tick 0 that steps through the
# weathers as a player would (rain, storm, fog, clear and wet, snow), so a
# frame is a function of the script alone: the same GPU draws the same
# frame every time. Outputs land in og-out/captures/, with their sha256 in
# og-out/captures/SHA256SUMS. The frames of record come from the L4 host;
# anything drawn on lavapipe is a preview.
#
# Environment:
#   OG      the og binary (default: this checkout's packaged one, else `og`)
#   SIZE    the frame size (default 1920x1080)
#   HOLD    seconds each weather builds before its frame (default 20)
#   SETTLE  seconds drawn before each frame, for the temporal passes (default 0.5)
#   TIMES   which times of day (default "night dawn")
#   SEASONS which seasons (default "spring summer autumn winter")
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
packaged="$here/../../packages/engine/native/og.linux-x64"
if [[ -z ${OG:-} ]]; then if [[ -x $packaged ]]; then OG="$packaged"; else OG=og; fi; fi
SIZE="${SIZE:-1920x1080}"; HOLD="${HOLD:-20}"; SETTLE="${SETTLE:-0.5}"
TIMES="${TIMES:-night dawn}"; SEASONS="${SEASONS:-spring summer autumn winter}"
WEATHERS="rain storm fog clear snow"
out="$here/og-out/captures"
mkdir -p "$out"
# The pinned models and maps, checked against og-assets.json.
bun "$here/../../scripts/assets.ts" ensure "$here"
what="${1:-all}"

# The run's script for one season at one time of day: the HUD off, the
# season and the time set at tick 0, then each weather held and captured.
script() {
  local time="$1" season="$2" t=0
  local commands="0:hud;0:season:$season;0:time:$time"
  for weather in $WEATHERS; do
    commands+=";$t:weather:$weather;$t:advance:$(awk -v h="$HOLD" 'BEGIN { print int(h * 60) }')"
    t="$(awk -v t="$t" -v h="$HOLD" -v s="$SETTLE" 'BEGIN { print t + h + s }')"
    commands+=";$t:screenshot:$time-$season-$weather.png"
    t="$(awk -v t="$t" 'BEGIN { print t + 0.1 }')"
  done
  echo "$commands;$t:quit"
}

frames() {
  cd "$here"
  for time in $TIMES; do
    for season in $SEASONS; do
      echo "capture: $time, $season"
      local log="$out/$time-$season.log"
      if ! "$OG" run . --headless --size "$SIZE" --out "$out" --commands "$(script "$time" "$season")" > "$log" 2>&1; then
        tail -n 20 "$log" >&2
        echo "capture: the $time $season run failed (its log: $log)" >&2
        exit 1
      fi
      for weather in $WEATHERS; do
        [[ -s "$out/$time-$season-$weather.png" ]] || { echo "capture: $time-$season-$weather.png was not written (the run's log: $log)" >&2; exit 1; }
      done
    done
  done
}

# The switching, as one take with the HUD on: the weathers in turn, the
# year turning under them, and the dawn breaking at the end.
recording() {
  cd "$here"
  local commands="4:weather:storm;14:weather:fog;22:season:summer;26:weather:clear;34:season:autumn;42:weather:rain;50:season:winter;56:weather:snow;70:time:dawn;84:season:spring;88:weather:clear;96:hud"
  echo "capture: the switching"
  "$OG" render . --commands "$commands" --from 0 --to 100 --fps 30 --size "$SIZE" --out "$out/switching.mp4"
}

case "$what" in
  frames) frames ;;
  recording) recording ;;
  all) frames; recording ;;
  *) echo "usage: tools/capture.sh [frames|recording|all]" >&2; exit 2 ;;
esac
(cd "$out" && find . -maxdepth 1 -type f \( -name '*.png' -o -name '*.mp4' \) -printf '%f\n' | sort | xargs -r sha256sum > SHA256SUMS)
echo "capture: $(grep -c . "$out/SHA256SUMS") files in $out"
