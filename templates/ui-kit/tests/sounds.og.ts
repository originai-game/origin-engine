// UI sounds are on by default (og-taste §6: every input is answered by a
// sound): without `sounds:on`, a press in the gallery starts a voice on the
// ui bus and is heard; after `sounds:off` the same press is silent.
import { playtest } from "@origingame/engine/playtest";
import { tapText } from "./lib.ts";

type WorldDump = { archetypes: { components: string[]; count: number }[] };

export default playtest({
  name: "a press sounds on the ui bus by default, and not after sounds:off",
  size: "1280x720",
  commands: [
    "0.3:touch_move_to:Screens",
    "0.35:scroll:0,400",
    tapText(0.5, "=shop"),
    "0.7:listen:on.wav:0.3",
    "0.75:dump_world:on.json",
    "0.8:show:gallery",
    "0.9:sounds:off",
    "1.0:touch_move_to:Screens",
    "1.05:scroll:0,400",
    tapText(1.2, "=shop"),
    "1.4:listen:off.wav:0.3",
    "1.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const world = run.json<WorldDump>("on.json");
    const voices = world.archetypes.filter((a) => a.components.includes("og.AudioSource")).reduce((n, a) => n + a.count, 0);
    t.ok(voices >= 1, `the press started a voice with no sounds:on (${voices})`);
    t.ok(run.wav("on.wav").peak > 0.01, `and it is heard (peak ${run.wav("on.wav").peak})`);
    t.ok(run.wav("off.wav").peak < 0.001, `after sounds:off the press is silent (peak ${run.wav("off.wav").peak})`);
  },
});
