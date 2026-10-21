// The text entry's textarea (src/web/text-input.ts) as assistive
// technology meets it (examples/WEB-FINDINGS.md #7): the main-thread shell
// (src/web/shell.ts) started against a stand-in Worker that answers its
// start and then sends what the engine Worker sends when the UI focuses a
// text field (`text`, then the frame's `a11y` tree), over a stand-in
// document (tests/support/dom.ts). No browser.
//
// The ways this could fail, each asserted below: the focused textarea, or
// an ancestor of it, is aria-hidden (Chrome blocks that, "Blocked
// aria-hidden on an element because its descendant retained focus", and a
// reader lands on an unnamed box); it is not named after the field; a
// reader meets the field twice (the mirror's element and the textarea);
// the textarea sits outside the dialog it belongs to; the name does not
// follow when the tree arrives after the focus, or the field's text
// changes; letting go leaves the textarea exposed or the field hidden.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { Shell } from "../src/web/shell.ts";
import type { A11yNode, A11yTree } from "../src/web/a11y.ts";
import type { TextTarget } from "../src/web/text-input.ts";
import { exposed, install, roleOf, StandInDocument, standInCanvas, type StandInElement } from "./support/dom.ts";

/** The engine Worker as the shell holds it: answers `start` with `ready`; `say` sends it a message. */
class StandInWorker extends EventTarget {
  postMessage(m: { type?: string }): void {
    if (m?.type === "start") setTimeout(() => this.say({ type: "ready", tier: "webgpu", lanes: { on: false, threads: 1, why: "a test" } }), 0);
  }
  say(data: unknown): void {
    this.dispatchEvent(new MessageEvent("message", { data }));
  }
  terminate(): void {}
}

const DIALOG = "Naming";
const FIELD = "Naming/Body/Name";
const node = (path: string, parent: string, role: string, name: string, extra: Partial<A11yNode> = {}): A11yNode => ({
  path, parent, role, name, description: "", value: "", states: [], x: 420, y: 200, width: 440, height: 40, ...extra,
});
const tree = (value: string): A11yTree => ({
  nodes: [
    node(DIALOG, "", "dialog", "Name your bell-keeper", { y: 160, height: 260 }),
    node(FIELD, DIALOG, "textbox", "Name", { description: "Wren", value, states: ["focusable", "focused"], y: 240 }),
    node("Naming/Body/Begin", DIALOG, "button", "Begin", { states: ["focusable"], y: 320 }),
  ],
  focusOrder: [FIELD, "Naming/Body/Begin"],
  focused: FIELD,
});
const target: TextTarget = { path: FIELD, rect: [420, 240, 440, 40], multiline: false, selection: "", composing: false };

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
/** Lets the shell's message handler run. */
const settle = () => new Promise((r) => setTimeout(r, 0));
/** The text boxes a reader can reach. */
const boxes = (doc: StandInDocument) => doc.all().filter((e) => roleOf(e) === "textbox" && exposed(e));
/** The mirror's element of a path. */
const mirrored = (doc: StandInDocument, path: string) => doc.all().find((e) => e.dataset["ogPath"] === path) as StandInElement;

for (const order of ["the tree first", "the focus first"] as const) {
  test(`a focused text field: the focus is on a named, exposed text box, the field's own, inside its dialog (${order})`, async () => {
    const { doc, worker, shell } = await start();
    // Before any field is focused the textarea is no text box a reader meets.
    worker.say({ type: "a11y", tree: { nodes: [node(DIALOG, "", "dialog", "Name your bell-keeper")], focusOrder: [], focused: "" } });
    await settle();
    expect(boxes(doc).length).toBe(0);
    // The engine Worker posts `text` before the frame's `a11y` (worker.ts relay); a later frame may bring the tree.
    const messages = [{ type: "text", target }, { type: "a11y", tree: tree("") }];
    for (const m of order === "the focus first" ? messages : messages.reverse()) {
      worker.say(m);
      await settle();
    }
    const area = doc.activeElement!;
    expect(area.tagName).toBe("TEXTAREA");
    // Chrome's rule: no aria-hidden on the focused element or above it.
    expect(exposed(area)).toBe(true);
    expect(area.getAttribute("aria-label")).toBe("Name");
    expect(area.getAttribute("aria-placeholder")).toBe("Wren");
    expect(area.getAttribute("aria-multiline")).toBe("false");
    // One text box, the focused one: the mirror's element for the field steps aside.
    expect(boxes(doc)).toEqual([area]);
    expect(mirrored(doc, FIELD).getAttribute("aria-hidden")).toBe("true");
    // Inside its dialog for a reader: the dialog's element owns the textarea.
    expect(mirrored(doc, DIALOG).getAttribute("aria-owns")).toBe(area.id);
    expect(doc.byId(area.id)).toBe(area);
    // What the field holds is read with it.
    worker.say({ type: "a11y", tree: tree("Mei") });
    await settle();
    expect(area.getAttribute("aria-description")).toBe("Mei");
    // The field lets go: the textarea leaves the reader's way, the field comes back.
    worker.say({ type: "text", target: null });
    await settle();
    expect(doc.activeElement).not.toBe(area);
    expect(area.getAttribute("aria-hidden")).toBe("true");
    expect(area.getAttribute("aria-label")).toBeNull();
    expect(mirrored(doc, FIELD).getAttribute("aria-hidden")).toBeNull();
    expect(mirrored(doc, DIALOG).getAttribute("aria-owns")).toBeNull();
    expect(boxes(doc).map((b) => b.dataset["ogPath"])).toEqual([FIELD]);
    shell.dispose();
  });
}

test("a field focused before the mirror has it is named by its id until the tree arrives", async () => {
  const { doc, worker, shell } = await start();
  worker.say({ type: "text", target });
  await settle();
  const area = doc.activeElement!;
  expect(exposed(area)).toBe(true);
  expect(area.getAttribute("aria-label")).toBe("Name");
  worker.say({ type: "a11y", tree: { ...tree(""), nodes: tree("").nodes.map((n) => (n.path === FIELD ? { ...n, name: "Your name" } : n)) } });
  await settle();
  expect(area.getAttribute("aria-label")).toBe("Your name");
  shell.dispose();
  // Disposed: nothing of the shell stays in the document.
  expect(doc.all().filter((e) => e.tagName !== "CANVAS")).toEqual([]);
});
