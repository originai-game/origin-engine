// The story screens (og-ui §2): New journey opens the abbess's words on
// og.ui.cutscene (letterbox, the speaker, the line typed out; the world held
// at time scale 0), walked line by line with Enter (a press shows the whole
// line, the next press moves on). Then, on a first journey, the tutorial
// (og.ui.tutorialOverlay) points at the HUD's strength, sigils, clock and
// bag, one step at a time; Next walks it, the last step's Done ends it, the
// progress records it (og.platform's store), the clock starts, and a second
// journey goes straight to the stair.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, press, script, tap, texts } from "./lib.ts";

const ENTERS = [1.0, 1.35, 1.7, 2.05, 2.4, 2.75, 3.1, 3.45];
const NEXTS = [4.2, 4.6, 5.0, 5.4];

export default playtest({
  name: "the intro cutscene walks with Enter, the tutorial spotlights the HUD step by step, and the progress keeps it seen",
  commands: script(
    "0.5:play",
    "0.95:dump_ui:line1.json",
    "0.95:get:Game:held.json",
    "0.95:get:og.TimeScale:scale.json",
    "1.2:screenshot:intro.png",
    ...ENTERS.map((t) => press("Enter", t)),
    "3.9:get:Game:after-intro.json",
    "3.95:dump_ui:step1.json",
    ...NEXTS.map((t) => tap("Next", t)),
    "5.8:get:Game:done.json",
    "6.8:get:Game:running.json",
    "6.9:dump_platform:calls.json",
    "7.0:play",
    "7.3:get:Game:second.json",
    "7.4:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const line1 = texts(run, "line1.json");
    t.ok(line1.includes("Abbess Maren") && line1.includes("Skip"), "the cutscene shows the speaker and Skip");
    t.ok(game(run, "held.json").held && run.json<{ value: { scale: number } }>("scale.json").value.scale === 0, "the world is held under it");
    const after = game(run, "after-intro.json");
    t.equal([after.scene, after.tutorial, after.tutorialStep], [0, true, 0], "eight presses of Enter walked the four lines; the tutorial opened");
    const step1 = elements(run, "step1.json");
    t.ok(step1.some((e) => e.text === "1 of 4") && step1.some((e) => e.text === "Your strength"), "its first step is the strength meter's");
    t.ok(step1.some((e) => e.id === "Health"), "with the HUD it points at drawn beneath");
    const done = game(run, "done.json");
    t.equal([done.tutorial, done.tutorialDone], [false, true], "Next walked the four steps and Done ended it");
    t.ok(game(run, "running.json").time > 0.5 && !game(run, "running.json").held, "the clock runs");
    const calls = run.json<{ calls: Array<{ method: string; args: { key?: string; value?: { tutorial?: boolean } } }> }>("calls.json").calls;
    t.ok(calls.some((c) => c.method === "store.set" && c.args.key === "progress" && c.args.value?.tutorial === true), "the progress records the tutorial as seen");
    const second = game(run, "second.json");
    t.equal(second.scene, 1, "a second journey opens with the abbess again");
    t.ok(!second.tutorial, "but not the tutorial");
  },
});
