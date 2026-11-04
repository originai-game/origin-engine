// The roster and wider-game previews on a phone in portrait (390×844 CSS px
// at 3×, a 47 px notch and a 34 px home indicator), driven by touch only,
// then right to left, in Japanese and at 150 % text: each frame compared
// with crates/og-ui/references/<tier>/kit-genres-phone-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { genresScript } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = genresScript(true);

export default playtest({
  name: "the roster and wider-game previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of ["lobby.json", "select-locked.json", "map-detail.json", "songs.json"]) {
      for (const e of ui(run, f).filter((x) => x.role === "button" && !x.states.includes("disabled") && !x.clipped)) {
        t.ok(e.width >= 43.5 && e.height >= 43.5, `${f}: ${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
      }
    }
    t.ok(ui(run, "lobby-found.json").some((e) => e.text === "Match found"), "the ready check stands over the lobby");
    t.ok(ui(run, "map-detail.json").some((e) => e.role === "region" && e.name === "Amberhold"), "a tap on Amberhold's pin opened its detail");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres-phone-");
  },
});
