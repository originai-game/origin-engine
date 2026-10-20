// The looks station: the same diorama through every preset of the
// engine's look catalogue (templates/looks, packages/knowledge/looks/
// catalogue.json), one frame per preset against references/<tier>/
// looks-<slug>.png. Each `look:<slug>` lands before its frame; the last
// frame is the diorama with the look taken off again, which must match
// the first (a look leaves nothing behind).
import { playtest } from "@origingame/engine/playtest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { LOOK_PRESETS } from "../src/looks.ts";
import { checkFrames, decodePng, difference, outDir } from "./frames.ts";

const STEP = 0.5;
const script = ["0.5:station:looks", "0.9:screenshot:looks-plain.png"];
LOOK_PRESETS.forEach((slug, i) => {
  const at = 1.0 + i * STEP;
  script.push(`${at.toFixed(2)}:look:${slug}`, `${(at + STEP - 0.1).toFixed(2)}:screenshot:looks-${slug}.png`);
});
const end = 1.0 + LOOK_PRESETS.length * STEP;
script.push(`${end.toFixed(2)}:look:none`, `${(end + 0.4).toFixed(2)}:screenshot:looks-none.png`, `${(end + 0.5).toFixed(2)}:quit`);

export default playtest({
  name: "the looks station shows every preset of the catalogue and takes it off again",
  size: "1280x720",
  commands: script.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const load = (f: string) => decodePng(readFileSync(join(outDir(), f)));
    const plain = load("looks-plain.png");
    const none = load("looks-none.png");
    const back = difference(plain, none);
    t.ok(back.rmse <= 0.01, `look:none restores the plain diorama (RMSE ${back.rmse.toFixed(5)})`);
    // Every preset changes the frame: none is a no-op by accident.
    for (const slug of LOOK_PRESETS) {
      const d = difference(plain, load(`looks-${slug}.png`));
      t.ok(d.rmse > 0.002, `${slug} changes the frame (RMSE ${d.rmse.toFixed(5)} against the plain diorama)`);
    }
    checkFrames(run, t, [...LOOK_PRESETS.map((s) => `looks-${s}.png`)], "");
  },
});
