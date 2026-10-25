#!/bin/bash
# usage: shots.sh <ogdir> <game> <out>
d=$1; g=$2; out=$3; mkdir -p $out
cd $(git rev-parse --show-toplevel)
cp $d/libog_host_node.so packages/engine/native/origin-engine.linux-x64.node
case $g in
hollowbell) s="0.1:play;0.5:key_down:KeyD;1.5:key_down:Space;1.8:key_up:Space;2:key_down:KeyX;2.05:key_up:KeyX;3:key_down:KeyX;3.05:key_up:KeyX;4:key_down:Space;4.4:key_up:Space";;
kestrel-stair) s="0.1:play:0:quick;0.5:key_down:KeyW;2:key_down:KeyJ;2.05:key_up:KeyJ;3:key_down:KeyD;5:key_up:KeyD";;
lantern-vale) s="0.1:run;0.5:key_down:KeyW;3:key_down:KeyD;5:key_up:KeyD";;
esac
s=$(python3 -c "
import sys
steps=sys.argv[1].split(';')+['1:screenshot:t1.png','2.2:screenshot:t2.png','3.4:screenshot:t3.png','4.6:screenshot:t4.png','4.7:dump_world:world.json','4.8:quit']
steps=[x for x in steps if float(x.split(':')[0])<=4.8]
print(';'.join(sorted(steps,key=lambda x: float(x.split(':')[0]))))" "$s")
OG_ENGINE_PACKAGE=$PWD/packages/engine $d/og run examples/$g --headless --json --size 1280x720 --commands "$s" --out $out > $out/stdout.txt 2> $out/stderr.txt
echo "$g exit $?"; tail -1 $out/stdout.txt | python3 -c "import json,sys; r=json.loads(sys.stdin.read()); print(r['hash'], r['tick'], r.get('unchecked'))"
