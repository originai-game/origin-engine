// Escape pauses: the world holds (the run's clock and the foes stop) and
// the pause menu shows. Its Settings opens the settings over it, where the
// language switches the whole interface; Done closes them. Escape again
// resumes, and Leave the arena goes back to the title.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, press, tap, texts, TITLE } from "./lib.ts";

export default playtest({
  name: "Escape pauses and holds the world, the settings and the language, resume, leave the arena",
  commands: [
    "0.3:run",
    press("Escape", 3.6),
    "3.8:get:Game:paused.json",
    "3.8:get:walker-0.og.Transform:w-a.json",
    "3.9:dump_ui:pause.json",
    tap("pause/Pause/Panel/Menu/Settings", 4.0),
    "4.3:dump_ui:settings.json",
    "4.4:language:zh-CN",
    "4.6:dump_ui:zh.json",
    "4.7:language:en",
    tap("Done", 4.9),
    "5.2:get:Game:still.json",
    "5.2:get:walker-0.og.Transform:w-b.json",
    // Every style's modal has closed by then (storybook's exit is the longest).
    "5.7:dump_ui:closed.json",
    press("Escape", 5.8),
    "6.4:get:Game:resumed.json",
    press("Escape", 6.5),
    tap("Quit", 6.8),
    "7.2:get:Game:left.json",
    "7.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const paused = game(run, "paused.json");
    t.ok(paused.paused && paused.phase === PLAY, "Escape pauses the run");
    t.ok(texts(run, "pause.json").includes("Paused") && texts(run, "pause.json").includes("Leave the arena"), "the pause menu shows");
    t.ok(texts(run, "settings.json").includes("Look speed") && texts(run, "settings.json").includes("Reduce motion"), "its Settings opens the settings");
    t.ok(texts(run, "zh.json").includes("设置") && texts(run, "zh.json").includes("完成"), "the language switches the interface to 简体中文");
    t.ok(!texts(run, "closed.json").includes("Look speed"), "Done closes the settings");
    const still = game(run, "still.json");
    const a = run.json<{ value: { position: number[] } }>("w-a.json").value.position;
    const b = run.json<{ value: { position: number[] } }>("w-b.json").value.position;
    t.ok(still.time === paused.time && Math.hypot(b[0]! - a[0]!, b[2]! - a[2]!) < 1e-4, `the world held while paused (the clock at ${still.time.toFixed(2)} s, the walker still)`);
    const resumed = game(run, "resumed.json");
    t.ok(!resumed.paused && resumed.time > still.time + 0.3, `Escape again resumes (${resumed.time.toFixed(2)} s)`);
    t.equal(game(run, "left.json").phase, TITLE, "Leave the arena goes back to the title");
  },
});
