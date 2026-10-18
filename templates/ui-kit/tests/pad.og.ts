// A gamepad (queued intents, as the shell maps a pad's d-pad, A and B): down
// four times focuses the fourth row, A opens it, B closes it again; in the
// widget screen the d-pad reaches the select and A opens its menu.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, ui } from "./lib.ts";

export default playtest({
  name: "gamepad navigation moves through the gallery, opens a screen and backs out",
  commands: [
    "0.5:pad:down",
    "0.6:pad:down",
    "0.7:pad:down",
    "0.8:pad:down",
    "0.9:pad:activate",
    "1.3:dump_entity:demo:opened.json",
    "1.4:pad:back",
    "1.9:dump_entity:demo:closed.json",
    "2.0:show:widgets",
    "2.4:pad:next",
    "2.5:pad:next",
    "2.6:pad:next",
    "2.7:pad:next",
    "2.75:pad:next",
    "2.8:pad:activate",
    "3.0:dump_ui:menu-ui.json",
    "3.1:pad:down",
    "3.2:pad:activate",
    "3.5:dump_entity:demo:picked.json",
    "3.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "opened.json").screen, SCREEN.settings, "four downs and A opened the fourth row (settings)");
    t.equal(demo(run, "closed.json").screen, SCREEN.gallery, "B closed it");
    t.ok(ui(run, "menu-ui.json").some((e) => e.parent === "Difficulty.menu" && e.text === "Hard"), "Tab order reached the select and A opened its menu");
    t.equal(demo(run, "picked.json").difficulty, 2, "down and A chose Hard");
  },
});
