// The far side streams: its pines, its arch and its banner (the far-side
// group of scenes/main.ogt) stay off the GPU until the camera comes within
// 26 m of the far side's heart; running there makes the group resident
// (og.StreamEvent, a toast, its bytes in og.stream.budget), walking back
// lets it go past 34 m. The pines' colliders stand either way, so the run
// replays to the same hash (two runs).
import { playtest } from "@origingame/engine/playtest";
import { game, RESIDENT, texts, UNLOADED } from "./lib.ts";

interface Stream { used: number; peak: number; groups: Array<{ name: string; state: string; loads: number; memory: number; trigger: { kind: string } }> }

export default playtest({
  name: "the far side's streaming group loads as the camera nears it and unloads as it leaves, replaying the same",
  runs: 2,
  commands: [
    "0.3:run",
    "0.5:dump_stream:start.json",
    "0.6:warp:2:-6",
    "0.7:key_down:KeyW",
    "0.7:key_down:ShiftLeft",
    "2.8:key_up:ShiftLeft",
    "2.8:key_up:KeyW",
    "3.4:dump_stream:near.json",
    "3.4:get:Game:near-game.json",
    "3.4:dump_ui:near-ui.json",
    "3.5:warp:0:24",
    "4.4:dump_stream:left.json",
    "4.4:get:Game:left-game.json",
    "4.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const group = (file: string) => run.json<Stream>(file).groups.find((g) => g.name === "far-side")!;
    const start = group("start.json");
    t.ok(start.state === "unloaded" && start.trigger.kind === "distance", "the far side starts unloaded, on a distance trigger");
    const near = group("near.json");
    t.ok(near.state === "resident" && near.loads === 1, "running north the camera nears it and it becomes resident");
    t.ok(run.json<Stream>("near.json").used >= near.memory && near.memory > 0, `its ${near.memory} bytes count in the budget`);
    const g = game(run, "near-game.json");
    t.ok(g.far === RESIDENT && g.farLoads === 1 && g.farUsed > 0, "the game saw og.StreamEvent and og.stream.budget");
    t.ok(texts(run, "near-ui.json").includes("The far side of the vale stirs"), "and says so");
    t.equal(group("left.json").state, "unloaded", "back at the south end it is let go");
    t.equal(game(run, "left-game.json").far, UNLOADED, "and the game saw that too");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
  },
});
