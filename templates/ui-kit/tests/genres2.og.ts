// The systems games' previews at desktop size (1280×720 CSS px at 1×): the
// garage (a part previewed, one refused, one bought, the livery's layers,
// a layer moved up); then each right to left, in Japanese and at 200 %
// text, each frame compared with
// crates/og-ui/references/<tier>/kit-genres2-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { genres2Script } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = genres2Script(false);

export default playtest({
  name: "the systems games' previews in their states at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.ok(ui(run, "garage.json").some((e) => e.role === "status" && e.name.includes("→")), "the performance card previews the sport tier's rating");
    t.ok(ui(run, "garage-bought.json").some((e) => e.role === "button" && e.name === "Equip"), "the bought sport tier offers Equip");
    t.ok(ui(run, "garage-layer.json").filter((e) => e.role === "option")[0]?.name.startsWith("Stripe"), "Up moved the stripe over the base coat");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres2-");
  },
});
