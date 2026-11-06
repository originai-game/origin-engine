import { resolve } from "node:path";

import { existsSync } from "node:fs";

import { dirname, join } from "node:path";

import { fileURLToPath } from "node:url";

import { defineSystem, og } from "@origingame/engine";

import type { World } from "@origingame/engine";

import { loadNativeHost } from "../../src/node/index.ts";

import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";

import { ogBin } from "./og-bin.ts";

import { type El } from "./ui.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

export const kit = join(resolve(dirname(fileURLToPath(import.meta.url)), ".."), "..", "..", "..", "tests", "fixtures", "ui-text");

/** A desktop surface with a 40 px top and 30 px right safe inset (a TV's overscan). */
export const DESKTOP_SAFE = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 40, right: 30, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };

{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

// The frames and the misc HUD tests are CPU-bound and nothing else: ticks of
// a dozen components on the dev (-O0) host, no sleep and no wall-clock wait.
// 1.3–1.6 s idle; a loaded gate ran them past 8 s, so the limit leaves that
// headroom three times over. Each advances only as far as its script needs:
// the tick is fixed (60 Hz), so the counts hold on any machine.
export const LOADED_MS = 30_000;

export const below = (els: El[], prefix: string): El[] => els.filter((x) => x.path.startsWith(`${prefix}/`));

export const states = (el: El | undefined): Record<string, string> => el?.visualStates ?? {};

/** An engine over the UI kit's project, its string tables loaded (Arabic lays out right to left). */
export function kitEngine(name: string, build: (world: World) => void): Engine {
  const sys = defineSystem(name, {}, (_q, _t, world: World) => build(world));
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, { root: kit });
  if (!r.ok) throw new Error(r.reason);
  for (const l of ["en", "ar"]) {
    const s = og.i18n.load(r.value.world, `strings/${l}.ogt`);
    if (!s.ok) throw new Error(s.reason);
  }
  og.ui.previewViewport(r.value.world, { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" } as never);
  return r.value;
}

export const VIEWPORTS = {
  desktop: { width: 1920, height: 1080, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" },
  ultrawide: { width: 3440, height: 1440, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" },
  tablet: { width: 1024, height: 768, dpr: 2, safeArea: { left: 0, top: 24, right: 0, bottom: 20 }, coarsePointer: true, breakpoint: "tablet" },
  phonePortrait: { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" },
  phoneLandscape: { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" },
};
