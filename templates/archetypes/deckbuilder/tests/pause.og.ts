// Pause and settings: Escape holds the fight (time scale 0, the play key
// plays nothing), the pause menu opens Settings, 简体中文 switches every
// string, the music slider moves, Done saves the settings through
// og.platform, and Resume gives the turn back, in the chosen language.
import { playtest } from "@origingame/engine/playtest";
import { game, type PlatformDump, press, script, tap, texts } from "./lib.ts";

export default playtest({
  name: "Escape pauses, settings switch the language and save, Resume gives the turn back",
  commands: script([
    "0.3:goto:0",
    press("Escape", 0.7),
    "0.9:get:Game:paused.json",
    "0.9:get:og.TimeScale.scale:scale.json",
    press("KeyE", 1.0),
    "1.2:get:Game:held.json",
    "1.3:screenshot:pause.png",
    tap("Settings", 1.4),
    "2.0:move_to_text:简体中文",
    "2.02:touch_start",
    "2.05:touch_end",
    "2.3:touch_move_to:Music/Slider",
    "2.32:touch_start",
    "2.35:touch_end",
    "2.8:screenshot:settings-zh.png",
    tap("Done", 3.0),
    "3.5:dump_platform:platform.json",
    tap("Resume", 3.6),
    "4.3:dump_ui:play-zh.json",
    press("KeyE", 4.4),
    "4.7:get:Game:resumed.json",
    "4.8:screenshot:play-zh.png",
    "4.9:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "paused.json").paused, true, "Escape paused the fight");
    t.equal(run.json<{ value: number }>("scale.json").value, 0, "the fight holds: time scale 0");
    t.equal(game(run, "held.json").played, 0, "the play key while paused plays nothing");
    const p = run.json<PlatformDump>("platform.json");
    const saves = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    const prefs = (saves[0]?.args["value"] ?? {}) as { language?: string; musicVolume?: number };
    t.equal(saves.length, 1, "Done saved once");
    t.equal(prefs.language, "zh-CN", "the settings went to og.platform with the language");
    t.ok((prefs.musicVolume ?? 1) < 0.7, `the music slider moved (${(prefs.musicVolume ?? 1).toFixed(2)})`);
    t.ok(texts(run, "play-zh.json").includes("结束回合"), "the HUD speaks 简体中文 (End turn: 结束回合)");
    const resumed = game(run, "resumed.json");
    t.equal(resumed.paused, false, "Resume closed the pause menu");
    t.equal(resumed.played, 1, "and the play key plays again");
    t.frame(["pause.png", "settings-zh.png", "play-zh.png"], () => {
      for (const f of ["pause.png", "settings-zh.png", "play-zh.png"]) t.ok(run.png(f).width > 0, `${f} captured`);
    });
  },
});
