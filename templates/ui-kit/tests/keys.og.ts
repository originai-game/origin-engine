// The keyboard: Tab and Enter open a screen from the gallery; in the
// settings dialog focus starts on the tabs, Down reaches the Master slider,
// Left steps it down twice, Escape closes and the save hook runs once,
// writing the settings through og.platform.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, key } from "./lib.ts";

export default playtest({
  name: "keyboard focus opens a screen, steps a slider and closes the settings with a save",
  commands: [
    key(0.5, "Tab"),
    key(0.7, "Tab"),
    key(0.9, "Enter"),
    "1.3:dump_entity:demo:opened.json",
    "1.4:show:settings",
    key(2.0, "ArrowDown"),
    key(2.2, "ArrowLeft"),
    key(2.4, "ArrowLeft"),
    "2.7:dump_ui:settings-ui.json",
    key(2.8, "Escape"),
    "3.3:dump_entity:demo:saved.json",
    "3.3:dump_platform:platform.json",
    "3.4:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "opened.json").screen, SCREEN.mainMenu, "Tab, Tab, Enter pressed the second row (mainMenu)");
    const saved = demo(run, "saved.json");
    t.near(saved.masterVolume, 0.9, 1e-4, "two Lefts on the Master slider stepped 1 down by 0.05 twice");
    t.equal(saved.screen, SCREEN.gallery, "Escape closed the settings");
    t.equal(saved.saves, 1, "closing after a change ran the save hook once");
    const calls = run.json<{ calls: Array<{ method: string; args: Record<string, unknown> }> }>("platform.json").calls;
    const stored = calls.filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    t.equal(stored.length, 1, "the save hook wrote og.settings once");
    t.near((stored[0]?.args["value"] as { masterVolume?: number } | undefined)?.masterVolume ?? -1, 0.9, 1e-4, "with the stepped Master volume");
  },
});
