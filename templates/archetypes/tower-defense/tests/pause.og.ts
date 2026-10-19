// Pause and settings: Escape holds the garden (time scale 0, the visitors
// stop), the pause menu opens Settings, 简体中文 switches every string, the
// music slider moves, Done saves the settings through og.platform, and
// Resume lets the garden go again in the chosen language.
import { playtest } from "@origingame/engine/playtest";
import { component, game, type PlatformDump, play, press, tap } from "./lib.ts";

export default playtest({
  name: "Escape pauses, settings switch the language and save, Resume runs the garden again",
  commands: [
    play(0.3),
    "4.4:dump_entity:visitor-0:moving.json",
    press("Escape", 4.5),
    "4.7:get:Game:paused.json",
    "4.7:get:og.TimeScale.scale:scale.json",
    "4.7:dump_entity:visitor-0:held-a.json",
    "5.2:dump_entity:visitor-0:held-b.json",
    "5.3:screenshot:pause.png",
    tap("settings", 5.4),
    "6.0:move_to_text:简体中文",
    "6.02:touch_start",
    "6.05:touch_end",
    "6.3:touch_move_to:Music/Slider",
    "6.32:touch_start",
    "6.35:touch_end",
    "6.8:screenshot:settings-zh.png",
    tap("Done", 7.0),
    "7.5:get:Prefs:prefs.json",
    "7.5:dump_platform:platform.json",
    "7.6:screenshot:pause-zh.png",
    tap("resume", 7.8),
    "8.4:get:Game:resumed.json",
    "8.4:dump_entity:visitor-0:again-a.json",
    "8.6:dump_entity:visitor-0:again-b.json",
    "8.7:screenshot:play-zh.png",
    "8.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const pos = (f: string) => component<{ position: number[] }>(run, f, "og.Transform").position;
    t.equal(game(run, "paused.json").paused, true, "Escape paused the round");
    t.equal(run.json<{ value: number }>("scale.json").value, 0, "the garden holds: time scale 0");
    t.equal(pos("held-b.json")[1], pos("held-a.json")[1], "the visitor stands still while paused");
    const prefs = run.json<{ value: { language: number; music: number; saves: number } }>("prefs.json").value;
    t.equal(prefs.language, 1, "简体中文 was chosen");
    t.ok(prefs.music < 0.7, `the music slider moved (${prefs.music.toFixed(2)})`);
    t.equal(prefs.saves, 1, "Done saved once");
    const p = run.json<PlatformDump>("platform.json");
    const saved = p.calls.find((c) => c.method === "store.set" && c.args["key"] === "og.settings");
    t.equal((saved?.args["value"] as { language?: string } | undefined)?.language, "zh-CN", "the settings went to og.platform with the language");
    const resumed = game(run, "resumed.json");
    t.equal(resumed.paused, false, "Resume closed the pause menu");
    t.ok(pos("again-b.json")[1]! < pos("again-a.json")[1]!, "the visitor walks again");
    for (const f of ["pause.png", "settings-zh.png", "pause-zh.png", "play-zh.png"]) t.equal(run.png(f).width, 1280, `${f} captured`);
    t.ok(pos("moving.json")[1]! < 12.5, "a visitor was on the lawn before the pause");
  },
});
