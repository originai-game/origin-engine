// The onboarding overlay: the action showcase's tutorial (the ejected
// tutorial document over the HUD, src/showcases.ts) on a phone in portrait.
// Each step's card sits just under the HUD part it teaches (the run's coins
// and timer on odd steps, the health on even ones), a press away from the
// card changes nothing, Next walks the five steps, and Next on the last one
// finishes it into the HUD.
// On a phone the card spans the screen (12 px from each edge): the desktop
// places it at the target's centre, where the odd steps' card runs past the
// screen's end edge (reported, not driven here).
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, tap, ui, type UiElement } from "./lib.ts";

const NEXT = "tutorial/Tutorial/Card/Foot/Next";

export default playtest({
  name: "the tutorial's card sits under each step's target and Next walks to the end",
  size: "1170x2532",
  commands: [
    "0.0:viewport:phone",
    "0.1:sc:tutorial:1",
    "0.2:show:actionTutorial",
    "0.8:dump_ui:step1.json",
    "0.9:touch_move:195,700",
    "0.95:touch_start",
    "1.0:touch_end",
    "1.1:dump_ui:blocked.json",
    "1.1:dump_entity:demo:blocked-demo.json",
    tap(1.2, NEXT),
    "1.5:dump_ui:step2.json",
    tap(1.6, NEXT),
    tap(1.9, NEXT),
    tap(2.2, NEXT),
    "2.5:dump_ui:step5.json",
    tap(2.6, NEXT),
    "3.0:dump_entity:demo:done.json",
    "3.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const at = (els: UiElement[], path: string) => els.find((e) => e.path === path);
    const card = (els: UiElement[]) => ({
      box: at(els, "tutorial/Tutorial/Card"),
      title: at(els, "tutorial/Tutorial/Card/Title")?.text ?? "",
      step: at(els, "tutorial/Tutorial/Card/Foot/Step")?.text ?? "",
    });
    const under = (els: UiElement[], target: string, n: number) => {
      const c = card(els).box;
      const to = at(els, target);
      t.ok(c !== undefined && to !== undefined && Math.abs(c.y - (to.y + to.height + 12)) <= 1, `step ${n}'s card sits 12 px under ${target} (card top ${c?.y}, target bottom ${to ? to.y + to.height : "missing"})`);
      t.ok(c !== undefined && c.x >= 0 && c.x + c.width <= 390, `step ${n}'s card is on the screen (${c?.x} to ${c ? c.x + c.width : "?"})`);
    };
    const s1 = ui(run, "step1.json");
    t.equal(card(s1).step, "1/5", "the tutorial opens on step 1 of 5");
    under(s1, "Run", 1);
    const blocked = ui(run, "blocked.json");
    t.equal([demo(run, "blocked-demo.json").screen, card(blocked).step], [SCREEN.actionTutorial, "1/5"], "a press away from the card changed nothing");
    const s2 = ui(run, "step2.json");
    t.equal(card(s2).step, "2/5", "Next walked to step 2");
    t.ok(card(s2).title !== card(s1).title, `with its own title (${card(s1).title} → ${card(s2).title})`);
    under(s2, "Player", 2);
    const s5 = ui(run, "step5.json");
    t.equal(card(s5).step, "5/5", "three more Nexts reached the last step");
    under(s5, "Run", 5);
    t.equal(demo(run, "done.json").screen, SCREEN.actionHud, "Next on the last step finished the tutorial into the HUD");
  },
});
