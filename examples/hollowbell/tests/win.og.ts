// The end. Up at the bell with fewer than three chimes is refused with a
// toast; with three the prompt says so, the bell swings and tolls (its
// flipbook's `toll` events play the bell), and the results come under the
// second look (looks/dawn.ogt: the engine's four-tone handheld palette,
// applied with og.render.applyLook): the score counted from the glimmers,
// the foes and the time, stars, the bell-keeper's name, a new best written
// to the store; Play again starts a fresh run under the game's own look.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, OVER, PLAY, press, script, tap, texts, value } from "./lib.ts";

export default playtest({
  name: "the bell refuses two chimes, rings with three, and the results under the dawn look keep a best; Play again",
  commands: script(
    "0.2:name:Mira",
    "0.3:play",
    "0.4:give:chimes:2",
    "0.5:warp:bell:1.2",
    press("ArrowUp", 1.0),
    "1.2:dump_ui:refused.json",
    "1.3:give:chimes:3",
    "1.3:give:glimmers:12",
    "1.5:dump_ui:prompt.json",
    press("ArrowUp", 1.6),
    "1.8:get:Game:ringing.json",
    "1.8:get:bell.og.Flipbook:swing.json",
    "3.4:get:sfx-4096.Sfx:tolls.json",
    "5.6:get:Game:over.json",
    "5.6:dump_ui:results.json",
    "5.6:dump_platform:calls.json",
    "5.7:get:og.Stylise:dawn.json",
    tap("again", 5.8),
    "6.6:get:Game:again.json",
    "6.6:get:og.Stylise:main.json",
    "6.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(texts(run, "refused.json").includes("The bell needs 1 more chime"), "two chimes: the bell refuses, and says what it needs");
    t.ok(texts(run, "prompt.json").includes("Up: ring the bell"), "three: the prompt offers the bell");
    t.ok(game(run, "ringing.json").ringing > 0, "Up rang it");
    t.equal(value<{ clip: number }>(run, "swing.json").clip, 1, "the bell swings (its ring clip)");
    t.ok(value<{ plays: number }>(run, "tolls.json").plays >= 2, `and tolls on its frames (${value<{ plays: number }>(run, "tolls.json").plays} tolls)`);
    const over = game(run, "over.json");
    t.equal(over.phase, OVER, "the results");
    t.ok(over.look === 1, "under the dawn look");
    const s = 12 * 10 + 0 + Math.max(0, Math.round((300 - over.time) * 2));
    t.ok(Math.abs(over.score - s) <= 2 && over.newBest && over.best === over.score, `the score ${over.score} is a new best`);
    const shown = texts(run, "results.json");
    t.ok(shown.includes("The bell rings again") && shown.includes("Mira") && shown.includes("New best"), "the results name the bell-keeper and the new best");
    t.ok(elements(run, "results.json").filter((e) => e.text === "star").length === 3, "three stars are drawn");
    const calls = run.json<{ calls: Array<{ method: string; args: Record<string, unknown> }> }>("calls.json").calls;
    t.ok(calls.some((c) => c.method === "store.set" && c.args["key"] === "best" && c.args["value"] === over.score), "the best is kept in og.platform's store");
    const dawn = value<{ pixelHeight: number }>(run, "dawn.json");
    t.equal(dawn.pixelHeight, 160, "the dawn look's coarser grid is applied");
    const again = game(run, "again.json");
    t.ok(again.phase === PLAY && again.chimes === 0 && again.look === 0, "Play again: a fresh run under the game's own look");
    t.equal(value<{ pixelHeight: number }>(run, "main.json").pixelHeight, 224, "its 224-line grid");
  },
});
