// Pause and settings: Escape holds the run (time scale 0, the clock stops,
// W moves nothing), the pause menu opens Settings, 简体中文 switches every
// string, the music slider moves, Done saves the settings through
// og.platform, and Resume gives the run back, in the chosen language.
import { playtest } from "@origingame/engine/playtest";
import { game, GATHER_ZH, type PlatformDump, position, press, tap, texts } from "./lib.ts";

export default playtest({
  name: "Escape pauses, settings switch the language and save, Resume gives the run back",
  size: "1280x720",
  commands: [
    "0.3:run",
    press("Escape", 0.8),
    "1.0:get:Game:paused.json",
    "1.0:get:og.TimeScale.scale:scale.json",
    "1.0:get:hero.og.Transform:before.json",
    "1.05:key_down:KeyW",
    "1.6:key_up:KeyW",
    "1.7:get:Game:held.json",
    "1.7:get:hero.og.Transform:after.json",
    "1.75:screenshot:pause.png",
    tap("Settings", 1.8),
    "2.4:move_to_text:简体中文",
    "2.42:touch_start",
    "2.45:touch_end",
    "2.7:touch_move_to:Music/Slider",
    "2.72:touch_start",
    "2.75:touch_end",
    "3.2:screenshot:settings-zh.png",
    tap("Done", 3.4),
    "3.9:dump_platform:platform.json",
    tap("Resume", 4.0),
    "4.3:warp:tree",
    "4.8:dump_ui:play-zh.json",
    "5.0:get:Game:resumed.json",
    "5.1:screenshot:play-zh.png",
    "5.2:quit",
  ].join(";"),
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
    const p = run.json<PlatformDump>("platform.json");
    const saved = p.calls.filter((c) => c.method === "settings.save" || (c.method === "store.set" && c.args["key"] === "og.settings"));
    t.equal(saved.length, 1, "Done saved the settings once through og.platform");
    const value = (saved[0]?.args["value"] ?? saved[0]?.args) as { language?: string; musicVolume?: number } | undefined;
    t.equal(value?.language, "zh-CN", "with 简体中文 chosen");
    t.ok((value?.musicVolume ?? 1) < 0.7, `and the music slider moved (${value?.musicVolume})`);
    t.ok(texts(run, "play-zh.json").includes(`按住 ${GATHER_ZH} 砍倒这棵树`), "the HUD speaks 简体中文");
    const resumed = game(run, "resumed.json");
    t.equal(resumed.paused, false, "Resume closed the pause menu");
    t.ok(resumed.time > held.time + 0.5, "and the clock runs again");
    t.frame(["pause.png", "settings-zh.png", "play-zh.png"], () => {
      for (const f of ["pause.png", "settings-zh.png", "play-zh.png"]) t.equal(run.png(f).width, 1280, `${f} captured`);
    });
  },
});
