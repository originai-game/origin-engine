// The onboarding overlay: the spotlight's hole sits on each step's target,
// presses on the dimmed parts stop there, Next walks the steps and Done
// finishes.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, tap, ui } from "./lib.ts";

export default playtest({
  name: "the tutorial spotlights each target and Next walks to the end",
  commands: [
    "0.2:show:tutorial",
    "0.8:dump_ui:step0.json",
    "0.9:touch_move:20,20",
    "0.95:touch_start",
    "1.0:touch_end",
    "1.1:dump_entity:demo:blocked.json",
    tap(1.2, "Tutorial.callout/Actions/Next"),
    "1.6:dump_ui:step1.json",
    tap(1.7, "Tutorial.callout/Actions/Next"),
    tap(2.0, "Tutorial.callout/Actions/Next"),
    "2.4:dump_entity:demo:done.json",
    "2.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const at = (els: ReturnType<typeof ui>, id: string) => els.find((e) => e.id === id);
    const s0 = ui(run, "step0.json");
    const hole = at(s0, "Hole");
    const play = at(s0, "play");
    t.ok(hole !== undefined && play !== undefined && hole.x <= play.x && hole.x + hole.width >= play.x + play.width && hole.y <= play.y && hole.y + hole.height >= play.y + play.height, "step 1's hole contains the Play button");
    const blocked = demo(run, "blocked.json");
    t.equal([blocked.screen, blocked.step], [SCREEN.tutorial, 0], "a press on the dimmed corner changed nothing");
    const s1 = ui(run, "step1.json");
    const hole1 = at(s1, "Hole");
    const settings = at(s1, "settings");
    t.ok(hole1 !== undefined && settings !== undefined && hole1.y <= settings.y && hole1.y + hole1.height >= settings.y + settings.height, "step 2's hole moved to Settings");
    t.equal(demo(run, "done.json").screen, SCREEN.gallery, "Done on the last step finished the tutorial");
  },
});
