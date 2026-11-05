// The starship screens on a phone in portrait (390×844 CSS px at 3×, a
// 47 px notch and a 34 px home indicator) in scifi, fantasy and kawaii, then
// right to left, in Japanese and at 150 % text: each frame compared with
// crates/og-ui/references/<tier>/kit-starship-phone-<file>.
import { checkFrames } from "./frames.ts";
import { starshipScript } from "./starship.ts";
import { playtest, ui } from "./lib.ts";

const script = starshipScript(true);

export default playtest({
  name: "the starship screens in scifi, fantasy and kawaii on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const e of ui(run, "scifi-hangar.json").filter((x) => x.role === "button" && !x.clipped)) {
      t.ok(e.width >= 43.5 && e.height >= 43.5, `${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
    }
    t.ok(ui(run, "scifi-turnOrder.json").some((e) => e.role === "listitem" && e.name === "Valiant, acting now" && e.states.includes("selected")), "the acting ship heads the order, selected");
    t.ok(ui(run, "scifi-abilityCards.json").some((e) => e.id === "EndTurn" && e.role === "button"), "End turn is there");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-starship-phone-");
  },
});
