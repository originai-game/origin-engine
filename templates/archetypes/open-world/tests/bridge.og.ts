// The strait and the bridge: at the landing with too little in the bag the
// prompt says what the bridge needs; walking on into the deep strait, the
// tide carries the explorer back to the camp with half the bag (the soft
// fail). With six wood and four stone, holding E at the landing builds the
// bridge (the bag pays for it, the deck rises out of the sea), and the
// explorer walks across it dry onto the north isle.
import { playtest } from "@origingame/engine/playtest";
import { game, GATHER, position, script, texts, VIEW } from "./lib.ts";

// North across the strait: W in third and first person (the warp faces the
// span), W and D together in isometric (its fixed heading is turned 45°).
const north = (t: number, seconds: number) => [`${t}:key_down:KeyW`, `${(t + seconds).toFixed(2)}:key_up:KeyW`, ...(VIEW === "isometric-2.5d" ? [`${t}:key_down:KeyD`, `${(t + seconds).toFixed(2)}:key_up:KeyD`] : [])];

export default playtest({
  name: "the strait washes an explorer back with half the bag; six wood and four stone build the bridge, which carries it across",
  size: "1280x720",
  commands: script([
    "0.3:run",
    "0.5:give:4:2",
    "0.6:warp:bridge",
    "0.9:dump_ui:need.json",
    ...north(1.0, 5.5),
    "6.6:get:Game:washed.json",
    "6.6:get:hero.og.Transform:washed-at.json",
    "6.6:dump_ui:washed-ui.json",
    "6.7:give:4:3",
    "6.8:warp:bridge",
    "7.1:dump_ui:build.json",
    "7.2:key_down:KeyE",
    "8.4:get:Game:building.json",
    "10.0:key_up:KeyE",
    "10.1:get:Game:built.json",
    "10.1:get:bridge-deck-3.og.Transform:deck.json",
    "10.1:dump_ui:built-ui.json",
    "10.2:warp:bridge",
    "10.4:get:hero.og.Transform:landing.json",
    ...north(10.5, 8.0),
    "18.7:get:Game:across.json",
    "18.7:get:hero.og.Transform:across-at.json",
    "18.7:dump_ui:across-ui.json",
    "18.8:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(texts(run, "need.json").includes("The bridge needs 6 wood and 4 stone (you have 4 and 2)"), "at the landing the prompt says what the bridge needs");
    const washed = game(run, "washed.json");
    t.equal(washed.washes, 1, "walking on into the deep strait, the tide carried the explorer back");
    t.ok(washed.wood === 2 && washed.stone === 1, `with half the bag (wood ${washed.wood}, stone ${washed.stone})`);
    const [, , wz] = position(run, "washed-at.json");
    t.ok(wz > 70, `to the camp on the south beach (z ${wz.toFixed(1)})`);
    t.ok(texts(run, "washed-ui.json").includes("The tide carried you back to camp, and half the bag with it"), "and the toast says so");
    t.equal(washed.phase, 1, "the run goes on");
    t.ok(texts(run, "build.json").includes(`Hold ${GATHER} to build the bridge`), "with six wood and four stone the prompt offers to build");
    const building = game(run, "building.json");
    t.ok(building.building > 0.8 && !building.bridge, `holding E builds it (${building.building.toFixed(2)} s)`);
    const built = game(run, "built.json");
    t.ok(built.bridge && built.wood === 0 && built.stone === 0, `after Tuning.buildSeconds the bridge stands, paid from the bag (${built.wood}, ${built.stone})`);
    t.ok(position(run, "deck.json")[1] > 0.5, `its deck rose out of the sea (y ${position(run, "deck.json")[1].toFixed(2)})`);
    t.ok(texts(run, "built-ui.json").includes("The bridge stands: the north isle is open"), "and the toast says so");
    const [, , lz] = position(run, "landing.json");
    const across = game(run, "across.json");
    const [, , az] = position(run, "across-at.json");
    t.ok(az < -66 && across.washes === 1, `the explorer walked across dry onto the north isle (z ${lz.toFixed(1)} → ${az.toFixed(1)}, washes ${across.washes})`);
    t.ok(texts(run, "across-ui.json").includes("The North Isle"), "where the map names the north isle");
  },
});
