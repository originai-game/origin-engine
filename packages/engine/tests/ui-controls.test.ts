// The component library's foundations (docs/ui.md §2, U6a) on the native
// host, driven as a player drives them: a key. Each test names the wrong
// implementation it would catch. The controls are document elements, tested
// in crates/og-ui.
import { expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import type * as Api from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

loadNativeHost();

type Element = { id: string; parent: string; path: string; kind: string; text: string; role: string; name: string; value: string; states: string[]; disabled: boolean; x: number; y: number; width: number; height: number; visualStates?: Record<string, string> };

let systems = 0;
/** An engine whose one system builds `build(world)` in a column at the centre of the surface. */
function run(build: (world: Api.World) => void, width = 640, root?: string, setup?: (world: Api.World) => void): Engine {
  let ready = false;
  const sys = defineSystem(`UcCtl${systems++}`, {}, (_q: unknown, _t: unknown, world: Api.World) => {
    if (!ready) {
      setup?.(world);
      ready = true;
    }
    og.ui.anchor("Root", { anchor: "center", direction: "column", gap: 12, width: { kind: "fixed", px: width } } as never, () => build(world));
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, root ? { root } : {});
  if (!r.ok) throw new Error(r.reason);
  r.value.advance(6);
  return r.value;
}

function elements(engine: Engine): Element[] {
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}
const find = (engine: Engine, pred: (e: Element) => boolean) => elements(engine).find(pred);
const focused = (engine: Engine) => find(engine, (e) => e.states.includes("focused"))?.path ?? "";

// A key press and the ticks for it to land: the tick is fixed (60 Hz), so
// the count holds on any machine.
function key(engine: Engine, code: string): void {
  engine.probe.commands(`0:key_down:${code};0.05:key_up:${code}`);
  engine.advance(5);
}
/** The focus is on a path ending in `end` (a failure shows the path it is on). */
function expectFocus(engine: Engine, end: string): void {
  const f = focused(engine);
  expect(f.endsWith(end) ? end : f).toBe(end);
}
/** Tab until the focus is on a path ending in `end` (at most 12 presses). */
function focusOn(engine: Engine, end: string): void {
  for (let i = 0; i < 12 && !focused(engine).endsWith(end); i++) key(engine, "Tab");
  expectFocus(engine, end);
}

// ── foundations ─────────────────────────────────────────────────────────

test("foundations: an effect layer shows what it holds", () => {
  // Wrong: an effect layer that hides its content.
  const engine = run(() => {
    og.ui.effectLayer("Shine", { material: "shine", sweep: true } as never, () => og.ui.label("Inner", "Legendary", {}));
  });
  expect(find(engine, (e) => e.text === "Legendary")).toBeDefined();
});
