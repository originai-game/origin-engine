// Pause and the settings, by keys and touch. Escape pauses the run (the time
// scale held at 0) and the pause menu shows; Settings opens the dialog over
// it: the Music slider pressed at its middle sets the music bus to a half,
// the language radio turns the dialog Chinese (its label mixing Latin and
// Chinese draws both), Reduce motion turns on; every control the
// accessibility tree reaches has a name; Done saves the settings through
// og.platform and returns to the pause menu; Resume lets the world move.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, press, script, tap, texts, value } from "./lib.ts";

export default playtest({
  name: "Escape pauses; the settings: a slider, Chinese, reduced motion, a named accessibility tree, saved; Resume",
  commands: script(
    "0.3:play",
    press("Escape", 0.8),
    "1.2:get:Game:paused.json",
    "1.2:get:og.TimeScale:held.json",
    "1.2:dump_ui:pause.json",
    tap("settings", 1.3),
    "1.9:dump_ui:settings.json",
    tap("Music/Slider", 2.0),
    "2.2:get:Prefs:slider.json",
    "2.2:get:og.Audio:audio.json",
    tap("Option#1", 2.3),
    "2.7:dump_ui:chinese.json",
    tap("Motion", 2.8),
    "3.0:get:Prefs:motion.json",
    "3.0:get:Game:a11y.json",
    tap("Done", 3.1),
    "3.6:get:Game:closed.json",
    "3.6:get:Prefs:saved.json",
    "3.6:dump_platform:calls.json",
    tap("resume", 3.7),
    "4.2:get:Game:resumed.json",
    "4.2:get:og.TimeScale:moving.json",
    "4.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const paused = game(run, "paused.json");
    t.ok(paused.paused && paused.held, "Escape paused the run");
    t.equal(value<{ scale: number }>(run, "held.json").scale, 0, "the time scale is held at 0");
    t.ok(texts(run, "pause.json").includes("Resume"), "the pause menu shows");
    const about = elements(run, "settings.json").find((e) => e.id === "About");
    t.equal(about?.text, "Hollowbell · 空心钟 · a bell-keeper's descent", "the settings carry a label mixing Latin and Chinese");
    const slider = value<{ music: number }>(run, "slider.json");
    t.ok(Math.abs(slider.music - 0.5) < 0.06, `the Music slider pressed at its middle set a half (${slider.music.toFixed(2)})`);
    t.ok(Math.abs(value<{ music: number }>(run, "audio.json").music - slider.music) < 1e-6, "and the music bus follows it");
    const zh = texts(run, "chinese.json");
    t.ok(zh.includes("设置") && zh.includes("音乐") && zh.includes("完成"), "Chinese turned the dialog into Chinese");
    t.ok(value<{ reducedMotion: boolean }>(run, "motion.json").reducedMotion, "Reduce motion is on");
    const a11y = game(run, "a11y.json");
    t.ok(a11y.a11yNodes > 8 && a11y.a11yUnnamed === 0, `every focusable node of og.ui.a11y has a name (${a11y.a11yNodes} nodes)`);
    t.ok(!game(run, "closed.json").settingsOpen && game(run, "closed.json").paused, "Done closed the dialog back to the pause menu");
    const saved = value<{ saves: number; language: number }>(run, "saved.json");
    t.ok(saved.saves === 1 && saved.language === 1, "Done saved the settings: Chinese");
    const calls = run.json<{ calls: Array<{ method: string; args: Record<string, unknown> }> }>("calls.json").calls;
    const save = calls.find((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    t.ok(save !== undefined && JSON.stringify(save.args).includes("zh-CN") && JSON.stringify(save.args).includes('"reducedMotion":true'), "through og.platform's settings");
    t.ok(!game(run, "resumed.json").paused, "Resume");
    t.equal(value<{ scale: number }>(run, "moving.json").scale, 1, "and the world moves again");
  },
});
