// The component library's preview screens (docs/ui.md §11): containers,
// navigation, the overlays (menus, popovers, dialogs, sheets, the radial
// wheel), data views, charts, and the graph and calendar. Each shows its
// components in their states; `demo:<key>:<value>` picks a state (which
// overlay is open, a data view's loading, empty or error state) so the
// playtests (tests/components*.og.ts) can draw each one.
//
// The previews' own state (the tab picked, the page, the sort) is the
// preview's, kept here between ticks: it is a gallery, never saved.

import { og } from "@origingame/engine";
import type { MenuEntry, Size, UiDevice, World } from "@origingame/engine";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

/** What the previews show and remember. */
const s = {
  overlay: "none",
  data: "rows",
  card: -1,
  open: [0] as number[],
  tab: 0,
  pill: 1,
  page: 4,
  slide: 0,
  split: 0.4,
  sortBy: 2,
  descending: true,
  filter: "",
  selectedRow: 3 as number | null,
  tree: ["weapons", "swords"] as string[],
  treeSel: "moonblade",
  zoom: 1,
  node: "cleave",
  detent: 0,
  day: "2026-11-13",
  month: 11,
  year: 2026,
  /** The wheel's stick (demo:aim): none, so the wheel opens unaimed. */
  radialAim: null as { x: number; y: number } | null,
  check: true,
  sort: "name",
  /** The action bar's room at the bottom (last tick's): the page keeps its content clear of it. */
  bar: 0,
};

/** `demo:<key>:<value>`: a preview's state (overlay, data, aim), the player's text size (text, percent), or the device prompts are drawn for (device: keyboardMouse, xbox, playstation, switch). */
export function demoCommand(world: World, args: readonly string[]): void {
  const [key, value = ""] = args;
  if (key === "text") {
    const r = og.ui.configure(world, { textScale: Math.min(2, Math.max(1, Number(value) / 100 || 1)) });
    if (!r.ok) throw new Error(r.reason);
  } else if (key === "device") og.ui.setDevice(world, value as UiDevice);
  else if (key === "overlay") {
    s.overlay = value;
    // Each opening of the wheel starts unaimed.
    s.radialAim = null;
  }
  else if (key === "data") s.data = value;
  else if (key === "aim") {
    const [x, y] = value.split(",").map(Number);
    s.radialAim = { x: x ?? 0, y: y ?? 0 };
  } else if (key === "reset") Object.assign(s, { overlay: "none", data: "rows", tab: 0, page: 4, slide: 0, detent: 0, zoom: 1, radialAim: null });
}

function panel(id: string, title: string, width: Size, build: () => void): void {
  const t = og.ui.frameTheme();
  og.ui.panel(id, { width, gap: t.space.sm, padding: { left: t.space.md, top: t.space.md, right: t.space.md, bottom: t.space.md } }, () => {
    og.ui.label("Heading", title, { step: "caption", color: { token: "inkMuted" }, width: grow });
    build();
  });
}

function phone(): boolean {
  const b = og.ui.frameViewport().breakpoint;
  return b === "phonePortrait" || b === "phoneLandscape";
}

/**
 * A page of the previews: a scrolling column on a phone, a wrapping row of panels elsewhere,
 * fading at an edge where more lies, and ending above the safe area and the action bar (`bar`:
 * its room at the bottom), so nothing hides under either.
 */
function page(id: string, build: () => void, bar = 0): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const top = phone() ? 96 : t.space.md;
  const bottom = Math.max(bar, vp.safeArea.bottom) + t.space.md;
  og.ui.anchor(id, { anchor: phone() ? "top" : "center", ...(phone() ? { margin: 0 } : {}), width: phone() ? grow : px(Math.min(1200, vp.width - 48)), direction: "column", padding: { left: t.space.md, top, right: t.space.md, bottom: t.space.md } }, () => {
    og.ui.scrollArea("Body", { width: grow, height: px(vp.height - vp.safeArea.top - top - bottom - (phone() ? 0 : 64)), gap: t.space.md, fadeInto: "backdrop", label: id }, () => {
      // One panel under another on a phone (a row that does not wrap would leave all but the first off screen).
      if (phone()) og.ui.column("Panels", { width: grow, gap: t.space.md, cross: "stretch" }, build);
      else og.ui.row("Panels", { width: grow, gap: t.space.md, wrap: true, cross: "start" }, build);
    });
  });
}

