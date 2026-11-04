// The story pattern previews at desktop size (1280×720 CSS px at 1×), fed
// by og.story: the conversation typing, its choices (a check with its odds,
// a passed insight, a toll that says why not, the timer), a visited choice,
// the d20's roll, the backlog, a voiced line; the visual novel's box with
// the narrator, Auto and its choices; the quest log's tabs, states and
// Track, and the HUD's tracker; then right to left, in Japanese, Korean
// and Chinese, and at 150 % text, each frame compared with
// crates/og-ui/references/<tier>/kit-story-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { storyScript } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = storyScript(false);

export default playtest({
  name: "the story pattern previews in their states at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const choices = ui(run, "dialogue-choices.json");
    t.ok(choices.some((e) => e.role === "status" && e.name === "Mira: The gate opens at dawn. Are you ready to cross?"), "Mira's line is read whole, with her name");
    t.ok(choices.some((e) => e.role === "button" && e.name.startsWith("2, Persuasion · 60 %, Tell me what waits beyond first.")), "the check says its stat and odds (d20 + 3 against 12: 60 %)");
    t.ok(choices.some((e) => e.role === "button" && e.name.startsWith("5, Pay the warden's toll., Needs 60 more credits") && e.states.includes("disabled")), "the toll the purse cannot pay is disabled and says why");
    t.ok(choices.some((e) => e.role === "timer"), "the choice timer is a timer");
    t.ok(ui(run, "dialogue-roll.json").some((e) => e.role === "status" && /^Persuasion: \d+, (Success|Failure)$/.test(e.name)), "the roll says its stat, its number and its result");
    t.ok(ui(run, "dialogue-log.json").some((e) => e.role === "listitem" && e.name.includes("→ Who else knows about the gate?")), "the backlog shows the reply chosen on Mira's line");
    t.ok(ui(run, "vn-line.json").some((e) => e.role === "status" && e.name.startsWith("Aria: There.")), "a tap went on from the narrator to Aria");
    const quests = ui(run, "quests.json");
    t.ok(quests.some((e) => e.role === "option" && e.name.startsWith("The Dawn Gate") && e.name.includes("Tracked")), "the tracked main quest is said tracked");
    t.ok(quests.some((e) => e.role === "listitem" && e.name.startsWith("Recover the star charts, 2 of 3")), "an objective says its count");
    t.ok(quests.some((e) => e.role === "listitem" && e.name === "???"), "a hidden objective is ???");
    t.ok(ui(run, "quests-tracked.json").some((e) => e.role === "option" && e.name.startsWith("Supply Run") && e.name.includes("Tracked")), "Track tracked the daily");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-story-");
  },
});
