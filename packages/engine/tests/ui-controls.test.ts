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
import { bindingControls, deviceSlots } from "../src/ui/controls/bindings.ts";
import { formatValue, snap } from "../src/ui/controls/base.ts";
import { filterSuggestions, fold } from "../src/ui/controls/fields.ts";
import { kanaMark, layoutRows, toKatakana } from "../src/ui/controls/keyboard.ts";
import { hsvToRgb, parseHex, rgbToHsv, toHex } from "../src/ui/controls/pickers.ts";
import { stepEnabled } from "../src/ui/controls/choices.ts";
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

// ── the pure parts ──────────────────────────────────────────────────────

test("values: snapping counts steps from min, formats follow the step, percent and word labels", () => {
  // Wrong: snapping to multiples of the step from 0 (min 0.25, step 0.5 lands on 0.5, not 0.75).
  expect(snap(0.8, 0.25, 2, 0.5)).toBeCloseTo(0.75, 6);
  expect(snap(9, 0, 5, 1)).toBe(5);
  expect(formatValue(0.5, { step: 0.05 })).toBe("0.50");
  expect(formatValue(3, { step: 1, unit: " m" })).toBe("3 m");
  expect(formatValue(0.42, { percent: true })).toBe("42%");
});

test("choices: a step skips disabled entries and wraps only when asked", () => {
  // Wrong: landing on the disabled index 2, or wrapping a non-wrapping cycler.
  const off = (i: number) => i === 2;
  expect(stepEnabled(4, 1, 1, off)).toBe(3);
  expect(stepEnabled(4, 3, 1, off)).toBe(0);
  expect(stepEnabled(4, 3, 1, off, false)).toBe(3);
});

test("colour: hsv and rgb round-trip, hex parses short and long forms and rejects junk", () => {
  // Wrong: a hue off by a sector (red's hue 0 read as 60), or #abc taken as #0abc00.
  const c = { r: 0.2, g: 0.6, b: 0.4 };
  const hsv = rgbToHsv(c);
  const back = hsvToRgb(hsv.h, hsv.s, hsv.v);
  expect(back.r).toBeCloseTo(c.r, 4);
  expect(back.g).toBeCloseTo(c.g, 4);
  expect(back.b).toBeCloseTo(c.b, 4);
  expect(rgbToHsv({ r: 1, g: 0, b: 0 }).h).toBeCloseTo(0, 4);
  expect(parseHex("#abc")).toMatchObject({ r: 0xaa / 255, g: 0xbb / 255, b: 0xcc / 255 });
  expect(parseHex("zzzzzz")).toBeNull();
  expect(toHex({ r: 1, g: 0.5, b: 0, a: 1 })).toBe("#ff8000");
});

test("search: suggestions match ignoring case and accents, the prefix ones first, then the shorter", () => {
  // Wrong: an accented `Élan` missed by `el`, or a mid-word match ranked over a prefix.
  expect(fold("Élan")).toBe("elan");
  expect(filterSuggestions(["Steel", "Élan", "Elm", "Bell"], "el", 3)).toEqual(["Elm", "Élan", "Bell"]);
});

test("keyboard: the kana marks voice, half-voice and shrink and cycle back; katakana is offered", () => {
  // Wrong: は taking ゛ twice into ぱ, or ば unable to take ゜.
  expect(kanaMark("は", "voice")).toBe("ば");
  expect(kanaMark("ば", "voice")).toBe("は");
  expect(kanaMark("ば", "semi")).toBe("ぱ");
  expect(kanaMark("つ", "small")).toBe("っ");
  expect(kanaMark("ん", "voice")).toBeNull();
  expect(toKatakana("ゆうしゃ")).toBe("ユウシャ");
  // Every layout ends with its action row, and kana is offered only when asked.
  expect(layoutRows("letters", false, false).flat().some((k) => k.kind === "layout" && k.to === "kana")).toBeFalse();
  expect(layoutRows("letters", false, true).flat().some((k) => k.kind === "layout" && k.to === "kana")).toBeTrue();
  expect(layoutRows("letters", true, false)[0]![0]).toEqual({ kind: "char", ch: "Q" });
  // A masked number (a PIN) takes digits only; an open one keeps its sign and point.
  const chars = (digits: boolean) => layoutRows("numeric", false, false, digits).flat().flatMap((k) => (k.kind === "char" ? [k.ch] : []));
  expect(chars(true).sort().join("")).toBe("0123456789");
  expect(chars(false)).toContain("-");
});

