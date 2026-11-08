import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect } from "bun:test";
import { og, type Result, type UiScope, type UiWritable, type UiCollection } from "@origingame/engine";
import { beginBundle, createEngine, type GameToken } from "../../src/runtime.ts";

// One run of the retained list proof on one host (node or web): a 1,000-row
// <list> scrolled to row 999 and back by the wheel with the focus held on
// row 999 by key, reordered, a row removed, the world restored, and a rich
// text's spans and body bound to stores. It prints what each step left, so
// the two hosts are compared whole.
const host = process.argv[2], root = process.argv[3];
if (!root) throw new Error("expected cooked list project");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else if (host === "web") {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import("../../dist/web/origin-engine.js");
  await loadWebHost(glue, readFileSync(fileURLToPath(new URL("../../dist/web/origin-engine_bg.wasm", import.meta.url))));
  function preload(directory: string): void {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) preload(path);
      else if (entry.isFile()) glue.og_web_put_file(path, readFileSync(path));
    }
  }
  preload(join(root, "cooked"));
} else throw new Error("expected node or web host");

function value<T>(result: Result<T>): T {
  if (!result.ok) throw new Error(result.reason);
  return result.value;
}

type Span = { name: string; position: number; length: number; properties: Record<string, unknown> };
let scope!: UiScope;
let rows!: UiCollection<{ id: string; label: string }>;
let body!: UiWritable<string>, spans!: UiWritable<Span[]>;
beginBundle();
const engine = value(createEngine(og.game({
  start(world) { value(og.load.document(world, "list.svelte")); },
  ui(world) {
    scope = og.ui.scope(world, "listed");
    rows = scope.collection("rows", Array.from({ length: 1000 }, (_, i) => ({ id: `r${i}`, label: `Item ${i}` })));
    body = scope.writable("body", "Plain words");
    spans = scope.writable<Span[]>("spans", []);
    value(scope.mount("view", "list", { rows, body, spans }));
  },
}) as unknown as GameToken, { root }));

type Element = { path: string; kind: string; text?: string; states: string[]; x: number; y: number; width: number; height: number; bindings?: unknown };
function elements(): Element[] {
  const dump: unknown = engine.probe.dump({ what: "ui" });
  if (!dump || typeof dump !== "object" || !("elements" in dump) || !Array.isArray(dump.elements)) throw new Error("missing UI dump");
  return dump.elements as Element[];
}
// The rows in view, by their labels, top first.
function shown(): string[] {
  return elements().filter(e => e.kind === "button" && typeof e.text === "string").map(e => e.text as string);
}
function rowPath(label: string): string {
  const row = elements().find(e => e.kind === "button" && e.text === label);
  if (!row) throw new Error(`${label} is not in view: ${shown().join(", ")}`);
  return row.path;
}
// The element in view that has the focus ("" when none does: the focused
// row is held by its key while it is not built), after one more frame.
function focusOf(): string {
  value(engine.advance(1));
  return elements().find(e => e.states.includes("focused"))?.path ?? "";
}
function click(path: string): void {
  const e = elements().find(x => x.path === path);
  if (!e) throw new Error(`${path} is not in view`);
  const x = e.x + e.width / 2, y = e.y + e.height / 2;
  if (!engine.probe.input({ kind: "pointer", action: "down", x, y, button: "left" } as never)) throw new Error("pointer refused");
  value(engine.advance(1));
  if (!engine.probe.input({ kind: "pointer", action: "up", x, y, button: "left" } as never)) throw new Error("pointer refused");
  value(engine.advance(1));
}
function wheel(deltaY: number, ticks: number): void {
  for (let t = 0; t < ticks; t++) {
    engine.probe.input({ kind: "wheel", x: 40, y: 120, deltaX: 0, deltaY } as never);
    value(engine.advance(1));
  }
}
const report: Record<string, unknown> = {};
try {
  value(engine.advance(1));
  report.first = shown();
  expect(shown()[0]).toBe("Item 0");
  // Down to the last row, focus it, and back to the top: the focus stays
  // on row 999 by its key while the row is not built.
  wheel(960, 40);
  report.bottom = shown();
  expect(shown()).toContain("Item 999");
  const far = rowPath("Item 999");
  click(far);
  expect(focusOf()).toBe(far);
  wheel(-960, 40);
  report.top = shown();
  expect(shown()[0]).toBe("Item 0");
  expect(shown()).not.toContain("Item 999");
  report.heldAway = focusOf();
  wheel(960, 40);
  expect(focusOf()).toBe(far);
  report.heldBack = focusOf();
  // Reorder and remove in view; the world restored brings both back.
  const saved = value(engine.world.snapshot());
  value(rows.move("r999", "r990"));
  value(engine.advance(1));
  report.moved = shown();
  value(rows.remove("r995"));
  value(engine.advance(1));
  report.removed = shown();
  expect(shown()).not.toContain("Item 995");
  value(engine.world.restore(saved));
  value(engine.advance(1));
  report.restored = shown();
  expect(report.restored).toEqual(report.bottom);
  expect(focusOf()).toBe(far);
  // <text spans={$spans}>{$body}</text>: both bindings land on the label.
  value(body.set("Gold and silver"));
  value(spans.set([{ name: "gold", position: 0, length: 4, properties: {} }]));
  value(engine.advance(1));
  const rich = elements().find(e => e.path.endsWith("Rich"));
  report.rich = { text: rich?.text, bindings: rich?.bindings };
  expect(rich?.text).toBe("Gold and silver");
  expect(scope.takeErrors()).toEqual([]);
  console.log(JSON.stringify(report));
} finally { engine.destroy(); }
