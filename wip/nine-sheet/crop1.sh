#!/bin/sh
# crop1.sh <og.png> <chrome.png> <scale> <x> <y> <w> <h> <zoom%> <out>: og above Chrome, region in CSS px.
a=$1; b=$2; s=$3; x=$4; y=$5; w=$6; h=$7; z=$8; o=$9
g="$(($w*$s))x$(($h*$s))+$(($x*$s))+$(($y*$s))"
magick \( $a -crop $g +repage -filter point -resize $z% \) \( $b -crop $g +repage -filter point -resize $z% \) -background '#f0f' -splice 0x3 -append $o