test("bindings: a device shows its own bindings by their index among the action's, then empty slots past the end", () => {
  // Wrong: a pad slot numbered from 0 on the pad (it would overwrite the key binding), or a composite read as unbound.
  const bs: Api.InputBinding[] = [{ control: "KeyW" }, { control: "pad.a" }, { up: "KeyI", left: "KeyJ", down: "KeyK", right: "KeyL" }];
  expect(bindingControls(bs[2]!)).toEqual(["KeyI", "KeyJ", "KeyK", "KeyL"]);
  expect(deviceSlots(bs, true, 2)).toEqual([{ slot: 1, binding: bs[1]! }, { slot: 3, binding: null }]);
  expect(deviceSlots(bs, false, 2).map((s) => s.slot)).toEqual([0, 2]);
});

// ── foundations ─────────────────────────────────────────────────────────

test("foundations: roles read as headings, art waits as a busy image, a labelled divider is a separator, counts past 99 cap", () => {
  // Wrong: a title a screen reader reads as plain text, a missing texture drawn as nothing, or `120` overflowing its pill.
  let yaw = 0;
  const engine = run(() => {
    og.ui.text("Title", "Inventory", { role: "title" });
    og.ui.text("Body", "Twelve slots free", { tone: "muted" });
    og.ui.prose("About", "The **Ember Blade** burns.\n\nForged in the *deep* mines.", {});
    og.ui.iconMark("Mail", "mail", { badge: "120", label: "Mail" });
    og.ui.artSlot("Hero", { kind: "portrait", aspect: 0.75, loading: true, label: "Hero portrait", width: { kind: "fixed", px: 120 } as never });
    og.ui.separator("Or", { label: "OR" });
    og.ui.shapeBox("Gem", { kind: "diamond", fill: "primary", glow: "primary" } as never);
    og.ui.space("Gap", { step: "lg" });
    og.ui.emitter("Sparks", { preset: "sparkle" } as never);
    og.ui.effectLayer("Shine", { material: "shine", sweep: true } as never, () => og.ui.label("Inner", "Legendary", {}));
    yaw = og.ui.viewFrame("Preview", { yaw, label: "Character preview", hint: "Drag to turn", height: { kind: "fixed", px: 160 } as never }).yaw;
  });
  expect(find(engine, (e) => e.text === "Inventory")!.role).toBe("heading");
  expect(find(engine, (e) => e.text === "99+")).toBeDefined();
  const art = find(engine, (e) => e.role === "image" && e.name === "Hero portrait")!;
  expect(art.states).toContain("busy");
  expect(art.height).toBeCloseTo(160, 0);
  // A portrait without art waits as a figure (head and shoulders), not a symbol on a grey box.
  expect(find(engine, (e) => e.path.endsWith("Hero/Head"))).toBeDefined();
  expect(find(engine, (e) => e.path.endsWith("Hero/Placeholder")) === undefined).toBeTrue();
  expect(find(engine, (e) => e.role === "separator" && e.name === "OR")).toBeDefined();
  expect(find(engine, (e) => e.text === "Legendary")).toBeDefined();
  expect(find(engine, (e) => e.text.includes("Ember Blade") || e.text.includes("burns"))).toBeDefined();
  // The viewport turns by a key step once focused, and its hint goes away.
  expect(find(engine, (e) => e.text === "Drag to turn")).toBeDefined();
  focusOn(engine, "Preview");
  key(engine, "ArrowRight");
  expect(yaw).toBe(15);
  engine.advance(4);
  expect(find(engine, (e) => e.text === "Drag to turn") === undefined).toBeTrue();
});

// ── buttons ─────────────────────────────────────────────────────────────

