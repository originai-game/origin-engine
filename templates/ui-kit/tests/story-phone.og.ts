// The story pattern previews on a phone in portrait (390×844 CSS px at 3×,
// a 47 px notch and a 34 px home indicator), driven by touch only, then
// right to left, in Japanese, Korean and Chinese, and at 150 % text: each
// frame compared with crates/og-ui/references/<tier>/kit-story-phone-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { storyScript } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = storyScript(true);

export default playtest({
  name: "the story pattern previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const choices = ui(run, "dialogue-choices.json");
    for (const e of choices.filter((x) => x.role === "button" && !x.states.includes("disabled"))) {
      t.ok(e.width >= 43.5 && e.height >= 43.5, `${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
    }
    t.ok(ui(run, "dialogue-roll.json").some((e) => e.role === "status" && /^Persuasion: \d+, (Success|Failure)$/.test(e.name)), "a tap took the check and the d20 rolled");
    t.ok(ui(run, "quests-detail.json").some((e) => e.role === "region" && e.name === "The Dawn Gate"), "a tap on a quest opened it over the list");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-story-phone-");
  },
});
