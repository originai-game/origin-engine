#!/bin/bash
# usage: phys.sh <og> <out>
og=$1; out=$2; mkdir -p $out
cd $(git rev-parse --show-toplevel)
s="0.1:play:0:quick;0.5:key_down:KeyW;2:key_down:KeyJ;2.05:key_up:KeyJ;3:key_down:KeyD;5:key_up:KeyD;5:key_down:Space;5.2:key_up:Space;7:key_down:KeyJ;7.05:key_up:KeyJ;9.5:key_up:KeyW"
OG_RENDERER=none OG_ENGINE_PACKAGE=$PWD/packages/engine $og run examples/kestrel-stair --headless --json --ticks 600 --size 1280x720 --commands "$s" --out $out > $out/stdout.txt 2> $out/stderr.txt
tail -1 $out/stdout.txt | python3 -c "import json,sys; r=json.loads(sys.stdin.read()); p=[x for x in r['systems'] if x['name']=='og.physics'][0]; print(r['hash'], 'physics mean %.3f'%p['meanMs'], 'step p95', r['frames']['stepP95Ms'])"
