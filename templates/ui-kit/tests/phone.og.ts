// The action showcase's HUD on a phone in portrait (390×844 at 3×, a 47 px
// notch, a 34 px home indicator): the corners sit inside the safe area,
// every touch control is at least 44 CSS px, and Attack answers a touch
// (the combo counts the hit).
import { playtest } from "@origingame/engine/playtest";
import { tap, ui } from "./lib.ts";

/** The HUD's combo climbs by one every 120 ticks on its own (src/huds.ts actionHud). */
const COMBO_TICKS = 120;

export default playtest({
  name: "on a phone the HUD keeps to the safe area and its touch controls to 44 px",
  size: "1170x2532",
  commands: ["0.0:viewport:phone", "0.1:show:actionHud", "0.6:dump_ui:hud.json", tap(0.7, "Buttons/Attack"), "1.0:dump_ui:after.json", "1.1:quit"].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const els = ui(run, "hud.json");
    const find = (path: string) => els.find((e) => e.path === path);
    for (const path of ["Move/Dpad/Up", "Move/Dpad/Down", "Move/Dpad/Left", "Move/Dpad/Right", "Buttons/Jump", "Buttons/Attack", "Run/Top/Pause"]) {
      const e = find(path);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${path} is at least 44×44 (${e ? `${e.width}×${e.height}` : "missing"})`);
    }
    for (const path of ["Player", "Run"]) {
      const e = find(path);
      t.ok(e !== undefined && e.y >= 47, `the ${path} group is below the notch (y ${e?.y})`);
    }
    for (const path of ["Move", "Buttons"]) {
      const e = find(path);
      t.ok(e !== undefined && e.y + e.height <= 844 - 34, `the ${path} group ends above the home indicator (${e ? e.y + e.height : "missing"})`);
    }
    const combo = (file: string) => {
      const d = run.json<{ tick: number; elements: Array<{ path: string; text: string }> }>(file);
      return { count: Number(d.elements.find((e) => e.path === "ComboHud/Combo/Count")?.text ?? NaN), climbs: Math.floor(d.tick / COMBO_TICKS) };
    };
    const [before, after] = [combo("hud.json"), combo("after.json")];
    t.equal(after.count - before.count - (after.climbs - before.climbs), 1, `a tap on Attack counted one hit on the combo (${before.count} → ${after.count})`);
  },
});
