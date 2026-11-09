// A HUD document placed at the CSS safe-area insets keeps out of the web
// shell's Back (Brasswick's finding #15, Viewport.covered given here through
// og.ui.previewViewport), on the native host, on a desktop, a phone in
// portrait (Back inside its notch's safe area) and a phone in landscape: no
// element of the HUD lies under the control, and moving past it never lands
// a readout on another of the HUD's readouts more than they met without it.
// The rule is og-ui's (env(safe-area-inset-*) folds the covered region into
// one edge's inset), so each HUD here puts a different readout in the
// top-left corner under Back. The command channel's `cover` lays a native
// run out the same way (Viewport.covered without a host).
//
// Ways it could fail: a health, resource or timer readout under Back; a
// readout moved onto its neighbour; the insets not re-read when the surface
// or the covered region changes; `cover` refused, ignored or never cleared.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";
import { cookProject } from "../../tests/support/og-bin.ts";

type Rect = { x: number; y: number; width: number; height: number };
type Element = Rect & { id: string; parent: string };
type Place = "topLeft" | "top" | "topRight" | "bottomRight";

// A HUD readout: a label and its value, as wide as a health bar or a clock.
const readout = (id: string, label: string, value: string, place: Place): string =>
  `<div id="${id}" class="readout ${place}"><span>${label}</span><span class="value">${value}</span></div>`;

const HUDS: Record<string, string[]> = {
  action: [
    readout("HP", "HP", "80 / 100", "topLeft"),
    readout("Clock", "Time", "1:20", "top"),
    readout("Res", "Coins", "1,200", "topRight"),
    readout("Ammo", "Ammo", "12 / 90", "bottomRight"),
  ],
  resources: [readout("Res", "Gold", "123,456", "topLeft"), readout("Clock", "Wave", "0:40", "topRight")],
  timer: [readout("Clock", "Lap", "0:03", "topLeft"), readout("Boost", "Boost", "40", "bottomRight")],
};

// The corners as a game's HUD document writes them: 16 px inside the insets.
const STYLE = `<style>
.readout { position: fixed; width: max-content; display: flex; gap: 8px; align-items: center; padding: 6px 8px; }
.value { width: 140px; }
.topLeft { left: calc(env(safe-area-inset-left) + 16px); top: calc(env(safe-area-inset-top) + 16px); }
.top { left: calc(50% - 100px); width: 200px; top: calc(env(safe-area-inset-top) + 16px); }
.topRight { right: calc(env(safe-area-inset-right) + 16px); top: calc(env(safe-area-inset-top) + 16px); }
.bottomRight { right: calc(env(safe-area-inset-right) + 16px); bottom: calc(env(safe-area-inset-bottom) + 16px); }
</style>`;

const SURFACES = {
  desktop: { viewport: { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" }, back: { x: 12, y: 12, width: 40, height: 40 } },
  phone: { viewport: { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" }, back: { x: 12, y: 59, width: 40, height: 40 } },
  landscape: { viewport: { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" }, back: { x: 59, y: 12, width: 40, height: 40 } },
};

let root = "";

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-hud-corner-"));
  mkdirSync(join(root, "assets"));
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "HUD corner"\nid "hud-corner"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  for (const [name, readouts] of Object.entries(HUDS)) {
    writeFileSync(join(root, "assets", `${name}.svelte`), `<meta name="component" content="${name}">\n<div>${readouts.join("\n")}</div>\n${STYLE}\n`);
  }
  cookProject(root, root);
});

afterAll(() => rmSync(root, { recursive: true, force: true }));

function hud(name: string): Engine {
  const r = createEngine(og.game({
    start(world) {
      const loaded = og.load.document(world, `${name}.svelte`);
      if (!loaded.ok) throw new Error(loaded.reason);
    },
    ui(world) {
      const mounted = og.ui.scope(world, "hud").mount("hud", name, {});
      if (!mounted.ok) throw new Error(mounted.reason);
    },
  }) as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const meet = (a: Rect, b: Rect): number => Math.max(0, Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x)) * Math.max(0, Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y));

function elements(engine: Engine): Element[] {
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}

const READOUTS = new Set(["HP", "Clock", "Res", "Ammo", "Boost"]);

/** How much the HUD's readouts cover each other, in CSS px². */
function crowding(ui: Element[]): number {
  const groups = ui.filter((e) => READOUTS.has(e.id));
  let sum = 0;
  for (let i = 0; i < groups.length; i++) for (let j = i + 1; j < groups.length; j++) sum += meet(groups[i]!, groups[j]!);
  return sum;
}

for (const name of Object.keys(HUDS)) {
  test(`the ${name} HUD keeps out of the shell's Back on a desktop and on a phone`, () => {
    const engine = hud(name);
    try {
      for (const [surface, { viewport, back }] of Object.entries(SURFACES)) {
        og.ui.previewViewport(engine.world, viewport as never);
        engine.advance(3);
        const free = elements(engine);
        expect(free.filter((e) => READOUTS.has(e.id)).length).toBe(HUDS[name]!.length);
        // Without Back, the top-left readout sits under where it will be: the test means something.
        expect(free.some((e) => e.width > 0 && e.height > 0 && meet(e, back) > 0)).toBe(true);
        og.ui.previewViewport(engine.world, { ...viewport, covered: back } as never);
        engine.advance(3);
        const ui = elements(engine);
        const under = ui.filter((e) => e.width > 0 && e.height > 0 && meet(e, back) > 0).map((e) => `${e.parent}/${e.id}`);
        expect(`${surface}: ${under.join(", ")}`).toBe(`${surface}: `);
        expect(crowding(ui)).toBeLessThanOrEqual(crowding(free) + 0.5);
      }
    } finally {
      engine.destroy();
    }
  });
}

test("the command channel's cover lays a native run out as the published page (Viewport.covered without a host)", () => {
  const engine = hud("timer");
  try {
    const clock = () => elements(engine).find((e) => e.id === "Clock")!;
    engine.advance(2);
    const free = clock();
    expect(free.x).toBeLessThan(40);
    // The page's Back: 40 CSS px square at (12, 12) on a desktop page (shell.css).
    const covered = engine.command("cover", ["12,12,40,40"]);
    expect(covered.ok).toBeTrue();
    engine.advance(2);
    const moved = clock();
    expect(moved.x).toBeGreaterThanOrEqual(52);
    expect(moved.y).toBeCloseTo(free.y, 3);
    // Cleared, the readout comes back; a malformed rectangle is refused with the usage.
    expect(engine.command("cover", ["none"]).ok).toBeTrue();
    engine.advance(2);
    expect(clock().x).toBeCloseTo(free.x, 3);
    const refused = engine.command("cover", ["12,12,40"]);
    expect(refused.ok).toBeFalse();
    if (!refused.ok) expect(refused.reason).toContain("x,y,width,height");
  } finally {
    engine.destroy();
  }
});
