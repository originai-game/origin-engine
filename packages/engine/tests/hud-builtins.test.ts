// Exercise the documents through the native host, as a game consumes them.
// The game supplies resources, phases, combo outcomes and sampled metrics.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const PATHS = ["base/pips", "hud/boss-bar", "hud/combo", "hud/pickup-feed", "hud/perf-readout"];
const members = PATHS.map((path) => {
  const text = readFileSync(join(ENGINE, `src/ui/components/${path}.svelte`), "utf8");
  const name = /name="component" content="([^"]+)"/.exec(text)![1]!;
  const fixtures = Object.fromEntries([...text.matchAll(/name="fixture:([^"]+)" content='([^']*)'/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
  return { name, fixtures };
});
let root = "";
type El = { path: string; text: string; value: string; role: string; name: string; disabled: boolean; states: string[]; x: number; y: number; width: number; height: number };
beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-hud-builtins-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "HUD builtins"\nid "hud-builtins"\nentry "main.ts"\nprofile "2d"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  for (const m of members) {
    const props = Object.keys(m.fixtures.default).map((p) => `${p}={$${p === "state" ? "statusValue" : p}}`).join(" ");
    writeFileSync(join(root, `assets/ui/${m.name}.svelte`), `<meta name="screen" content="Host_${m.name}" data-layer="menu" data-input="menu">\n<div id="Page" style="width: stretch; padding: 16px"><${m.name} ${props} /></div>`);
  }
  const cooked = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, stdout: "ignore", stderr: "pipe" });
  if (cooked.exitCode) throw new Error(new TextDecoder().decode(cooked.stderr));
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });

function mount(name: string, stores: Record<string, unknown>, phone = false): Engine {
  const r = createEngine(og.game({
    start(w) {
      og.ui.configure(w, { style: null });
      const d = og.load.document(w, `ui/${name}.svelte`);
      if (!d.ok) throw new Error(d.reason);
    },
    ui(w) {
      const scope = og.ui.scope(w, "test");
      const inputs = Object.fromEntries(Object.entries(stores).map(([k, v]) => [k === "state" ? "statusValue" : k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const m = scope.mount("page", `Host_${name}`, inputs);
      if (!m.ok) throw new Error(m.reason);
    },
  }) as unknown as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  og.ui.previewViewport(r.value.world, { width: phone ? 390 : 1280, height: phone ? 844 : 720, dpr: 1, safeArea: { left: 0, right: 0, top: 0, bottom: 0 }, coarsePointer: phone, breakpoint: phone ? "phonePortrait" : "desktopHd" });
  r.value.advance(3);
  return r.value;
}
const elements = (e: Engine) => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements;
const words = (e: Engine) => elements(e).filter((x) => x.width > 0).map((x) => x.text);
function check(name: string, fixture: string, fn: (e: Engine) => void, over: Record<string, unknown> = {}) {
  const m = members.find((x) => x.name === name)!;
  const e = mount(name, { ...m.fixtures.default, ...m.fixtures[fixture], ...over });
  try { fn(e); } finally { e.destroy(); }
}

for (const phone of [false, true]) for (const m of members) test(`${m.name}: all fixtures are readable in default style on ${phone ? "phone" : "desktop"}`, () => {
  for (const [name, fixture] of Object.entries(m.fixtures)) {
    const e = mount(m.name, { ...m.fixtures.default, ...fixture as object }, phone);
    try {
      expect(og.ui.scope(e.world, "test").takeErrors()).toEqual([]);
      const warnings: string[] = JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id)).warnings ?? [];
      expect(warnings).toEqual([]);
      const boxes = elements(e).filter((x) => x.width > 0);
      expect(boxes.length).toBeGreaterThan(2);
      for (const b of boxes) expect(`${m.name}/${name} ${b.path}: ${b.x + b.width <= (phone ? 390 : 1280) + 0.5}`).toEndWith("true");
    } finally { e.destroy(); }
  }
}, 120_000);

test("resource displays distinguish fractional/empty units, phases and game-supplied outcomes", () => {
  check("Pips", "default", (e) => expect(elements(e).filter((x) => x.role === "meter").map((x) => x.value)).toEqual(["100%", "50%", "0%"]));
  check("Pips", "empty", (e) => expect(words(e)).toContain("No units"));
  check("BossBar", "default", (e) => {
    expect(words(e)).toContain("7,200 / 10,000");
    expect(words(e)).toContain("Current");
    expect(words(e)).toContain("Complete");
  });
  check("BossBar", "default", (e) => {
    expect(words(e)).toContain("0 / 0");
    expect(words(e)).not.toContain("Defeated");
  }, { max: 0 });
  check("Combo", "default", (e) => {
    expect(words(e)).toContain("17");
    expect(words(e)).toContain("×2.5");
    expect(words(e)).toContain("Active");
  }, { remaining: 0 });
  check("PickupFeed", "default", (e) => {
    expect(words(e)).toContain("+12");
    expect(words(e)).toContain("-40");
  });
  check("PerfReadout", "connecting", (e) => {
    expect(words(e).filter((x) => x === "Unavailable")).toHaveLength(4);
    expect(words(e)).not.toContain("0");
    expect(words(e)).not.toContain("0 ms");
  });
  check("PerfReadout", "zero", (e) => {
    expect(words(e)).toContain("0");
    expect(words(e)).toContain("0 ms");
    expect(words(e)).not.toContain("Unavailable");
  });
  check("PerfReadout", "default", (e) => expect(words(e)).toContain("16.7 ms"));
  for (const name of ["Pips", "PickupFeed"]) check(name, "loading", (e) => {
    expect(elements(e).some((x) => x.role === "progressbar" && x.name === "Loading")).toBeTrue();
    expect(words(e)).toContain("Loading");
    expect(words(e)).not.toContain("+12");
    expect(elements(e).filter((x) => x.role === "meter")).toHaveLength(0);
  });
});
