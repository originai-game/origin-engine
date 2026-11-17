#!/usr/bin/env bash
# Draws the minimap's picture of the district, assets/textures/map.png: the
# alley's footprints from above, 50 m a side (x -25 to 25 across, z -40 to
# 10 down), in the positions src/set.ts builds them at. Needs ImageMagick.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# A box about (x, z) of half sizes (hx, hz), in pixels of a 512 px map.
box() { awk -v x="$1" -v z="$2" -v hx="$3" -v hz="$4" 'BEGIN { k = 512 / 50; printf "rectangle %.1f,%.1f %.1f,%.1f", (x - hx + 25) * k, (z - hz + 40) * k, (x + hx + 25) * k, (z + hz + 40) * k }'; }
magick -size 512x512 xc:'#14121c' \
  -fill '#2b2838' -draw "$(box 0 -15 3.4 25)" -draw "$(box 0 -34 25 2.6)" -draw "$(box 0 7 25 3)" \
  -fill '#3d394e' -stroke '#6a6485' -strokewidth 2 \
  -draw "$(box -11.4 -1.5 5.5 5.5)" -draw "$(box -8.6 -14 4.8 5.3)" -draw "$(box -9.6 -27 4.9 4.9)" \
  -draw "$(box 9.4 -2.5 4.8 5.3)" -draw "$(box 9.6 -15 5.5 5.5)" -draw "$(box 8.6 -27.5 5.3 5.8)" \
  -fill '#57506e' -draw "$(box -3.7 3.1 1.1 2.2)" \
  -fill '#4a4560' -stroke none -draw "$(box 0 -20.5 4.5 1.1)" \
  -fill '#8a5560' -draw "$(box 0 -24 3.2 0.3)" \
  -strip "$here/assets/textures/map.png"
echo "draw-map: $here/assets/textures/map.png"
