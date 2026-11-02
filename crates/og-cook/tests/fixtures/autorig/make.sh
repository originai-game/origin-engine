#!/bin/sh
# Remakes the auto-rig fixtures with ImageMagick 7: a 100x200 side-view
# stick figure facing right, one canvas-sized PNG per part in stick/
# (named NN-<part>.png, back to front), and the same parts as the layers
# of stick.psd. Made here; CC0 (README.md).
set -e
cd "$(dirname "$0")"
rm -rf stick stick.psd
mkdir stick
part() { # n name colour draw
  magick -size 100x200 xc:none -fill "$3" -draw "$4" "PNG32:stick/$1-$2.png"
}
part 01 arm-upper-l '#7f8c8d' 'roundrectangle 44,48 51,79 3,3'
part 02 arm-lower-l '#95a5a6' 'roundrectangle 44,76 51,105 3,3'
part 03 hand-l      '#bdc3c7' 'roundrectangle 43,102 52,113 3,3'
part 04 leg-upper-l '#34495e' 'roundrectangle 43,108 54,149 4,4'
part 05 leg-lower-l '#2c3e50' 'roundrectangle 43,146 54,185 4,4'
part 06 foot-l      '#1a252f' 'roundrectangle 43,182 65,193 3,3'
part 07 pelvis      '#8e44ad' 'roundrectangle 38,95 61,114 5,5'
part 08 torso       '#c0392b' 'roundrectangle 40,42 59,99 6,6'
part 09 head        '#f1c27d' 'circle 50,28 50,12'
part 10 leg-upper-r '#2980b9' 'roundrectangle 45,108 56,149 4,4'
part 11 leg-lower-r '#3498db' 'roundrectangle 45,146 56,185 4,4'
part 12 foot-r      '#1f618d' 'roundrectangle 45,182 67,193 3,3'
part 13 arm-upper-r '#e67e22' 'roundrectangle 47,48 54,79 3,3'
part 14 arm-lower-r '#f39c12' 'roundrectangle 47,76 54,105 3,3'
part 15 hand-r      '#f8c471' 'roundrectangle 46,102 55,113 3,3'
# The PSD: a flattened composite first (ImageMagick's PSD writer stores
# the first image as the composite), then each part as a layer labelled
# with its part name. PNG32 parts: lower-depth PNGs make ImageMagick
# write bit-packed layer rows that no reader (its own included) decodes.
set --
for f in stick/*.png; do
  n=$(basename "$f" .png); n=${n#??-}
  set -- "$@" \( "$f" -set label "$n" \)
done
magick \( stick/*.png -background none -flatten \) "$@" -compress RLE stick.psd
