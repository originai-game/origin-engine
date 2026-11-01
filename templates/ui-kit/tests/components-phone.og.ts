// The component library's previews on a phone in portrait (390×844 CSS px
// at 3×, a 47 px notch and a 34 px home indicator), driven by touch only:
// each frame compared with
// crates/og-ui/references/<tier>/kit-components-phone-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { componentsScript } from "./components.ts";
import { ui } from "./lib.ts";

const script = componentsScript(true);

export default playtest({
  name: "the component previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const nav = ui(run, "navigation.json");
    t.ok(nav.some((e) => e.role === "tab" && e.name === "Skills, 3" && e.states.includes("selected")), "a tap picked the second tab");
    t.ok(!nav.some((e) => e.id === "Prev" && e.parent === "Bar"), "no bumper hints under a finger");
    for (const e of nav.filter((x) => x.role === "button" || x.role === "tab" || x.role === "link")) {
      t.ok(e.width >= 43.5 && e.height >= 43.5, `${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
    }
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-components-phone-");
  },
});
