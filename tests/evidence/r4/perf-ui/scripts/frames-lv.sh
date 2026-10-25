#!/bin/bash
d=$1; out=$2; mkdir -p $out
cd $(git rev-parse --show-toplevel)
cp $d/libog_host_node.so packages/engine/native/origin-engine.linux-x64.node
s="1.0:screenshot:title.png;1.1:run;2.2:screenshot:start.png;2.3:warp:0;2.4:key_down:KeyE;3.8:key_up:KeyE;4.0:screenshot:lantern.png;4.1:warp:-19:0;4.6:screenshot:slime.png;4.65:dump_world:world.json;4.7:warp:8:-20;6.0:screenshot:far.png;6.1:language:zh-CN;6.2:settings;7.0:screenshot:settings-zh.png;7.1:quit"
OG_ENGINE_PACKAGE=$PWD/packages/engine $d/og run examples/lantern-vale --headless --json --size 1280x720 --commands "$s" --out $out > $out/stdout.txt 2> $out/stderr.txt
echo "exit $?"; tail -1 $out/stdout.txt | python3 -c "import json,sys; r=json.loads(sys.stdin.read()); print(r['hash'], r['tick'], r.get('warnings'))"
