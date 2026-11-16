import { afterAll, beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { og } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

const source = readFileSync(new URL("../src/ui/components/base/currency.svelte", import.meta.url), "utf8");
const fixtures = Object.fromEntries([...source.matchAll(/<meta name="fixture:([^\"]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
let root = "";
beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-currency-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Currency test"\nid "currency-test"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});');
  const props = [...source.matchAll(/name="prop:([^\"]+)"/g)].map((m) => `${m[1]}={$${m[1]}}`).join(" ");
  writeFileSync(join(root, "assets/ui/page.svelte"), `<meta name="component" content="CurrencyTest"><div style="padding: 16px; width: stretch"><Currency ${props} /></div>`);
  const cooked = Bun.spawnSync([ogBin(), "cook", root], { stdout: "ignore", stderr: "pipe" });
  if (cooked.exitCode) throw new Error(new TextDecoder().decode(cooked.stderr));
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });

for (const width of [1280, 390]) test(`Currency: all balances/states at ${width}px, no stale figures or overflow`, () => {
  for (const [name, values] of Object.entries(fixtures)) {
    const data = { ...fixtures.default, ...values };
    const result = createEngine(og.game({
      start(world) {
        const loaded = og.load.document(world, "ui/page.svelte");
        if (!loaded.ok) throw new Error(loaded.reason);
      },
      ui(world) {
        const scope = og.ui.scope(world, "currency");
        const mounted = scope.mount("page", "CurrencyTest", Object.fromEntries(Object.entries(data).map(([k, v]) => [k, scope.writable(k, v)])));
        if (!mounted.ok) throw new Error(mounted.reason);
      },
    }) as unknown as GameToken, { root });
    if (!result.ok) throw new Error(result.reason);
    const e = result.value;
    try {
      og.ui.previewViewport(e.world, { width, height: 844, dpr: 1, safeArea: { top: 0, bottom: 0, left: 0, right: 0 }, coarsePointer: width < 600, breakpoint: width < 600 ? "phonePortrait" : "desktop" } as never);
      e.advance(3);
      expect(og.ui.scope(e.world, "currency").takeErrors()).toEqual([]);
      const frame = JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id));
      expect((frame.warnings ?? []).filter((w: string) => w.includes("computed-value"))).toEqual([]);
      const els = (e.probe.dump({ what: "ui" }) as unknown as { elements: { text: string; width: number; height: number; x: number; role: string; clipped?: boolean }[] }).elements.filter((x) => x.width > 0 && x.height > 0);
      const texts = els.map((x) => x.text);
      expect(texts).toContain(data.name);
      expect(els.some((x) => x.role === "button")).toBeFalse();
      if (name === "loading" || name === "unavailable") {
        expect(texts).toContain(name === "loading" ? "Loading" : "Unavailable");
        expect(texts).not.toContain("999");
        expect(texts).not.toContain("+48/s");
      } else if (name === "empty") expect(texts).toContain("0");
      else if (name === "full") expect(texts).toContain("Full");
      else if (name === "overflow") expect(texts).toContain("123.45 quattuordecillion");
      for (const el of els.filter((x) => x.text)) {
        expect(`${name}: ${el.text} fits`).toBe(`${name}: ${el.text} ${el.x >= 0 && el.x + el.width <= width + 0.5 && !el.clipped ? "fits" : "overflows"}`);
      }
    } finally { e.destroy(); }
  }
}, 60_000);
