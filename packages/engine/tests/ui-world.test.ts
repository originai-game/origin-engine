// Mass element timing on the native host (og.ui.defineMass). The world
// tags, ground marks and mass components are component documents now
// (src/ui/components), cooked and built by og-ui's component document test.
import { expect, test } from "bun:test";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { engineFor } from "./support/ui.ts";

loadNativeHost();

const DESKTOP = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };
const engine = (name: string, build: (w: World) => void) => engineFor(name, build, DESKTOP, [og.Camera]);

test("mass timing accepts shared curves and refuses removed or invalid names", () => {
  const e = engine("MassTiming", () => {
    for (const ease of ["cubic-bezier(0.17,0.85,0.63,0.08)", "steps(4,start)", "power1.out"] as const) {
      expect(og.ui.defineMass(ease, { kind: "particle", keys: [{ t: 1, ease }] }).ok).toBe(true);
    }
    for (const ease of ["out", "step", "steps(0)", "cubic-bezier(2,0,0,1)"]) {
      // Runtime validation also protects untyped callers.
      const result = og.ui.defineMass(ease, JSON.parse(JSON.stringify({ keys: [{ t: 1, ease }] })));
      expect(result.ok).toBe(false);
      if (!result.ok) expect(result.reason).toContain(ease);
    }
  });
  e.advance(1);
  e.destroy();
});