test("pushButton: two taps and Enter press it; a disabled one with a reason takes the focus but never fires", () => {
  // Wrong: the disabled button firing on Enter, or skipping the focus so its reason cannot be read.
  let fired = 0;
  let dead = 0;
  const engine = run(() => {
    if (og.ui.pushButton("Play", "Play", { variant: "primary", icon: "play", badge: "3" }).clicked) fired++;
    if (og.ui.pushButton("Shop", "Shop", { disabled: true, disabledReason: "Reach level 5" }).clicked) dead++;
  });
  const b = find(engine, (e) => e.role === "button" && e.name === "Play")!;
  expect(b).toBeDefined();
  tap(engine, b.path);
  tap(engine, b.path);
  expect(fired).toBe(2);
  focusOn(engine, "Play");
  key(engine, "Enter");
  expect(fired).toBe(3);
  const shop = find(engine, (e) => e.name === "Shop")!;
  expect(shop.disabled).toBeTrue();
  focusOn(engine, "Shop");
  key(engine, "Enter");
  tap(engine, shop.path);
  expect(dead).toBe(0);
});

test("every control: on a coarse pointer each target a finger presses is at least 44 px (or reaches it by its hit slop)", () => {
  // Wrong: a 28 px icon button, checkbox box or stepper arrow a thumb misses.
  const taps: Record<string, number> = {};
  const hit = (name: string, r: { clicked: boolean }) => {
    if (r.clicked) taps[name] = (taps[name] ?? 0) + 1;
  };
  const engine = run(() => {
    hit("Close", og.ui.iconButton("Close", "cross", { label: "Close", size: "small" }));
    hit("Small", og.ui.pushButton("Small", "Small", { size: "small" }));
    og.ui.checkField("Check", false, { label: "Check" });
    og.ui.toggleSwitch("Switch", false, { label: "Switch" });
    og.ui.radioGroup("Radio", { label: "Radio", options: [{ label: "A" }, { label: "B" }], row: true });
    og.ui.segmented("Seg", { label: "Seg", items: [{ label: "A" }, { label: "B" }], size: "small" });
    og.ui.cycler("Cyc", ["A", "B"], 0, { label: "Cyc" });
    og.ui.valueSlider("Slider", 0.5, { label: "Slider" });
    og.ui.numberStepper("Step", 1, { label: "Step" });
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
  // And the icon button is 44 px itself.
  const close = find(engine, (e) => e.name === "Close")!;
  expect(Math.min(close.width, close.height)).toBeGreaterThanOrEqual(44);
});

test("holdButton: letting go early confirms nothing and drains; holding the whole time confirms once; the toggle alternative confirms on a second press", () => {
  // Wrong: confirming on the press (no hold), on release before it filled, or twice.
  let confirmed = 0;
  let progress = 0;
  const engine = run(() => {
    const r = og.ui.holdButton("Delete", "Delete save", { variant: "danger", holdMs: 600 });
    if (r.confirmed) confirmed++;
    progress = r.progress;
  });
  const b = find(engine, (e) => e.name.startsWith("Delete save"))!;
  engine.probe.commands(`0:touch_move_to:${b.path};0.05:touch_start;0.3:touch_end`);
  engine.advance(14);
  expect(progress).toBeGreaterThan(0);
  expect(confirmed).toBe(0);
  engine.advance(60);
  expect(progress).toBe(0);
  engine.probe.commands(`0:touch_move_to:${b.path};0.05:touch_start;1.0:touch_end`);
  engine.advance(70);
  expect(confirmed).toBe(1);
  // The alternative (XAG 107): a tap starts the fill, which runs on by itself; a second tap stops it.
  let alt = 0;
  let altProgress = 0;
  const e2 = run(() => {
    const r = og.ui.holdButton("Delete", "Delete save", { variant: "danger", toggle: true, holdMs: 600 });
    if (r.confirmed) alt++;
    altProgress = r.progress;
  });
  const b2 = find(e2, (e) => e.name.startsWith("Delete save"))!;
  tap(e2, b2.path);
  expect(altProgress).toBeGreaterThan(0);
  tap(e2, b2.path);
  e2.advance(60);
  expect(alt).toBe(0);
  expect(altProgress).toBe(0);
  tap(e2, b2.path);
  e2.advance(60);
  expect(alt).toBe(1);
  // A key starts it the same way (the button never needs holding Enter).
  focusOn(e2, "Button");
  key(e2, "Enter");
  e2.advance(60);
  expect(alt).toBe(2);
});

// ── toggles and choices ─────────────────────────────────────────────────

test("toggleSwitch and checkField: the whole row takes the tap and Space; the state is the game's", () => {
  // Wrong: only the knob taking the tap, or the component keeping its own state (game state ignored).
  let on = false;
  let checked = false;
  const engine = run(() => {
    on = og.ui.toggleSwitch("Music", on, { label: "Music", description: "Background tracks", width: { kind: "grow" } as never }).on;
    checked = og.ui.checkField("Terms", checked, { label: "I accept the terms" }).on;
  });
  const sw = find(engine, (e) => e.role === "switch")!;
  expect(sw.name).toContain("Music");
  // A tap at the row's start, on the label, far from the track.
  const label = find(engine, (e) => e.text === "Music")!;
  engine.probe.commands(`0:touch_move:${Math.round(label.x + 4)},${Math.round(label.y + label.height / 2)};0.05:touch_start;0.1:touch_end`);
  engine.advance(12);
  expect(on).toBeTrue();
  focusOn(engine, "Terms");
  key(engine, "Space");
  expect(checked).toBeTrue();
  expect(find(engine, (e) => e.role === "checkbox")!.states).toContain("checked");
});

test("radioGroup: arrows move the choice past a disabled option, mirrored in right to left", () => {
  // Wrong: ArrowRight going back in RTL, or selecting the disabled option.
  for (const dir of ["ltr", "rtl"] as const) {
    let sel = 0;
    const engine = run(() => {
      sel = og.ui.radioGroup("Diff", { label: "Difficulty", row: true, selected: sel, options: [{ label: "Story" }, { label: "Normal", disabled: true }, { label: "Hard" }, { label: "Nightmare" }] }).selected;
    }, 640, dir === "rtl" ? kit : undefined);
    if (dir === "rtl") {
      const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
      if (!loaded.ok) throw new Error(loaded.reason);
      og.i18n.setLocale(engine.world, "ar-EG");
      engine.advance(6);
    }
    expect(find(engine, (e) => e.role === "radiogroup")).toBeDefined();
    focusOn(engine, "Option#0");
    // Forward skips the disabled one (backward would wrap to Nightmare, 3).
    key(engine, dir === "rtl" ? "ArrowLeft" : "ArrowRight");
    expect(`${dir} ${sel}`).toBe(`${dir} 2`);
    // In RTL, the third option sits left of the first.
    const [a, c] = [byPath(engine, "Option#0")!, byPath(engine, "Option#2")!];
    expect(dir === "rtl" ? c.x < a.x : c.x > a.x).toBeTrue();
  }
});

test("segmented: a tap selects a segment, the bumpers do not (it is not a tab bar), Left and Right step", () => {
  let sel = 0;
  const engine = run(() => {
    sel = og.ui.segmented("View", { label: "View", selected: sel, items: [{ label: "Grid" }, { label: "List" }, { label: "Map" }] }).selected;
  });
  tap(engine, byPath(engine, "Item#2")!.path);
  expect(sel).toBe(2);
  focusOn(engine, "Item#2");
  key(engine, "ArrowLeft");
  expect(sel).toBe(1);
});

test("cycler: the pad's d-pad cycles the focused option with wrap; its arrows are 44 px on touch", () => {
  // Wrong: Left on the first option stopping (wrap ignored), or arrows too small for a thumb.
  let idx = 0;
  const engine = run(() => {
    idx = og.ui.cycler("Quality", ["Low", "Medium", "High"], idx, { label: "Quality" }).selected;
  });
  focusOn(engine, "Quality");
  pad(engine, "left");
  expect(idx).toBe(2);
  pad(engine, "right");
  expect(idx).toBe(0);
});

// ── ranges ──────────────────────────────────────────────────────────────

test("valueSlider: keys step it and commit, a drag follows the finger and commits on release, RTL runs from the right", () => {
  // Wrong: a drag that commits every tick, or a slider that fills from the left in RTL.
  for (const dir of ["ltr", "rtl"] as const) {
    let v = 0.5;
    let commits = 0;
    const engine = run(() => {
      const r = og.ui.valueSlider("Volume", v, { label: "Volume", min: 0, max: 1, step: 0.1, format: { percent: true } as never });
      v = r.value;
      if (r.committed) commits++;
    }, 640, dir === "rtl" ? kit : undefined);
    if (dir === "rtl") {
      const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
      if (!loaded.ok) throw new Error(loaded.reason);
      og.i18n.setLocale(engine.world, "ar-EG");
      engine.advance(6);
    }
    const s = find(engine, (e) => e.role === "slider")!;
    expect(s.value).toBe("50%");
    focusOn(engine, s.path.split("/").pop()!);
    key(engine, dir === "rtl" ? "ArrowLeft" : "ArrowRight");
    expect(`${dir} ${v.toFixed(2)}`).toBe(`${dir} 0.60`);
    expect(commits).toBe(1);
    // Drag the knob to the track's far end.
    const track = byPath(engine, "Track")!;
    const end = dir === "rtl" ? track.x + 2 : track.x + track.width - 2;
    engine.probe.commands(`0:touch_move_to:${s.path};0.05:touch_start;0.1:touch_move:${Math.round(end)},${Math.round(track.y + track.height / 2)};0.2:touch_move:${Math.round(end)},${Math.round(track.y + track.height / 2)};0.3:touch_end`);
    engine.advance(30);
    expect(v).toBeCloseTo(1, 5);
    expect(commits).toBe(2);
  }
});

test("rangeSlider: the knobs never cross and keep the minimum gap", () => {
  let lo = 20;
  let hi = 40;
  const engine = run(() => {
    const r = og.ui.rangeSlider("Price", lo, hi, { label: "Price", min: 0, max: 100, step: 5, minGap: 10 });
    lo = r.low;
    hi = r.high;
  });
  const low = find(engine, (e) => e.role === "slider" && e.name.includes("Minimum"))!;
  focusOn(engine, low.path.split("/").pop()!);
  for (let i = 0; i < 6; i++) key(engine, "ArrowRight");
  expect(lo).toBe(30);
  expect(hi).toBe(40);
});

test("numberStepper: the buttons step to the bounds and disable there; typed text is not needed", () => {
  // Wrong: going past max, or leaving + enabled at max.
  let v = 8;
  const engine = run(() => {
    v = og.ui.numberStepper("Players", v, { label: "Players", min: 1, max: 10, step: 1 }).value;
  });
  // On a fine pointer + is the pointer's shortcut for Right on the focused stepper: a named image to the pad graph.
  const plus = find(engine, (e) => e.name.startsWith("Increase"))!;
  expect(plus.role).toBe("image");
  tap(engine, plus.path);
  tap(engine, plus.path);
  tap(engine, plus.path);
  expect(v).toBe(10);
  expect(find(engine, (e) => e.name.startsWith("Increase"))!.disabled).toBeTrue();
});

// ── fields and pickers ──────────────────────────────────────────────────

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

test("searchBox: typing shows matching suggestions, Down walks them, Enter chooses one", () => {
  // Wrong: Down leaving the suggestions closed, or Enter submitting the query instead of the focused suggestion.
  let q = "";
  let chosen = null as string | null;
  const engine = run(() => {
    const r = og.ui.searchBox("Find", q, { suggestions: ["Iron sword", "Iron shield", "Oak bow"], filter: true });
    q = r.query;
    if (r.chosen !== null) chosen = r.chosen;
  });
  focusOn(engine, "Field");
  engine.probe.commands("0:type:iro");
  engine.advance(10);
  expect(q).toBe("iro");
  expect(find(engine, (e) => e.text === "Oak bow") === undefined).toBeTrue();
  key(engine, "ArrowDown");
  key(engine, "ArrowDown");
  key(engine, "Enter");
  expect(chosen).toBe("Iron shield");
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

test("colorPicker: a swatch commits its colour and the hex reads it", () => {
  let c: Api.PickedColor = { r: 1, g: 1, b: 1, a: 1 };
  let hex = "";
  const engine = run(() => {
    const r = og.ui.colorPicker("Paint", c, { swatches: ["#e63946", "#2a9d8f"], label: "Livery" });
    c = r.color;
    hex = r.hex;
  });
  tap(engine, byPath(engine, "Swatch#1")!.path);
  expect(hex).toBe("#2a9d8f");
  expect(c.g).toBeCloseTo(0x9d / 255, 3);
});

// ── key bindings ────────────────────────────────────────────────────────

test("keyBindingRow: a press waits for a key, a free key lands at once, a used one asks and Swap trades them", () => {
  // Wrong: the capture's key also moving the focus, a conflict applied without asking, or Swap leaving both on one key.
  const out: { r?: Api.KeyBindingResult } = {};
  const engine = run((world) => {
    out.r = og.ui.keyBindingRow(world, "Jump", { action: "jump", device: "keyboardMouse" });
    og.ui.keyBindingRow(world, "Crouch", { action: "crouch", device: "keyboardMouse" });
  }, 640, undefined, (world) => {
    og.input.defineAction(world, { name: "jump", label: "Jump", bindings: [{ control: "Space" }, { control: "pad.a" }] });
    og.input.defineAction(world, { name: "crouch", label: "Crouch", bindings: [{ control: "KeyC" }] });
  });
  const slot = byPath(engine, "Jump/Main/Bound/Slots/Slot#0")!;
  expect(slot.name).toContain("Jump");
  tap(engine, slot.path);
  expect(out.r!.capturing).toBeTrue();
  expect(find(engine, (e) => e.text.startsWith("Press a key"))).toBeDefined();
  // A free key lands.
  key(engine, "KeyJ");
  engine.advance(4);
  const jump = () => og.input.actions(engine.world).find((a) => a.name === "jump")!;
  expect(jump().bindings[0]!.control).toBe("KeyJ");
  expect(out.r!.capturing).toBeFalse();
  // Crouch's key asks.
  tap(engine, byPath(engine, "Jump/Main/Bound/Slots/Slot#0")!.path);
  key(engine, "KeyC");
  engine.advance(4);
  expect(out.r!.conflict).toBeTrue();
  expect(find(engine, (e) => e.role === "alert")).toBeDefined();
  expect(jump().bindings[0]!.control).toBe("KeyJ");
  tap(engine, byPath(engine, "Jump/Conflict/Swap")!.path);
  expect(jump().bindings[0]!.control).toBe("KeyC");
  expect(og.input.actions(engine.world).find((a) => a.name === "crouch")!.bindings[0]!.control).toBe("KeyJ");
  // Rebound, it offers a reset.
  tap(engine, byPath(engine, "Jump/Main/Bound/Reset")!.path);
  expect(jump().bindings[0]!.control).toBe("Space");
});

test("keyBindingRow: Escape cancels a capture and the focus stays on the slot", () => {
  const out: { r?: Api.KeyBindingResult } = {};
  const engine = run((world) => {
    out.r = og.ui.keyBindingRow(world, "Jump", { action: "jump", device: "keyboardMouse" });
  }, 640, undefined, (world) => {
    og.input.defineAction(world, { name: "jump", label: "Jump", bindings: [{ control: "Space" }] });
  });
  focusOn(engine, "Slot#0");
  key(engine, "Enter");
  expect(out.r!.capturing).toBeTrue();
  key(engine, "Escape");
  engine.advance(4);
  expect(out.r!.capturing).toBeFalse();
  expect(og.input.actions(engine.world)[0]!.bindings[0]!.control).toBe("Space");
  expectFocus(engine, "Slot#0");
});

// ── the on-screen keyboard ──────────────────────────────────────────────

// A keyboard's test is CPU-bound and nothing else: ~60 ticks of ~70 keys on
// the dev (-O0) host, no sleep and no wall-clock wait. 1.2–2.3 s idle; a gate
// at load 23 ran the kana one 6× slower (10.4 s when it took 114 ticks), so
// the limit leaves that headroom twice over.
const KEYBOARD_MS = 30_000;

test("onScreenKeyboard: taps type at the caret, shift types one capital, backspace erases, Done submits", () => {
  let value = "";
  let submitted = false;
  const engine = run(() => {
    const r = og.ui.onScreenKeyboard("Kb", true, { value, title: "Name your hero" });
    value = r.value;
    if (r.submitted) submitted = true;
  });
  const keyNamed = (n: string) => find(engine, (e) => e.role === "button" && e.name === n)!;
  tap(engine, keyNamed("Shift").path);
  tap(engine, keyNamed("A").path);
  tap(engine, keyNamed("b").path);
  tap(engine, keyNamed("c").path);
  tap(engine, keyNamed("Backspace").path);
  expect(value).toBe("Ab");
  tap(engine, keyNamed("Done").path);
  expect(submitted).toBeTrue();
}, KEYBOARD_MS);

test("onScreenKeyboard: kana compose underlined until a candidate commits; a pad walks the keys and the bumpers move the caret", () => {
  // Wrong: kana going straight into the text (no composition), or the katakana candidate missing.
  let value = "";
  let comp = "";
  const engine = run(() => {
    const r = og.ui.onScreenKeyboard("Kb", true, { value, layout: "kana", kana: true });
    value = r.value;
    comp = r.composition;
  });
  const keyNamed = (n: string) => find(engine, (e) => e.role === "button" && e.name === n)!;
  tap(engine, keyNamed("ゆ").path);
  tap(engine, keyNamed("う").path);
  expect(comp).toBe("ゆう");
  expect(value).toBe("");
  tap(engine, find(engine, (e) => e.role === "button" && e.name === "ユウ")!.path);
  expect(value).toBe("ユウ");
  expect(comp).toBe("");
  // The pad: the d-pad walks the keys (the focus shows), A types the focused one, a bumper moves the caret.
  pad(engine, "down");
  expect(focused(engine)).toContain("/Keys/Row#");
  const typed = find(engine, (e) => e.states.includes("focused"))!.name;
  pad(engine, "a");
  expect(comp).toBe(typed);
  // Enter commits the composition at the caret; LB moves the caret back one and Y erases before it.
  tap(engine, keyNamed("Done").path);
  expect(value).toBe(`ユウ${typed}`);
  pad(engine, "leftBumper");
  pad(engine, "y");
  expect(value).toBe(`ユ${typed}`);
}, KEYBOARD_MS);

// ── every breakpoint, direction, locale and text size ──────────────────

/** The controls a settings or a shop screen holds, labelled through `l` (the pseudo-locale's longer words). */
function gallery(l: (s: string) => string, world: Api.World): void {
  og.ui.pushButton("Play", l("Continue"), { variant: "primary", icon: "play", badge: "3" });
  og.ui.row("Buttons", { gap: 8, wrap: true }, () => {
    og.ui.pushButton("Ghost", l("Credits"), { variant: "ghost" });
    og.ui.pushButton("Danger", l("Delete"), { variant: "danger" });
    og.ui.iconButton("Close", "cross", { label: l("Close") });
  });
  og.ui.holdButton("Hold", l("Delete save"), { hint: l("Hold to delete") });
  og.ui.toggleSwitch("Music", true, { label: l("Music"), description: l("Background tracks"), width: { kind: "grow" } as never });
  og.ui.checkField("Terms", false, { label: l("I accept the terms") });
  og.ui.radioGroup("Diff", { label: l("Difficulty"), row: true, selected: 0, options: [{ label: l("Story") }, { label: l("Normal") }, { label: l("Hard") }] });
  og.ui.segmented("View", { label: l("View"), selected: 0, items: [{ label: l("Grid") }, { label: l("List") }] });
  og.ui.cycler("Quality", [l("Low"), l("Medium"), l("High")], 1, { label: l("Quality") });
  og.ui.valueSlider("Volume", 0.6, { label: l("Volume"), format: { percent: true } as never });
  og.ui.rangeSlider("Price", 20, 60, { label: l("Price"), min: 0, max: 100, step: 5 });
  og.ui.numberStepper("Players", 2, { label: l("Players"), min: 1, max: 8 });
  og.ui.inputField("Name", "", { label: l("Hero name"), placeholder: l("Type a name"), helper: l("Shown to other players") });
  og.ui.dropdown("Lang", { label: l("Language"), selected: 0, options: [{ label: "English" }, { label: "日本語" }] });
  og.ui.keyBindingRow(world, "Jump", { action: "jump", device: "keyboardMouse" });
}
const defineJump = (world: Api.World) => {
  og.input.defineAction(world, { name: "jump", label: "Jump", bindings: [{ control: "Space" }] });
};

test("right to left and the pseudo-locale: rows run from the right, slider fills from the right, the longer labels overlap nothing", () => {
  // Wrong: a control placing its parts by physical left, or a fixed width its longer label overflows.
  const engine = run((w) => gallery(pseudo, w), 640, kit, defineJump);
  const overlaps = () => (og.ui.overlaps() as unknown as Array<{ path: string; other: string }>).filter((o) => !/Popup|Thumb|Fill|Badge/.test(o.path + o.other));
  expect(overlaps()).toEqual([]);
  const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
  if (!loaded.ok) throw new Error(loaded.reason);
  og.i18n.setLocale(engine.world, "ar-EG");
  engine.advance(10);
  const x = (end: string) => byPath(engine, end)!.x;
  // A switch row: the label at the start (right), the track at the end (left).
  expect(x("Music/Track")).toBeLessThan(x("Music/Text/Label"));
  expect(x("Diff/Options/Option#0")).toBeGreaterThan(x("Diff/Options/Option#1"));
  expect(x("View/Strip/Track/Item#0")).toBeGreaterThan(x("View/Strip/Track/Item#1"));
  // The volume's fill (60 %) runs from the rail's right end, half a knob in from the track's edge.
  const track = byPath(engine, "Volume/Track")!;
  const fill = find(engine, (e) => e.path.startsWith(track.path + "/") && /Fill$/.test(e.path))!;
  expect(track.x + track.width - (fill.x + fill.width)).toBeLessThan(16);
  expect(fill.x - track.x).toBeGreaterThan(track.width * 0.3);
  expect(overlaps()).toEqual([]);
});

test("150 % text: every control grows with the text and cuts none of it, on a desktop and a phone", () => {
  // Wrong: a row, a cap or a knob sized in fixed pixels around its text, so at the largest text size it clips.
  const ui = native() as unknown as { og_node_ui_frame(id: number): string };
  for (const [bp, w, h, coarse] of [["desktop", 1920, 2400, false], ["phonePortrait", 390, 2400, true]] as const) {
    const frames: UiFrameSnapshot[] = [];
    for (const scale of [1, 1.5]) {
      const engine = run((wd) => gallery((s) => s, wd), Math.min(640, w - 32), kit, defineJump);
      og.ui.previewViewport(engine.world, { width: w, height: h, dpr: 1, coarsePointer: coarse, breakpoint: bp } as never);
      const font = og.load.font(engine.world, "fonts/NotoSansSC-Regular-subset.ttf");
      if (!font.ok) throw new Error(font.reason);
      og.ui.configure(engine.world, { font: font.value, textScale: scale });
      engine.advance(30);
      frames.push(JSON.parse(ui.og_node_ui_frame(engine.id)) as UiFrameSnapshot);
      engine.destroy();
    }
    const cell: UiCell = { screen: "controls", breakpoint: bp, locale: "", style: "", scale: 150, highContrast: false };
    expect(frames[1]!.texts.find((x) => x.text === "Volume")!.size).toBeCloseTo(frames[0]!.texts.find((x) => x.text === "Volume")!.size * 1.5, 1);
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
