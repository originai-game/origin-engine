// Pause and the settings, by touch and keys: Escape pauses the run (the
// time scale held at 0), Settings opens the dialog over it. The Music
// slider pressed at its middle sets the music bus to a half; the
// lamplighter's name is typed (a Latin part, then a Chinese part through an
// input method's composition and commit); the label mixing Latin and
// Chinese draws both; Chinese turns the whole dialog into Chinese; Done
// saves the settings through og.platform and the results name the
// lamplighter. Every control the accessibility tree reaches has a name.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, PLAY, press, tap, texts, value } from "./lib.ts";

export default playtest({
  name: "pause, then the settings: a slider, a typed name with an input method, a mixed-script label, Chinese, saved",
  commands: [
    "0.3:run",
    press("Escape", 0.6),
    "1.2:get:Game:paused.json",
    "1.2:get:og.TimeScale:held.json",
    "1.2:dump_ui:pause.json",
    tap("settings", 1.3),
    "2.0:dump_ui:settings.json",
    tap("Music/Slider", 2.1),
    "2.3:get:Prefs:slider.json",
    "2.3:get:og.Audio:audio.json",
    tap("Name", 2.4),
    "2.6:type:Mei",
    "2.8:compose:xiao deng",
    "2.9:compose:小灯",
    "3.0:commit:小灯",
    "3.2:dump_ui:typed.json",
    "3.2:get:Game:a11y.json",
    "3.3:screenshot:settings.png",
    tap("Option#1", 3.4),
    "3.9:dump_ui:chinese.json",
    "4.0:screenshot:settings-zh.png",
    tap("Done", 4.1),
    "4.6:get:Prefs:saved.json",
    "4.6:dump_platform:platform.json",
    "4.6:get:Game:closed.json",
    "4.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const paused = game(run, "paused.json");
    t.ok(paused.phase === PLAY && paused.paused, "Escape paused the run");
    t.equal(value<{ scale: number }>(run, "held.json").scale, 0, "the time scale is held at 0");
    t.ok(texts(run, "pause.json").includes("Resume"), "the pause menu shows");
    const about = elements(run, "settings.json").find((e) => e.id === "About");
    t.equal(about?.text, "Lantern Vale · 灯笼谷 · a lamplighter's dusk", "the settings carry a label mixing Latin and Chinese");
    const slider = value<{ music: number }>(run, "slider.json");
    t.ok(Math.abs(slider.music - 0.5) < 0.06, `the Music slider pressed at its middle set a half (${slider.music.toFixed(2)})`);
    t.ok(Math.abs(value<{ music: number }>(run, "audio.json").music - slider.music) < 1e-6, "and the music bus follows it");
    const name = elements(run, "typed.json").find((e) => e.id === "Name");
    t.equal(name?.text, "Mei小灯", "the name was typed, its Chinese part through an input method");
    const a11y = game(run, "a11y.json");
    t.ok(a11y.a11yNodes > 10 && a11y.a11yUnnamed === 0, `every focusable node of og.ui.a11y has a name (${a11y.a11yNodes} nodes)`);
    const zh = texts(run, "chinese.json");
    t.ok(zh.includes("设置") && zh.includes("音乐") && zh.includes("完成"), "Chinese turned the dialog into Chinese");
    const saved = value<{ saves: number; language: number; name: string }>(run, "saved.json");
    t.ok(saved.saves === 1 && saved.language === 1, "Done saved the settings: Chinese");
    t.equal(saved.name, "Mei小灯", "the name is world state: Prefs.name, a str field");
    const calls = run.json<{ calls: Array<{ method: string; args: Record<string, unknown> }> }>("platform.json").calls;
    const save = calls.find((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    t.ok(save !== undefined && JSON.stringify(save.args).includes("Mei小灯"), "through og.platform, the name with them");
    const closed = game(run, "closed.json");
    t.ok(!closed.settingsOpen && closed.paused, "the dialog closed back to the pause menu");
  },
});
