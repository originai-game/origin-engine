// The kit's accessibility tree is complete on every screen: the probe's
// dump_ui gives each element its role, name, value and states (og.ui.a11y
// answers the same tree to game code), every control keys and pads reach is
// named, every element showing text has a role, and keyboard Tab walks the
// focusable elements in the tree's order. The demo reads og.ui.a11y each
// tick; on the main menu its focus order is the probe's focusable elements.
// Ways it could fail: a widget without a role or a control without a name
// (an icon button, a card), Tab taking another order than the tree's, or
// og.ui.a11y answering a different tree than the probe's.
import { playtest } from "@origingame/engine/playtest";
import { FRAME_SCREENS } from "./script.ts";
import { type UiElement, demo, key, ui } from "./lib.ts";

const SCREENS = [...FRAME_SCREENS, "players"] as const;
/** Roles a person operates: each needs a name. */
const CONTROLS = new Set(["button", "link", "checkbox", "radio", "switch", "slider", "spinbutton", "textbox", "combobox", "option", "tab"]);
const TABS = 6;

const steps: string[] = [];
let t = 0.5;
for (const s of SCREENS) {
  steps.push(`${t.toFixed(2)}:show:${s}`);
  steps.push(`${(t + 0.6).toFixed(2)}:dump_ui:${s}.json`);
  t += 0.7;
}
steps.push(`${t.toFixed(2)}:show:mainMenu`);
t += 0.6;
steps.push(`${t.toFixed(2)}:dump_ui:menu-0.json`);
steps.push(`${(t + 0.05).toFixed(2)}:dump_entity:demo:menu.json`);
for (let k = 1; k <= TABS; k++) {
  t += 0.2;
  steps.push(key(t, "Tab"));
  steps.push(`${(t + 0.1).toFixed(2)}:dump_ui:menu-${k}.json`);
}
steps.push(`${(t + 0.2).toFixed(2)}:quit`);

const path = (els: UiElement[], e: UiElement): string => {
  const parts = [e.id];
  let p = e.parent;
  while (p) {
    parts.unshift(p);
    p = els.find((x) => x.id === p)?.parent ?? "";
  }
  return parts.join("/");
};
const has = (e: UiElement, s: string) => e.states.includes(s);

export default playtest({
  name: "every screen's accessibility tree names its controls, and Tab follows its focus order",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    for (const s of SCREENS) {
      const els = ui(run, `${s}.json`);
      const unnamed = els.filter((e) => (has(e, "focusable") || CONTROLS.has(e.role)) && (e.role === "" || e.name === "")).map((e) => `${path(els, e)} (${e.kind}, role "${e.role}")`);
      t.equal(unnamed, [], `${s}: every control has a role and a name`);
      const silent = els.filter((e) => e.text !== "" && e.role === "").map((e) => `${path(els, e)} (${e.kind})`);
      t.equal(silent, [], `${s}: every element showing text has a role`);
      t.ok(els.some((e) => e.role === "heading" || e.role === "dialog" || CONTROLS.has(e.role)), `${s}: the tree has controls or a heading (${els.filter((e) => e.role).length} nodes)`);
    }
    const menu = ui(run, "menu-0.json");
    const order = menu.filter((e) => has(e, "focusable")).map((e) => path(menu, e));
    t.ok(order.length >= 4, `the main menu has a focus order (${order.join(", ")})`);
    const walked: string[] = [];
    for (let k = 1; k <= TABS; k++) {
      const els = ui(run, `menu-${k}.json`);
      const focused = els.filter((e) => has(e, "focused"));
      t.equal(focused.length, 1, `Tab ${k}: one element is focused`);
      if (focused[0]) walked.push(path(els, focused[0]));
    }
    const expected = Array.from({ length: TABS }, (_, k) => order[k % order.length]!);
    t.equal(walked, expected, "Tab walks the focusable elements in the tree's order, then wraps");
    const d = demo(run, "menu.json");
    t.equal(d.a11yFocus, order.length, "og.ui.a11y's focus order is as long as the probe's");
    t.equal(d.a11yNodes, menu.filter((e) => e.role !== "").length, "og.ui.a11y has a node for each element with a role");
  },
});
