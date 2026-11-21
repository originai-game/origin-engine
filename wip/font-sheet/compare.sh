#!/bin/sh
# compare.sh <engine.png> <chrome.png> <out.png> <label>: stacked sheets plus 4x crops of the 12-16 px rows.
E=$1; C=$2; O=$3; L=$4
magick $E -crop 600x70+0+0 +repage -filter point -resize 400% /tmp/_e1.png
magick $C -crop 600x70+0+0 +repage -filter point -resize 400% /tmp/_c1.png
magick $E -crop 600x70+0+186 +repage -filter point -resize 400% /tmp/_e2.png
magick $C -crop 600x70+0+186 +repage -filter point -resize 400% /tmp/_c2.png
magick \( $E -crop 1200x540+0+0 +repage -gravity northwest -fill '#ff6' -pointsize 18 -annotate +900+10 "og ($L)" \) \
       \( $C -crop 1200x540+0+0 +repage -gravity northwest -fill '#6ff' -pointsize 18 -annotate +900+10 "Chrome ($L)" \) -append /tmp/_full.png
magick \( /tmp/_e1.png /tmp/_c1.png +append \) \( /tmp/_e2.png /tmp/_c2.png +append \) -append -resize 50% /tmp/_zoom.png
magick /tmp/_full.png /tmp/_zoom.png -background '#000' -gravity center -append $O