// ── containers ──────────────────────────────────────────────────────────

export function containersScreen(): void {
  const t = og.ui.frameTheme();
  page("Containers", () => {
    panel("Cards", "Cards: raised, interactive, selected, disabled, outlined", phone() ? grow : px(560), () => {
      og.ui.row("Row", { width: grow, gap: t.space.sm, wrap: true }, () => {
        og.ui.card("Plain", { title: "Daily quest", subtitle: "Win three matches", eyebrow: "Quest", width: px(170) }, () => og.ui.meter("Progress", 2, 3, { kind: "xp", label: "2 / 3" }));
        [0, 1, 2].forEach((k) => {
          const r = og.ui.card(og.ui.idi("Pick", k), {
            interactive: true,
            selected: s.card === k,
            disabled: k === 2,
            media: { icon: ["sword", "shield", "potion"][k] ?? "image", aspect: 4 / 3 },
            eyebrow: ["Epic", "Rare", "Locked"][k] ?? "",
            title: ["Moonblade", "Bulwark", "Elixir"][k] ?? "",
            badge: k === 0 ? "New" : "",
            badgeTone: "primary",
            width: px(110),
          }, () => {});
          if (r.clicked) s.card = k;
        });
        og.ui.card("Outlined", { variant: "outlined", title: "Outlined", subtitle: "No elevation, a hairline", width: px(170) }, () => {});
      });
    });
    panel("Accordions", "Accordion: one open, a disabled section", phone() ? grow : px(380), () => {
      s.open = og.ui.accordion("Faq", {
        sections: [{ title: "How do ranks work?", subtitle: "Seasons and placement" }, { title: "Rewards", badge: "2", badgeTone: "success" }, { title: "Reporting a player" }, { title: "Esports (soon)", disabled: true }],
        expanded: s.open,
        single: true,
      }, (i) => og.ui.label("Text", ["Each season starts with five placement matches; your rank moves with every win and loss after them.", "Claim rewards from the pass screen.", "Open the player's card and choose Report."][i] ?? "", { width: grow, color: { token: "inkMuted" } })).expanded.slice();
    });
    panel("Scrolls", "Scroll area: edges fade where more lies", phone() ? grow : px(280), () => {
      og.ui.scrollArea("List", { height: px(180), label: "Patch notes" }, () => {
        for (let k = 0; k < 14; k++) og.ui.label(og.ui.idi("Line", k), `• Change ${k + 1}: balance and fixes`, { width: grow });
      });
    });
    panel("Splits", "Split view: drag or press the divider", phone() ? grow : px(560), () => {
      s.split = og.ui.splitView("Split", { ratio: s.split, height: px(160), label: "Panes" }, () => {
        og.ui.label("List", "Mail", { step: "base" });
        og.ui.label("Item", "Season 4 rewards", { color: { token: "inkMuted" } });
      }, () => {
        og.ui.label("Detail", "Your rewards are ready to claim.", { width: grow });
      }).ratio;
    });
  });
}

// ── navigation ──────────────────────────────────────────────────────────

