// The systems games' previews on a phone in portrait (390×844 CSS px at 3×,
// a 47 px notch and a 34 px home indicator), driven by touch only, then
// right to left, in Japanese and at 200 % text: each frame compared with
// crates/og-ui/references/<tier>/kit-genres2-phone-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { genres2Script } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = genres2Script(true);

export default playtest({
  name: "the systems games' previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of ["garage.json", "garage-layer.json", "ledger.json", "citymap-power.json", "chat.json", "chat-mention.json", "rts.json"]) {
      for (const e of ui(run, f).filter((x) => x.role === "button" && !x.states.includes("disabled") && !x.clipped)) {
        t.ok(e.width >= 43.5 && e.height >= 43.5, `${f}: ${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
      }
    }
    t.ok(ui(run, "garage-bought.json").some((e) => e.role === "button" && e.name === "Equip"), "the bought sport tier offers Equip");
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
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres2-phone-");
  },
});
