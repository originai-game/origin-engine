// The systems games' previews at desktop size (1280×720 CSS px at 1×): the
// garage (a part previewed, one refused, one bought, the livery's layers,
// a layer moved up), the ledger (a service underfunded, sorted, the year)
// and the city's overlays (traffic, power, none); then each right to left, in Japanese and at 200 %
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
    const rows = (f: string): string[] => ui(run, f).filter((e) => e.role === "row").map((e) => e.name);
    t.ok(rows("ledger-sorted.json")[0]?.startsWith("Education"), "Amount sorted the services high to low: the dearest first");
    t.ok(rows("ledger-underfunded.json").some((n) => n.startsWith("Fire") && n.includes("60%")), "fire's budget at 60 % is in its row");
    t.ok(ui(run, "citymap-power.json").some((e) => e.role === "status" && e.name.startsWith("Power")), "a press on Power showed its legend");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres2-");
  },
});
