// The component library's containers, navigation and data components
// (docs/ui.md §11) on the native host, driven as a player drives them: a
// tap, a key, a bumper, Back. Each test names the wrong implementation it
// would catch.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineSystem } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { pseudo } from "../src/node/ui-check.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { tableOrder } from "../src/ui/data/table.ts";
import { niceTicks, formatDelta, formatNumber } from "../src/ui/data/format.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "templates", "ui-kit");

// At module level, not in a hook (a hook's 5 s would not cover a cook on a
// loaded machine): the kit's string tables, Arabic among them, cooked
// (incrementally) by the og this package built.
{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

type Element = { id: string; parent: string; path: string; kind: string; text: string; role: string; name: string; value: string; states: string[]; disabled: boolean; x: number; y: number; width: number; height: number };

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
  expect(elements(engine).some((e) => e.parent === "Cell#2" && e.text === formatNumber(top, "grouped") && e.path.startsWith(first().path))).toBeTrue();
  tap(engine, "Header/Col#2");
  expect(descending).toBeFalse();
  expect(byPath(engine, "Header/Col#2")?.value).toBe("sorted ascending");
});

test("table: a tick over 10,000 sorted rows stays within budget once the order is cached", () => {
  const engine = run(() => {
    og.ui.table("Board", { rows: ROWS, columns: COLUMNS, sortBy: 2, descending: true, filter: "1", height: { kind: "fixed", px: 400 } });
  }, 720);
  const t0 = performance.now();
  engine.advance(60);
  const per = (performance.now() - t0) / 60;
  // A tick of the whole UI (layout and the a11y tree too), well under a frame.
  expect(per).toBeLessThan(8);
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
