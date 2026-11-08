// The UI's CPU budget on the TypeScript side (docs/ui.md §6.3): the dense
// menu of crates/og-ui/tests/perf.rs (a shop of 48 cards, 300 elements, 40
// in a spring at every tick) built through og.ui by a system, so the tick
// pays every TS↔engine crossing: one host call per widget, its desc as JSON.
// It prints the median and 95th percentile tick and the crossings per tick.
//
// A measurement for the reference machines on the release addon, run by the
// perf lane (scripts/perf.sh sets OG_PERF=1 after `OG_RELEASE=1 sh
// scripts/build.sh --native`); skipped elsewhere, where a dev addon's times
// say nothing.
import { expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import type * as Api from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { bindHost, createEngine, type GameToken } from "../src/runtime.ts";

const BUDGET_MS = 0.5;
/** What menu() builds: the panel, its header row, its title and four buttons, the grid, and six per card. */
const BUILT = 8 + 48 * 6;
const WARM = 30;
const TICKS = 240;

let tick = 0;
function card(k: number): void {
  // Retargeted every 12 ticks, before a 300 ms spring settles: always in flight.
  const lifted = k < 40 && (Math.floor(tick / 12) + k) % 2 === 0;
  og.ui.open(`Card${k}`, {
    width: { kind: "fixed", px: 200 },
    height: { kind: "fixed", px: 150 },
    padding: { left: 8, top: 8, right: 8, bottom: 8 },
    gap: 4,
    radius: 6,
    interactive: true,
  } as Api.ElementDesc);
  og.ui.icon("Art", "sword");
  og.ui.label("Name", `Item ${k}`);
  og.ui.label("Price", `${100 + k * 25} gold`);
  og.ui.badge("Rarity", ["Common", "Rare", "Epic"][k % 3]!, "primary");
  // The stock as a bar of its own width (og-ui's meter in perf.rs).
  og.ui.open("Stock", { width: { kind: "fixed", px: 4 + (k % 10) * 18 }, height: { kind: "fixed", px: 6 }, radius: 3 } as Api.ElementDesc);
  og.ui.close();
  og.ui.close();
  og.ui.props({
    transform: { scaleX: lifted ? 1.05 : 1, scaleY: lifted ? 1.05 : 1 },
    opacity: lifted ? 1 : 0.9,
    motion: { kind: "spring", durationMs: 300, bounce: 0.1 },
  } as Api.ElementProps);
}

function menu(): void {
  og.ui.column("Shop", {}, () => {
    og.ui.row("Header", {}, () => {
      og.ui.label("Title", "Shop");
      for (const t of ["Weapons", "Armour", "Potions", "Gems"]) og.ui.button(t, t);
    });
    og.ui.open("Cards", { gap: 8 } as Api.ElementDesc);
    for (let k = 0; k < 48; k++) card(k);
    og.ui.close();
    og.ui.props({ layout: "grid", grid: { columns: Array.from({ length: 8 }, () => ({ kind: "fr", fr: 1 })) } } as Api.ElementProps);
  });
}

test.skipIf(!process.env.OG_PERF)("perf_ui_ts_dense_menu_300_elements_40_animating", () => {
  const binding = loadNativeHost();
  const real = binding.abi as unknown as Record<string, unknown>;
  let crossings = 0;
  // bindHost wraps the abi it is given (runtime.ts), so the count is the proxy it binds.
  bindHost({ ...binding, abi: new Proxy(real, {
    get(target, key) {
      const f = target[key as string];
      if (typeof f !== "function" || typeof key !== "string" || !key.startsWith("og_ui_")) return f;
      return (...args: unknown[]) => {
        crossings++;
        return (f as (...a: unknown[]) => unknown).apply(target, args);
      };
    },
  }) as unknown as typeof binding.abi });
  try {
    const sys = defineSystem("UiPerfMenu", {}, () => menu());
    const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken);
    if (!r.ok) throw new Error(r.reason);
    const engine = r.value;
    const times: number[] = [];
    let perTick = 0;
    for (tick = 0; tick < WARM + TICKS; tick++) {
      crossings = 0;
      const t = performance.now();
      engine.advance(1);
      const ms = performance.now() - t;
      if (tick >= WARM) {
        times.push(ms);
        perTick = crossings;
      }
    }
    times.sort((a, b) => a - b);
    const median = times[times.length >> 1]!;
    const p95 = times[Math.floor(times.length * 0.95)]!;
    const elements = (engine.probe.dump({ what: "ui" }) as unknown as { elements: unknown[] }).elements.length;
    console.log(`perf ui ts dense menu: median ${median.toFixed(3)} ms, p95 ${p95.toFixed(3)} ms a tick; ${elements} elements, ${perTick} og.ui crossings a tick (${((median * 1000) / perTick).toFixed(2)} µs each, tick included)`);
    // A measurement, not a verdict (the coordinator's call, 2026-11-07): the budget is printed beside it.
    console.log(`perf ui ts dense menu: budget ${BUDGET_MS} ms; ${BUILT} elements built a tick, ${elements} on screen (the probe lists what the viewport shows)`);
    expect(perTick).toBeGreaterThan(BUILT);
  } finally {
    bindHost(binding);
  }
});
