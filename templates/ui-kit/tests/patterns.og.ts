// The genre pattern previews at desktop size (1280×720 CSS px at 1×): the
// gear screen (inventory grid and paperdoll) idle, hovered, dragged over a
// slot that accepts, a refused footprint, a swap and a merge, equipped, the
// item menu, a lit tag, a selection, the key cursor and a key carry, then
// right to left and at 200 % text, each frame compared with
// crates/og-ui/references/<tier>/kit-patterns-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { patternsScript } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = patternsScript(false);

export default playtest({
  name: "the genre pattern previews in their states at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const equipped = ui(run, "gear-equipped.json");
    t.ok(equipped.some((e) => e.role === "button" && e.name.startsWith("Head, Crown of Thorns")), "the helm dragged onto the head slot is worn");
    t.ok(!equipped.some((e) => e.id === "helm" && e.role === "gridcell"), "and left the backpack");
    const menu = ui(run, "gear-menu.json");
    t.ok(menu.some((e) => e.role === "menu"), "a long press opened the item menu");
    const keys = ui(run, "gear-keys.json");
    t.ok(keys.some((e) => e.role === "grid" && e.states.includes("focused")), "Tab put the focus on the backpack");
    const aim = ui(run, "cards-aim.json");
    t.ok(aim.some((e) => e.role === "status" && e.name === "valid: Cultist"), "the arc dragged onto the cultist says it is a valid target");
    t.ok(!ui(run, "cards-played.json").some((e) => e.role === "option" && e.name.startsWith("Bash")), "released there, Bash was played and left the hand");
    t.ok(ui(run, "cards-inspect.json").some((e) => e.role === "dialog" && e.name === "Inflame"), "the inspected card is shown large");
    t.ok(ui(run, "cards-keys.json").some((e) => e.role === "status" && e.name === "invalid: Red Louse"), "keys stepped the target onto the taunted enemy and the arc says invalid");
    t.ok(ui(run, "rewards-banish.json").some((e) => e.id === "Banish" && e.role === "button" && e.states.includes("selected")), "Banish armed is said selected");
    t.ok(ui(run, "rewards-selected.json").filter((e) => e.role === "checkbox" && e.states.includes("checked")).length === 2, "two rewards selected for a two-pick choice");
    t.ok(ui(run, "gacha-short.json").some((e) => e.role === "button" && e.name.includes("not enough")), "a ten-pull the purse cannot pay is said not enough");
    t.ok(ui(run, "gacha-reveal.json").some((e) => e.role === "status" && e.name.startsWith("Aria, Tidecaller, 5 stars, new")), "the fifth tap reached the 5★ reveal");
    t.ok(ui(run, "gacha-summary.json").filter((e) => e.role === "listitem").length === 10, "Skip went to a summary of all ten");
    t.ok(ui(run, "cards-refused.json").some((e) => e.id === "Note" && e.text === "Behind a taunt"), "a card let go on the taunted foe is refused there and the foe says why");
    t.ok(ui(run, "cards-unaffordable.json").some((e) => e.role === "option" && e.name.startsWith("Bash") && e.states.includes("disabled")), "with one energy, Bash (2) is said disabled");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-patterns-");
  },
});
