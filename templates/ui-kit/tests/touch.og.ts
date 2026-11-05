// Touch through the probe's ids and texts: the gallery, scrolled, opens the
// store by its row's text; a touch on an offer asks to confirm and Buy
// spends its price and marks it owned; Escape returns; the action
// showcase's results take a touch on Play again into the HUD; a generator's
// Buy buys one; the visual novel's box finishes a typing line on the first
// press, then advances on the second.
import { demo, key, playtest, SCREEN, tap, tapText, type UiElement, ui } from "./lib.ts";

const STORE = "Shop/Body/View/Content/Store";
const LINE = "Novel/Talk/Caption/Words/Line";

export default playtest({
  name: "touch_move_to and move_to_text drive the gallery, the store, results, generators and the visual novel",
  commands: [
    // The store is the forty-second row (88 px rows that hold a two-line blurb; seventeen showcases lead): 3,356 px brings it whole into the list's view.
    "0.3:touch_move_to:Screens",
    "0.35:scroll:0,3356",
    tapText(0.5, "=store"),
    "1.2:dump_entity:demo:store-open.json",
    tap(1.3, "Offers/ashbringer"),
    "1.6:dump_ui:confirm-ui.json",
    tap(1.7, "Buy/Panel/Actions/confirm"),
    "2.3:dump_ui:bought-ui.json",
    key(2.4, "Escape"),
    "2.8:dump_entity:demo:closed.json",
    "2.9:show:actionResults",
    tap(3.3, "Footer/Again"),
    "3.6:dump_entity:demo:again.json",
    "3.7:show:generators",
    "4.3:dump_ui:generators-before.json",
    tap(4.4, "Generators/solar/Buy"),
    "4.7:dump_ui:generators-after.json",
    "4.8:show:vn",
    "5.1:dump_ui:vn-typing.json",
    tap(5.2, "Talk/Hit"),
    "5.4:dump_ui:vn-shown.json",
    tap(5.5, "Talk/Hit"),
    "5.7:dump_ui:vn-next.json",
    "5.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const at = (file: string, path: string): UiElement | undefined => ui(run, file).find((e) => e.path === path);
    t.equal(demo(run, "store-open.json").screen, SCREEN.store, "scrolled down, the store row of the gallery opened the store");
    t.equal(at("confirm-ui.json", "Buy/Panel")?.name, "Ashbringer", "a touch on Ashbringer asked to confirm its purchase");
    t.equal(at("bought-ui.json", `${STORE}/Head/Wallet/gems/Amount/Amount`)?.text, "600", "Buy took Ashbringer's 1,800 of the 2,400 gems");
    t.ok(at("bought-ui.json", `${STORE}/Sections/featured/Offers/ashbringer`)?.name.includes("Owned") ?? false, "and marked it owned");
    t.equal(at("bought-ui.json", "Buy/Panel"), undefined, "the confirm closed");
    t.equal(demo(run, "closed.json").screen, SCREEN.gallery, "Escape closed the store");
    t.equal(demo(run, "again.json").screen, SCREEN.actionHud, "Play again on the results went back into the HUD");
    const owned = (file: string) => Number(/(\d+)/.exec(at(file, "Gens/Body/View/Content/Split/Left/Generators/solar/Info/Meta/Owned")?.text ?? "")?.[1] ?? NaN);
    t.equal(owned("generators-after.json") - owned("generators-before.json"), 1, "Buy on the solar row bought one");
    const reveal = (file: string) => run.json<{ motion: Array<{ path: string; states: Record<string, string> }> }>(file).motion.find((m) => m.path === LINE)?.states["reveal"];
    t.equal(reveal("vn-typing.json"), "typing", "the visual novel's first line is typing");
    t.equal([reveal("vn-shown.json"), at("vn-shown.json", LINE)?.name], ["shown", at("vn-typing.json", LINE)?.name], "the first press finished the line, still on it");
    const next = at("vn-next.json", LINE);
    t.ok(next !== undefined && next.name.startsWith("Aria:") && reveal("vn-next.json") === "typing", `the second advanced to the next line, typing (${next?.name})`);
  },
});
