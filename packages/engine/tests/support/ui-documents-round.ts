import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, type TextSpan } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const host = process.argv[2];
const root = process.argv[3];
if (!root) throw new Error("expected cooked document project");
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
} else {
  throw new Error("expected node or web host");
}

const spans: TextSpan[] = [
  { name: "emphasis", position: 2, length: 1, properties: { large: true } },
  { name: "unknown", position: 0, length: 0, properties: { marker: 7 } },
];
const created = createEngine(og.game({
  start(world) {
    const strings = og.i18n.load(world, "game.en.ogt");
    if (!strings.ok) throw new Error(strings.reason);
    const document = og.load.document(world, "card.svelte");
    if (!document.ok) throw new Error(document.reason);
    const shown = og.ui.push(world, "card", { body: "A🦊B <b>C</b>", spans, classes: "other chosen" });
    if (!shown.ok) throw new Error(shown.reason);
  },
}) as unknown as GameToken, { root });
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
function element(id: string) {
  const dump: unknown = engine.probe.dump({ what: "ui" });
  if (!dump || typeof dump !== "object" || !("elements" in dump) || !Array.isArray(dump.elements)) throw new Error("missing UI dump");
  for (const item of dump.elements) {
    if (item && typeof item === "object" && "id" in item && item.id === id && "text" in item && typeof item.text === "string" && "height" in item && typeof item.height === "number" && "width" in item && typeof item.width === "number" && "x" in item && typeof item.x === "number") {
      return { text: item.text, height: item.height, width: item.width, x: item.x };
    }
  }
  throw new Error(`missing element ${id}`);
}
try {
  const advanced = engine.advance(2);
  if (!advanced.ok) throw new Error(advanced.reason);
  const rich = element("Rich"), plain = element("Plain");
  const saved = engine.world.snapshot();
  if (!saved.ok) throw new Error(saved.reason);
  engine.advance(1);
  const anim = element("Anim").x;
  engine.advance(3);
  const restored = engine.world.restore(saved.value);
  if (!restored.ok) throw new Error(restored.reason);
  engine.advance(1);
  console.log(JSON.stringify({ title: element("Title").text, base: element("Base").text, body: rich.text, larger: rich.height > plain.height, restored: JSON.stringify(element("Rich")) === JSON.stringify(rich), widths: ["Inherited", "Nested", "Sibling", "Last"].map(id => element(id).width), animMoved: anim - element("Last").x, animX: anim, animRestored: element("Anim").x === anim, queried: element("Sibling").height }));
} finally {
  engine.destroy();
}
