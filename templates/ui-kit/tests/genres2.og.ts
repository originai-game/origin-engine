// The systems games' previews at desktop size (1280×720 CSS px at 1×): the
// garage (a part previewed, one refused, one bought, the livery's layers,
// a layer moved up), the ledger (a service underfunded, sorted, the year)
// the city's overlays (traffic, power, none) and the chat (links and a
// mention, a mention suggested and sent, the party's tab, the HUD's chat)
// and the RTS console (its orders, Build, Barracks targeting, a group); then each right to left, in Japanese and at 150 %
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
    t.ok(ui(run, "chat.json").some((e) => e.role === "tab" && e.name.startsWith("Party") && e.name.endsWith("mentioned")), "the party's tab counts its unread messages and says the player is mentioned there");
    t.ok(ui(run, "chat.json").some((e) => e.role === "listitem" && e.name.includes("[Ashbringer]")), "an item link stands in its message in brackets");
    t.ok(ui(run, "chat-mention.json").some((e) => e.role === "option" && e.name === "Riven"), "@Ri offers Riven");
    t.ok(ui(run, "chat-sent.json").some((e) => e.role === "listitem" && e.name.endsWith("Lumen: gg @Riven")), "the suggestion wrote the name and Send posted it");
    t.ok(ui(run, "rts-target.json").some((e) => e.role === "status" && e.name.startsWith("Barracks")), "Barracks waits for its target, the prompt naming it");
    t.ok(ui(run, "rts-group.json").filter((e) => e.parent.endsWith("Tiles")).map((e) => e.id).join(",") === "k1,k2", "group 2's tab selected the knights alone");
    t.ok(ui(run, "rts-assigned.json").some((e) => e.role === "tab" && e.id === "g3" && e.value === "2"), "Ctrl+3 set group 3 to the two knights");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres2-");
  },
});
