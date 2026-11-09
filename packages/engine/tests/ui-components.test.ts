// og.ui's number formats and the player's colour vision setting on the
// native host. Each test names the wrong implementation it would catch.
import { expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { formatNumber } from "../src/ui/data/format.ts";

loadNativeHost();

/** An engine with no systems, advanced past its first frames. */
function run(): Engine {
  const r = createEngine(og.game({}) as unknown as GameToken);
  if (!r.ok) throw new Error(r.reason);
  r.value.advance(6);
  return r.value;
}

test("formats: true minus signs, percentages and compact units", () => {
  expect(formatNumber(-12840, "grouped")).toBe("−12,840");
  expect(formatNumber(0.425, "percent", 1)).toBe("42.5%");
  expect(formatNumber(12400, "compact")).toBe("12K");
});

test("colorVision: the player's colour-blind mode reaches the theme, partial configure keeps it, null clears it", () => {
  // Wrong: a setting read but not reported (a settings screen cannot show it), one lost when
  // another field is configured, or a strength left unclamped.
  const engine = run();
  const theme = () => og.ui.theme(engine.world) as unknown as { colorVision: string; colorVisionFilter: string; colorVisionStrength: number };
  expect(theme().colorVision).toBe("none");
  og.ui.configure(engine.world, { colorVision: "deuteranopia", colorVisionStrength: 3 } as never);
  engine.advance(2);
  expect(theme()).toMatchObject({ colorVision: "deuteranopia", colorVisionFilter: "correct", colorVisionStrength: 1 });
  og.ui.configure(engine.world, { textScale: 1.2 });
  engine.advance(2);
  expect(theme().colorVision).toBe("deuteranopia");
  og.ui.configure(engine.world, { colorVision: null, colorVisionFilter: "simulate" } as never);
  engine.advance(2);
  expect(theme()).toMatchObject({ colorVision: "none", colorVisionFilter: "simulate" });
  engine.destroy();
});
