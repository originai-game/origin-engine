// The starship screens at desktop size (1280×720 CSS px at 1×): ship status,
// turn order, ability cards, sector map and hangar in the scifi pack (their
// showcase), fantasy and kawaii, the hangar with another class picked, then
// right to left, in Japanese and at 200 % text, each frame compared with
// crates/og-ui/references/<tier>/kit-starship-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { starshipScript } from "./starship.ts";
import { ui } from "./lib.ts";

const script = starshipScript(false);

export default playtest({
  name: "the starship screens in scifi, fantasy and kawaii at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const status = ui(run, "scifi-shipStatus.json");
    t.ok(status.some((e) => e.role === "heading" && e.name === "AFS Valiant"), "the ship's name is the status's heading");
    t.ok(status.some((e) => e.role === "progressbar" && e.name === "Shields" && e.value === "3 / 4"), "the shields are pips, three of four");
    const order = ui(run, "scifi-turnOrder.json").filter((e) => e.role === "option");
    t.ok(order.length === 8 && order[0]!.name === "Valiant" && order[0]!.states.includes("selected"), "the acting ship heads the order, selected");
    const cards = ui(run, "scifi-abilityCards.json");
    t.equal(cards.filter((e) => e.id === "Card" && e.role === "button").length, 6, "a hand of six cards");
    t.ok(cards.some((e) => e.id === "EndTurn" && e.role === "button"), "End turn is there");
    t.ok(ui(run, "scifi-sectorMap.json").some((e) => e.role === "image" && e.name.endsWith("current sector")), "the current sector is marked");
    t.ok(ui(run, "scifi-hangar-carriers.json").some((e) => e.role === "option" && e.name.startsWith("Carriers") && e.states.includes("selected")), "a class picked is the selected row");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-starship-");
  },
});
