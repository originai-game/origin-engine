#!/bin/bash
# usage: norender.sh <og> <addon> <game> <out>
og=$1; addon=$2; g=$3; out=$4; mkdir -p $out
cd $(git rev-parse --show-toplevel)
cp $addon packages/engine/native/origin-engine.linux-x64.node
case $g in
hollowbell) s="0.1:play;0.5:key_down:KeyD;1.5:key_down:Space;1.8:key_up:Space;2:key_down:KeyX;2.05:key_up:KeyX;3:key_down:KeyX;3.05:key_up:KeyX;4:key_down:Space;4.4:key_up:Space;6:key_up:KeyD;6:key_down:KeyA;8:key_down:KeyX;8.05:key_up:KeyX;9.5:key_up:KeyA";;
kestrel-stair) s="0.1:play:0:quick;0.5:key_down:KeyW;2:key_down:KeyJ;2.05:key_up:KeyJ;3:key_down:KeyD;5:key_up:KeyD;5:key_down:Space;5.2:key_up:Space;7:key_down:KeyJ;7.05:key_up:KeyJ;9.5:key_up:KeyW";;
lantern-vale) s="0.1:run;0.5:key_down:KeyW;3:key_down:KeyD;5:key_up:KeyD;6:key_up:KeyW;6:warp:0;6.2:key_down:KeyE;8:key_up:KeyE;8.2:key_down:KeyS;9.9:key_up:KeyS";;
esac
OG_RENDERER=none OG_ENGINE_PACKAGE=$PWD/packages/engine $og run examples/$g --headless --json --ticks 600 --size 1280x720 --commands "$s" --out $out > $out/stdout.txt 2> $out/stderr.txt
