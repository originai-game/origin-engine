// Pause, the bag and the settings: Escape holds the run (time scale 0,
// the clock stops, W moves nothing); the pause menu opens the bag, the rpg
// kit's inventory of the party's things and its journal of the quest;
// Escape closes it; Settings switches every string to 简体中文 and moves
// the music, Done saves them through og.platform, and Resume gives the run
// back, in the chosen language.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, type PlatformDump, position, press, script, tap, texts } from "./lib.ts";

export default playtest({
  name: "Escape pauses, the bag shows the party's things and the journal, settings switch the language and save",
  commands: script(
    "0.3:run",
    "0.4:quest:clear",
    press("Escape", 0.8),
    "1.0:get:Game:paused.json",
    "1.0:get:og.TimeScale.scale:scale.json",
    "1.0:get:hero.og.Transform:before.json",
    "1.05:key_down:KeyW",
    "1.6:key_up:KeyW",
    "1.7:get:Game:held.json",
    "1.7:get:hero.og.Transform:after.json",
    tap("Bag", 1.8),
    "2.3:get:Game:bag-open.json",
    "2.3:dump_ui:bag.json",
    press("Escape", 2.4),
    "2.7:get:Game:closed.json",
    tap("Settings", 2.9),
    "3.4:move_to_text:简体中文",
    "3.42:touch_start",
    "3.45:touch_end",
    "3.7:touch_move_to:Music/Slider",
    "3.72:touch_start",
    "3.75:touch_end",
    tap("Done", 4.0),
    "4.5:dump_platform:platform.json",
    tap("Resume", 4.6),
    "5.2:dump_ui:play-zh.json",
    "5.3:get:Game:resumed.json",
    "5.4:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const paused = game(run, "paused.json");
    t.equal(paused.paused, true, "Escape paused the run");
    t.equal(run.json<{ value: number }>("scale.json").value, 0, "the run holds: time scale 0");
    const held = game(run, "held.json");
    t.equal(held.time, paused.time, "the clock stopped");
    const [bx, , bz] = position(run, "before.json");
    const [ax, , az] = position(run, "after.json");
    t.ok(Math.hypot(ax - bx, az - bz) < 1e-3, "W while paused moves nothing");
    t.equal(game(run, "bag-open.json").bagOpen, true, "Bag and journal opened the bag");
    const bag = texts(run, "bag.json");
    const parts = elements(run, "bag.json");
    t.ok(parts.some((e) => e.id === "Grid") && parts.some((e) => e.id === "Log"), "the rpg kit's inventory grid and quest log");
    t.ok(bag.includes("2"), "the party's two draughts in the grid");
    t.ok(bag.includes("The Warden's Seal"), "the journal holds the quest");
    t.equal(game(run, "closed.json").bagOpen, false, "Escape closed the bag");
    // The settings are this device's (src/device.ts), never the world's: what Done saved through og.platform.
    const p = run.json<PlatformDump>("platform.json");
    const saves = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    t.equal(saves.length, 1, "Done saved once");
    const saved = (saves[0]?.args["value"] ?? {}) as { language?: string; musicVolume?: number };
    t.equal(saved.language, "zh-CN", "the settings went to og.platform with 简体中文");
    t.ok((saved.musicVolume ?? 1) < 0.7, `the music slider moved (${(saved.musicVolume ?? 1).toFixed(2)})`);
    t.ok(texts(run, "play-zh.json").includes("奥德里克"), "the HUD speaks 简体中文");
    const resumed = game(run, "resumed.json");
    t.equal(resumed.paused, false, "Resume closed the pause menu");
    t.ok(resumed.time > held.time + 0.3, "and the clock runs again");
  },
});
