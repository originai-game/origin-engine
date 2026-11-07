import { resolve } from "node:path";

import { expect, setDefaultTimeout } from "bun:test";

import { og, defineSystem } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { existsSync } from "node:fs";

import { dirname, join } from "node:path";

import { fileURLToPath } from "node:url";

import { loadNativeHost } from "../../src/node/index.ts";

import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";

import { ogBin } from "./og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

export const kit = join(resolve(dirname(fileURLToPath(import.meta.url)), ".."), "..", "..", "..", "tests", "fixtures", "ui-text");

// At module level, not in a hook: the kit's string tables (Arabic for right to left), cooked incrementally.
{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

export type Element = { id: string; parent: string; path: string; kind: string; text: string; role: string; name: string; value: string; states: string[]; disabled: boolean; x: number; y: number; width: number; height: number; clipped?: boolean };

// A test here drives hundreds of ticks of a whole screen through the native host (a drag, a reveal, a scroll): 4 to 6 s
// each on the r41 host, past bun's 5 s default, so which ones timed out varied from run to run. Each is given 20 s, as
// the slowest already was; a test still fails on what it asserts, never on the machine's load.
setDefaultTimeout(20000);

export const systemIds = { next: 0 };

/** An engine whose one system builds `build` at the top start of a 1280×720 desktop surface. */
export function run(build: () => void, viewport?: Partial<Api.Viewport>, rtl = false): Engine {
  const sys = defineSystem(`UpBuild${systemIds.next++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column", padding: { left: 24, top: 24, right: 24, bottom: 24 } } as never, build);
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, rtl ? { root: kit } : {});
  if (!r.ok) throw new Error(r.reason);
  if (rtl) {
    const loaded = og.i18n.load(r.value.world, "strings/ar.ogt");
    if (!loaded.ok) throw new Error(loaded.reason);
    og.i18n.setLocale(r.value.world, "ar-EG");
  }
  if (viewport) og.ui.previewViewport(r.value.world, { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, ...viewport } as never);
  r.value.advance(6);
  return r.value;
}

export function elements(engine: Engine): Element[] {
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}

export const byPath = (engine: Engine, end: string) => elements(engine).find((e) => e.path.endsWith(end));

export const centre = (e: Element) => `${Math.round(e.x + e.width / 2)},${Math.round(e.y + e.height / 2)}`;

/** A drag from an element's centre to a point, in steps, then a release. */
export function drag(engine: Engine, from: Element, to: string, mods = ""): void {
  const [tx, ty] = to.split(",").map(Number) as [number, number];
  const [fx, fy] = centre(from).split(",").map(Number) as [number, number];
  const steps = [`0:touch_move:${fx},${fy}`];
  if (mods) steps.push(`0.02:key_down:${mods}`);
  steps.push("0.05:touch_start");
  for (let k = 1; k <= 4; k++) steps.push(`${(0.05 + k * 0.05).toFixed(2)}:touch_move:${Math.round(fx + ((tx - fx) * k) / 4)},${Math.round(fy + ((ty - fy) * k) / 4)}`);
  steps.push("0.35:touch_end");
  if (mods) steps.push(`0.4:key_up:${mods}`);
  const r = engine.probe.commands(steps.join(";")) as unknown as { warnings: string[] };
  expect(r.warnings).toEqual([]);
  engine.advance(36);
}

export function key(engine: Engine, code: string): void {
  engine.probe.commands(`0:key_down:${code};0.05:key_up:${code}`);
  engine.advance(8);
}

/** The point at the centre of cell (cx, cy) of the grid. */
export function cell(engine: Engine, cx: number, cy: number): string {
  const g = byPath(engine, "Bag/Cells")!;
  return `${Math.round(g.x + cx * 50 + 25)},${Math.round(g.y + cy * 50 + 25)}`;
}

/** Pressed on `from` and held over `to` (not let go): the drag's state that tick. */
export function hold(engine: Engine, from: Element, to: string): void {
  const [tx, ty] = to.split(",").map(Number) as [number, number];
  const [fx, fy] = centre(from).split(",").map(Number) as [number, number];
  const steps = [`0:touch_move:${fx},${fy}`, "0.05:touch_start"];
  for (let k = 1; k <= 4; k++) steps.push(`${(0.05 + k * 0.05).toFixed(2)}:touch_move:${Math.round(fx + ((tx - fx) * k) / 4)},${Math.round(fy + ((ty - fy) * k) / 4)}`);
  engine.probe.commands(steps.join(";"));
  engine.advance(24);
}

export const release = (engine: Engine) => {
  engine.probe.commands("0:touch_end");
  engine.advance(24);
};

export const cardEl = (engine: Engine, k: string) => byPath(engine, `Fan/${k}`)!;

// ── story: the conversation, the backlog, the quest log and tracker ─────

export const storyFixture = join(resolve(dirname(fileURLToPath(import.meta.url)), ".."), "fixtures", "story");

{
  const cooked = Bun.spawnSync([ogBin(), "cook", storyFixture]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
}

export const ok = <T>(r: unknown): T => {
  const x = r as { ok: boolean; value: T; reason?: string };
  if (!x.ok) throw new Error(x.reason);
  return x.value;
};

/** An engine over tests/fixtures/story (the smith, the rescue), its one system building `build(world)` on a 1280×720 desktop. */
export function storyRun(build: (w: Api.World) => void): Engine {
  let world: Api.World | null = null;
  const sys = defineSystem(`StoryBuild${systemIds.next++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "topLeft", margin: 0, width: { kind: "fixed", px: 1280 }, height: { kind: "fixed", px: 720 }, direction: "column" } as never, () => build(world!));
  });
  const r = createEngine(og.game({ seed: 42, systems: [sys] }) as unknown as GameToken, { root: storyFixture, locale: "en" });
  if (!r.ok) throw new Error(r.reason);
  world = r.value.world;
  ok(og.i18n.load(world, "strings/en.ogt"));
  ok(og.story.load(world, "story/smith.yarn"));
  ok(og.story.load(world, "story/rescue.ogt"));
  og.ui.previewViewport(world, { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 } } as never);
  r.value.advance(2);
  return r.value;
}

export const tapOn = (engine: Engine, el: Element): void => {
  engine.probe.commands(`0:touch_move:${centre(el)};0.05:touch_start;0.1:touch_end`);
  engine.advance(10);
};

// ── RTS command layer ───────────────────────────────────────────────────

export const ACT = (name: string, keys: string[]): Api.UiActionDesc => ({ name, keys, buttons: [], trigger: "press", holdMs: 0, toggle: false, label: name });
