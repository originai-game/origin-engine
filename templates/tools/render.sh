#!/bin/sh
# Renders the template's opening to a video with its sound, a contact sheet
# and frames.json (docs/design.md §2.9, og-game §19):
#
#   sh templates/tools/render.sh [project] [out.mp4]
#
# The script presses Start and walks the trickster for a few seconds; the
# fps (30) divides the template's tick (60), so every frame is a tick.
# Iterate at 640x360 (about 0.1 s a frame on a CPU-only sandbox), render
# 1280x720 once; --jobs splits the range over processes and gives the same
# frames. Without ffmpeg (og doctor) the frames are written instead.
# Once timelines exist (stream S2), `--timeline intro` renders
# assets/timelines/intro.ogt over its duration instead of --commands.
set -eu
project="${1:-templates/game}"
out="${2:-$project/og-out/intro.mp4}"
og render "$project" \
  --commands "0.2:touch_move:60,130;0.4:touch_start;0.5:touch_end;1:key_down:KeyW;3:key_up:KeyW;3.2:key_down:KeyD;5:key_up:KeyD" \
  --fps 30 --from 0 --to 6 --size 640x360 \
  --out "$out" --audio --contact-sheet 6 --jobs 4
