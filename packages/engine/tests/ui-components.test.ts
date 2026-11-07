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
import { tableOrder } from "../src/ui/data/table.ts";
import { niceTicks, formatDelta, formatNumber } from "../src/ui/data/format.ts";
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

// ── tabs ────────────────────────────────────────────────────────────────

test("tabBar: the bumper steps to the next enabled tab, skipping a disabled one, and a tap selects", () => {
  // Wrong: a bumper that lands on the disabled tab, or ignores wrap.
  let sel = 1;
  const engine = run(() => {
    sel = og.ui.tabBar("Tabs", { tabs: [{ label: "Gear" }, { label: "Skills" }, { label: "Map", disabled: true }, { label: "Codex" }], selected: sel }, (k) => og.ui.label("Body", `Panel ${k}`, {})).selected;
  });
  expect(find(engine, (e) => e.role === "tablist")).toBeDefined();
  expect(find(engine, (e) => e.id === "Tab#1")?.states).toContain("selected");
  key(engine, "KeyE");
  expect(sel).toBe(3);
  key(engine, "KeyE");
  expect(sel).toBe(0);
  key(engine, "KeyQ");
  expect(sel).toBe(3);
  tap(engine, "Tab#1");
  expect(sel).toBe(1);
  expect(byPath(engine, "Tabs/Panel")?.role).toBe("tabpanel");
  expect(byPath(engine, "Panel/Body")?.text).toBe("Panel 1");
});

