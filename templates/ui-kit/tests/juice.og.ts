// Motion and sound: a screen change goes through og.ui.screenTransition
// (the old screen stays until the iris closes, then the new one is built
// under it as it opens: the action showcase's results document), and with
// sounds on a dialog's open (the store's purchase confirm), a hover and a
// press each start a voice on the ui bus. The frames it writes are the E3
// transition transcript.
import { playtest } from "@origingame/engine/playtest";
import { ui } from "./lib.ts";

const has = (els: { id: string }[], id: string): boolean => els.some((e) => e.id === id);

export default playtest({
  name: "an iris transition switches screens at full cover, UI events sound",
  size: "1280x720",
  commands: [
    "0.10:style:cartoon", "0.15:transition:iris", "0.20:sounds:on",
    "0.30:show:actionResults",
    "0.45:screenshot:iris-closing.png", "0.45:dump_ui:covering.json",
    "0.75:screenshot:iris-opening.png", "0.75:dump_ui:revealing.json",
    "2.40:screenshot:landed.png", "2.40:dump_ui:landed.json",
    "2.50:transition:none", "2.55:show:store",
    "3.20:touch_move_to:Offers/ashbringer", "3.25:touch_start", "3.30:touch_end",
    "3.80:dump_ui:confirm.json",
    "3.90:touch_move_to:Buy/Panel/Actions/cancel", "3.95:touch_start", "4.00:touch_end",
    "4.10:dump_world", "4.20:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const covering = ui(run, "covering.json");
    t.ok(has(covering, "Screens") && !has(covering, "Results"), "while the iris closes the old screen (the gallery) is still built");
    const revealing = ui(run, "revealing.json");
    t.ok(has(revealing, "Results") && !has(revealing, "Screens"), "once it has closed the results are built under it");
    t.ok(has(ui(run, "landed.json"), "Results"), "and stay once it has opened");
    t.ok(ui(run, "confirm.json").some((e) => e.path === "Buy/Panel" && e.role === "dialog"), "the touch on an offer opened the purchase confirm");
    const world = run.json<{ archetypes: { components: string[]; count: number }[] }>(run.files.find((f) => f.startsWith("dump_world"))!);
    const voices = world.archetypes.filter((a) => a.components.includes("og.AudioSource")).reduce((n, a) => n + a.count, 0);
    t.ok(voices >= 3, `the dialog's open, the hover and the press started voices (${voices})`);
  },
});