export function navigationScreen(): void {
  page("Navigation", () => {
    panel("TabsUnderline", "Tabs with bumpers, a badge, a disabled tab", phone() ? grow : px(560), () => {
      s.tab = og.ui.tabBar("Tabs", { tabs: [{ label: "Gear", icon: "shield" }, { label: "Skills", badge: "3" }, { label: "Map", disabled: true }, { label: "Codex" }], selected: s.tab }, (k) => og.ui.label("Body", ["Equip up to six items.", "Three skill points to spend.", "", "142 entries discovered."][k] ?? "", { color: { token: "inkMuted" } })).selected;
    });
    panel("TabsPill", "Pill tabs, filling the row", phone() ? grow : px(420), () => {
      s.pill = og.ui.tabBar("Pills", { tabs: [{ label: "Daily" }, { label: "Weekly" }, { label: "Season" }], selected: s.pill, variant: "pill", fill: true, bumpers: false }, () => {}).selected;
    });
    panel("Carousels", "Carousel: drag, flick, arrows, dots", phone() ? grow : px(560), () => {
      s.slide = og.ui.carousel("Featured", { count: 4, index: s.slide, height: px(120), label: "Featured" }, (k) => {
        og.ui.card(og.ui.idi("Banner", k), { media: { icon: ["star", "gift", "trophy", "map"][k] ?? "image", aspect: 3.2 }, title: ["Season 4", "Gift pack", "Tournament", "New map"][k] ?? "", variant: "flat", width: grow }, () => {});
      }).index;
    });
    panel("Crumbs", "Breadcrumbs, collapsed in the middle", phone() ? grow : px(560), () => {
      og.ui.breadcrumbs("Path", { items: [{ label: "Home", icon: "home" }, { label: "Inventory" }, { label: "Weapons" }, { label: "Swords" }, { label: "Rare" }, { label: "Moonblade" }], maxShown: 4 });
    });
    panel("Pages", "Pagination", phone() ? grow : px(420), () => {
      s.page = og.ui.pagination("Pager", { pages: 20, page: s.page }).page;
      og.ui.pagination("Compact", { pages: 20, page: s.page, compact: true });
    });
  }, s.bar);
  s.bar = og.ui.actionHints("Hints", { hints: [{ action: "confirm", hint: "Select", text: "Enter", icon: "key.enter" }, { action: "back", hint: "Back", text: "Esc", icon: "key.escape" }, { action: "details", hint: "Hold to sell", text: "X", icon: "key.x", hold: true, progress: 0.6 }] }).height;
}

// ── overlays ────────────────────────────────────────────────────────────

const MENU: MenuEntry[] = [
  { kind: "header", label: "Item" },
  { id: "equip", label: "Equip", icon: "shield", shortcut: "E" },
  { id: "compare", label: "Compare", shortcut: "C" },
  { id: "favourite", label: "Favourite", kind: "check", checked: true },
  { id: "sort", label: "Sort by" },
  { id: "name", label: "Name", kind: "radio", checked: true, parent: "sort" },
  { id: "rarity", label: "Rarity", kind: "radio", parent: "sort" },
  { id: "level", label: "Level", kind: "radio", parent: "sort" },
  { kind: "separator" },
  { id: "sell", label: "Sell", disabled: true },
  { id: "destroy", label: "Destroy", danger: true },
];

