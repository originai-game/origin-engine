// Progress kept in og.platform's store across a relaunch (store: "keep").
// The first process lights a lantern (its light, a toast, the hearts back,
// and the run written to the key `save` at the revision read), then Save
// and quit returns to the title. The relaunch reads the save on its first
// tick: the title offers Continue (the primary action) and names the slot
// ("Mira: 1 chime kept"); Continue puts Wren by the lantern with the
// chime, the glimmers and the name, and the beaten beetle stays beaten.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, hold, PLAY, press, script, tap, TITLE, value } from "./lib.ts";

export default playtest({
  name: "a lantern saves the run, Save and quit, and a relaunch continues it by the lantern",
  store: "keep",
  commands: script(
    "0.2:name:Mira",
    "0.3:play",
    "0.4:give:glimmers:7",
    "0.5:give:chimes:1",
    "0.6:hearts:2",
    "1.2:warp:lantern-0:-1",
    hold("ArrowRight", 1.6, 0.3),
    "2.2:get:Game:lit.json",
    "2.2:get:lantern-0.og.Light2D:light.json",
    "2.3:dump_platform:calls.json", // not platform.json: that is the kept store (FINDINGS.md #F-storefile)
    press("Escape", 2.5),
    tap("quit", 2.9),
    "3.6:get:Game:quit.json",
    "3.7:quit",
  ),
  then: [
    {
      commands: [
        "0.8:dump_ui:title.json",
        "0.8:get:Game:at-title.json",
        "0.9:touch_move_to:continue",
        "0.917:touch_start",
        "0.95:touch_end",
        "1.8:get:Game:continued.json",
        "1.8:get:Prefs:prefs.json",
        "1.8:get:lantern-0.og.Light2D:light.json",
        "1.8:dump_ui:play.json",
        "1.9:quit",
      ].join(";"),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first process ran without warnings");
    const lit = game(run, "lit.json");
    t.ok(lit.checkpoint === 0 && (lit.litMask & 1) === 1, "touching lantern-0 lit it");
    t.equal(lit.hearts, 3, "and gave back the hearts");
    t.ok(value<{ intensity: number }>(run, "light.json").intensity > 0.5, "its 2D light is on");
    t.ok(lit.saves === 1 && lit.hasSave, "the store took the save");
    const calls = run.json<{ calls: Array<{ method: string; args: { key?: string; value?: { name?: string; chimeMask?: number; glimmers?: number } } }> }>("calls.json").calls;
    const set = calls.find((c) => c.method === "store.set" && c.args.key === "save");
    t.ok(set !== undefined && set.args.value?.name === "Mira" && set.args.value?.chimeMask === 1 && set.args.value?.glimmers === 7, "the save names its slot and holds the chime and glimmers");
    const quit = game(run, "quit.json");
    t.ok(quit.phase === TITLE && quit.saves >= 2, `Save and quit wrote again and went to the title (${quit.saves} saves)`);

    const next = run.then[0]!;
    t.equal(next.warnings.join("; "), "", "the relaunch ran without warnings");
    t.ok(game(next, "at-title.json").hasSave, "the relaunch read the save on its first tick");
    const title = elements(next, "title.json");
    const cont = title.find((e) => e.id === "continue");
    t.ok(cont !== undefined && !cont.states.includes("disabled"), "Continue is offered");
    t.ok(title.some((e) => e.text === "Mira: 1 chime kept"), "the title names the slot");
    const c = game(next, "continued.json");
    t.equal(c.phase, PLAY, "Continue started the run");
    t.equal([c.chimes, c.glimmers, c.checkpoint, c.loads], [1, 7, 0, 1], "with the chime, the glimmers and the lantern");
    t.ok(Math.abs(c.heroX - 27.5) < 0.6 && Math.abs(c.heroY - 16) < 0.2, `by the lantern (${c.heroX.toFixed(2)}, ${c.heroY.toFixed(2)})`);
    t.equal(value<{ name: string }>(next, "prefs.json").name, "Mira", "the bell-keeper's name came back with it (Prefs.name, a str field)");
    t.ok(value<{ intensity: number }>(next, "light.json").intensity > 0.5, "the lantern is lit again");
    t.ok(elements(next, "play.json").some((e) => e.text === "Welcome back, Mira"), "a toast welcomes Mira back");
  },
});
