#!/bin/sh
# Renders the characters' parts (knight.py, ...) with headless Chrome into
# canvas-sized PNGs and stacks them into a layered PSD with ImageMagick 7:
# assets/chars/<name>.psd, one layer per part named for it, back to front.
# CHROME names the browser (agent-browser's Chrome for Testing works).
set -e
cd "$(dirname "$0")"
CHROME=${CHROME:-$(ls -d "$HOME"/.agent-browser/browsers/chrome-*/chrome | tail -1)}
for name in ${*:-knight}; do
  tmp=$(mktemp -d)
  python3 -B "$name.py" "$tmp"
  w=$(python3 -B -c "import $name; print($name.W)"); h=$(python3 -B -c "import $name; print($name.H)")
  set --
  for svg in "$tmp"/[0-9]*.svg; do
    png=${svg%.svg}.png
    "$CHROME" --headless --no-sandbox --disable-gpu --hide-scrollbars \
      --default-background-color=00000000 --window-size="$w,$h" \
      --screenshot="$png" "file://$svg" >/dev/null 2>&1
    magick "$png" -trim "PNG32:$png"  # trimmed, its offset kept: a small layer where it sits
    part=$(basename "$svg" .svg); part=${part#??-}
    set -- "$@" \( "$png" -set label "$part" \)
  done
  magick \( "$tmp"/[0-9]*.png -background none -flatten \) "$@" -compress RLE "../assets/chars/$name.psd"
  cp "$tmp"/all.svg "$name.svg"
  rm -rf "$tmp"
done
