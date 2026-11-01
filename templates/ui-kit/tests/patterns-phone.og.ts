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
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-patterns-phone-");
  },
});
