// The keyboard: Tab and Enter open a screen from the gallery; on the
// settings tree (its Audio page) Tab walks the page tabs to the Master
// slider, Left steps it down twice, the change waits as pending (the row
// marked changed, Apply and Revert enabled), and Escape closes the screen.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, key, ui } from "./lib.ts";

/** Tab presses from the screen's start to the Master slider: the Gallery button, the six page tabs, then the slider. */
const TABS = 8;

const tabs: string[] = [];
for (let k = 0; k < TABS; k++) tabs.push(key(2.0 + k * 0.15, "Tab"));

export default playtest({
  name: "keyboard focus opens a screen, steps a slider on the settings tree to a pending change and closes it",
  commands: [
    key(0.5, "Tab"),
    key(0.7, "Tab"),
    key(0.9, "Enter"),
    "1.3:dump_entity:demo:opened.json",
    "1.35:pat:set:page:audio",
    "1.4:show:settingsTree",
    ...tabs,
    key(3.3, "ArrowLeft"),
    key(3.5, "ArrowLeft"),
    "3.7:dump_ui:settings-ui.json",
    key(3.8, "Escape"),
    "4.2:dump_entity:demo:closed.json",
    "4.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "opened.json").screen, SCREEN.shooterHud, "Tab, Tab, Enter pressed the second row (the shooter showcase, on its HUD)");
    const els = ui(run, "settings-ui.json");
    const at = (path: string) => els.find((e) => e.path === path);
    const rows = "Settings/Body/Tree/Body/Page/Rows";
    t.equal(els.filter((e) => e.states.includes("focused")).map((e) => e.path), [`${rows}/master/Control/Slider/Track`], "Tab reached the Master slider");
    t.equal(at(`${rows}/master/Control/Slider/Head/Value`)?.text, "90%", "two Lefts stepped Master from 100 % down by 5 % twice");
    t.ok(at(`${rows}/master/Words/Name/Changed`) !== undefined, "the Master row is marked changed (pending)");
    for (const b of ["Apply", "Revert"]) {
      const e = at(`Settings/Body/Tree/Foot/${b}`);
      t.ok(e !== undefined && !e.states.includes("disabled"), `${b} is enabled by the pending change`);
    }
    t.equal(demo(run, "closed.json").screen, SCREEN.gallery, "Escape closed the settings");
  },
});
