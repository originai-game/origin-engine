import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect } from "bun:test";
import { og, type Result, type UiScope, type UiWritable, type UiCollection, type UiReadable } from "@origingame/engine";
import { beginBundle, createEngine, type GameToken } from "../../src/runtime.ts";

const host = process.argv[2], root = process.argv[3];
if (!root) throw new Error("expected cooked mount project");
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

let scope!: UiScope, title!: UiWritable<string>;
let rows!: UiCollection<{ id: string; label: string }>;
let sensor!: UiReadable<number>;
let starts = 0, stops = 0;
beginBundle();
const engine = value(createEngine(og.game({
  start(world) { value(og.load.document(world, "mount.svelte")); },
  ui(world) {
    scope = og.ui.scope(world, "mounted");
    title = scope.writable("title", "First");
    rows = scope.collection("rows", [{ id: "a", label: "Alpha" }, { id: "b", label: "Beta" }]);
    sensor = scope.readable("sensor", 0, set => { starts++; set(17); return () => { stops++; }; });
    value(scope.mount("view", "mount", { title, rows, sensor }));
  },
}) as unknown as GameToken, { root }));

function labels(): string[] {
  const dump: unknown = engine.probe.dump({ what: "ui" });
  if (!dump || typeof dump !== "object" || !("elements" in dump) || !Array.isArray(dump.elements)) throw new Error("missing UI dump");
  return dump.elements.flatMap((element: unknown) => {
    if (element && typeof element === "object" && "kind" in element && element.kind === "label"
      && "text" in element && typeof element.text === "string") return [element.text];
    return [];
  });
}
try {
  value(engine.advance(1));
  expect(labels()).toEqual(["First", "17", "Alpha", "Beta"]);
  value(scope.mount("view", "mount", { title, rows, sensor }));
  expect([starts, stops]).toEqual([1, 0]);
  const hash = engine.hash();
  value(title.set("Second"));
  value(rows.move("b", "a"));
  expect(engine.hash()).toBe(hash);
  value(engine.advance(1));
  expect(labels()).toEqual(["Second", "17", "Beta", "Alpha"]);
  const saved = value(engine.world.snapshot());
  const probeSaved = engine.probe.snapshot();
  if (!probeSaved) throw new Error("missing probe snapshot");
  value(title.set("Third"));
  value(rows.remove("a"));
  value(engine.advance(1));
  expect(labels()).toEqual(["Third", "17", "Beta"]);
  value(engine.world.restore(saved));
  value(engine.advance(1));
  expect(labels()).toEqual(["Second", "17", "Beta", "Alpha"]);
  value(scope.unmount("view"));
  value(engine.advance(1));
  expect(labels()).toEqual([]);
  expect([starts, stops]).toEqual([1, 1]);
  value(scope.unmount("view"));
  expect([starts, stops]).toEqual([1, 1]);
  const unmounted = value(engine.world.snapshot());
  value(engine.world.restore(saved));
  value(engine.advance(1));
  expect(labels()).toEqual(["Second", "17", "Beta", "Alpha"]);
  expect([starts, stops]).toEqual([2, 1]);
  value(engine.world.restore(unmounted));
  value(engine.advance(1));
  expect(labels()).toEqual([]);
  expect([starts, stops]).toEqual([2, 2]);
  expect(engine.probe.restore(probeSaved.id)).not.toBeNull();
  value(engine.advance(1));
  expect(labels()).toEqual(["Second", "17", "Beta", "Alpha"]);
  expect([starts, stops]).toEqual([3, 2]);
  value(scope.unmount("view"));
  beginBundle();
  og.game({
    start() { throw new Error("reload ran gameplay start"); },
    ui(world) {
      scope = og.ui.scope(world, "mounted");
      title = scope.writable("new-title", "Replacement");
      rows = scope.collection("rows", []);
      sensor = scope.readable("sensor", 0, set => { starts++; set(17); return () => { stops++; }; });
      value(scope.mount("replacement", "mount", { title, rows, sensor }));
    },
  });
  expect(engine.probe.reload()).not.toBeNull();
  value(engine.advance(1));
  expect(labels()).toEqual(["Replacement", "17", "Beta", "Alpha"]);
  value(title.set("Changed"));
  value(engine.world.restore(saved));
  value(engine.advance(1));
  expect(labels()).toEqual(["Replacement", "17", "Beta", "Alpha"]);
  value(scope.dispose());
  value(engine.world.restore(saved));
  value(engine.advance(1));
  expect(labels()).toEqual([]);
  expect([starts, stops]).toEqual([4, 4]);
  expect(scope.takeErrors()).toEqual([]);
  console.log(JSON.stringify({ restored: true, starts, stops, labels: labels() }));
} finally { engine.destroy(); }
