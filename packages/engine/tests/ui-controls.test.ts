// The component library's foundations and controls (docs/ui.md §2, U6a)
// on the native host, driven as a player drives them: a tap, a drag, a
// key, a pad, a hold. Each test names the wrong implementation it would
// catch.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineSystem } from "@origingame/engine";
import type * as Api from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { checkFrame, measureScale, pseudo, type UiCell, type UiFrameSnapshot } from "../src/node/ui-check.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "fixtures", "ui-text");
{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

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
const byPath = (engine: Engine, end: string) => find(engine, (e) => e.path.endsWith(end));
const focused = (engine: Engine) => find(engine, (e) => e.states.includes("focused"))?.path ?? "";

// The helpers advance only as far as their script ends plus two ticks for the
// click to land and the build to see it: the tick is fixed (60 Hz), so these
// counts hold on any machine, and every tick spent here is CPU a loaded gate
// multiplies (a keyboard's tick is ~70 controls on the dev host).
function tap(engine: Engine, path: string): void {
  const r = engine.probe.commands(`0:touch_move_to:${path};0.05:touch_start;0.1:touch_end`) as unknown as { warnings: string[] };
  expect(r.warnings).toEqual([]);
  engine.advance(8);
}
function key(engine: Engine, code: string): void {
  engine.probe.commands(`0:key_down:${code};0.05:key_up:${code}`);
  engine.advance(5);
}
function pad(engine: Engine, button: string): void {
  engine.probe.commands(`0:gamepad:${button};0.05:gamepad:-`);
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

test("foundations: an effect layer shows what it holds; a view frame turns by a key and drops its hint", () => {
  // Wrong: an effect layer that hides its content, or a view frame that ignores its keys.
  let yaw = 0;
  const engine = run(() => {
    og.ui.emitter("Sparks", { preset: "sparkle" } as never);
    og.ui.effectLayer("Shine", { material: "shine", sweep: true } as never, () => og.ui.label("Inner", "Legendary", {}));
    yaw = og.ui.viewFrame("Preview", { yaw, label: "Character preview", hint: "Drag to turn", height: { kind: "fixed", px: 160 } as never }).yaw;
  });
  expect(find(engine, (e) => e.text === "Legendary")).toBeDefined();
  // The viewport turns by a key step once focused, and its hint goes away.
  expect(find(engine, (e) => e.text === "Drag to turn")).toBeDefined();
  focusOn(engine, "Preview");
  key(engine, "ArrowRight");
  expect(yaw).toBe(15);
  engine.advance(4);
  expect(find(engine, (e) => e.text === "Drag to turn") === undefined).toBeTrue();
});

// ── buttons ─────────────────────────────────────────────────────────────

test("every control: on a coarse pointer each target a finger presses is at least 44 px (or reaches it by its hit slop)", () => {
  // Wrong: a 28 px stepper arrow or clear button a thumb misses.
  const engine = run(() => {
    og.ui.inputField("Field", "", { label: "Field", clearable: true });
    og.ui.dropdown("Drop", { label: "Drop", options: [{ label: "A" }] });
  }, 360);
  og.ui.previewViewport(engine.world, { width: 390, height: 1400, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait" } as never);
  engine.advance(12);
  const pressable = new Set(["button", "checkbox", "switch", "radio", "slider", "combobox", "textbox", "spinbutton"]);
  const small = elements(engine).filter((e) => pressable.has(e.role) && Math.min(e.width, e.height) < 44 - 0.5);
  // What is drawn smaller reaches 44 px: a touch just inside the 44 px square around its centre lands on it.
  for (const e of small) {
    const cx = e.x + e.width / 2;
    const cy = e.y + e.height / 2;
    const r = engine.probe.commands(`0:touch_move:${Math.round(e.width < 44 ? cx + 20 : cx)},${Math.round(e.height < 44 ? cy + 20 : cy)};0.02:touch_start`) as unknown as { warnings: string[] };
    expect(r.warnings).toEqual([]);
    engine.advance(6);
    const now = find(engine, (x) => x.path === e.path)!;
    expect(`${e.path} ${now.visualStates?.interaction === "pressed" ? "reached" : "missed"}`).toBe(`${e.path} reached`);
    engine.probe.commands("0:touch_end");
    engine.advance(6);
  }
});

test("inputField: an error marks it invalid with its message described, the clear button empties it", () => {
  let value = "Arthur";
  let cleared = false;
  const engine = run(() => {
    const r = og.ui.inputField("Name", value, { label: "Hero name", error: value.length > 4 ? "At most 4 letters" : "", clearable: true });
    value = r.value;
    if (r.cleared) cleared = true;
  });
  expect(find(engine, (e) => e.text === "At most 4 letters")).toBeDefined();
  const clear = find(engine, (e) => e.name.toLowerCase().includes("clear"))!;
  tap(engine, clear.path);
  expect(cleared).toBeTrue();
  expect(value).toBe("");
});

test("dropdown: a tap opens the list with the focus on the chosen option, a tap on another chooses and closes, Back closes", () => {
  let sel = 1;
  let open = false;
  const engine = run(() => {
    const r = og.ui.dropdown("Lang", { label: "Language", selected: sel, options: [{ label: "English" }, { label: "Français" }, { label: "日本語" }] });
    sel = r.selected;
    open = r.open;
  });
  const trig = find(engine, (e) => e.role === "combobox")!;
  tap(engine, trig.path);
  engine.advance(12);
  expect(open).toBeTrue();
  expectFocus(engine, "Option#1");
  tap(engine, find(engine, (e) => e.text === "日本語")!.path);
  engine.advance(12);
  expect(sel).toBe(2);
  expect(open).toBeFalse();
  tap(engine, trig.path);
  engine.advance(12);
  key(engine, "Escape");
  engine.advance(12);
  expect(open).toBeFalse();
  expect(sel).toBe(2);
});

// ── every breakpoint, direction, locale and text size ──────────────────

/** The controls a settings or a shop screen holds, labelled through `l` (the pseudo-locale's longer words). */
function gallery(l: (s: string) => string): void {
  og.ui.inputField("Name", "", { label: l("Hero name"), placeholder: l("Type a name"), helper: l("Shown to other players") });
  og.ui.dropdown("Lang", { label: l("Language"), selected: 0, options: [{ label: "English" }, { label: "日本語" }] });
}

test("right to left and the pseudo-locale: rows run from the right, the longer labels overlap nothing", () => {
  // Wrong: a control placing its parts by physical left, or a fixed width its longer label overflows.
  const engine = run(() => gallery(pseudo), 640, kit);
  const overlaps = () => (og.ui.overlaps() as unknown as Array<{ path: string; other: string }>).filter((o) => !/Popup|Thumb|Fill|Badge/.test(o.path + o.other));
  expect(overlaps()).toEqual([]);
  const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
  if (!loaded.ok) throw new Error(loaded.reason);
  og.i18n.setLocale(engine.world, "ar-EG");
  engine.advance(10);
  const x = (end: string) => byPath(engine, end)!.x;
  // A switch row: the label at the start (right), the track at the end (left).
  expect(x("Music/Track")).toBeLessThan(x("Music/Text/Label"));
  expect(overlaps()).toEqual([]);
});

test("150 % text: every control grows with the text and cuts none of it, on a desktop and a phone", () => {
  // Wrong: a row, a cap or a knob sized in fixed pixels around its text, so at the largest text size it clips.
  const ui = native() as unknown as { og_node_ui_frame(id: number): string };
  for (const [bp, w, h, coarse] of [["desktop", 1920, 2400, false], ["phonePortrait", 390, 2400, true]] as const) {
    const frames: UiFrameSnapshot[] = [];
    for (const scale of [1, 1.5]) {
      const engine = run(() => gallery((s) => s), Math.min(640, w - 32), kit);
      og.ui.previewViewport(engine.world, { width: w, height: h, dpr: 1, coarsePointer: coarse, breakpoint: bp } as never);
      const font = og.load.font(engine.world, "fonts/NotoSansSC-Regular-subset.ttf");
      if (!font.ok) throw new Error(font.reason);
      og.ui.configure(engine.world, { font: font.value, textScale: scale });
      engine.advance(30);
      frames.push(JSON.parse(ui.og_node_ui_frame(engine.id)) as UiFrameSnapshot);
      engine.destroy();
    }
    const cell: UiCell = { screen: "controls", breakpoint: bp, locale: "", style: "", scale: 150, highContrast: false };
    expect(frames[1]!.texts.find((x) => x.text === "Players")!.size).toBeCloseTo(frames[0]!.texts.find((x) => x.text === "Players")!.size * 1.5, 1);
    // At 150 % no text is cut, and every text is drawn at 150 % of its size (within 5 %).
    const cuts = checkFrame(frames[1]!, cell).filter((f) => f.rule === "text-cut").map((f) => `${f.paths.join(" × ")}: ${f.measured}`);
    const short = measureScale(frames[0]!, frames[1]!, cell).filter((m) => m.value < 0.95 * 150).map((m) => `${m.paths.join(" × ")}: ${m.detail}`);
    expect([...cuts, ...short]).toEqual([]);
    const els = frames[1]!.elements;
    const spills: string[] = [];
    const parentOf = (i: number): number => {
      let j = i - 1;
      while (j >= 0 && els[j]!.id !== els[i]!.parent) j--;
      return j;
    };
    els.forEach((e, i) => {
      if (e.kind !== "label" || !e.text || e.clipped) return;
      for (let j = parentOf(i); j >= 0; j = parentOf(j)) {
        const p = els[j]!;
        if (p.kind === "scroll" || p.kind === "panel" || p.clipped) break;
        if (p.height > 0 && (e.y < p.y - 0.5 || e.y + e.height > p.y + p.height + 0.5)) {
          spills.push(`${bp} ${p.id} ⊅ ${e.id} "${e.text}": ${e.y}+${e.height} in ${p.y}+${p.height}`);
          break;
        }
        // A corner badge overhangs its host by design: it holds its own text, and nothing above it need.
        if (p.id === "Badge") break;
      }
    });
    expect(spills).toEqual([]);
  }
}, 30_000);
