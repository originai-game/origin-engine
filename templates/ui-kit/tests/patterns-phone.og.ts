// The genre pattern previews on a phone in portrait (390×844 CSS px at 3×,
// a 47 px notch and a 34 px home indicator), driven by touch only, then
// right to left and at 200 % text: each frame compared with
// crates/og-ui/references/<tier>/kit-patterns-phone-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { patternsScript } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = patternsScript(true);

export default playtest({
  name: "the genre pattern previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const equipped = ui(run, "gear-equipped.json");
    t.ok(equipped.some((e) => e.role === "button" && e.name.startsWith("Head, Crown of Thorns")), "a finger dragged the helm onto the head slot");
    for (const e of equipped.filter((x) => x.role === "button")) {
      t.ok(e.width >= 43.5 && e.height >= 43.5, `${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
    }
    t.ok(ui(run, "cards-aim.json").some((e) => e.role === "status" && e.name === "valid: Cultist"), "a finger dragged Bash's arc onto the cultist");
    t.ok(!ui(run, "cards-played.json").some((e) => e.role === "option" && e.name.startsWith("Bash")), "and played it");
    t.ok(!ui(run, "cards-tapped.json").some((e) => e.role === "option" && e.name.startsWith("Strike,")), "a tap picked Strike, a second aimed it and a tap on the cultist played it");
    t.ok(ui(run, "rewards-banish.json").some((e) => e.id === "Banish" && e.role === "button" && e.states.includes("selected")), "Banish armed is said selected");
    t.ok(ui(run, "rewards-selected.json").filter((e) => e.role === "checkbox" && e.states.includes("checked")).length === 2, "two rewards selected for a two-pick choice");
    t.ok(ui(run, "gacha-short.json").some((e) => e.role === "button" && e.name.includes("not enough")), "a ten-pull the purse cannot pay is said not enough");
    t.ok(ui(run, "gacha-reveal.json").some((e) => e.role === "status" && e.name.startsWith("Aria, Tidecaller, 5 stars, new")), "the fifth tap reached the 5★ reveal");
    t.ok(ui(run, "gacha-summary.json").filter((e) => e.role === "listitem").length === 10, "Skip went to a summary of all ten");
    t.ok(ui(run, "cards-refused.json").some((e) => e.id === "Note" && e.text === "Behind a taunt"), "a card let go on the taunted foe is refused there and the foe says why");
    t.ok(ui(run, "cards-unaffordable.json").some((e) => e.role === "option" && e.name.startsWith("Bash") && e.states.includes("disabled")), "with one energy, Bash (2) is said disabled");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-patterns-phone-");
  },
});
