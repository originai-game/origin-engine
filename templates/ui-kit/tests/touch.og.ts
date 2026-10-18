// Touch through the probe's ids and texts: the gallery opens the shop by its
// row's text, Buy spends the price and shows a toast, Escape returns; the
// results' Share calls the share hook; the idle Tap counts; the cutscene's
// box finishes a line, then advances.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, key, tap, tapText, ui } from "./lib.ts";

export default playtest({
  name: "touch_move_to and move_to_text drive the gallery, the shop, results, idle and the cutscene",
  commands: [
    "0.3:touch_move_to:Screens",
    "0.35:scroll:0,400",
    tapText(0.5, "=shop"),
    "1.2:dump_entity:demo:shop-open.json",
    tap(1.3, "Detail/Buy"),
    "1.6:dump_entity:demo:bought.json",
    "1.6:dump_ui:bought-ui.json",
    key(1.8, "Escape"),
    "2.4:dump_entity:demo:closed.json",
    "2.5:show:results",
    tap(4.2, "Share"),
    "4.5:dump_ui:shared-ui.json",
    "4.6:show:idle",
    tap(5.2, "Tap"),
    tap(5.5, "Tap"),
    tap(5.8, "Tap"),
    tap(6.1, "Row#0/Buy"),
    "6.3:dump_entity:demo:idle.json",
    "6.4:show:cutscene",
    tap(6.8, "Dialogue"),
    tap(7.1, "Dialogue"),
    "7.4:dump_entity:demo:story.json",
    "7.4:dump_ui:story-ui.json",
    "7.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "shop-open.json").screen, SCREEN.shop, "scrolled down, the shop row of the gallery opened the shop");
    const bought = demo(run, "bought.json");
    t.equal([bought.balance, bought.owned & 1, bought.bought], [950, 1, 1], "Buy took the sword's 300 and marked it owned");
    t.ok(ui(run, "bought-ui.json").some((e) => e.parent === "Toasts" && e.text === "Bought Iron sword"), "a toast says what was bought");
    t.equal(demo(run, "closed.json").screen, SCREEN.gallery, "Escape closed the shop");
    t.ok(ui(run, "shared-ui.json").some((e) => e.text.startsWith("Shared: I scored 12,840")), "Share called the share hook with the text");
    const idle = demo(run, "idle.json");
    t.equal(idle.taps, 3, "three taps on Tap");
    t.equal(idle.gen0, 2, "Buy on the farm row bought a level");
    t.equal(demo(run, "story.json").line, 1, "the first press finished the line, the second advanced it");
    t.ok(ui(run, "story-ui.json").some((e) => e.id === "Line" && e.text.startsWith("Ready")), "the second line is typing");
  },
});
