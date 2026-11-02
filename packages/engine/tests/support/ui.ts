// The UI tests' shared harness: an engine whose one system builds the UI
// every tick with the viewport pinned, and the UI's dump by path.
import { og, defineSystem } from "@origingame/engine";
import type { DrawDesc, World } from "@origingame/engine";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";
import { kit } from "../../src/ui-kit/bind.ts";

export type El = {
  id: string; parent: string; path: string; text: string; role: string; name: string; value: string;
  x: number; y: number; width: number; height: number; visualStates?: Record<string, string>;
};

const DESKTOP = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };

/** An engine whose one system calls `build` every tick, with the UI's viewport pinned. */
export function engineFor(name: string, build: (world: World) => void, viewport: object = DESKTOP, reads: readonly unknown[] = []): Engine {
  const sys = defineSystem(name, { reads } as never, (_q, _t, world: World) => build(world));
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken);
  if (!r.ok) throw new Error(r.reason);
  og.ui.previewViewport(r.value.world, viewport as never);
  return r.value;
}

export const dump = (e: Engine): El[] => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements;
export const at = (els: El[], path: string): El | undefined => els.find((x) => x.path === path);

/**
 * Every draw og.ui's components give while `run` runs (og.ui.setDraw), by the
 * path of the element open at the time: what a part is drawn with (its shape,
 * its fills), which the probe's dump does not carry.
 */
export function drawsDuring(run: () => void): Map<string, DrawDesc[]> {
  const out = new Map<string, DrawDesc[]>();
  const was = kit.setDraw;
  const k = kit as { setDraw: (d?: DrawDesc) => void };
  k.setDraw = (d?: DrawDesc) => {
    const p = kit.openPath();
    out.set(p, [...(out.get(p) ?? []), d ?? {}]);
    was(d);
  };
  try {
    run();
  } finally {
    k.setDraw = was;
  }
  return out;
}
