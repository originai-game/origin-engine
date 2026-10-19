// Pause and settings: Escape holds the orb (time scale 0, the embers make
// nothing); Settings switch to 简体中文 and move the music; Done saves the
// settings through og.platform; Resume lets the orb make sparks again; the
// numbers now read in the Chinese units (万, 亿).
import { playtest } from "@origingame/engine/playtest";
import { game, type PlatformDump, press, tap, text } from "./lib.ts";

export default playtest({
  name: "Escape pauses, settings switch to 简体中文 and save, numbers read in 万 and 亿",
  commands: [
    "0.3:play",
    "0.9:own:0:4",
    "1.0:sparks:123456789",
    press("Escape", 1.2),
    "1.4:get:Game:paused.json",
    "1.4:get:og.TimeScale.scale:scale.json",
    "1.9:get:Game:held.json",
    "2.0:screenshot:pause.png",
    tap("settings", 2.1),
    "2.7:move_to_text:简体中文",
    "2.72:touch_start",
    "2.75:touch_end",
    "3.0:touch_move_to:Music/Slider",
    "3.02:touch_start",
    "3.05:touch_end",
    "3.4:screenshot:settings-zh.png",
    tap("Done", 3.6),
    "4.0:get:Prefs:prefs.json",
    "4.0:dump_platform:platform.json",
    tap("resume", 4.1),
    "4.8:dump_ui:play-zh.json",
    "4.8:get:Game:resumed.json",
    "4.85:screenshot:play-zh.png",
    "5.4:get:Game:running.json",
    "5.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const paused = game(run, "paused.json");
    t.equal(paused.paused, true, "Escape paused");
    t.equal(run.json<{ value: number }>("scale.json").value, 0, "the orb holds: time scale 0");
    t.equal(game(run, "held.json").sparks, paused.sparks, "the embers made nothing while paused");
    const prefs = run.json<{ value: { language: number; music: number; saves: number } }>("prefs.json").value;
    t.equal(prefs.language, 1, "简体中文 was chosen");
    t.ok(prefs.music < 0.7, `the music slider moved (${prefs.music.toFixed(2)})`);
    t.equal(prefs.saves, 1, "Done saved once");
    const p = run.json<PlatformDump>("platform.json");
    const saved = p.calls.find((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    t.equal((saved?.args["value"] as { language?: string } | undefined)?.language, "zh-CN", "the settings went to og.platform with the language");
    t.equal(text(run, "play-zh.json", "Amount", "Sparks"), "1.23亿", "123,456,789 reads 1.23亿");
    t.equal(text(run, "play-zh.json", "Rate"), "每秒 +2", "the rate reads 每秒 +2");
    t.equal(text(run, "play-zh.json", "Name", "Head"), "余烬", "the first card is 余烬");
    t.equal(game(run, "resumed.json").paused, false, "Resume closed the pause menu");
    t.ok(game(run, "running.json").sparks > game(run, "resumed.json").sparks, "and the embers make sparks again");
    for (const f of ["pause.png", "settings-zh.png", "play-zh.png"]) t.equal(run.png(f).width, 1280, `${f} captured`);
  },
});
