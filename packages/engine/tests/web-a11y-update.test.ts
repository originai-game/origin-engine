// The accessibility mirror (src/web/a11y.ts) takes the Worker's updates as
// deltas: a node sent again replaces what was mirrored of it, the nodes not
// sent stay as they were, and a node its parent's children no longer list
// leaves the page. A live node carries aria-live and aria-atomic, so the
// page announces its text changing. The main-thread shell runs against a
// stand-in Worker over a stand-in document (tests/support/dom.ts).
//
// The ways this could fail, each asserted below: an update that drops the
// nodes it did not carry, a node changed in the tree but not in the page,
// a removed node left behind, a live region without its live setting.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { Shell } from "../src/web/shell.ts";
import type { A11yWireNode } from "../src/web/a11y.ts";
import { install, StandInDocument, standInCanvas, type StandInElement } from "./support/dom.ts";

class StandInWorker extends EventTarget {
  postMessage(m: { type?: string }): void {
    if (m?.type === "start") setTimeout(() => this.say({ type: "ready", tier: "webgpu", lanes: { on: false, threads: 1, why: "a test" } }), 0);
  }
  say(data: unknown): void {
    this.dispatchEvent(new MessageEvent("message", { data }));
  }
  terminate(): void {}
}

const node = (path: string, role: string, name: string, extra: Partial<A11yWireNode> = {}): A11yWireNode => ({
  path, role, name, description: "", value: "", states: [], x: 10, y: 10, width: 100, height: 20, children: [], ...extra,
});
const root = (children: string[]) => node("0", "", "", { children });

let uninstall: () => void;
beforeAll(() => {
  uninstall = install();
});
afterAll(() => uninstall());

test("an update replaces the nodes it carries, keeps the others and drops what no node reaches", async () => {
  const doc = new StandInDocument();
  const worker = new StandInWorker();
  const shell = await Shell.start({ canvas: standInCanvas(doc), worker: worker as unknown as Worker, glue: "unused.js", wasm: "unused.wasm", autoplay: false });
  const settle = () => new Promise((r) => setTimeout(r, 0));
  const at = (path: string) => doc.all().find((e) => e.dataset["ogPath"] === path) as StandInElement | undefined;
  const hud = node("4", "group", "HUD", { children: ["7", "9"] });
  worker.say({
    type: "a11y",
    update: { full: true, focus: "", nodes: [root(["4"]), hud, node("7", "status", "Wave 1", { live: "polite", atomic: true }), node("9", "button", "Pause", { states: ["focusable"] })] },
  });
  await settle();
  const pause = at("9");
  expect(pause?.getAttribute("aria-label")).toBe("Pause");
  expect([at("7")?.getAttribute("aria-live"), at("7")?.getAttribute("aria-atomic"), at("7")?.textContent]).toEqual(["polite", "true", "Wave 1"]);
  // The status alone changed: the button is the same element, as it was.
  worker.say({ type: "a11y", update: { full: false, focus: "9", nodes: [node("7", "status", "Wave 2", { live: "polite", atomic: true })] } });
  await settle();
  expect(at("7")?.textContent).toBe("Wave 2");
  expect(at("9")).toBe(pause);
  expect(shell.a11y?.tree.focused).toBe("9");
  // The HUD lost its button: only the HUD was sent, and the button is gone.
  worker.say({ type: "a11y", update: { full: false, focus: "", nodes: [{ ...hud, children: ["7"] }] } });
  await settle();
  expect(shell.a11y?.node("9")).toBeUndefined();
  expect(shell.a11y?.element("9")).toBeUndefined();
  expect(shell.a11y?.tree.nodes.map((n) => [n.path, n.parent])).toEqual([["4", ""], ["7", "4"]]);
  shell.dispose();
});
