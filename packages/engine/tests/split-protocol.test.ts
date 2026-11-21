// The split-wasm placeholder protocol is one module (src/web/split-protocol.js),
// read by the engine Worker's loader (src/web/split.ts) and, copied beside
// wasm-bindgen-rayon's helper by scripts/rayon-worker.sh, by the lanes pool
// Workers' installer (scripts/og-pool-split.js). This pins both readers to
// the one module and the protocol's pieces to each other: no reader restates
// the placeholders regex or the namespace, the installer's copy imports it
// relatively the way the deployed layout lays the files out, and the
// namespace and the imports wrapper answer as the split wasm asks.
import { expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const engine = join(dirname(fileURLToPath(import.meta.url)), "..");
const protocol = join(engine, "src/web/split-protocol.js");
const loader = readFileSync(join(engine, "src/web/split.ts"), "utf8");
const installer = readFileSync(join(engine, "scripts/og-pool-split.js"), "utf8");

test("the placeholders regex and the namespace are stated once, in the protocol module", () => {
  const once = readFileSync(protocol, "utf8");
  expect([...once.matchAll(/%table_\\d/g)].length).toBe(1);
  expect([...once.matchAll(/"placeholder\."/g)].length).toBe(1);
  for (const [name, source] of [["split.ts", loader], ["og-pool-split.js", installer]] as const) {
    expect(source.includes("%table_\\d"), name).toBe(false);
    expect(source.includes('"placeholder."'), name).toBe(false);
  }
});

test("the pool installer imports the protocol relatively, as the helper's directory lays it out", () => {
  expect(installer.includes('from "./split-protocol.js"')).toBe(true);
});

test("the protocol answers the split wasm's asks", async () => {
  const { NAMESPACE, PLACEHOLDERS, moduleUrl, placeholderTable, placeholderNamespace, withPlaceholderNamespaces } = await import(
    pathToFileURL(protocol).href
  );
  expect(PLACEHOLDERS.test("%table_12")).toBe(true);
  expect(PLACEHOLDERS.test("%table")).toBe(false);
  expect(NAMESPACE).toBe("placeholder.");

  const table = new WebAssembly.Table({ element: "anyfunc", initial: 2 });
  expect(placeholderTable({ "%table_0": table, other: 1 })).toBe(table);
  expect(placeholderTable({ other: 1 })).toBeNull();

  // The namespace: each slot loads its module once and calls the slot's function.
  const slots = new Map([
    [0, (...args: unknown[]) => `slot0(${args.join(",")})`],
    [2, () => "slot2"],
  ]);
  const fake = { get: (index: number) => slots.get(index) } as unknown as WebAssembly.Table;
  const loaded: string[] = [];
  const load = (name: string) => {
    if (!loaded.includes(name)) loaded.push(name);
  };
  const ns = placeholderNamespace("physics", load, () => fake);
  expect((ns[0] as (n: number) => string)(7)).toBe("slot0(7)");
  expect((ns[0] as (n: number) => string)(8)).toBe("slot0(8)");
  expect(loaded).toEqual(["physics"]);
  expect(ns[2]).toBeInstanceOf(Function);
  expect((ns as unknown as Record<symbol | string, unknown>)[Symbol("other")]).toBeUndefined();

  // The imports wrapper: the namespaces a split primary asks for are added,
  // everything given stays.
  const calls: string[] = [];
  const given = { env: { memory: 1 } };
  const wrapped = withPlaceholderNamespaces(given, (name: string) => calls.push(name), () => fake);
  const placeholders = (wrapped as Record<string, unknown>)[`${NAMESPACE}nav`] as Record<string, unknown>;
  expect(placeholders).toBeInstanceOf(Object);
  expect((wrapped as Record<string, unknown>).env).toEqual({ memory: 1 });
  expect((wrapped as Record<string, unknown>)[`${NAMESPACE}nav`]).toBe(placeholders);

  // A module's file: the build's, else modules/<name>.wasm under the base.
  expect(moduleUrl("physics", { physics: "m/engine-physics.wasm" }, "https://cdn/x/").href).toBe("https://cdn/x/m/engine-physics.wasm");
  expect(moduleUrl("nav", {}, "https://cdn/x/").href).toBe("https://cdn/x/modules/nav.wasm");
});

test("the pool installer runs from the deployed layout: the helper's directory beside the protocol", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-pool-split-"));
  try {
    writeFileSync(join(dir, "og-pool-split.js"), installer);
    writeFileSync(join(dir, "split-protocol.js"), readFileSync(protocol, "utf8"));
    const { installPoolSplit } = await import(pathToFileURL(join(dir, "og-pool-split.js")).href);
    expect(installPoolSplit(null)).toBeUndefined();
    // An unsplit primary (no split handed) leaves WebAssembly alone.
    const original = WebAssembly.instantiate;
    installPoolSplit({ base: "https://cdn/x/", files: {} });
    expect(WebAssembly.instantiate).not.toBe(original);
    WebAssembly.instantiate = original;
  } finally {
    rmSync(dir, { recursive: true });
  }
});
