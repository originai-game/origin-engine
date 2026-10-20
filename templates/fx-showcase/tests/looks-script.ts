// The looks station's script (tests/looks.og.ts): the station at 0.5 s, a
// plain frame, then each preset of the catalogue applied in turn with its
// frame STEP seconds later, and the look taken off again. Its own module,
// with no engine import, so the browser reference tiers
// (packages/engine/tests/web/reference.ts --looks) run the same commands
// and take each preset's frame at its screenshot's tick.
import { LOOK_PRESETS } from "../src/look-presets.ts";

/** Seconds between two presets. */
export const STEP = 0.5;

const script = ["0.5:station:looks", "0.9:screenshot:looks-plain.png"];
LOOK_PRESETS.forEach((slug, i) => {
  const at = 1.0 + i * STEP;
  script.push(`${at.toFixed(2)}:look:${slug}`, `${(at + STEP - 0.1).toFixed(2)}:screenshot:looks-${slug}.png`);
});
const end = 1.0 + LOOK_PRESETS.length * STEP;
script.push(`${end.toFixed(2)}:look:none`, `${(end + 0.4).toFixed(2)}:screenshot:looks-none.png`, `${(end + 0.5).toFixed(2)}:quit`);

/** The whole `og run --commands` script, screenshots and quit included. */
export const LOOK_SCRIPT = script.join(";");

const screenshot = (c: string) => c.split(":")[1] === "screenshot";

/** The script's commands without its screenshots and quit, in order. */
export const LOOK_COMMANDS: string[] = script.filter((c) => !screenshot(c) && c.split(":")[1] !== "quit");

/** Every frame the script takes, with the second its screenshot runs at. */
export const LOOK_SHOTS: Array<{ file: string; at: number }> = script.filter(screenshot).map((c) => {
  const [at, , file] = c.split(":");
  return { file: file!, at: Number(at) };
});

/** The preset frames compared with references/<tier>/ (the plain and none frames are compared with each other). */
export const LOOK_FRAMES: string[] = LOOK_PRESETS.map((s) => `looks-${s}.png`);
