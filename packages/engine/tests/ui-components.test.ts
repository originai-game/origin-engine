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

test("menu: entries take menu roles, a tap chooses, a submenu opens beside its row and chooses", () => {
  // Wrong: a check entry announced as a plain item, a submenu that never opens, a chosen id from the wrong level.
  let open = true;
  let chosen = null as string | null;
  const engine = run(() => {
    pressable("Opener", "Options");
    const at = og.ui.lastPath();
    const r = og.ui.menu("Menu", open, {
      anchor: at,
      entries: [
        { id: "copy", label: "Copy", shortcut: "Ctrl+C" },
        { id: "grid", label: "Show grid", kind: "check", checked: true },
        { kind: "separator" },
        { id: "sort", label: "Sort by" },
        { id: "byName", label: "Name", kind: "radio", checked: true, parent: "sort" },
        { id: "byLevel", label: "Level", kind: "radio", parent: "sort" },
        { id: "delete", label: "Delete", danger: true },
      ],
    });
    if (r.chosen !== null) chosen = r.chosen;
    if (r.dismissed) open = false;
  });
  engine.advance(10);
  const roles = elements(engine).filter((e) => e.role.startsWith("menu")).map((e) => `${e.role}:${e.name}`);
  expect(roles.some((r) => r.startsWith("menu:"))).toBeTrue();
  expect(roles).toContain("menuitem:Copy");
  expect(roles).toContain("menuitemcheckbox:Show grid");
  expect(find(engine, (e) => e.name === "Show grid")?.states).toContain("checked");
  tap(engine, "Content/Item#3");
  engine.advance(12);
  expect(find(engine, (e) => e.role === "menuitemradio" && e.name === "Level")).toBeDefined();
  const level = find(engine, (e) => e.role === "menuitemradio" && e.name === "Level")!;
  const sortRow = find(engine, (e) => e.path.endsWith("Content/Item#3"))!;
  // Beside its row (at its end), not over it.
  expect(level.x).toBeGreaterThanOrEqual(sortRow.x + sortRow.width - 4);
  tapAt(engine, level);
  expect(chosen).toBe("byLevel");
  expect(open).toBeFalse();
});

test("menu: Back closes it, answering dismissed without a choice", () => {
  let open = true;
  let chosen = null as string | null;
  const engine = run(() => {
    const r = og.ui.menu("Menu", open, { at: { x: 200, y: 200 }, entries: [{ id: "a", label: "Alpha" }, { id: "b", label: "Beta" }] });
    if (r.chosen !== null) chosen = r.chosen;
    if (r.dismissed) open = false;
  });
  engine.advance(10);
  key(engine, "Escape");
  expect(open).toBeFalse();
  expect(chosen).toBeNull();
});

test("menu: the forward arrow opens a submenu into its first entry, the back arrow closes it to its row; mirrored in right to left", () => {
  // Wrong: Right moves the focus off the row (or wraps), Left in the submenu leaves it open, or the arrows are not mirrored.
  for (const dir of ["ltr", "rtl"] as const) {
    let chosen = null as string | null;
    const engine = run(() => {
      pressable("Opener", "Options");
      const r = og.ui.menu("Menu", true, {
        anchor: og.ui.lastPath(),
        entries: [
          { id: "copy", label: "Copy" },
          { id: "sort", label: "Sort by" },
          { id: "byName", label: "Name", parent: "sort" },
          { id: "byLevel", label: "Level", parent: "sort" },
        ],
      });
      if (r.chosen !== null) chosen = r.chosen;
    }, 640, dir === "rtl" ? kit : undefined);
    if (dir === "rtl") {
      const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
      if (!loaded.ok) throw new Error(loaded.reason);
      og.i18n.setLocale(engine.world, "ar-EG");
    }
    engine.advance(12);
    const [fwd, back] = dir === "rtl" ? ["ArrowLeft", "ArrowRight"] : ["ArrowRight", "ArrowLeft"];
    // Opened, the focus is on its first entry; Down reaches the submenu's row.
    expect(focused(engine).endsWith("Content/Item#0")).toBeTrue();
    key(engine, "ArrowDown");
    expect(focused(engine).endsWith("Content/Item#1")).toBeTrue();
    key(engine, fwd);
    engine.advance(12);
    expect(find(engine, (e) => e.role === "menuitem" && e.name === "Name")).toBeDefined();
    expect(find(engine, (e) => e.name === "Sort by")?.states).toContain("expanded");
    expect(focused(engine)).toContain("Menu.sub/");
    key(engine, back);
    engine.advance(12);
    expect(find(engine, (e) => e.role === "menuitem" && e.name === "Name") === undefined).toBeTrue();
    expect(focused(engine).endsWith("Content/Item#1")).toBeTrue();
    // The pad's d-pad does the same.
    const [padFwd, padBack] = dir === "rtl" ? ["left", "right"] : ["right", "left"];
    engine.probe.commands(`0:gamepad:${padFwd};0.05:gamepad:-`);
    engine.advance(16);
    expect(focused(engine)).toContain("Menu.sub/");
    engine.probe.commands(`0:gamepad:${padBack};0.05:gamepad:-`);
    engine.advance(16);
    expect(focused(engine).endsWith("Content/Item#1")).toBeTrue();
    // Forward again and Enter chooses in the submenu.
    key(engine, fwd);
    engine.advance(12);
    key(engine, "Enter");
    expect(chosen).toBe("byName");
  }
});

