// The racing HUD on a phone in portrait (racing.og.ts on a desktop): idle
// in scifi, at 150 % text (the tower dropped, the map shrunk), the
// countdown, and the arcade look in cartoon. Each frame is compared with
// crates/og-ui/references/<tier>/kit-racing-phone-<file>.
import { checkFrames } from "./frames.ts";
import { playtest, ui } from "./lib.ts";

const SHOTS: ReadonlyArray<[string, string[]]> = [
  ["idle.png", ["viewport:phone", "style:scifi", "show:raceHud"]],
  ["text-150.png", ["demo:text:150"]],
  ["countdown.png", ["demo:text:100", "sc:race:countdown"]],
  ["arcade.png", ["sc:reset", "style:cartoon"]],
];

const steps: string[] = [];
let t = 0.2;
for (const [file, cmds] of SHOTS) {
  for (const c of cmds) steps.push(`${t.toFixed(2)}:${c}`);
  t += 1.4;
  steps.push(`${t.toFixed(2)}:dump_ui:${file.replace(/\.png$/, ".json")}`, `${t.toFixed(2)}:screenshot:${file}`);
  t += 0.1;
}
steps.push(`${t.toFixed(2)}:quit`);
const FILES = SHOTS.map(([f]) => f);

export default playtest({
  name: "the racing HUD on a phone in portrait, at 150 % text and in the arcade look matches its reference frames",
  size: "1170x2532",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const has = (f: string, part: string) => ui(run, f).some((e) => e.path.includes(part));
    t.ok(has("idle.json", "Standings/You"), "the tower shows the player between its neighbours");
    t.ok(!has("text-150.json", "Standings/"), "at 150 % text the tower drops before the instrument shrinks");
    t.ok(ui(run, "countdown.json").some((e) => e.text === "2"), "the countdown shows its 2");
    t.ok(has("arcade.json", "Item"), "the arcade look holds an item");
    t.frame(FILES, () => {
      for (const f of FILES) t.ok(run.png(f).width > 0, `${f} was drawn`);
    });
    checkFrames(run, t, FILES, "kit-racing-phone-");
  },
});
