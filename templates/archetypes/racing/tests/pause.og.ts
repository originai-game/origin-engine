// Pause and settings: Escape holds the race (time scale 0, the clock
// stops, Throttle moves nothing), the pause menu opens Settings, 简体中文
// switches every string, the music slider moves, Done saves the settings
// through og.platform, and Resume gives the race back, in the chosen
// language. Then the pause menu's Retire goes back to the title.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, press, sorted, tap, texts, TITLE, type PlatformDump } from "./lib.ts";

const position = (run: { json<T>(f: string): T }, f: string) => run.json<{ value: { position: [number, number, number] } }>(f).value.position;

export default playtest({
  name: "Escape pauses, settings switch the language and save, Resume gives the race back, Retire leaves it",
  size: "1280x720",
  commands: sorted([
    "0.3:race:go",
    "0.5:warp:300:0:20",
    press("Escape", 0.7),
    "0.9:get:Game:paused.json",
    "0.9:get:og.TimeScale.scale:scale.json",
    "0.9:get:car-0.og.Transform:before.json",
    hold("KeyW", 0.95, 0.5),
    "1.5:get:Game:held.json",
    "1.5:get:car-0.og.Transform:after.json",
    "1.55:screenshot:pause.png",
    tap("Settings", 1.6),
    "2.2:move_to_text:简体中文",
    "2.22:touch_start",
    "2.25:touch_end",
    "2.5:touch_move_to:Music/Slider",
    "2.52:touch_start",
    "2.55:touch_end",
    "2.9:screenshot:settings-zh.png",
    tap("Done", 3.1),
    "3.5:dump_platform:platform.json",
    tap("Resume", 3.7),
    "4.2:dump_ui:race-zh.json",
    "4.2:get:Game:resumed.json",
    "4.25:screenshot:race-zh.png",
    press("Escape", 4.4),
    tap("Quit", 4.8),
    "5.6:get:Game:title.json",
    "5.7:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const paused = game(run, "paused.json");
    t.ok(paused.paused && paused.held, "Escape paused the race");
    t.equal(run.json<{ value: number }>("scale.json").value, 0, "the race holds: time scale 0");
    const held = game(run, "held.json");
    t.equal(held.clock, paused.clock, "the clock stopped");
    const [bx, , bz] = position(run, "before.json");
    const [ax, , az] = position(run, "after.json");
    t.ok(Math.abs(ax - bx) + Math.abs(az - bz) < 1e-3, "Throttle while paused moves nothing");
    const p = run.json<PlatformDump>("platform.json");
    const saved = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "og.settings").at(-1)?.args["value"] as { language?: string; musicVolume?: number } | undefined;
    t.equal(saved?.language, "zh-CN", "Done saved the settings to og.platform with the language");
    t.ok(saved?.musicVolume !== undefined && Math.abs(saved.musicVolume - 0.6) > 0.01, `and the music slider's move (${saved?.musicVolume})`);
    const words = texts(run, "race-zh.json");
    t.ok(words.includes("圈") && words.includes("名次"), "the HUD speaks 简体中文");
    const resumed = game(run, "resumed.json");
    t.ok(!resumed.paused && resumed.clock > held.clock + 20, "Resume gave the race back and its clock runs");
    t.equal(game(run, "title.json").phase, TITLE, "Retire went back to the title");
    t.frame(["pause.png", "settings-zh.png", "race-zh.png"], () => {
      for (const f of ["pause.png", "settings-zh.png", "race-zh.png"]) t.equal(run.png(f).width, 1280, `${f} captured`);
    });
  },
});