test("menu: on a phone a submenu drills in, its rows replacing the menu's under a back row; back and Back go up a level", () => {
  // Wrong: a submenu that opens beside its row on a 390 px screen (covering the parent), a back row
  // that does nothing, or Back that closes the whole menu from inside the submenu.
  let open = true;
  let chosen = null as string | null;
  const engine = run(() => {
    pressable("Opener", "Options");
    const r = og.ui.menu("Menu", open, {
      anchor: og.ui.lastPath(),
      entries: [
        { id: "copy", label: "Copy" },
        { id: "sort", label: "Sort by" },
        { id: "byName", label: "Name", parent: "sort" },
        { id: "byLevel", label: "Level", parent: "sort" },
        { id: "destroy", label: "Destroy", danger: true },
      ],
    });
    if (r.chosen !== null) chosen = r.chosen;
    if (r.dismissed) open = false;
  }, 358);
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as never);
  engine.advance(12);
  const names = () => elements(engine).filter((e) => e.role.startsWith("menuitem")).map((e) => e.name);
  expect(names()).toEqual(["Copy", "Sort by", "Destroy"]);
  tap(engine, "Content/Item#1");
  engine.advance(12);
  // Drilled in: one menu, its rows the submenu's under the back row naming where it came from.
  expect(names()).toEqual(["Sort by", "Name", "Level"]);
  expect(find(engine, (e) => e.path.includes("Menu.sub")) === undefined).toBeTrue();
  // The back row goes up.
  tap(engine, "Content/Up");
  engine.advance(12);
  expect(names()).toEqual(["Copy", "Sort by", "Destroy"]);
  // In again; Back (Escape) goes up a level, not out; a second Back closes the menu.
  tap(engine, "Content/Item#1");
  engine.advance(12);
  key(engine, "Escape");
  engine.advance(6);
  expect(open).toBeTrue();
  expect(names()).toEqual(["Copy", "Sort by", "Destroy"]);
  key(engine, "Escape");
  expect(open).toBeFalse();
  expect(chosen).toBeNull();
  // A choice from a drilled-in submenu is that entry's.
  open = true;
  engine.advance(12);
  tap(engine, "Content/Item#1");
  engine.advance(12);
  const level = find(engine, (e) => e.role === "menuitem" && e.name === "Level")!;
  tapAt(engine, level);
  expect(chosen).toBe("byLevel");
});

test("menu: a press while it still eases in chooses (nothing waits for the open motion)", () => {
  // Wrong: the surface inert for its 140 ms entrance, so a quick player's press is dropped.
  let open = false;
  let chosen = null as string | null;
  const engine = run(() => {
    if (pressable("Opener", "Options").clicked) open = true;
    const at = og.ui.lastPath();
    const r = og.ui.menu("Menu", open, { anchor: at, entries: [{ id: "a", label: "Alpha" }, { id: "b", label: "Beta" }] });
    if (r.chosen !== null) chosen = r.chosen;
    if (r.dismissed) open = false;
  });
  // Where Beta rests, from an open that settled; then closed again.
  tap(engine, "Opener");
  engine.advance(20);
  const beta = find(engine, (e) => e.role === "menuitem" && e.name === "Beta")!;
  expect(beta).toBeDefined();
  key(engine, "Escape");
  engine.advance(20);
  expect(open).toBeFalse();
  // Open, and press Beta three ticks after it first shows (its motion runs nine).
  const x = Math.round(beta.x + beta.width / 2), y = Math.round(beta.y + beta.height / 2);
  engine.probe.commands(`0:touch_move_to:Opener;0.02:touch_start;0.05:touch_end;0.1:touch_move:${x},${y};0.1:touch_start;0.13:touch_end`);
  engine.advance(12);
  expect(chosen).toBe("b");
  expect(open).toBeFalse();
});

// ── sheet ───────────────────────────────────────────────────────────────

test("sheet: a bottom sheet rests at its detent from the bottom edge and Back dismisses it", () => {
  let open = true;
  const engine = run(() => {
    const r = og.ui.sheet("Shop", open, { title: "Shop", detents: [0.4, 0.9] }, () => og.ui.label("Line", "Wares", {}));
    if (r.dismissed) open = false;
  });
  engine.advance(60);
  const frame = byPath(engine, "Shop/Panel/Content")!;
  expect(frame).toBeDefined();
  // The visible top of the sheet sits near 60 % down (detent 0.4 of the room).
  const handle = byPath(engine, "Content/Handle")!;
  expect(handle.y).toBeGreaterThan(720 * 0.45);
  expect(handle.y).toBeLessThan(720 * 0.7);
  key(engine, "Escape");
  expect(open).toBeFalse();
});