export function overlaysScreen(): void {
  const t = og.ui.frameTheme();
  let anchor = "";
  page("Overlays", () => {
    panel("Openers", "Menus, popovers, dialogs, sheets and the wheel", phone() ? grow : px(560), () => {
      og.ui.row("Row", { gap: t.space.sm, wrap: true }, () => {
        for (const o of ["menu", "popover", "dialog", "confirm", "sheet", "sideSheet", "radial"]) {
          if (og.ui.button(o, o, { variant: s.overlay === o ? "primary" : "secondary", size: "small" }).clicked) s.overlay = o;
          if (o === "menu") anchor = og.ui.lastPath();
        }
      });
    });
  });
  const close = () => (s.overlay = "none");
  const o = s.overlay;
  {
    const r = og.ui.menu("Menu", o === "menu" || o === "submenu", { anchor, entries: MENU.map((e) => (e.parent === "sort" ? { ...e, checked: e.id === s.sort } : e.id === "favourite" ? { ...e, checked: s.check } : e)), label: "Item actions" });
    if (r.chosen === "favourite") s.check = !s.check;
    else if (r.chosen && ["name", "rarity", "level"].includes(r.chosen)) s.sort = r.chosen;
    if (r.dismissed) close();
  }
  if (og.ui.contextMenu("Context", o === "context", { x: 420, y: 260 }, { entries: MENU.slice(1, 5) }).dismissed) close();
  if (og.ui.popover("Pop", o === "popover", { anchor, side: "below", title: "Moonblade" }, () => {
    og.ui.label("Text", "A curved blade that glows under the moon. +18 attack at night.", { width: px(260), color: { token: "inkMuted" } });
  }).dismissed) close();
  const d = og.ui.dialog("Dialog", o === "dialog", { title: "Season 4 has begun", message: "New ranks, a new map and [b]40 rewards[/b] to earn.", tone: "info", actions: [{ id: "later", label: "Later", variant: "quiet" }, { id: "pass", label: "See the pass", variant: "primary", autofocus: true }] }, () => {});
  if (d.action || d.dismissed) close();
  const c = og.ui.confirm("Confirm", o === "confirm", { title: "Destroy Moonblade?", message: "It is gone for good: [color=danger]this cannot be undone[/color].", confirmLabel: "Destroy", danger: true });
  if (c.confirmed || c.cancelled) close();
  const sh = og.ui.sheet("Sheet", o === "sheet", { title: "Filters", detents: [0.45, 0.9], detent: s.detent }, () => {
    og.ui.checkboxList("Kinds", ["Weapons", "Armor", "Potions"], [true, true, false]);
  });
  s.detent = sh.detent;
  if (sh.dismissed) close();
  if (og.ui.sheet("Side", o === "sideSheet", { edge: "end", title: "Friends" }, () => {
    for (const [k, n] of ["Ava", "Kenji", "Lena"].entries()) og.ui.label(og.ui.idi("Friend", k), n, {});
  }).dismissed) close();
  const w = og.ui.radialMenu("Wheel", o === "radial", { items: [{ label: "Sword", icon: "sword", detail: "Melee" }, { label: "Wand", icon: "magic-wand", detail: "Arcane" }, { label: "Bomb", icon: "bomb", detail: "3 left" }, { label: "Potion", icon: "potion" }, { label: "Shield", icon: "shield", disabled: true }, { label: "Map", icon: "map" }], ...(s.radialAim ? { aim: s.radialAim } : {}), title: "Quick select" });
  if (w.chosen !== null || w.dismissed) close();
}

// ── data ────────────────────────────────────────────────────────────────

const NAMES = ["Ava", "Bram", "Cleo", "Dax", "Esme", "Finn", "Gus", "Hana", "Ivo", "Juno", "Kai", "Lior"];
const BOARD = Array.from({ length: 10_000 }, (_, i) => ({ rank: i + 1, name: `${NAMES[i % NAMES.length]}${Math.floor(i / NAMES.length) || ""}`, guild: ["Ash", "Brine", "Cinder", "Dusk"][(i * 7) % 4], wins: 900 - ((i * 37) % 900), ratio: ((i * 7919) % 1000) / 1000, trend: i % 3 === 0 ? "down" : "up" }));