test("tabBar and actionHints: the bumpers and hints show the device's glyphs, following it live (keyboard, Xbox, PlayStation, Switch)", () => {
  // Wrong: fixed Q and E caps, or a hint that keeps the text it was given when the player picks up a pad.
  let device: Api.UiDevice = "keyboardMouse";
  const sys = defineSystem(`UcBuild${systems++}`, {}, (_q, _t, world: World) => {
    og.ui.setDevice(world, device);
    og.ui.anchor("Root", { anchor: "center", direction: "column", width: { kind: "fixed", px: 640 } } as never, () => {
      og.ui.tabBar("Tabs", { tabs: [{ label: "Gear" }, { label: "Skills" }], selected: 0 }, () => {});
      og.ui.actionHints("Hints", { hints: [{ action: "confirm", hint: "Select", text: "Given", icon: "key.given" }, { action: "back", hint: "Back" }, { action: "gameOwn", hint: "Own", text: "F", icon: "key.f" }] });
    });
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, {});
  if (!r.ok) throw new Error(r.reason);
  const engine = r.value;
  // A cap's text, or for a drawn symbol (PlayStation's face buttons) the name the cap gives it.
  const cap = (p: string) => byPath(engine, `${p}/Glyph/Text`)?.text ?? (byPath(engine, `${p}/Glyph/Symbol`) ? byPath(engine, `${p}/Glyph`)?.name : undefined);
  const caps = () => ["Prev", "Next", "Hint#0", "Hint#1", "Hint#2"].map(cap);
  const expected: Array<[Api.UiDevice, Array<string | undefined>]> = [
    ["keyboardMouse", ["Q", "E", "Enter", "Esc", "F"]],
    ["xbox", ["LB", "RB", "A", "B", "F"]],
    ["playstation", ["L1", "R1", "×", "○", "F"]],
    ["switch", ["L", "R", "B", "A", "F"]],
    ["keyboardMouse", ["Q", "E", "Enter", "Esc", "F"]],
  ];
  for (const [d, want] of expected) {
    device = d;
    engine.advance(4);
    expect([d, ...caps()]).toEqual([d, ...want]);
    // Wrong: × ○ set as text, which a style's font may not have (tofu); they are drawn, never text.
    const drawn = d === "playstation";
    expect([d, !!byPath(engine, "Hint#0/Glyph/Symbol"), !!byPath(engine, "Hint#0/Glyph/Text")]).toEqual([d, drawn, !drawn]);
  }
});

test("tabBar: tabs that do not fit scroll the selected one into view clear of the fades, fade where they are cut, and stop short of the bumpers", () => {
  // Wrong: a native scroll that never moves to the selected tab (Codex cut against RB), a fade that
  // never shows because a cut tab's box is the part in view, a strip that runs under the caps.
  let sel = 0;
  const engine = run(() => {
    sel = og.ui.tabBar("Tabs", { tabs: ["Equipment", "Skills", "Inventory", "Codex", "Journal"].map((label) => ({ label })), selected: sel }, () => {}).selected;
  }, 300);
  const clip = () => byPath(engine, "Tabs/Bar/Clip")!;
  const fades = () => ["FadeStart", "FadeEnd"].filter((f) => byPath(engine, `Clip/${f}`));
  expect(fades()).toEqual(["FadeEnd"]);
  for (let k = 0; k < 4; k++) key(engine, "KeyE");
  engine.advance(30);
  expect(sel).toBe(4);
  const tab = byPath(engine, "Tabs/Tab#4")!;
  const c = clip();
  // Whole: the last tab rests at the end, where nothing more lies and no fade is drawn.
  expect(tab.x).toBeGreaterThanOrEqual(c.x - 0.5);
  expect(tab.x + tab.width).toBeLessThanOrEqual(c.x + c.width + 0.5);
  expect(tab.clipped).toBeFalse();
  // A middle tab revealed clears the fade at its edge.
  key(engine, "KeyQ");
  engine.advance(30);
  const mid = byPath(engine, "Tabs/Tab#3")!;
  expect(mid.x).toBeGreaterThanOrEqual(clip().x + 27.5);
  key(engine, "KeyE");
  engine.advance(30);
  expect(fades()).toEqual(["FadeStart"]);
  const next = byPath(engine, "Tabs/Bar/Next")!;
  expect(c.x + c.width).toBeLessThanOrEqual(next.x + 0.5);
  key(engine, "KeyQ");
  key(engine, "KeyQ");
  key(engine, "KeyQ");
  key(engine, "KeyQ");
  engine.advance(30);
  expect(sel).toBe(0);
  expect(fades()).toEqual(["FadeEnd"]);
  expect(byPath(engine, "Tabs/Tab#0")!.x).toBeGreaterThanOrEqual(clip().x - 0.5);
});

// ── visual states ───────────────────────────────────────────────────────

test("states: parts move through §7.3 state groups (selection, interaction, expansion) a style's motion runs", () => {
  // Wrong: a part animated by the component's own tween (no group, so a style's motion and the probe's forceState never reach it).
  let sel = 0;
  let open: number[] = [];
  // Read inside the tick (og.ui answers the UI being built): each watched path's groups.
  const watched: Array<[path: string, group: string]> = [];
  let seen = new Map<string, string>();
  const engine = run(() => {
    sel = og.ui.tabBar("Tabs", { tabs: [{ label: "Gear" }, { label: "Skills" }], selected: sel }, () => {}).selected;
    open = og.ui.accordion("Faq", { sections: [{ title: "Rules" }], expanded: open }, () => og.ui.label("Text", "Body", {})).expanded.slice();
    seen = new Map(watched.map(([p, g]) => [`${p}:${g}`, og.ui.visualState(p, g)]));
  });
  const path = (end: string) => byPath(engine, end)!.path;
  for (const [end, groups] of [["Tab#0", ["selection", "interaction"]], ["Tab#1", ["selection", "interaction"]], ["Section#0/Header", ["expansion"]], ["Section#0/Header/Chevron", ["expansion"]]] as const) {
    for (const g of groups) watched.push([path(end), g]);
  }
  engine.advance(1);
  const vs = (end: string, group: string) => seen.get(`${path(end)}:${group}`);
  expect([vs("Tab#0", "selection"), vs("Tab#1", "selection")]).toEqual(["on", "off"]);
  expect(vs("Tab#1", "interaction")).toBe("normal");
  engine.probe.commands("0:touch_move_to:Tab#1");
  engine.advance(4);
  expect(vs("Tab#1", "interaction")).toBe("hover");
  tap(engine, "Tab#1");
  expect(sel).toBe(1);
  expect([vs("Tab#0", "selection"), vs("Tab#1", "selection")]).toEqual(["off", "on"]);
  expect([vs("Section#0/Header", "expansion"), vs("Section#0/Header/Chevron", "expansion")]).toEqual(["collapsed", "collapsed"]);
  tap(engine, "Section#0/Header");
  expect(open).toEqual([0]);
  expect([vs("Section#0/Header", "expansion"), vs("Section#0/Header/Chevron", "expansion")]).toEqual(["expanded", "expanded"]);
});

// ── accordion ───────────────────────────────────────────────────────────

test("accordion: a tap opens a section and says expanded; single closes the other", () => {
  // Wrong: a body built while closed, a header that does not say its state, single that keeps two open.
  let open: number[] = [0];
  const engine = run(() => {
    open = og.ui.accordion("Faq", { sections: [{ title: "Rules" }, { title: "Scoring", subtitle: "How points add up" }, { title: "Credits" }], expanded: open, single: true }, (i) => og.ui.label("Text", `Body ${i}`, {})).expanded.slice();
  });
  const header = (i: number) => find(engine, (e) => e.path.endsWith(`Section#${i}/Header`))!;
  expect(header(0).states).toContain("expanded");
  expect(header(1).states).toContain("collapsed");
  expect(elements(engine).filter((e) => e.id === "Body").length).toBe(1);
  tap(engine, "Section#1/Header");
  expect(open).toEqual([1]);
  expect(header(1).states).toContain("expanded");
  expect(find(engine, (e) => e.path.endsWith("Section#1/Body"))?.name).toBe("Scoring");
});

// ── menus ───────────────────────────────────────────────────────────────

test("menu: entries take menu roles, a tap chooses, a submenu opens beside its row and chooses", () => {
  // Wrong: a check entry announced as a plain item, a submenu that never opens, a chosen id from the wrong level.
  let open = true;
  let chosen: string | null = null;
  const engine = run(() => {
    og.ui.button("Opener", "Options", {});
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
  let chosen: string | null = null;
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
    let chosen: string | null = null;
    const engine = run(() => {
      og.ui.button("Opener", "Options", {});
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
  let chosen: string | null = null;
  const engine = run(() => {
    og.ui.button("Opener", "Options", {});
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
  let chosen: string | null = null;
  const engine = run(() => {
    if (og.ui.button("Opener", "Options", {}).clicked) open = true;
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

// ── dialogs ─────────────────────────────────────────────────────────────

test("confirm: the action answers confirmed, Escape cancels; a danger confirm says its tone", () => {
  let open = true;
  let result = "";
  const engine = run(() => {
    const r = og.ui.confirm("Delete", open, { title: "Delete save?", message: "This cannot be undone.", confirmLabel: "Delete", danger: true });
    if (r.confirmed) (result = "confirmed", open = false);
    if (r.cancelled) (result = "cancelled", open = false);
  });
  engine.advance(20);
  expect(find(engine, (e) => e.id === "Tone")).toBeDefined();
  tap(engine, "Actions/confirm");
  expect(result).toBe("confirmed");
  open = true;
  result = "";
  engine.advance(30);
  key(engine, "Escape");
  expect(result).toBe("cancelled");
});

test("dialog: on a phone the actions stack with the primary first; on a desktop the primary is last in the row", () => {
  // Wrong: the same order on both, or a row on a phone.
  const engine = run(() => {
    og.ui.dialog("Quit", true, { title: "Leave the match?", actions: [{ id: "stay", label: "Stay", variant: "secondary" }, { id: "leave", label: "Leave", variant: "primary" }] }, () => {});
  });
  engine.advance(20);
  const order = () => elements(engine).filter((e) => e.parent === "Actions").sort((a, b) => a.y - b.y || a.x - b.x).map((e) => e.id);
  expect(order()).toEqual(["stay", "leave"]);
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as never);
  engine.advance(20);
  expect(order()).toEqual(["leave", "stay"]);
  const [a, b] = elements(engine).filter((e) => e.parent === "Actions");
  expect(a!.x).toBe(b!.x);
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

// ── navigation ──────────────────────────────────────────────────────────

test("pagination: the first, the last and the current page's neighbours with gaps; next pages on", () => {
  // Wrong: every page listed, or no gap between 1 and 9.
  let page = 9;
  const engine = run(() => {
    page = og.ui.pagination("Pages", { pages: 20, page }).page;
  });
  const labels = () => elements(engine).filter((e) => e.parent === "Pages" && (e.id.startsWith("Page#") || e.id.startsWith("Gap#"))).sort((a, b) => a.x - b.x).map((e) => e.text || e.name);
  expect(labels()).toEqual(["1", "…", "9", "10", "11", "…", "20"]);
  tap(engine, "Pages/Next");
  expect(page).toBe(10);
  expect(find(engine, (e) => e.role === "navigation")?.value).toBe("11 / 20");
});

test("carousel: the next arrow moves on a page, the dots say which is current", () => {
  let index = 0;
  const engine = run(() => {
    index = og.ui.carousel("Featured", { count: 4, index, height: { kind: "fixed", px: 160 } }, (k) => og.ui.label("Title", `Banner ${k}`, {})).index;
  });
  // A page is as wide as the viewport between the arrows, and the arrows stay beside it.
  const clip = byPath(engine, "Viewport/Clip")!;
  const page = byPath(engine, "Track/Page#0")!;
  expect(page.width).toBeGreaterThan(clip.width * 0.9);
  expect(byPath(engine, "Viewport/Next")!.x).toBeGreaterThanOrEqual(clip.x + clip.width - 1);
  tap(engine, "Viewport/Next");
  expect(index).toBe(1);
  expect(find(engine, (e) => e.id === "Dot#1")?.states).toContain("selected");
  tap(engine, "Dots/Dot#3");
  expect(index).toBe(3);
  expect(find(engine, (e) => e.id === "Next")?.disabled).toBeTrue();
});

test("breadcrumbs: a long path collapses its middle into a menu that goes back to a hidden crumb", () => {
  let chosen: number | null = null;
  const engine = run(() => {
    const r = og.ui.breadcrumbs("Path", { items: [{ label: "Home" }, { label: "Inventory" }, { label: "Weapons" }, { label: "Swords" }, { label: "Rare" }, { label: "Moonblade" }], maxShown: 4 });
    if (r.chosen !== null) chosen = r.chosen;
  });
  const crumbs = elements(engine).filter((e) => e.parent === "Path" && e.id.startsWith("Crumb#")).map((e) => e.id);
  expect(crumbs).toEqual(["Crumb#0", "Crumb#4", "Crumb#5"]);
  expect(byPath(engine, "Path/Crumb#5")?.states).toContain("selected");
  tap(engine, "Path/More");
  engine.advance(12);
  const weapons = find(engine, (e) => e.role === "menuitem" && e.name === "Weapons")!;
  tapAt(engine, weapons);
  expect(chosen).toBe(2);
});

test("splitView: a press of the focused divider steps to the next snap ratio", () => {
  let ratio = 0.5;
  const engine = run(() => {
    ratio = og.ui.splitView("Split", { ratio, height: { kind: "fixed", px: 200 } }, () => og.ui.label("A", "List", {}), () => og.ui.label("B", "Detail", {})).ratio;
  });
  engine.advance(4);
  tap(engine, "Split/Divider");
  expect(ratio).toBeCloseTo(0.67, 2);
  expect(byPath(engine, "Split/Divider")?.role).toBe("separator");
});

test("radialMenu: a stick's aim chooses the slice it points at, a release picks it", () => {
  // Wrong: slices counted from 3 o'clock, or counterclockwise.
  let picked: number | null = null;
  let aim = { x: 1, y: 0 };
  let release = false;
  const engine = run(() => {
    const r = og.ui.radialMenu("Wheel", true, { items: [{ label: "Up" }, { label: "Right" }, { label: "Down" }, { label: "Left" }], aim, release });
    if (r.chosen !== null) picked = r.chosen;
  });
  engine.advance(4);
  expect(find(engine, (e) => e.role === "menu")?.value).toBe("Right");
  aim = { x: 0, y: 1 };
  release = true;
  engine.advance(2);
  expect(picked).toBe(2);
});

test("radialMenu: the pointer picks by angle anywhere on a slice; a release in the hole cancels; Right steps round", () => {
  // Wrong: slices hit-tested as overlapping rectangles (the topmost wins everywhere), or a press in the hole that chooses.
  let picked: number | null = null;
  let hovered: number | null = null;
  const engine = run(() => {
    const r = og.ui.radialMenu("Wheel", true, { items: [{ label: "Up" }, { label: "Right" }, { label: "Down" }, { label: "Left" }, { label: "Bag", disabled: true }, { label: "Map" }] });
    if (r.chosen !== null) picked = r.chosen;
    hovered = r.hovered;
  });
  engine.advance(20);
  const w = find(engine, (e) => e.role === "menu")!;
  const cx = w.x + w.width / 2, cy = w.y + w.height / 2, R = w.width / 2;
  // Six slices of 60° from the top, clockwise: a point near the edge between Down (2) and Left (3), on Left's side,
  // near the rim and near the hole.
  const at = (deg: number, r: number) => `${Math.round(cx + Math.sin((deg * Math.PI) / 180) * r)},${Math.round(cy - Math.cos((deg * Math.PI) / 180) * r)}`;
  for (const [deg, r, want] of [[150 + 4, R * 0.95, 3], [150 - 4, R * 0.95, 2], [150 + 6, R * 0.5, 3], [0, R * 0.7, 0], [330 + 25, R * 0.8, 0], [300, R * 0.8, 5]] as const) {
    engine.probe.commands(`0:touch_move:${at(deg, r)}`);
    engine.advance(3);
    expect([deg, hovered]).toEqual([deg, want]);
  }
  engine.probe.commands(`0:touch_move:${at(190, R * 0.8)};0.05:touch_start;0.1:touch_end`);
  engine.advance(12);
  expect(picked).toBe(3);
  picked = null;
  engine.probe.commands(`0:touch_move:${Math.round(cx)},${Math.round(cy)};0.05:touch_start;0.1:touch_end`);
  engine.advance(12);
  expect(picked).toBeNull();
  // Right steps clockwise, skipping the disabled Bag (4).
  key(engine, "ArrowRight");
  expect(hovered).toBe(5);
  key(engine, "Enter");
  expect(picked).toBe(5);
});

test("radialMenu: a slice's cost shows under its label; an unaffordable slice stays choosable", () => {
  // Wrong: the cost dropped, or an unaffordable slice made unchoosable (the game refuses it and says why).
  let picked: number | null = null;
  const engine = run(() => {
    const r = og.ui.radialMenu("Wheel", true, { items: [{ label: "Laser", icon: "bolt", cost: "100" }, { label: "Cannon", icon: "bomb", cost: "150", unaffordable: true }, { label: "Frost", icon: "drop", cost: "120" }], aim: { x: 0.9, y: 0.4 }, release: true });
    if (r.chosen !== null) picked = r.chosen;
  });
  engine.advance(20);
  const cost = elements(engine).filter((e) => e.path.endsWith("/Label/Cost/Text"));
  expect(cost.map((e) => e.text).sort()).toEqual(["100", "120", "150"]);
  expect(picked).toBe(1);
});

// ── data ────────────────────────────────────────────────────────────────

const ROWS = Array.from({ length: 10_000 }, (_, i) => ({ name: `Player ${String(i).padStart(5, "0")}`, score: (i * 7919) % 10_007, guild: ["Ash", "Brine", "Cinder"][i % 3] }));
const COLUMNS = [{ key: "name", label: "Name" }, { key: "guild", label: "Guild" }, { key: "score", label: "Score", numeric: true }];

test("table: 10,000 rows build only a screenful; the score header sorts high to low, then low to high", () => {
  // Wrong: every row built, a sort by text on a numeric column, a sort that runs every tick.
  let sortBy = -1;
  let descending = false;
  const engine = run(() => {
    const r = og.ui.table("Board", { rows: ROWS, columns: COLUMNS, sortBy, descending, height: { kind: "fixed", px: 400 }, label: "Leaderboard" });
    sortBy = r.sortBy;
    descending = r.descending;
  }, 720);
  const rows = () => elements(engine).filter((e) => e.role === "row");
  expect(rows().length).toBeGreaterThan(5);
  expect(rows().length).toBeLessThan(30);
  expect(find(engine, (e) => e.role === "table")?.value).toBe("10000 rows");
  tap(engine, "Header/Col#2");
  expect([sortBy, descending]).toEqual([2, true]);
  const first = () => rows().sort((a, b) => a.y - b.y)[0]!;
  const top = Math.max(...ROWS.map((r) => r.score));
  expect(elements(engine).some((e) => e.path.endsWith("/Cell#2") && e.text === formatNumber(top, "grouped") && e.path.startsWith(first().path))).toBeTrue();
  tap(engine, "Header/Col#2");
  expect(descending).toBeFalse();
  expect(byPath(engine, "Header/Col#2")?.value).toBe("sorted ascending");
});

test("table: over 10,000 sorted rows a tick reads only the rows in view; the filter and sort run once per change", () => {
  // The cost is counted, not timed: every read of a row goes through this
  // proxy, so a cache that misses (or a sort run every tick) reads all
  // 10,000 again and fails on any machine, however loaded.
  let reads = 0;
  const rows = new Proxy(ROWS, {
    get(target, p, recv) {
      if (typeof p === "string" && /^\d+$/.test(p)) reads++;
      return Reflect.get(target, p, recv);
    },
  });
  let sortBy = 2;
  const engine = run(() => {
    og.ui.table("Board", { rows, columns: COLUMNS, sortBy, descending: true, filter: "1", height: { kind: "fixed", px: 400 } });
  }, 720);
  const tick = (): number => {
    reads = 0;
    engine.advance(1);
    return reads;
  };
  // Cached: a screenful of rows by three columns (and a few reads beside), never the table.
  const screenful = 30 * (COLUMNS.length + 1);
  const steady = Array.from({ length: 30 }, tick);
  expect(Math.max(...steady)).toBeLessThan(screenful);
  // A new sort reads every row once, on the tick it changes; the ticks after it are cheap again.
  sortBy = 0;
  expect(tick()).toBeGreaterThanOrEqual(ROWS.length);
  expect(Math.max(...Array.from({ length: 10 }, tick))).toBeLessThan(screenful);
  // The wall-clock time is the perf lane's to judge; here it is only reported.
  const t0 = performance.now();
  engine.advance(60);
  console.info(`table: ${((performance.now() - t0) / 60).toFixed(2)} ms per tick over 10,000 sorted rows (not asserted)`);
});

test("tableOrder: filters by every filterable column, sorts numbers by value, keeps equal keys in row order", () => {
  const rows = [{ n: "b", v: 10 }, { n: "a", v: 9 }, { n: "c", v: 10 }, { n: "ab", v: 100 }];
  const cols = [{ key: "n" }, { key: "v", numeric: true }];
  expect(Array.from(tableOrder(rows, cols, 1, false, ""))).toEqual([1, 0, 2, 3]);
  expect(Array.from(tableOrder(rows, cols, 1, true, ""))).toEqual([3, 0, 2, 1]);
  expect(Array.from(tableOrder(rows, cols, 0, false, "a"))).toEqual([1, 3]);
  expect(Array.from(tableOrder(rows, cols, -1, false, "10"))).toEqual([0, 2, 3]);
});

test("table: a filter that keeps nothing says no matches, an error offers Retry", () => {
  let retry = false;
  let error = "";
  const engine = run(() => {
    if (og.ui.table("Board", { rows: ROWS.slice(0, 50), columns: COLUMNS, filter: "zzz", error }).retry) retry = true;
  });
  expect(find(engine, (e) => e.role === "status")?.name).toBe("No matches");
  error = "Could not load the leaderboard";
  engine.advance(4);
  expect(find(engine, (e) => e.role === "alert")?.name).toBe("Could not load the leaderboard");
  tap(engine, "State/Retry");
  expect(retry).toBeTrue();
});

test("dataList: loading says busy, empty says so, rows are virtualised", () => {
  let desc: Record<string, unknown> = { loading: true };
  const engine = run(() => {
    og.ui.dataList("Mail", { ...desc, height: { kind: "fixed", px: 300 } }, (i) => og.ui.label("Subject", `Letter ${i}`, {}));
  });
  expect(find(engine, (e) => e.role === "status")?.states).toContain("busy");
  desc = { count: 0, emptyLabel: "No mail" };
  engine.advance(2);
  expect(find(engine, (e) => e.role === "status")?.name).toBe("No mail");
  desc = { count: 5000 };
  engine.advance(2);
  const built = elements(engine).filter((e) => e.id === "Subject").length;
  expect(built).toBeGreaterThan(3);
  expect(built).toBeLessThan(20);
});

test("tree: a tap on a branch opens it, showing its children one level deeper", () => {
  let expanded: string[] = [];
  let chosen: string | null = null;
  const engine = run(() => {
    const r = og.ui.tree("Codex", { items: [{ id: "beasts", label: "Beasts" }, { id: "wolf", parent: "beasts", label: "Wolf" }, { id: "bear", parent: "beasts", label: "Bear" }, { id: "places", label: "Places" }], expanded });
    expanded = r.expanded.slice();
    if (r.chosen !== null) chosen = r.chosen;
  });
  const items = () => elements(engine).filter((e) => e.role === "treeitem");
  expect(items().map((e) => e.name)).toEqual(["Beasts", "Places"]);
  expect(items()[0]!.states).toContain("collapsed");
  tapAt(engine, items()[0]!);
  expect(expanded).toEqual(["beasts"]);
  expect(items().map((e) => `${e.name}:${e.value}`)).toEqual(["Beasts:level 1", "Wolf:level 2", "Bear:level 2", "Places:level 1"]);
  tapAt(engine, items()[2]!);
  expect(chosen).toBe("bear");
});

test("nodeGraph: zoom in scales it, a node tap chooses it, a locked node says so", () => {
  let zoom = 1;
  let chosen: string | null = null;
  const engine = run(() => {
    const r = og.ui.nodeGraph("Skills", { zoom, nodes: [{ id: "root", x: 0, y: 0, label: "Strength", state: "done" }, { id: "cleave", x: 120, y: 120, label: "Cleave", state: "available" }, { id: "rage", x: 240, y: 0, label: "Rage", state: "locked" }], links: [{ from: "root", to: "cleave" }, { from: "root", to: "rage" }] });
    zoom = r.zoom;
    if (r.chosen !== null) chosen = r.chosen;
  });
  expect(find(engine, (e) => e.name === "Rage")?.value).toBe("locked");
  tap(engine, "Zoom/In");
  expect(zoom).toBeCloseTo(1.25, 5);
  tapAt(engine, find(engine, (e) => e.role === "button" && e.name === "Cleave")!);
  expect(chosen).toBe("cleave");
});

test("nodeGraph: a triangle node and a node's side: allied and hostile differ by shape and by colour, with no letters over them", () => {
  // Wrong: a triangle drawn as the circle (an unknown shape), sides told by colour alone or not at all (every node
  // on the panel's ground), the player's side in a primary a style may share with danger, a sided node's label's
  // first letters over its mark, no ground texture, or caption room kept under nodes that show none.
  const nodes: Api.GraphNode[] = [
    { id: "home", x: 0, y: 0, label: "Allied", shape: "triangle", team: 0, state: "current" },
    { id: "rim", x: 120, y: 300, label: "Hostile", shape: "diamond", team: 1 },
    { id: "far", x: 240, y: 0, label: "Unclaimed" },
  ];
  let palette: Record<string, string> = {};
  let ground: unknown = null;
  const engine = run(() => {
    palette = og.ui.frameTheme().palette as Record<string, string>;
    og.ui.nodeGraph("Map", { nodes, labels: false, height: { kind: "fixed", px: 300 }, ground: ground as never });
  }, 640, kit);
  const tex = og.load.texture(engine.world, "textures/backdrop.png");
  if (!tex.ok) throw new Error(tex.reason);
  ground = tex.value;
  engine.advance(4);
  const draws = drawsDuring(() => engine.advance(1));
  const node = (i: number) => [...draws.entries()].find(([p]) => p.endsWith(`/Canvas/Node#${i}`))?.[1]?.find((d) => d.shape) ?? {};
  expect(node(0).shape).toMatchObject({ kind: "polygon", sides: 3 });
  expect(node(1).shape).toMatchObject({ kind: "polygon", sides: 4 });
  const fill = (i: number) => JSON.stringify(node(i).fills ?? null);
  // The sides' fills differ from each other and from an unsided node's ground.
  expect(fill(0)).not.toBe(fill(1));
  expect(fill(0)).not.toBe(fill(2));
  expect(fill(1)).not.toBe(fill(2));
  expect(fill(2)).toContain("panelRaised");
  // The player's side in the ink, never the primary (fantasy's gold primary beside its gold danger told the sides
  // by shape alone): the fill's top stop is the ink lightened.
  const rgb = (h: string) => [1, 3, 5].map((i) => Number.parseInt(h.slice(i, i + 2), 16) / 255);
  const top = (node(0).fills?.[0] as unknown as { stops: { color: { color: { r: number; g: number; b: number } } }[] }).stops[0]!.color.color;
  const lightened = (h: string) => rgb(h).map((c) => c + (1 - c) * 0.2);
  const off = (h: string) => Math.max(...lightened(h).map((c, i) => Math.abs(c - [top.r, top.g, top.b][i]!)));
  expect(palette.ink).not.toBe(palette.primary);
  expect(off(palette.ink!)).toBeLessThan(0.01);
  // The ground's texture tiled faintly over the field, in the ink.
  const canvas = [...draws.entries()].find(([p]) => p.endsWith("/Canvas"))?.[1]?.[0];
  expect(canvas?.fills?.[0]).toMatchObject({ kind: "image", fit: "pattern" });
  expect(JSON.stringify(canvas?.fills?.[0])).toContain("ink");
  // With no captions the fit keeps no caption's room under the bottom row: it sits a node and the fade above the
  // window's foot (two caption lines higher before).
  const view = byPath(engine, "Map/Window/View")!;
  const low = byPath(engine, "Canvas/Node#1")!;
  expect(view.y + view.height - (low.y + low.height)).toBeLessThan(28 + 8);
  // The side's mark is its shape and colour: no letters; an unsided node keeps its initials.
  expect(byPath(engine, "Node#0/Label")).toBe(undefined);
  expect(byPath(engine, "Node#1/Label")).toBe(undefined);
  expect(byPath(engine, "Node#2/Label")?.text).toBe("Un");
  // A screen reader still hears each by its label and state.
  expect(find(engine, (e) => e.role === "button" && e.name === "Allied")?.value).toBe("current");
});

test("nodeGraph: at 200 % text no node or caption overlaps another, and the fit keeps every node clear of the zoom controls", () => {
  // Wrong: a fit that packs rows by the graph's units alone (two-line captions at 200 % run into
  // the next row's nodes), or one that lets the bottom row sit under the zoom controls.
  const nodes = [
    { id: "root", x: 200, y: 0, label: "Warrior" }, { id: "cleave", x: 60, y: 110, label: "Cleave" }, { id: "guard", x: 200, y: 110, label: "Guard" },
    { id: "rage", x: 340, y: 110, label: "Rage" }, { id: "whirl", x: 60, y: 220, label: "Whirlwind" }, { id: "wall", x: 200, y: 220, label: "Iron wall" }, { id: "berserk", x: 340, y: 220, label: "Berserk" },
  ];
  for (const scale of [1, 2]) {
    const engine = run(() => og.ui.nodeGraph("Skills", { nodes, height: { kind: "fixed", px: 640 } } as never), 1100);
    og.ui.configure(engine.world, { textScale: scale });
    engine.advance(12);
    const boxes = elements(engine).filter((e) => /\/(Node|Caption)#\d+$/.test(e.path));
    expect(boxes.length).toBe(nodes.length * 2);
    const hit = (a: Element, b: Element) => a.x < b.x + b.width - 0.5 && b.x < a.x + a.width - 0.5 && a.y < b.y + b.height - 0.5 && b.y < a.y + a.height - 0.5;
    const overlaps: string[] = [];
    boxes.forEach((a, i) => boxes.slice(i + 1).forEach((b) => hit(a, b) && overlaps.push(`${scale}: ${a.path.split("/").pop()} × ${b.path.split("/").pop()}`)));
    expect(overlaps).toEqual([]);
    const zoom = find(engine, (e) => /\/Zoom$/.test(e.path))!;
    expect(boxes.filter((b) => hit(b, zoom)).map((b) => b.path)).toEqual([]);
    engine.destroy();
  }
});

test("nodeGraph: a graph wider than its window opens centred on its root, fades where it is cut, and a drag pans it", () => {
  // Wrong: a graph that opens at its top-left corner (the root half out of view), cut edges with
  // no sign that more lies there, or a canvas that does not follow the finger.
  const nodes = [
    { id: "root", x: 200, y: 0, label: "Warrior" }, { id: "cleave", x: 60, y: 110, label: "Cleave" }, { id: "guard", x: 200, y: 110, label: "Guard" },
    { id: "rage", x: 340, y: 110, label: "Rage" }, { id: "whirl", x: 60, y: 220, label: "Whirlwind" }, { id: "wall", x: 200, y: 220, label: "Iron wall" }, { id: "berserk", x: 340, y: 220, label: "Berserk" },
  ];
  const links = [{ from: "root", to: "cleave" }, { from: "root", to: "guard" }, { from: "root", to: "rage" }];
  // Nested in a page, as games and the kit place it: its window is found by its full path.
  const engine = run(() => og.ui.column("Page", { width: { kind: "grow" } } as never, () => og.ui.nodeGraph("Skills", { nodes, links, height: { kind: "fixed", px: 340 } } as never)), 358);
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as never);
  og.ui.configure(engine.world, { textScale: 1.5 });
  engine.advance(30);
  const view = find(engine, (e) => e.path.endsWith("Skills/Window/View"))!;
  const root = () => find(engine, (e) => e.path.endsWith("Canvas/Node#0"))!;
  const r0 = root();
  expect(Math.abs(r0.x + r0.width / 2 - (view.x + view.width / 2))).toBeLessThan(2);
  // Centred on the top row, the root is whole and clear of the top fade (28 px), not cut by the window.
  expect(r0.y).toBeGreaterThanOrEqual(view.y + 28 - 0.5);
  // Panned, any node may lie anywhere in the window: the zoom controls sit in a bar below it.
  const zoomBar = find(engine, (e) => /\/Zoom$/.test(e.path))!;
  expect(zoomBar.y).toBeGreaterThanOrEqual(view.y + view.height - 0.5);
  // With a node of the second row selected (it and the root do not both fit at 200 %), its row
  // starts the window below the top fade: the row above is out of view, never a node cut in half.
  {
    const e2 = run(() => og.ui.column("Page", { width: { kind: "grow" } } as never, () => og.ui.nodeGraph("Skills", { nodes, links, selected: "guard", height: { kind: "fixed", px: 340 } } as never)), 358);
    og.ui.previewViewport(e2.world, { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as never);
    og.ui.configure(e2.world, { textScale: 1.5 });
    e2.advance(30);
    const v2 = find(e2, (e) => e.path.endsWith("Skills/Window/View"))!;
    const sel = find(e2, (e) => e.path.endsWith("Canvas/Node#2"))!;
    expect(sel.y + sel.height).toBeLessThanOrEqual(v2.y + v2.height + 0.5);
    // No node is shown cut at the top: each one in view starts below the top fade.
    const cut = elements(e2).filter((e) => /Canvas\/Node#\d+$/.test(e.path) && e.height > 0 && e.y < v2.y + 28 - 0.5);
    expect(cut.map((e) => `${e.path} ${e.y}`)).toEqual([]);
    e2.destroy();
  }
  // Cut on both sides: a fade over each.
  expect(find(engine, (e) => e.path.endsWith("Skills/Window/FadeStart"))).toBeDefined();
  expect(find(engine, (e) => e.path.endsWith("Skills/Window/FadeEnd"))).toBeDefined();
  // A drag to the left pans the graph with the finger.
  const y = Math.round(view.y + view.height - 20);
  engine.probe.commands(`0:touch_move:${Math.round(view.x + view.width / 2)},${y};0.05:touch_start;0.1:touch_move:${Math.round(view.x + view.width / 2 - 60)},${y};0.15:touch_move:${Math.round(view.x + view.width / 2 - 120)},${y};0.2:touch_end`);
  engine.advance(40);
  // The root follows the finger left, or past the window's edge with the fling.
  const r1 = find(engine, (e) => e.path.endsWith("Canvas/Node#0"));
  expect(r1 === undefined || r1.x < r0.x - 40).toBeTrue();
  // More of it now lies before the window: the start fade shows.
  expect(find(engine, (e) => e.path.endsWith("Skills/Window/FadeStart"))).toBeDefined();
});

test("chart: a figure described from its data; each category a target that answers its press", () => {
  // Wrong: an unnamed picture, categories only reachable by pointer.
  let clicked: number | null = null;
  const engine = run(() => {
    const r = og.ui.chart("Gold", { kind: "bar", title: "Gold per day", categories: ["Mon", "Tue", "Wed", "Thu"], series: [{ label: "Earned", values: [120, 340, 90, 410] }, { label: "Spent", values: [80, 200, 150, 60] }] });
    if (r.clicked !== null) clicked = r.clicked;
  });
  engine.advance(30);
  const fig = find(engine, (e) => e.role === "figure")!;
  expect(fig.name).toBe("Gold per day");
  const cats = elements(engine).filter((e) => e.id.startsWith("Category#"));
  expect(cats.map((e) => e.name)).toEqual(["Mon: Earned 120, Spent 80", "Tue: Earned 340, Spent 200", "Wed: Earned 90, Spent 150", "Thu: Earned 410, Spent 60"]);
  expect(cats.every((e) => e.states.includes("focusable"))).toBeTrue();
  tapAt(engine, cats[1]!);
  expect(clicked).toBe(1);
  // The value axis reaches past the largest value on a round step.
  const ticks = elements(engine).filter((e) => e.parent.startsWith("Tick#")).map((e) => e.text);
  expect(ticks[0]).toBe("500");
});

test("chart: a pointer anywhere inside a pie's wedge picks that wedge, at its edges and by the centre too; mirrored in right to left", () => {
  // Wrong: wedges hit-tested by rectangles (a neighbour's overlaps a wedge's edge and the centre), or angles counted the wrong way.
  for (const dir of ["ltr", "rtl"] as const) {
    let hovered: number | null = null;
    let clicked: number | null = null;
    const engine = run(() => {
      const r = og.ui.chart("Loot", { kind: "pie", categories: ["A", "B", "C"], series: [{ values: [1, 1, 2] }] });
      hovered = r.hovered;
      if (r.clicked !== null) clicked = r.clicked;
    }, 640, dir === "rtl" ? kit : undefined);
    if (dir === "rtl") {
      const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
      if (!loaded.ok) throw new Error(loaded.reason);
      og.i18n.setLocale(engine.world, "ar-EG");
    }
    engine.advance(40);
    const plot = byPath(engine, "Body/Plot")!;
    const cx = plot.x + plot.width / 2, cy = plot.y + plot.height / 2, R = plot.width / 2 - 8;
    // Clockwise from 12 o'clock (counter-clockwise in right to left): A a quarter, B a quarter, C a half.
    const s = dir === "rtl" ? -1 : 1;
    const at = (deg: number, r: number) => `${Math.round(cx + s * Math.sin((deg * Math.PI) / 180) * r)},${Math.round(cy - Math.cos((deg * Math.PI) / 180) * r)}`;
    for (const [deg, r, want] of [[86, 0.55, 0], [94, 0.55, 1], [176, 0.9, 1], [184, 0.9, 2], [45, 0.12, 0], [135, 0.12, 1], [300, 0.5, 2], [3, 0.95, 0]] as const) {
      engine.probe.commands(`0:touch_move:${at(deg, R * r)}`);
      engine.advance(3);
      expect([dir, deg, r, hovered]).toEqual([dir, deg, r, want]);
    }
    engine.probe.commands(`0:touch_move:${at(135, R * 0.6)};0.05:touch_start;0.1:touch_end`);
    engine.advance(12);
    expect(clicked).toBe(1);
  }
});

test("chart: a donut lists its wedges with their shares; a pie with no data shows the empty state", () => {
  let values = [3, 1];
  const engine = run(() => {
    og.ui.chart("Loot", { kind: "donut", categories: ["Common", "Rare"], series: [{ values }] });
  });
  engine.advance(30);
  expect(elements(engine).filter((e) => e.id.startsWith("Row#")).map((e) => e.name)).toEqual(["Common, 3, 75%", "Rare, 1, 25%"]);
  values = [0, 0];
  engine.advance(2);
  expect(find(engine, (e) => e.role === "status")?.name).toBe("No data yet");
});

test("statCompare: each row says its change and whether it is better, lower-is-better turning a fall green", () => {
  const engine = run(() => {
    og.ui.statCompare("Compare", { rows: [{ label: "Attack", current: 120, next: 138 }, { label: "Weight", current: 12, next: 9, lowerIsBetter: true }, { label: "Speed", current: 5, next: 5 }] });
  });
  expect(elements(engine).filter((e) => e.role === "listitem").map((e) => e.name)).toEqual([
    "Attack: 120 to 138, +18, better",
    "Weight: 12 to 9, −3, better",
    "Speed: 5, no change",
  ]);
});

test("calendar: November 2026 from Monday, today ringed and named, a tap selects, the bumper turns the month", () => {
  // Wrong: the month starting on the wrong weekday, a day from the next month selectable.
  let selected = "";
  let month = 11;
  let year = 2026;
  const engine = run(() => {
    const r = og.ui.calendar("Login", { year, month, today: "2026-11-01", selected, bumpers: true, marks: [{ date: "2026-11-03", kind: "claimed" }] });
    if (r.selected !== null) selected = r.selected;
    month = r.month;
    year = r.year;
  });
  const cells = () => elements(engine).filter((e) => e.role === "gridcell");
  expect(cells().length).toBe(42);
  // 1 November 2026 is a Sunday: the first row is 26 October to 1 November.
  const first = cells().find((e) => e.path.includes("Week#0/Cell#0"))!;
  expect(first.name).toBe("Monday, October 26, 2026");
  expect(first.disabled).toBeTrue();
  expect(cells().find((e) => e.path.includes("Week#0/Cell#6"))!.name).toBe("Sunday, November 1, 2026, today");
  expect(cells().some((e) => e.name === "Tuesday, November 3, 2026, claimed")).toBeTrue();
  tapAt(engine, cells().find((e) => e.name.startsWith("Friday, November 13"))!);
  expect(selected).toBe("2026-11-13");
  key(engine, "KeyE");
  expect([year, month]).toEqual([2026, 12]);
  key(engine, "KeyE");
  expect([year, month]).toEqual([2027, 1]);
});

test("card: a number plate at the art's top start and a cost disc on its bottom end, small, the body's text at the card's whole width, mirrored right to left", () => {
  // Wrong: the number or cost dropped (no slot), drawn over the body's text, as large as the art, beside the body
  // so its text wraps short of it (ja's effect lost a third of a narrow card), or left where it was when the
  // layout mirrors.
  const engine = run(() => {
    og.ui.card("Ability", { interactive: true, width: { kind: "fixed", px: 200 }, media: { aspect: 4 / 3 }, number: "1", cost: "3", padding: 12 }, () => {
      og.ui.label("Effect", "Deal high damage to a target in range of the ship.", { width: { kind: "grow" } });
    });
    og.ui.card("Plain", { width: { kind: "fixed", px: 200 }, cost: "2", padding: 12 }, () => {
      og.ui.label("Effect", "Shields up for one turn.", { width: { kind: "grow" } });
    });
  }, 640, kit);
  const box = (end: string) => byPath(engine, `Ability/${end}`)!;
  const read = () => {
    const card = byPath(engine, "Root/Ability")!;
    const art = box("Media");
    const plate = box("Number");
    const disc = box("Media/Cost");
    const effect = box("Body/Effect");
    return { card, art, plate, disc, effect };
  };
  let { card, art, plate, disc, effect } = read();
  expect(box("Number/Text").text).toBe("1");
  expect(box("Media/Cost/Text").text).toBe("3");
  // The plate on the art's top start corner, the disc on its bottom end corner, both inside the art.
  expect(plate.x - art.x).toBeLessThan(16);
  expect(plate.y - art.y).toBeLessThan(16);
  expect(plate.x + plate.width).toBeLessThan(art.x + art.width / 2);
  expect(art.x + art.width - (disc.x + disc.width)).toBeGreaterThanOrEqual(0);
  expect(art.x + art.width - (disc.x + disc.width)).toBeLessThan(16);
  expect(art.y + art.height - (disc.y + disc.height)).toBeGreaterThanOrEqual(0);
  expect(art.y + art.height - (disc.y + disc.height)).toBeLessThan(16);
  // Small beside the art: the art stays the one dominant element.
  expect(plate.width * plate.height).toBeLessThan(art.width * art.height * 0.08);
  expect(disc.width * disc.height).toBeLessThan(art.width * art.height * 0.12);
  // The body's text runs the card's whole width but its padding, under the art and clear of the disc.
  expect(effect.width).toBeGreaterThanOrEqual(card.width - 2 * 12 - 1);
  expect(effect.y).toBeGreaterThanOrEqual(disc.y + disc.height);
  // With no art the disc stands beside the body at its bottom end, and the text wraps short of it.
  const plainDisc = byPath(engine, "Plain/Body/Cost")!;
  const plainText = byPath(engine, "Plain/Body/Content/Effect")!;
  expect(plainText.x + plainText.width).toBeLessThanOrEqual(plainDisc.x + 0.5);
  // Right to left both corners mirror.
  const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
  if (!loaded.ok) throw new Error(loaded.reason);
  og.i18n.setLocale(engine.world, "ar-EG");
  engine.advance(8);
  ({ card, art, plate, disc, effect } = read());
  expect(art.x + art.width - (plate.x + plate.width)).toBeLessThan(16);
  expect(disc.x - art.x).toBeGreaterThanOrEqual(0);
  expect(disc.x - art.x).toBeLessThan(16);
});

// ── direction, locale ───────────────────────────────────────────────────

test("right to left: breadcrumbs, tabs and the calendar's arrows run from the right; the pseudo-locale's longer labels overlap nothing", () => {
  // Wrong: a component that places its parts by physical left, or draws its arrows unmirrored.
  const engine = run(() => {
    og.ui.breadcrumbs("Path", { items: [{ label: pseudo("Home") }, { label: pseudo("Inventory") }, { label: pseudo("Swords") }] });
    og.ui.tabBar("Tabs", { tabs: [{ label: pseudo("Equipment") }, { label: pseudo("Skills") }, { label: pseudo("Achievements") }] }, () => og.ui.label("Body", pseudo("Nothing equipped yet"), {}));
    og.ui.calendar("Days", { year: 2026, month: 11, today: "2026-11-01" });
  }, 640, kit);
  const x = (end: string) => find(engine, (e) => e.path.endsWith(end))!.x;
  expect(x("Path/Crumb#0")).toBeLessThan(x("Path/Crumb#1"));
  expect(x("Tab#0")).toBeLessThan(x("Tab#1"));
  expect(x("Header/Prev")).toBeLessThan(x("Header/Next"));
  const overlaps = og.ui.overlaps() as unknown as Array<{ path: string; other: string }>;
  expect(overlaps.filter((o) => /Path|Tabs|Days/.test(o.path))).toEqual([]);
  const loaded = og.i18n.load(engine.world, "strings/ar.ogt");
  if (!loaded.ok) throw new Error(loaded.reason);
  expect((og.i18n.setLocale(engine.world, "ar-EG") as unknown as { direction: string }).direction).toBe("rtl");
  engine.advance(8);
  expect(x("Path/Crumb#0")).toBeGreaterThan(x("Path/Crumb#1"));
  expect(x("Tab#0")).toBeGreaterThan(x("Tab#1"));
  expect(x("Header/Prev")).toBeGreaterThan(x("Header/Next"));
  expect(x("Week#0/Cell#0")).toBeGreaterThan(x("Week#0/Cell#6"));
});

// ── 200 % text ──────────────────────────────────────────────────────────

test("150 % text: the containers, navigation and data components grow with the text and cut none of it, on a desktop and a phone", () => {
  // Wrong: a part sized in fixed pixels around its text (a row, a chip, a header), so at the
  // player's largest text size it clips, or a text drawn at a fixed size the scale never reaches.
  const ui = native() as unknown as { og_node_ui_frame(id: number): string };
  const nav = () => {
    og.ui.tabBar("Tabs", { tabs: [{ label: "Equipment" }, { label: "Skills" }, { label: "Codex" }] }, () => {});
    og.ui.breadcrumbs("Path", { items: [{ label: "Home" }, { label: "Inventory" }, { label: "Swords" }] });
    og.ui.pagination("Pages", { page: 3, pages: 9 });
    og.ui.accordion("Faq", { sections: [{ title: "Rules", subtitle: "How a round is played" }], expanded: [0] }, () => og.ui.label("Text", "Each round lasts three minutes.", {}));
    og.ui.actionHints("Hints", { hints: [{ action: "confirm", text: "Select" }, { action: "back", text: "Back" }, { action: "details", text: "Hold to sell", hold: true, progress: 0.4 }] });
    og.ui.carousel("Featured", { count: 3, index: 0, height: { kind: "fixed", px: 140 } }, (k) => og.ui.label("Title", `Banner ${k}`, {}));
    og.ui.card("Card", { title: "Ember Blade", eyebrow: "Legendary", interactive: true } as never, () => og.ui.label("Body", "Deals fire damage over time.", {}));
    // A narrow card at a fixed width: it grows with the text instead of breaking its title mid-word.
    og.ui.card("Narrow", { title: "Moonblade", eyebrow: "Epic", badge: "New", interactive: true, width: { kind: "fixed", px: 110 }, media: { icon: "sword", aspect: 4 / 3 } } as never, () => {});
    og.ui.tabBar("Pills", { tabs: [{ label: "Daily" }, { label: "Weekly" }, { label: "Season" }], selected: 1, variant: "pill", fill: true, bumpers: false }, () => {});
    og.ui.breadcrumbs("Long", { items: [{ label: "Home" }, { label: "Inventory" }, { label: "Weapons" }, { label: "Swords" }, { label: "Rare" }, { label: "Moonblade" }], maxShown: 4 });
    og.ui.menu("Menu", true, { at: { x: 40, y: 40 }, entries: [{ id: "copy", label: "Copy", shortcut: "Ctrl+C" }, { id: "compare", label: "Compare", shortcut: "C" }, { id: "sort", label: "Sort by" }, { id: "destroy", label: "Destroy", danger: true }, { id: "n", label: "Name", parent: "sort" }] });
    og.ui.confirm("Delete", true, { title: "Delete save?", message: "This cannot be undone.", confirmLabel: "Delete", danger: true });
  };
  const data = () => {
    og.ui.statCompare("Compare", { rows: [{ label: "Attack", current: 120, next: 138 }, { label: "Weight", current: 12, next: 9, lowerIsBetter: true }] });
    // A narrow compare with headings: at 200 % its labels go above their values, inside the panel.
    og.ui.statCompare("Narrow", { currentLabel: "Equipped", nextLabel: "Moonblade", width: { kind: "fixed", px: 316 }, rows: [{ label: "Attack", current: 120, next: 142 }, { label: "Speed", current: 1.4, next: 1.2, digits: 1 }] } as never);
    // The guild drops first where the columns do not fit (a phone at 200 %); the score stays.
    og.ui.table("Board", { columns: COLUMNS.map((c, k) => ({ ...c, priority: [2, 0, 1][k] })), rows: ROWS.slice(0, 40), height: { kind: "fixed", px: 220 } as never });
    og.ui.keyValueRows("Stats", { title: "Run", rows: [{ label: "Time", value: "12:40", mono: true }, { label: "Gold", value: "1,280", hint: "+120 today" }] });
    og.ui.calendar("Days", { year: 2026, month: 11, today: "2026-11-01", selected: "2026-11-12" } as never);
    og.ui.tree("Codex", { items: [{ id: "beasts", label: "Beasts" }, { id: "wolf", parent: "beasts", label: "Wolf" }, { id: "moon", parent: "wolf", label: "Moonblade", badge: "New" }, { id: "places", label: "Places" }], expanded: ["beasts", "wolf"], width: { kind: "fixed", px: 300 }, height: { kind: "fixed", px: 360 } } as never);
    og.ui.dataList("Mail", { count: 3, height: { kind: "fixed", px: 320 }, width: { kind: "fixed", px: 300 } } as never, (i) => og.ui.label("Subject", `Letter ${i + 1}: a gift for you`, { width: { kind: "grow" }, maxLines: 2, ellipsis: true } as never));
    og.ui.chart("Gold", { kind: "bar", title: "Gold per day", categories: ["Mon", "Tue", "Wed", "Thu"], series: [{ label: "Gold", values: [120, 340, 260, 410] }] });
    // A state in a list or grid of a set height grows it, never spills (Retry inside the panel).
    og.ui.dataList("Down", { count: 0, error: "Could not reach the leaderboard", height: { kind: "fixed", px: 200 }, width: { kind: "fixed", px: 276 } } as never, () => {});
    og.ui.dataGrid("Bag", { count: 0, emptyLabel: "No matches played yet", emptyHint: "Play a ranked match to appear here.", cellWidth: 56, height: { kind: "fixed", px: 190 }, width: { kind: "fixed", px: 276 } } as never, () => {});
    // A week's axis thins its labels whole, never "M…"; a radar's axis names keep whole outside the ring.
    og.ui.chart("Week", { kind: "line", categories: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"], series: [{ label: "Runs", values: [3, 5, 2, 6, 4, 8, 7] }], width: { kind: "fixed", px: 300 } } as never);
    og.ui.chart("Build", { kind: "radar", categories: ["Damage", "Speed", "Range", "Defense", "Magic"], series: [{ label: "Hero", values: [8, 6, 4, 7, 5] }], width: { kind: "fixed", px: 340 } } as never);
  };
  // Tall surfaces: everything is laid out on screen, nothing scrolled away.
  for (const [bp, w, h, coarse, build, probe] of [["desktop", 1280, 2400, false, nav, "Equipment"], ["desktop", 1280, 3000, false, data, "Attack"], ["phonePortrait", 390, 2400, true, nav, "Equipment"], ["phonePortrait", 390, 3000, true, data, "Attack"]] as const) {
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
    // The narrow stat compare keeps every part inside its width (its columns and chips once ran past it).
    const at = els.findIndex((e) => e.id === "Narrow");
    if (at >= 0) {
      const narrow = els[at]!;
      const under = (i: number): boolean => {
        for (let j = parentOf(i); j >= 0; j = parentOf(j)) if (j === at) return true;
        return false;
      };
      const inside = els.filter((e, i) => i > at && e.width > 0 && under(i));
      expect(inside.length).toBeGreaterThan(4);
      expect(inside.filter((e) => e.x < narrow.x - 0.5 || e.x + e.width > narrow.x + narrow.width + 0.5).map((e) => `${bp} ${e.id}`)).toEqual([]);
    }
    // Nothing truncated to a stub but data in a fixed column (a table's cells): headers, menus,
    // crumbs, tabs, cards, the tree and the list's rows wrap or reflow instead.
    const cut = checkFrame(frames[1]!, cell).filter((f) => f.rule === "text-truncated" && !f.paths.some((p) => p.includes("/Cell#")));
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

test("actionHints: under a finger a bar across the bottom that reaches into the safe area and answers its room; content kept above it meets none of it", () => {
  // Wrong: a floating bar over the page's last rows, or one whose room leaves out the safe area.
  let room = 0;
  const sys = defineSystem(`UcBuild${systems++}`, {}, () => {
    const vp = og.ui.frameViewport();
    // A page from the safe area's top to the bar's room: what a screen does with the answer.
    og.ui.anchor("Page", { anchor: "top", margin: 0, width: { kind: "grow" }, height: { kind: "fixed", px: Math.max(0, vp.height - vp.safeArea.top - room) } } as never, () => og.ui.label("Last", "The page's last line", {}));
    room = og.ui.actionHints("Hints", { hints: [{ action: "confirm", hint: "Select" }, { action: "back", hint: "Back" }, { action: "details", hint: "Hold to sell", hold: true, progress: 0.5 }] }).height;
  });
  const created = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, {});
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as never);
  engine.advance(12);
  const bar = byPath(engine, "Hints")!;
  expect(bar.y + bar.height).toBeCloseTo(844, 0);
  expect(bar.width).toBeCloseTo(390, 0);
  expect(room).toBeCloseTo(844 - bar.y, 0);
  // Its buttons sit above the home indicator, each a whole target.
  for (const k of [0, 1, 2]) {
    const h = byPath(engine, `Hint#${k}`)!;
    expect(h.y + h.height).toBeLessThanOrEqual(844 - 34 + 0.5);
    expect(Math.min(h.width, h.height)).toBeGreaterThanOrEqual(43.5);
  }
  const page = elements(engine).filter((e) => e.path === "Page" || e.path.startsWith("Page/"));
  expect(page.length).toBeGreaterThan(0);
  expect(page.filter((e) => e.y + e.height > bar.y + 0.5).map((e) => e.path)).toEqual([]);
});

// ── formats ─────────────────────────────────────────────────────────────

test("formats: round axis steps, true minus signs, signed deltas", () => {
  expect(niceTicks(0, 410, 4)).toMatchObject({ lo: 0, hi: 500, step: 100 });
  expect(niceTicks(-3, 7, 4).ticks).toEqual([-4, -2, 0, 2, 4, 6, 8]);
  expect(niceTicks(0, 0.9, 4)).toMatchObject({ lo: 0, hi: 1 });
  expect(niceTicks(0, 0.9, 4).step).toBeCloseTo(0.2, 9);
  expect(formatNumber(-12840, "grouped")).toBe("−12,840");
  expect(formatNumber(0.425, "percent", 1)).toBe("42.5%");
  expect(formatNumber(12400, "compact")).toBe("12.4K");
  expect(formatDelta(18, "grouped")).toBe("+18");
  expect(formatDelta(-3, "grouped")).toBe("−3");
  expect(formatDelta(0, "grouped")).toBe("±0");
});
