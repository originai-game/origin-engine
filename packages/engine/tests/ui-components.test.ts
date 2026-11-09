// The component library's containers, navigation and data components
// (docs/ui.md §2) on the native host, driven as a player drives them: a
// tap, a key, a bumper, Back. Each test names the wrong implementation it
// would catch.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineSystem, type World } from "@origingame/engine";
import type * as Api from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { checkFrame, measureScale, pseudo, type UiCell, type UiFrameSnapshot } from "../src/node/ui-check.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { formatNumber } from "../src/ui/data/format.ts";
import { ogBin } from "./support/og-bin.ts";
import { drawsDuring } from "./support/ui.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "ui-text");

// At module level, not in a hook (a hook's 5 s would not cover a cook on a
// loaded machine): the kit's string tables, Arabic among them, cooked
// (incrementally) by the og this package built.
{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

type Element = { clipped?: boolean; id: string; parent: string; path: string; kind: string; text: string; role: string; name: string; value: string; states: string[]; disabled: boolean; x: number; y: number; width: number; height: number };

let systems = 0;
/** An engine whose one system builds `build` in a 640 px column at the centre of the surface. */
function run(build: () => void, width = 640, root?: string): Engine {
  const sys = defineSystem(`UcBuild${systems++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "center", direction: "column", width: { kind: "fixed", px: width } } as never, build);
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
const byPath = (engine: Engine, end: string) => find(engine, (e) => e.path.endsWith(end));
/** The path of the element keys and pads focus, from the dump's states ("" when none). */
/** A pressable box holding its label (what a document's `<button>` builds). */
const pressable = (id: string, text: string): Api.Response =>
  og.ui.row(id, { interactive: true, focusable: true, padding: { left: 12, top: 8, right: 12, bottom: 8 } }, () => og.ui.label("Label", text));
const focused = (engine: Engine) => find(engine, (e) => e.states.includes("focused"))?.path ?? "";

function tap(engine: Engine, path: string): void {
  const r = engine.probe.commands(`0:touch_move_to:${path};0.05:touch_start;0.1:touch_end`) as unknown as { warnings: string[] };
  expect(r.warnings).toEqual([]);
  engine.advance(12);
}
function tapAt(engine: Engine, e: Element): void {
  engine.probe.commands(`0:touch_move:${Math.round(e.x + e.width / 2)},${Math.round(e.y + e.height / 2)};0.05:touch_start;0.1:touch_end`);
  engine.advance(12);
}
function key(engine: Engine, code: string): void {
  engine.probe.commands(`0:key_down:${code};0.05:key_up:${code}`);
  engine.advance(8);
}

// ── menus ───────────────────────────────────────────────────────────────

// ── formats ─────────────────────────────────────────────────────────────

test("formats: true minus signs, percentages and compact units", () => {
  expect(formatNumber(-12840, "grouped")).toBe("−12,840");
  expect(formatNumber(0.425, "percent", 1)).toBe("42.5%");
  expect(formatNumber(12400, "compact")).toBe("12K");
});

test("colorVision: the player's colour-blind mode reaches the theme, partial configure keeps it, null clears it", () => {
  // Wrong: a setting read but not reported (a settings screen cannot show it), one lost when
  // another field is configured, or a strength left unclamped.
  const engine = run(() => og.ui.label("T", "Status"));
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

test("userSelect: a drag over labels in a box that allows it selects across them in order; a press on the box's button selects nothing", () => {
  // Wrong: the prop not reaching og-ui (nothing selected), labels joined
  // without a break, a drag back up reversing their order, or a control's
  // label taken for text.
  const engine = run(() => {
    og.ui.open("Notes", { direction: "column", gap: 8 } as Api.ElementDesc);
    og.ui.label("First", "Alpha beta gamma");
    og.ui.label("Second", "Delta epsilon");
    pressable("Go", "Go");
    og.ui.close();
    og.ui.props({ userSelect: "text" } as Api.ElementProps);
  });
  const selection = () => (engine.probe.dump({ what: "ui" }) as unknown as { selection: { text: string } | null }).selection;
  const a = byPath(engine, "First")!;
  const b = byPath(engine, "Second")!;
  const drag = (from: [number, number], to: [number, number]) => {
    const mid = [(from[0] + to[0]) / 2, (from[1] + to[1]) / 2];
    engine.probe.commands(`0:touch_move:${from.join(",")};0.02:touch_start;0.06:touch_move:${mid.join(",")};0.1:touch_move:${to.join(",")};0.14:touch_end`);
    engine.advance(14);
  };
  drag([a.x + 1, a.y + a.height / 2], [b.x + b.width + 20, b.y + b.height / 2]);
  expect(selection()?.text).toBe("Alpha beta gamma\nDelta epsilon");
  // Back up from the second's last glyph to the first's first.
  drag([b.x + b.width - 0.5, b.y + b.height / 2], [a.x + 1, a.y + a.height / 2]);
  expect(selection()?.text).toBe("Alpha beta gamma\nDelta epsilon");
  const go = byPath(engine, "Go")!;
  drag([go.x + go.width / 2, go.y + go.height / 2], [go.x + go.width / 2 + 2, go.y + go.height / 2]);
  expect(selection()).toBeNull();
});
