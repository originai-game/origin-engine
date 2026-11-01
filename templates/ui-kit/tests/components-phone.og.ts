// The component library's previews on a phone in portrait (390×844 CSS px
// at 3×, a 47 px notch and a 34 px home indicator), driven by touch only,
// then again at 200 % text (text200-*): each frame compared with
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
    // Read before the page scrolls on to the breadcrumbs (a row scrolled out of view is not listed).
    t.ok(ui(run, "navigation-tab.json").some((e) => e.role === "tab" && e.name === "Skills, 3" && e.states.includes("selected")), "a tap picked the second tab");
    t.ok(!nav.some((e) => e.id === "Prev" && e.parent === "Bar"), "no bumper hints under a finger");
    // Every target a finger can reach: one a scroll container cuts at its edge scrolls into view whole.
    for (const e of nav.filter((x) => (x.role === "button" || x.role === "tab" || x.role === "link") && !x.clipped)) {
      t.ok(e.width >= 43.5 && e.height >= 43.5, `${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
    }
    const tabs = (f: string) => ui(run, f).filter((e) => e.role === "tab").map((e) => e.height);
    t.ok(Math.min(...tabs("navigation-200.json")) > Math.max(...tabs("navigation.json")) * 1.2, "at 200 % text the tabs grow with their labels");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-components-phone-");
  },
});
