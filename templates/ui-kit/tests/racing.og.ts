// The racing showcase's HUD (src/race.ts) over its engine-rendered circuit
// (src/raceScene.ts), in the art director's brief's states: idle in scifi,
// the countdown at "2", a checkpoint's split, the wrong way, the final lap
// and the finish; idle in broadcast (golden hour) and pixel (the digital
// readout); the arcade look in cartoon and kawaii (racing-phone.og.ts: on
// a phone). Each frame is compared with
// crates/og-ui/references/<tier>/kit-racing-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { ui } from "./lib.ts";

/** [file, the commands before it] in order; each is taken 1.6 s after its commands, once the moment has played in. */
const SHOTS: ReadonlyArray<[string, string[]]> = [
  ["idle.png", ["style:scifi", "show:raceHud"]],
  ["countdown.png", ["sc:race:countdown"]],
  ["checkpoint.png", ["sc:race:checkpoint"]],
  ["wrongway.png", ["sc:race:wrongWay"]],
  ["finallap.png", ["sc:race:finalLap"]],
  ["finish.png", ["sc:race:finish"]],
  ["broadcast.png", ["sc:reset", "style:broadcast"]],
  ["pixel.png", ["style:pixel"]],
  ["arcade-cartoon.png", ["style:cartoon"]],
  ["arcade-kawaii.png", ["style:kawaii"]],
];

const steps: string[] = [];
let t = 0.2;
for (const [file, cmds] of SHOTS) {
  for (const c of cmds) steps.push(`${t.toFixed(2)}:${c}`);
  // The countdown's "2" shows from 1 s to 2 s after it starts.
  t += 1.4;
  steps.push(`${t.toFixed(2)}:dump_ui:${file.replace(/\.png$/, ".json")}`, `${t.toFixed(2)}:screenshot:${file}`);
  t += 0.1;
}
steps.push(`${t.toFixed(2)}:quit`);
const FILES = SHOTS.map(([f]) => f);

export default playtest({
  name: "the racing HUD over its circuit in its moments, packs and surfaces matches its reference frames",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const texts = (f: string) => ui(run, f).map((e) => e.text);
    const has = (f: string, path: string) => ui(run, f).some((e) => e.path === path || e.path.endsWith(`/${path}`));
    t.ok(texts("countdown.json").includes("2"), "the countdown shows its 2 after a second");
    t.ok(texts("checkpoint.json").includes("Checkpoint 2"), "a checkpoint names itself");
    t.ok(has("wrongway.json", "Banner/WrongWay/Label"), "the wrong way banner shows");
    t.ok(texts("finish.json").includes("3rd") && texts("finish.json").includes("3:41.287"), "the finish shows the place and the race time");
    t.ok(texts("idle.json").includes("Meridian Loop"), "the circuit's map names it");
    t.ok(has("pixel.json", "Instrument/Speed") && !has("arcade-cartoon.json", "Instrument/Speed/Num#2"), "pixel keeps its readout; the arcade look has no tach numbers");
    t.ok(texts("arcade-cartoon.json").includes("3rd"), "the arcade look's place is its ordinal");
    t.frame(FILES, () => {
      for (const f of FILES) t.ok(run.png(f).width > 0, `${f} was drawn`);
    });
    checkFrames(run, t, FILES, "kit-racing-");
  },
});
