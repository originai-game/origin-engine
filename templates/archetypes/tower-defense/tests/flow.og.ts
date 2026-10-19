// The core loop through real input: Play on the title, a hedge picked on
// the tray and tapped onto the lawn (the tap also moves the cursor there),
// two thorns set beside it with the keys (1, the arrows, Space), then the first wave: visitors come through the gate and
// walk to the doorstep, the thorns see them (og.ai), shoot, and every
// visitor turned away pays its bounty.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, play, press, TITLE, tap, tapCell } from "./lib.ts";

export default playtest({
  name: "title, plants set by tap and by keys, the first wave walks in and is shot down",
  commands: [
    "0.3:screenshot:title.png",
    "0.4:get:Game:title.json",
    play(0.5),
    tap("Plant#2/Pick", 1.3),
    tapCell(4, 4, 1.5),
    press("Digit1", 1.8),
    press("ArrowLeft", 2.3),
    press("Space", 2.4),
    press("ArrowRight", 2.5),
    press("ArrowRight", 2.6),
    press("Space", 2.7),
    "2.9:get:Game:set.json",
    "2.95:screenshot:play.png",
    "3.9:dump_entity:visitor-0:walker.json",
    "4.2:dump_entity:visitor-0:walker2.json",
    "8.0:screenshot:wave.png",
    "13.0:get:Game:fought.json",
    "13.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const set = game(run, "set.json");
    t.equal(set.phase, PLAY, "Play started a round");
    t.equal(set.placed, 3, "a hedge by tap and two thorns by keys");
    t.equal(set.gold, 150 - 10 - 50 - 50, "each plant took its price");
    t.equal(set.selected, 0, "1 picked the thorn back");
    t.ok(set.cursorX === 5 && set.cursorY === 4, `the tap put the cursor on (4, 4) and the arrows walked it to (5, 4) (${set.cursorX}, ${set.cursorY})`);
    t.equal(set.refused, 0, "nothing was refused");
    const a = run.json<{ components: Record<string, { position?: number[]; seeking?: boolean }> }>("walker.json").components;
    const b = run.json<{ components: Record<string, { position?: number[] }> }>("walker2.json").components;
    t.equal(a["og.NavAgent"]!.seeking, true, "the first visitor walks the flow field");
    const step = Math.hypot(b["og.Transform"]!.position![0]! - a["og.Transform"]!.position![0]!, b["og.Transform"]!.position![1]! - a["og.Transform"]!.position![1]!);
    t.ok(step > 0.25 && step < 0.5, `it walks at a beetle's pace (${step.toFixed(2)} m in 0.3 s)`);
    const fought = game(run, "fought.json");
    t.equal(fought.wave, 1, "the first wave came");
    t.ok(fought.shots >= 6, `the thorns shot at what they saw (${fought.shots} shots)`);
    t.ok(fought.kills >= 3, `visitors were turned away (${fought.kills})`);
    t.ok(fought.gold > set.gold, `bounties filled the purse (${set.gold} to ${fought.gold})`);
    for (const f of ["title.png", "play.png", "wave.png"]) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
  },
});
