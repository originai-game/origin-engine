#!/bin/sh
# run.sh OUT: the three shots polish flags, with their UI dumps.
set -e
og=/Volumes/app/origin-engine-web-host-tools/packages/engine/native/og.darwin-arm64
proj=/Volumes/app/origin-engine-web-host-tools/templates/archetypes/third-person
out="$1"; rm -rf "$out"; mkdir -p "$out"
$og run $proj --headless --size 1280x720 --out "$out/a" --commands "0.9:screenshot:title.png;0.9:dump_ui:title.json;1.0:quit" > "$out/a.log" 2>&1 &
$og run $proj --headless --size 1280x720 --out "$out/b" --commands "0.3:run;0.8:key_down:Escape;0.834:key_up:Escape;1.75:screenshot:pause.png;1.75:dump_ui:pause.json;1.8:touch_move_to:settings;1.817:touch_start;1.85:touch_end;2.4:move_to_text:简体中文;2.42:touch_start;2.45:touch_end;3.2:screenshot:settings-zh.png;3.2:dump_ui:settings-zh.json;3.3:quit" > "$out/b.log" 2>&1 &
$og run $proj --headless --size 1280x720 --out "$out/c" --commands "1.0:run;1.1:warp:0;1.3:key_down:KeyF;3.5:key_up:KeyF;3.6:light:1;3.7:light:2;3.8:light:3;7.2:screenshot:dawn.png;7.2:dump_ui:dawn.json;7.3:quit" > "$out/c.log" 2>&1 &
wait
M=$(dirname "$0")/measure.py
python3 $M "$out/a/title.png" "$out/a/title.json" play,Footer,settings
python3 $M "$out/b/pause.png" "$out/b/pause.json" resume,settings,quit
python3 $M "$out/b/settings-zh.png" "$out/b/settings-zh.json" Done,LanguageLabel
python3 $M "$out/c/dawn.png" "$out/c/dawn.json" Again,Title
