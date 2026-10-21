// A slider and a stepper's spinbutton as the accessibility mirror
// (src/web/a11y.ts) puts them in the page: native ranges carrying the UI's
// bounds and value (aria-valuemin, aria-valuemax, aria-valuenow from the
// tree's valueMin, valueMax and valueNow), which a screen reader's own
// adjust gesture moves: the range's `input` event becomes an `increment`
// or `decrement` request for the control and the range goes back to the
// UI's value. The main-thread shell (src/web/shell.ts) runs against a
// stand-in Worker over a stand-in document (tests/support/dom.ts). No
// browser, no reader: the events a reader's adjust fires, as Chrome and
// Safari fire them on a native range.
//
// The ways this could fail, each asserted below: a range without bounds
// (the reader says no minimum or maximum, or the native range's own ±1),
// a stepper without bounds given some, a bound gone stale after the tree
// moves; a reader's step up or down sent the wrong way or not at all, sent
// for a step back to the UI's value, or the range left where the reader
// put it; a control that is no longer a range keeping the attributes.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { Shell } from "../src/web/shell.ts";
import type { A11yNode, A11yTree } from "../src/web/a11y.ts";
import { install, StandInDocument, standInCanvas, type StandInElement, type StandInInput } from "./support/dom.ts";

/** The engine Worker as the shell holds it: answers `start`, keeps the input events it is sent. */
class StandInWorker extends EventTarget {
  readonly sent: Array<{ kind: string; action?: string; path?: string }> = [];
  postMessage(m: { type?: string; event?: { kind: string; action?: string; path?: string } }): void {
    if (m?.type === "start") setTimeout(() => this.say({ type: "ready", tier: "webgpu", lanes: { on: false, threads: 1, why: "a test" } }), 0);
    if (m?.type === "input" && m.event) this.sent.push(m.event);
  }
  say(data: unknown): void {
    this.dispatchEvent(new MessageEvent("message", { data }));
  }
  terminate(): void {}
}

const node = (path: string, role: string, name: string, extra: Partial<A11yNode> = {}): A11yNode => ({
  path, parent: "Settings", role, name, description: "", value: "", states: ["focusable"], x: 400, y: 200, width: 300, height: 32, ...extra,
});
const tree = (volume: number, count: number, extra: A11yNode[] = []): A11yTree => ({
  nodes: [
    { ...node("Settings", "dialog", "Settings", { parent: "", states: [] }), valueMin: null, valueMax: null, valueNow: null },
    node("Settings/Volume", "slider", "Volume", { value: String(volume), valueMin: 0, valueMax: 100, valueNow: volume }),
    node("Settings/Count", "spinbutton", "Count", { value: String(count), valueMin: 1, valueMax: 9, valueNow: count, y: 240 }),
    node("Settings/Free", "spinbutton", "Free", { value: "-3", valueMin: null, valueMax: null, valueNow: -3, y: 280 }),
    ...extra,
  ],
  focusOrder: ["Settings/Volume", "Settings/Count", "Settings/Free"],
  focused: "Settings/Volume",
});

let uninstall: () => void;
beforeAll(() => {
  uninstall = install();
});
afterAll(() => uninstall());

async function start(): Promise<{ doc: StandInDocument; worker: StandInWorker; shell: Shell }> {
  const doc = new StandInDocument();
  const worker = new StandInWorker();
  const shell = await Shell.start({ canvas: standInCanvas(doc), worker: worker as unknown as Worker, glue: "unused.js", wasm: "unused.wasm", autoplay: false });
  return { doc, worker, shell };
}
const settle = () => new Promise((r) => setTimeout(r, 0));
const mirrored = (doc: StandInDocument, path: string) => doc.all().find((e) => e.dataset["ogPath"] === path) as StandInElement;
/** What a reader's adjust does to a native range: moves its value a step and fires `input` (bubbling). */
const adjust = (el: StandInElement, up: boolean) => {
  const range = el as StandInInput;
  range.value = String(Number(range.value) + (up ? 1 : -1) * Number(range.step || "1"));
  range.dispatchEvent(new Event("input", { bubbles: true }));
};

test("a slider's and a spinbutton's native range carry the UI's bounds and value", async () => {
  const { doc, worker, shell } = await start();
  worker.say({ type: "a11y", tree: tree(35, 4) });
  await settle();
  const volume = mirrored(doc, "Settings/Volume");
  expect(volume.tagName).toBe("INPUT");
  expect([volume.getAttribute("aria-valuemin"), volume.getAttribute("aria-valuemax"), volume.getAttribute("aria-valuenow")]).toEqual(["0", "100", "35"]);
  const count = mirrored(doc, "Settings/Count");
  expect(count.getAttribute("role")).toBe("spinbutton");
  expect([count.getAttribute("aria-valuemin"), count.getAttribute("aria-valuemax"), count.getAttribute("aria-valuenow")]).toEqual(["1", "9", "4"]);
  // A stepper without bounds says its value alone.
  const free = mirrored(doc, "Settings/Free");
  expect([free.getAttribute("aria-valuemin"), free.getAttribute("aria-valuemax"), free.getAttribute("aria-valuenow")]).toEqual([null, null, "-3"]);
  // The bounds follow the tree; a node that stops being a range loses them.
  worker.say({ type: "a11y", tree: { ...tree(35, 4), nodes: tree(35, 4).nodes.map((n) => (n.path === "Settings/Volume" ? { ...n, valueMax: 200 } : n.path === "Settings/Free" ? { ...n, role: "text", valueNow: null } : n)) } });
  await settle();
  expect(mirrored(doc, "Settings/Volume").getAttribute("aria-valuemax")).toBe("200");
  const text = mirrored(doc, "Settings/Free");
  expect(text.tagName).toBe("DIV");
  expect([text.getAttribute("aria-valuemin"), text.getAttribute("aria-valuemax"), text.getAttribute("aria-valuenow")]).toEqual([null, null, null]);
  shell.dispose();
});

test("a reader's adjust on the mirrored range asks the UI to step, and the range goes back to the UI's value", async () => {
  const { doc, worker, shell } = await start();
  worker.say({ type: "a11y", tree: tree(35, 4) });
  await settle();
  const volume = mirrored(doc, "Settings/Volume") as StandInInput;
  // Held one step from either end, so a step either way lands on one side.
  expect([volume.min, volume.value, volume.max]).toEqual(["34", "35", "36"]);
  adjust(volume, true);
  expect(worker.sent.filter((e) => e.kind === "a11y")).toEqual([{ kind: "a11y", action: "increment", path: "Settings/Volume" }]);
  expect(volume.value).toBe("35");
  const count = mirrored(doc, "Settings/Count") as StandInInput;
  adjust(count, false);
  expect(worker.sent.filter((e) => e.kind === "a11y").at(-1)).toEqual({ kind: "a11y", action: "decrement", path: "Settings/Count" });
  expect(count.value).toBe("4");
  // The UI stepped: the next tree moves the range and its value with it.
  worker.say({ type: "a11y", tree: tree(40, 3) });
  await settle();
  expect([volume.min, volume.value, volume.max, volume.getAttribute("aria-valuenow")]).toEqual(["39", "40", "41", "40"]);
  expect(count.getAttribute("aria-valuenow")).toBe("3");
  // A `change` after the `input` (Safari sends both) at the UI's value asks nothing more.
  const before = worker.sent.length;
  volume.dispatchEvent(new Event("change", { bubbles: true }));
  expect(worker.sent.length).toBe(before);
  shell.dispose();
});
