// The Berry Rush: no berry comes up before the first third of a second, then
// one every nine ticks until six are up (Tuning.berriesUp), away from the
// critters; Pip's sweep of the island takes some (the HUD counts them under
// an apple); a dip in the sea costs one and a moment off the island, then
// back at a random place near the middle; the whistle ranks by berries, the
// most sets the isle's record, which the scoreboard announces and og.platform
// keeps under `records`.
import { playtest } from "@origingame/engine/playtest";
import { calls, game, OVER, PLAY, script, sweep, texts, twoPlayers, value } from "./lib.ts";

interface BerryV { up: boolean; gold: boolean; age: number }

const N = 10;
const berries = (t: number, name: string) => Array.from({ length: N }, (_, b) => `${t}:get:berry-${b}.Berry:${name}-${b}.json`);

export default playtest({
  name: "the berry rush: berries come up to six, a sweep takes some, a dip costs one and respawns, the whistle ranks by berries and records the most",
  commands: script(
    ...twoPlayers(0.2, 2),
    "0.7:get:Game:early.json",
    ...berries(0.7, "early"),
    ...berries(2.0, "up"),
    "2.0:get:Game:up.json",
    ...sweep(2.1),
    "6.3:get:Game:swept.json",
    "6.3:dump_ui:hud.json",
    "6.4:knock:0",
    "6.7:get:Game:dipped.json",
    "8.0:get:Game:back.json",
    "8.1:finish",
    "8.3:get:Game:over.json",
    "10.4:dump_ui:scores.json",
    "10.4:dump_platform:calls.json",
    "10.5:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const early = game(run, "early.json");
    t.ok(early.phase === PLAY && early.round === 1 && early.played < 20, `the berry round is in play (${early.played} ticks in)`);
    const upAt = (name: string) => Array.from({ length: N }, (_, b) => value<BerryV>(run, `${name}-${b}.json`)).filter((v) => v.up).length;
    t.equal(upAt("early"), 0, "no berry is up in the first twenty ticks");
    t.equal(upAt("up"), 6, "six berries are up a second and a half in");
    const up = game(run, "up.json");
    t.ok(up.berriesTaken === 0 && Math.abs(up.x0 - 2.4) < 0.1, "none taken yet: the critters have not moved and the berries come up away from them");
    const swept = game(run, "swept.json");
    t.ok(swept.berriesTaken >= 1 && swept.berries0 >= 1, `Pip's sweep of the island took berries (${swept.berries0} berries, ${swept.berriesTaken} taken)`);
    t.ok(Math.abs(swept.x0 - 2.4) < 1.5 && Math.abs(swept.z0 - 2.4) < 1.5, `and brought him back about where he began (${swept.x0.toFixed(2)}, ${swept.z0.toFixed(2)})`);
    const hud = texts(run, "hud.json");
    t.ok(hud.includes("apple") && hud.includes(`${swept.berries0}`), `the HUD counts Pip's berries under an apple (${swept.berries0})`);
    const dipped = game(run, "dipped.json");
    t.ok(!dipped.alive0 && dipped.splashes === 1 && dipped.berries0 === swept.berries0 - 1, `a dip in the sea splashed and cost a berry (${dipped.berries0} left)`);
    t.ok(dipped.phase === PLAY && dipped.standing === 2, "the round goes on: nobody is out of a berry rush");
    const back = game(run, "back.json");
    t.ok(back.alive0 && back.y0 > -0.5 && Math.hypot(back.x0, back.z0) < 3.2, `Pip is back on the island after a moment, near the middle (${back.x0.toFixed(2)}, ${back.z0.toFixed(2)})`);
    const over = game(run, "over.json");
    t.equal(over.phase, OVER, "the whistle ended the round");
    const most = Math.max(over.berries0, over.berries1);
    const first = over.berries0 > over.berries1 ? 0 : 1;
    t.ok((first === 0 ? over.place0 : over.place1) === 1 && (first === 0 ? over.points0 : over.points1) === 3, `the most berries is 1st for 3 points (${over.berries0} against ${over.berries1})`);
    t.ok(over.newRecord && over.bestBerries === most, `the most set the isle's record (${most})`);
    const scores = texts(run, "scores.json");
    t.ok(scores.includes("Round 2: Berry Rush"), "the scoreboard names the round");
    t.ok(scores.includes(`${most} ${most === 1 ? "berry" : "berries"}`), `and the berries of each (${most})`);
    t.ok(scores.includes(`New record: ${most} ${most === 1 ? "berry" : "berries"}!`), "and announces the record");
    const written = calls(run, "calls.json").find((c) => c.method === "store.set" && c.args.key === "records");
    t.ok(written !== undefined && written.args.value?.berries === most && written.result?.ok === true, "og.platform keeps the record under `records`");
  },
});
