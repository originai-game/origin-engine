#!/bin/sh
# Remakes the auto-rig fixtures with ImageMagick 7: a 100x200 side-view
# stick figure facing right, one canvas-sized PNG per part in stick/
# (named NN-<part>.png, back to front), and the same parts as the layers
# of stick.psd. Made here; CC0 (README.md).
set -e
cd "$(dirname "$0")"
rm -rf stick stick.psd front dog blob
mkdir stick front dog blob
# part [dir WxH] n name colour draw
part() {
  dir=stick size=100x200
  case $1 in */*) dir=${1%%/*} size=${1#*/}; shift ;; esac
  magick -size "$size" xc:none -fill "$3" -draw "$4" -define png:exclude-chunks=date,time "PNG32:$dir/$1-$2.png"
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
# The front view (humanoid-front): the same figure facing the viewer, its
# image-left (`-l`) parts on the left, the feet turned out.
F=front/100x200
part $F 01 leg-upper-l '#34495e' 'roundrectangle 39,108 49,149 4,4'
part $F 02 leg-lower-l '#2c3e50' 'roundrectangle 39,146 49,185 4,4'
part $F 03 foot-l      '#1a252f' 'roundrectangle 31,182 49,193 3,3'
part $F 04 leg-upper-r '#2980b9' 'roundrectangle 51,108 61,149 4,4'
part $F 05 leg-lower-r '#3498db' 'roundrectangle 51,146 61,185 4,4'
part $F 06 foot-r      '#1f618d' 'roundrectangle 51,182 69,193 3,3'
part $F 07 pelvis      '#8e44ad' 'roundrectangle 37,95 63,114 5,5'
part $F 08 torso       '#c0392b' 'roundrectangle 38,42 62,99 6,6'
part $F 09 head        '#f1c27d' 'circle 50,28 50,12'
part $F 10 arm-upper-l '#7f8c8d' 'roundrectangle 30,46 40,79 3,3'
part $F 11 arm-lower-l '#95a5a6' 'roundrectangle 30,76 40,105 3,3'
part $F 12 hand-l      '#bdc3c7' 'roundrectangle 29,102 41,113 3,3'
part $F 13 arm-upper-r '#e67e22' 'roundrectangle 60,46 70,79 3,3'
part $F 14 arm-lower-r '#f39c12' 'roundrectangle 60,76 70,105 3,3'
part $F 15 hand-r      '#f8c471' 'roundrectangle 59,102 71,113 3,3'
# A dog (quadruped) facing right: the far legs behind the body, a tail
# straight back from the rump, a neck up to the head.
D=dog/200x120
part $D 01 leg-back-upper-l  '#7f5539' 'roundrectangle 56,55 68,85 4,4'
part $D 02 leg-back-lower-l  '#6f4518' 'roundrectangle 56,82 66,107 4,4'
part $D 03 paw-back-l        '#432818' 'roundrectangle 54,104 72,113 3,3'
part $D 04 leg-front-upper-l '#7f5539' 'roundrectangle 122,55 132,85 4,4'
part $D 05 leg-front-lower-l '#6f4518' 'roundrectangle 122,82 132,107 4,4'
part $D 06 paw-front-l       '#432818' 'roundrectangle 120,104 138,113 3,3'
part $D 07 tail              '#b08968' 'roundrectangle 18,36 58,44 4,4'
part $D 08 body              '#ddb892' 'roundrectangle 50,30 150,70 15,15'
part $D 09 neck              '#c9a37a' 'roundrectangle 136,16 154,50 6,6'
part $D 10 head              '#e6ccb2' 'roundrectangle 142,4 186,32 8,8'
part $D 11 leg-back-upper-r  '#9c6644' 'roundrectangle 64,55 76,85 4,4'
part $D 12 leg-back-lower-r  '#99582a' 'roundrectangle 64,82 74,107 4,4'
part $D 13 paw-back-r        '#582f0e' 'roundrectangle 62,104 80,113 3,3'
part $D 14 leg-front-upper-r '#9c6644' 'roundrectangle 130,55 140,85 4,4'
part $D 15 leg-front-lower-r '#99582a' 'roundrectangle 130,82 140,107 4,4'
part $D 16 paw-front-r       '#582f0e' 'roundrectangle 128,104 146,113 3,3'
# A blob (creature): a round body on four short legs, two eyes.
B=blob/120x120
part $B 01 limb-1 '#2d6a4f' 'roundrectangle 32,70 41,110 4,4'
part $B 02 limb-2 '#2d6a4f' 'roundrectangle 48,74 57,112 4,4'
part $B 03 limb-3 '#2d6a4f' 'roundrectangle 63,74 72,112 4,4'
part $B 04 limb-4 '#2d6a4f' 'roundrectangle 79,70 88,110 4,4'
part $B 05 body   '#52b788' 'ellipse 60,52 34,30 0,360'
part $B 06 eye-l  '#081c15' 'circle 48,46 48,40'
part $B 07 eye-r  '#081c15' 'circle 72,46 72,40'
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