// ── data ────────────────────────────────────────────────────────────────

// ── 200 % text ──────────────────────────────────────────────────────────

test("150 % text: the containers and navigation components grow with the text and cut none of it, on a desktop and a phone", () => {
  // Wrong: a part sized in fixed pixels around its text (a row, a chip, a header), so at the
  // player's largest text size it clips, or a text drawn at a fixed size the scale never reaches.
  const ui = native() as unknown as { og_node_ui_frame(id: number): string };
  const nav = () => {
    og.ui.menu("Menu", true, { at: { x: 40, y: 40 }, entries: [{ id: "copy", label: "Copy", shortcut: "Ctrl+C" }, { id: "compare", label: "Compare", shortcut: "C" }, { id: "sort", label: "Sort by" }, { id: "destroy", label: "Destroy", danger: true }, { id: "n", label: "Name", parent: "sort" }] });
  };
  // Tall surfaces: everything is laid out on screen, nothing scrolled away.
  for (const [bp, w, h, coarse, build, probe] of [["desktop", 1280, 2400, false, nav, "Copy"], ["phonePortrait", 390, 2400, true, nav, "Copy"]] as const) {
    const frames: UiFrameSnapshot[] = [];
    for (const scale of [1, 1.5]) {
      const engine = run(build, Math.min(640, w - 32), kit);
      og.ui.previewViewport(engine.world, { width: w, height: h, dpr: 1, coarsePointer: coarse, breakpoint: bp } as never);
      // A font, so text is measured as drawn (without one og-ui estimates and draws none).
      const font = og.load.font(engine.world, "fonts/NotoSansSC-Regular-subset.ttf");
      if (!font.ok) throw new Error(font.reason);
      og.ui.configure(engine.world, { font: font.value, textScale: scale });
      engine.advance(30);
      frames.push(JSON.parse(ui.og_node_ui_frame(engine.id)) as UiFrameSnapshot);
      engine.destroy();
    }
    const cell: UiCell = { screen: "components", breakpoint: bp, locale: "", style: "", scale: 150, highContrast: false };
    expect(frames[1]!.texts.find((x) => x.text === probe)!.size).toBeCloseTo(frames[0]!.texts.find((x) => x.text === probe)!.size * 1.5, 1);
    // At 150 % no text is cut, and every text is drawn at 150 % of its size (within 5 %).
    const cuts = checkFrame(frames[1]!, cell).filter((f) => f.rule === "text-cut").map((f) => `${f.paths.join(" × ")}: ${f.measured}`);
    const short = measureScale(frames[0]!, frames[1]!, cell).filter((m) => m.value < 0.95 * 150).map((m) => `${m.paths.join(" × ")}: ${m.detail}`);
    expect([...cuts, ...short]).toEqual([]);
    // And no text spills out of the part that holds it (a fixed-height row its text outgrew,
    // which nothing clips, so the cut rule cannot see it).
    const els = frames[1]!.elements;
    const spills: string[] = [];
    const parentOf = (i: number): number => {
      let j = i - 1;
      while (j >= 0 && els[j]!.id !== els[i]!.parent) j--;
      return j;
    };
    els.forEach((e, i) => {
      if (e.kind !== "label" || !e.text || e.clipped) return;
      // Every holder up to the first that scrolls (or is cut by one) holds it whole.
      for (let j = parentOf(i); j >= 0; j = parentOf(j)) {
        const p = els[j]!;
        // A floating surface's content is moved off its modal panel by a transform.
        if (p.kind === "scroll" || p.kind === "panel" || p.clipped) break;
        // A zero-height holder centres its text on a line by design (a chart's axis ticks).
        if (p.height > 0 && (e.y < p.y - 0.5 || e.y + e.height > p.y + p.height + 0.5)) {
          spills.push(`${bp} ${p.id} ⊅ ${e.id} "${e.text}": ${e.y}+${e.height} in ${p.y}+${p.height}`);
          break;
        }
      }
    });
    expect(spills).toEqual([]);
    // Nothing truncated to a stub: headers, menus and tabs wrap or reflow instead.
    const cut = checkFrame(frames[1]!, cell).filter((f) => f.rule === "text-truncated");
    expect(cut.map((f) => `${bp} ${f.paths.join(" × ")}: ${f.measured}`)).toEqual([]);
    // Under a finger every target is a whole 44 CSS px, at both text sizes (a dot, a crumb, a step).
    if (coarse) {
      for (const f of frames) {
        const small = f.elements.filter((e) => /^(button|tab|link|menuitem|menuitemcheckbox|menuitemradio|columnheader)$/.test(e.role ?? "") && !e.clipped && !e.disabled && (e.width < 43.5 || e.height < 43.5));
        expect(small.map((e) => `${bp} ${e.id} (${e.name}) ${Math.round(e.width)}×${Math.round(e.height)}`)).toEqual([]);
      }
    }
  }
}, 60_000);

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