export function dataScreen(): void {
  const state = { loading: s.data === "loading", error: s.data === "error" ? "Could not reach the leaderboard" : "", emptyLabel: "No matches played yet", emptyHint: "Play a ranked match to appear here." };
  page("Data", () => {
    panel("Board", "Table: 10,000 rows, sort, filter", phone() ? grow : px(640), () => {
      const r = og.ui.table("Leaderboard", {
        rows: s.data === "empty" ? [] : BOARD,
        columns: [
          { key: "rank", label: "#", numeric: true, width: 56, priority: 3 },
          { key: "name", label: "Player", priority: 4 },
          { key: "guild", label: "Guild", width: 90, priority: 1 },
          { key: "wins", label: "Wins", numeric: true, width: 80, priority: 2, toneKey: "trend" },
          { key: "ratio", label: "Win rate", numeric: true, format: "percent", digits: 1, width: 90, priority: 0 },
        ],
        sortBy: s.sortBy,
        descending: s.descending,
        filter: s.filter,
        selected: s.selectedRow,
        height: px(300),
        label: "Leaderboard",
        ...state,
      });
      s.sortBy = r.sortBy;
      s.descending = r.descending;
      s.filter = r.filter;
      if (r.clicked !== null) s.selectedRow = r.clicked;
    });
    panel("Lists", "List with states", phone() ? grow : px(300), () => {
      og.ui.dataList("Mail", { count: s.data === "empty" ? 0 : 400, selected: 1, height: px(200), label: "Mail", ...state }, (i) => {
        og.ui.icon("Icon", i % 4 === 0 ? "gift" : "chat-bubble", { size: 18 });
        og.ui.label("Subject", `Letter ${i + 1}: ${i % 4 === 0 ? "a gift for you" : "match report"}`, { width: grow, maxLines: 2, ellipsis: true });
      });
    });
    panel("Trees", "Tree: a codex", phone() ? grow : px(300), () => {
      const r = og.ui.tree("Codex", {
        items: [
          { id: "weapons", label: "Weapons", icon: "sword" }, { id: "swords", parent: "weapons", label: "Swords" }, { id: "moonblade", parent: "swords", label: "Moonblade", badge: "New", badgeTone: "primary" },
          { id: "sabre", parent: "swords", label: "Sabre" }, { id: "bows", parent: "weapons", label: "Bows" }, { id: "longbow", parent: "bows", label: "Longbow" },
          { id: "armor", label: "Armor", icon: "shield" }, { id: "helm", parent: "armor", label: "Helm" }, { id: "lore", label: "Lore (locked)", disabled: true },
        ],
        expanded: s.tree,
        selected: s.treeSel,
        height: px(220),
        label: "Codex",
      });
      s.tree = r.expanded.slice();
      if (r.chosen) s.treeSel = r.chosen;
    });
    panel("Grids", "Grid: an inventory", phone() ? grow : px(300), () => {
      og.ui.dataGrid("Bag", { count: s.data === "empty" ? 0 : 60, cellWidth: 56, selected: 2, height: px(190), label: "Bag", ...state }, (i) => og.ui.icon("Icon", ["sword", "shield", "potion", "gem", "key"][i % 5] ?? "gem", { size: 28 }));
    });
    panel("Stats", "Key-value rows and stat compare", phone() ? grow : px(340), () => {
      og.ui.keyValueRows("Sheet", { title: "Moonblade", rows: [{ label: "Damage", value: "142", icon: "sword" }, { label: "Crit chance", value: "12%", hint: "+4% at night" }, { label: "Element", value: "Lunar", tone: "info" }, { label: "Durability", value: "18 / 60", tone: "warn" }] });
      og.ui.statCompare("Compare", { currentLabel: "Equipped", nextLabel: "Moonblade", rows: [{ label: "Attack", current: 120, next: 142, max: 200 }, { label: "Speed", current: 1.4, next: 1.2, digits: 1, max: 2 }, { label: "Weight", current: 12, next: 9, lowerIsBetter: true, max: 20 }, { label: "Crit", current: 0.12, next: 0.12, format: "percent" }] });
    });
  });
}

// ── charts ──────────────────────────────────────────────────────────────

const DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function chartsScreen(): void {
  const w = phone() ? grow : px(370);
  const state = { loading: s.data === "loading", error: s.data === "error" ? "Stats are unavailable" : "" };
  const empty = s.data === "empty";
  page("Charts", () => {
    panel("Line", "Line", w, () => og.ui.chart("Players", { kind: "line", title: "Players online", subtitle: "This week", categories: DAYS, series: empty ? [] : [{ label: "EU", values: [1200, 1350, 1280, 1600, 2100, 2650, 2400] }, { label: "NA", values: [900, 950, 1020, 1100, 1700, 2200, 2050] }], format: "compact", height: 180, ...state }));
    panel("Area", "Area, stacked", w, () => og.ui.chart("Gold", { kind: "area", stacked: true, title: "Gold earned", categories: DAYS, series: empty ? [] : [{ label: "Quests", values: [120, 180, 150, 220, 260, 310, 280] }, { label: "Trade", values: [60, 40, 90, 70, 120, 100, 140] }], height: 180, ...state }));
    panel("Bar", "Bars, grouped", w, () => og.ui.chart("Matches", { kind: "bar", title: "Matches", categories: ["Duel", "Squad", "Ranked", "Event"], series: empty ? [] : [{ label: "Won", values: [34, 52, 18, 9] }, { label: "Lost", values: [21, 40, 22, 3] }], height: 180, selected: 2, ...state }));
    panel("Stacked", "Bars, stacked", w, () => og.ui.chart("Damage", { kind: "bar", stacked: true, title: "Damage by type", categories: ["R1", "R2", "R3", "R4", "R5"], series: empty ? [] : [{ label: "Fire", values: [40, 55, 30, 70, 60] }, { label: "Ice", values: [20, 15, 35, 25, 40] }, { label: "Poison", values: [10, 20, 15, 10, 30] }], height: 180, ...state }));
    panel("Radar", "Radar", w, () => og.ui.chart("Hero", { kind: "radar", title: "Hero stats", categories: ["Power", "Speed", "Range", "Defense", "Utility"], series: empty ? [] : [{ label: "Ava", values: [8, 6, 4, 7, 5] }, { label: "Kenji", values: [5, 9, 7, 3, 6] }], max: 10, height: 220, ...state }));
    panel("Donut", "Donut", w, () => og.ui.chart("Loot", { kind: "donut", title: "Loot by rarity", categories: ["Common", "Rare", "Epic", "Legendary"], series: empty ? [] : [{ values: [62, 25, 10, 3] }], height: 180, totalLabel: "Drops", ...state }));
  });
}

// ── graph and calendar ──────────────────────────────────────────────────

export function graphScreen(): void {
  page("Graphs", () => {
    panel("Skills", "Skill tree: pan, pinch, zoom, keys and pads", phone() ? grow : px(620), () => {
      const r = og.ui.nodeGraph("Tree", {
        nodes: [
          { id: "root", x: 200, y: 0, label: "Warrior", state: "done", icon: "sword", shape: "hex" },
          { id: "cleave", x: 60, y: 110, label: "Cleave", state: "done" },
          { id: "guard", x: 200, y: 110, label: "Guard", state: "current", icon: "shield" },
          { id: "rage", x: 340, y: 110, label: "Rage", state: "available" },
          { id: "whirl", x: 60, y: 220, label: "Whirlwind", state: "active" },
          { id: "wall", x: 200, y: 220, label: "Iron wall", state: "locked", shape: "square" },
          { id: "berserk", x: 340, y: 220, label: "Berserk", state: "locked", shape: "diamond", kind: "ultimate" },
        ],
        links: [{ from: "root", to: "cleave" }, { from: "root", to: "guard" }, { from: "root", to: "rage" }, { from: "cleave", to: "whirl" }, { from: "guard", to: "wall" }, { from: "rage", to: "berserk" }],
        zoom: s.zoom,
        selected: s.node,
        height: px(340),
        label: "Warrior skills",
      });
      s.zoom = r.zoom;
      if (r.chosen) s.node = r.chosen;
    });
    panel("Days", "Calendar: today, selected, range, marks, claimed", phone() ? grow : px(380), () => {
      const r = og.ui.calendar("Login", {
        year: s.year,
        month: s.month,
        today: "2026-11-01",
        selected: s.day,
        rangeStart: "2026-11-20",
        rangeEnd: "2026-11-24",
        min: "2026-10-01",
        disabled: ["2026-11-28"],
        marks: [{ date: "2026-11-01", kind: "claimed" }, { date: "2026-11-02", kind: "claimed" }, { date: "2026-11-03", kind: "claimed" }, { date: "2026-11-07", tone: "gold", label: "Double rewards" }, { date: "2026-11-14", tone: "danger", label: "Boss raid" }, { date: "2026-11-14", tone: "info", label: "Patch" }],
        bumpers: true,
      });
      if (r.selected) s.day = r.selected;
      s.month = r.month;
      s.year = r.year;
    });
  });
}
