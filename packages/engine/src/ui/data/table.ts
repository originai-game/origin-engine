// Virtualised lists and grids with their empty, loading and error states,
// and the sortable, filterable table (docs/ui.md §11, Data; §15.1: a
// 10,000-row table scrolls within budget). Only the rows in view are
// built (og.ui.openList); sorting and filtering run once per change of
// the rows, the sort or the filter, never per tick: the order is cached by
// the rows array (a cache of derived indices, rebuilt from the same inputs
// on a restored world, so it holds no state).

import type * as Api from "@origingame/engine";
import {
  all, byText, chevron, target, fit, flags, frame, grow, kit, mark, px, role, spring, surface, text, token, xy,
} from "../containers/base.ts";
import { formatNumber, groupNumber } from "./format.ts";

// ── states: empty, loading, error ───────────────────────────────────────

/** What a data view shows instead of rows: a skeleton while loading, a message and Retry on error, a message when empty. Answers whether Retry was pressed. */
/** A list's or grid's set height while it shows a state: its least, so a
 * message that wraps at a large text size grows the panel, never spills. */
function stateHeight(h: Api.Size | null | undefined): Api.Size | null {
  return h && h.kind === "fixed" ? fit(h.px) : h ?? null;
}

export function dataState(id: string, s: Api.DataStateDesc, rows: number, rowHeight: number): boolean {
  const { t } = frame();
  let retry = false;
  if (s.loading) {
    kit.column(id, { width: grow(), gap: t.sm, padding: xy(t.md, t.md), cross: "stretch" }, () => {
      for (let k = 0; k < Math.max(1, Math.min(rows, 6)); k++) kit.skeleton(kit.idi("Row", k), { width: grow(), height: Math.max(16, rowHeight - 16) });
    });
    kit.semantics({ role: "status", name: s.loadingLabel || "Loading", busy: true });
    return false;
  }
  const error = !!s.error;
  kit.column(id, { width: grow(), padding: all(t.xl), gap: t.sm, cross: "center", align: "center" }, () => {
    kit.open("Mark", { width: px(48), height: px(48), radius: 24, align: "center", cross: "center", background: token(error ? "danger" : "control", error ? 0.16 : 1), part: `Data.stateMark${flags("round", error ? "error" : "empty")}`, partClass: "avatar" });
    mark("Glyph", error ? "cross" : "minus", 22, role(error ? "danger" : "inkMuted"), 2.5);
    kit.close();
    text("Title", error ? s.error ?? "" : s.emptyLabel || "Nothing here yet", { step: "base", heading: true, align: "center", width: grow() });
    if (!error && s.emptyHint) text("Hint", s.emptyHint, { step: "sm", color: "inkMuted", align: "center", width: grow() });
    if (error && (s.retry ?? true)) {
      if ((kit.button("Retry", s.retryLabel || "Retry", {}) as Api.Response).clicked) retry = true;
    }
  });
  kit.semantics({ role: error ? "alert" : "status", name: error ? s.error ?? "" : s.emptyLabel || "Nothing here yet" });
  return retry;
}

// ── list ────────────────────────────────────────────────────────────────

/**
 * A virtualised list with its states: `count` rows of `rowHeight`, only
 * those in view built (a list of 10,000 builds about a screenful), the
 * selected row marked; rows the game builds take keys and pads (Row#i)
 * and scroll into view as focus moves. Empty, loading and error states
 * replace the rows. Answers the row pressed and Retry.
 */
export function dataList(id: string, desc: Api.DataListDesc, build: Api.UiItemBuild): Api.DataListResult {
  const { t } = frame();
  const count = Math.max(0, desc.count ?? 0);
  const rowH = byText(desc.rowHeight ?? Math.max(t.row, 48));
  let clicked: number | null = null;
  let retry = false;
  const state = !!(desc.loading || desc.error || count === 0);
  kit.open(id, { direction: "column", width: desc.width ?? grow(), height: state ? stateHeight(desc.height) : desc.height ?? null, cross: "stretch", part: "List.frame", partClass: "panel" });
  try {
    if (state) {
      retry = dataState("State", desc, count || 4, rowH);
    } else {
      const r = kit.openList("Rows", { count, rowHeight: rowH, gap: desc.gap ?? 0, width: grow(), height: desc.height ? grow() : null, padding: xy(0, 0) }) as Api.ListRange;
      for (let i = r.first; i < r.last; i++) {
        const sel = desc.selected === i;
        const resp = kit.listItem(i, sel) as Api.Response;
        try {
          kit.row("Cells", { width: grow(), height: grow(), padding: xy(t.md, 0), gap: t.md, cross: "center", part: `List.row${flags(sel && "selected", i % 2 === 1 && desc.striped && "odd")}`, partClass: "slot" }, () => build(i));
        } finally {
          kit.close();
        }
        if (resp.clicked) clicked = i;
      }
      kit.closeList();
      kit.semantics({ role: "listbox", name: desc.label ?? "", value: `${count}` });
    }
  } finally {
    kit.close();
  }
  if (clicked !== null) kit.feedback("select", -1);
  return { clicked, retry };
}

// ── grid ────────────────────────────────────────────────────────────────

/**
 * A virtualised grid of equal cells (an inventory, a level select, a card
 * collection) with its states: only the rows of cells in view are built,
 * the selected cell marked, cells the game builds take keys and pads in
 * two dimensions and scroll into view as focus moves. Empty, loading and
 * error states replace the cells. Answers the cell pressed and Retry.
 */
export function dataGrid(id: string, desc: Api.DataGridDesc, build: Api.UiItemBuild): Api.DataListResult {
  const { t } = frame();
  const count = Math.max(0, desc.count ?? 0);
  const cellW = desc.cellWidth ?? 0;
  let clicked: number | null = null;
  let retry = false;
  const state = !!(desc.loading || desc.error || count === 0);
  kit.open(id, { direction: "column", width: desc.width ?? grow(), height: state ? stateHeight(desc.height) : desc.height ?? null, cross: "stretch", part: "Grid.frame", partClass: "panel" });
  try {
    if (state) {
      retry = dataState("State", desc, 4, desc.cellHeight || cellW || 64);
    } else {
      const r = kit.openGrid("Cells", { count, cellWidth: cellW, cellHeight: desc.cellHeight ?? 0, columns: desc.columns ?? 0, gap: desc.gap ?? t.sm, width: grow(), height: desc.height ? grow() : null, padding: desc.padding ?? xy(0, 0) }) as Api.ListRange;
      for (let i = r.first; i < r.last; i++) {
        const sel = desc.selected === i;
        const resp = kit.gridItem(i, sel) as Api.Response;
        try {
          kit.open("Cell", { direction: "column", width: grow(), height: grow(), align: "center", cross: "center", part: `Grid.cell${flags(sel && "selected")}`, partClass: "slot" });
          try {
            build(i);
          } finally {
            kit.close();
          }
        } finally {
          kit.close();
        }
        if (resp.clicked) clicked = i;
      }
      kit.closeList();
      kit.semantics({ role: "grid", name: desc.label ?? "", value: `${count}` });
    }
  } finally {
    kit.close();
  }
  if (clicked !== null) kit.feedback("select", -1);
  return { clicked, retry };
}

// ── table ───────────────────────────────────────────────────────────────

interface Order { rows: readonly unknown[]; key: string; idx: Int32Array }
const orders = new WeakMap<readonly unknown[], Order>();

function cell(row: unknown, c: Api.TableColumn): unknown {
  return (row as Record<string, unknown> | null)?.[c.key ?? ""];
}

function cellText(row: unknown, c: Api.TableColumn): string {
  const v = cell(row, c);
  if (v === null || v === undefined) return "";
  if (typeof v === "number") return formatNumber(v, c.format, c.digits ?? 0, c.unit ?? "");
  return String(v);
}

/** The rows' order after the filter and the sort: computed once per change, kept with the rows array. */
export function tableOrder(rows: readonly unknown[], columns: readonly Api.TableColumn[], sortBy: number, descending: boolean, filter: string): Int32Array {
  const key = `${sortBy}|${descending ? 1 : 0}|${filter}|${rows.length}|${columns.map((c) => c.key).join(",")}`;
  const hit = orders.get(rows);
  if (hit && hit.key === key) return hit.idx;
  const q = filter.trim().toLowerCase();
  const searchable = columns.filter((c) => c.filterable !== false);
  const kept: number[] = [];
  for (let i = 0; i < rows.length; i++) {
    if (!q || searchable.some((c) => cellText(rows[i], c).toLowerCase().includes(q))) kept.push(i);
  }
  const c = columns[sortBy];
  if (c && sortBy >= 0) {
    const dir = descending ? -1 : 1;
    const num = c.numeric ?? typeof cell(rows[kept[0] ?? 0], c) === "number";
    kept.sort((a, b) => {
      const x = cell(rows[a], c);
      const y = cell(rows[b], c);
      let d: number;
      if (num) d = (Number(x) || 0) - (Number(y) || 0);
      else d = String(x ?? "").localeCompare(String(y ?? ""));
      // Stable: equal keys keep the rows' own order.
      return d !== 0 ? d * dir : a - b;
    });
  }
  const idx = Int32Array.from(kept);
  orders.set(rows, { rows, key, idx });
  return idx;
}

/**
 * A table of rows (objects) by columns: a header that sorts by a column
 * when pressed (again to turn the direction; an arrow says which), a
 * search field that filters by every filterable column, numeric columns
 * aligned to the end in the mono face, a selected row, striping, and only
 * the rows in view built, so 10,000 rows and more scroll within budget.
 * Columns narrower than the screen allows drop (lowest `priority` first)
 * on a phone. Empty, loading and error states replace the body. Answers
 * the sort, the filter and the row pressed, to keep in game state.
 */
export function table(id: string, desc: Api.TableDesc): Api.TableResult {
  const { t, phone } = frame();
  const rows = (desc.rows ?? []) as readonly unknown[];
  const all_ = desc.columns ?? [];
  // A column's width grows with the player's text size (its numbers and its header are text).
  const widthOf = (c: Api.TableColumn): number | null => (c.width ? byText(c.width) : null);
  // Keep the columns that fit the table's width (as laid out last tick; the screen's before it
  // was), highest priority first: a growing column needs room for its text too. On a phone, or at
  // a large text size on a desktop, the low-priority columns drop instead of every column cutting.
  const room = kit.bounds(id)?.width ?? frame().viewport.width - 2 * t.gutter;
  const cols = (() => {
    const want = [...all_].map((c, k) => ({ c, k })).sort((a, b) => (b.c.priority ?? 0) - (a.c.priority ?? 0));
    let used = 2 * t.sm;
    const keep = new Set<number>();
    for (const { c, k } of want) {
      const w = widthOf(c) ?? byText(80);
      if (keep.size === 0 || used + w <= room) {
        keep.add(k);
        used += w;
      }
    }
    return all_.filter((_, k) => keep.has(k));
  })();
  let sortBy = desc.sortBy ?? -1;
  let descending = desc.descending ?? false;
  let filter = desc.filter ?? "";
  let clicked: number | null = null;
  let retry = false;
  const rowH = byText(desc.rowHeight ?? (phone ? 48 : 40));
  const colIndex = (c: Api.TableColumn) => all_.indexOf(c);
  // A small text's rough advance at the text size: a cell whose value is likely cut says it whole in a tooltip.
  const charW = 9 * Math.max(1, frame().theme.textScale ?? 1);
  const colW = new Map(cols.map((c) => [c, widthOf(c)] as const));
  // A state (loading, error, empty, no match) grows the table past its set height rather than spill.
  const state = !!(desc.loading || desc.error || tableOrder(rows, all_, sortBy, descending, filter).length === 0);
  kit.open(id, { direction: "column", width: desc.width ?? grow(), height: state ? stateHeight(desc.height) : desc.height ?? null, cross: "stretch", gap: t.sm });
  try {
    if (desc.search ?? true) {
      const f = kit.textField("Search", filter, { placeholder: desc.searchPlaceholder || "Search", width: grow() }) as Api.FieldResult;
      if (f.response.changed) filter = f.value;
    }
    kit.open("Grid", { direction: "column", width: grow(), height: desc.height && !state ? grow() : null, cross: "stretch", radius: t.panel, background: token("panel"), part: "Table.frame", partClass: "panel" });
    try {
      // The header: one cell a column, a button for the sortable ones.
      kit.row("Header", { width: grow(), height: fit(Math.max(40, t.controlMd)), padding: xy(t.sm, 0), cross: "center", background: token("panelRaised"), part: "Table.header", partClass: "bar" }, () => {
        cols.forEach((c) => {
          const k = colIndex(c);
          const on = k === sortBy;
          const end = c.numeric ?? false;
          const label = c.label ?? c.key ?? "";
          const cw = widthOf(c);
          const width: Api.Size = cw ? px(cw) : grow();
          if (c.sortable === false) {
            kit.row(kit.idi("Col", k), { width, padding: xy(t.xs, 0), align: end ? "end" : "start" }, () => text("Label", label, { step: "caption", color: "inkMuted", maxLines: 2, align: end ? "end" : "start" }));
            kit.semantics({ role: "columnheader", name: label });
            return;
          }
          const r = surface(kit.idi("Col", k), {
            part: "Table.column",
            cls: "tab",
            flags: [on ? "sorted" : "", on ? (descending ? "descending" : "ascending") : ""],
            fill: null,
            width,
            height: fit(target(36)),
            padding: xy(t.xs, 0),
            gap: t.xxs,
            align: end ? "end" : "start",
            semantics: { role: "columnheader", name: label, value: on ? (descending ? "sorted descending" : "sorted ascending") : "" },
          }, () => {
            // A header wraps onto a second line rather than cut to a stub.
            text("Label", label, { step: "caption", color: on ? "ink" : "inkMuted", maxLines: 2, font: on ? "display" : "body", align: end ? "end" : "start" });
            if (on) chevron("Sort", "forward", 14, role("primary"), { group: "sort", current: descending ? "descending" : "ascending", turns: { ascending: -1, descending: 1 } }, 2);
          });
          if (r.clicked) {
            if (on) descending = !descending;
            else {
              sortBy = k;
              descending = c.numeric ?? false;
            }
            kit.feedback("select", -1);
          }
        });
      });
      kit.divider("Rule", false);
      const order = tableOrder(rows, all_, sortBy, descending, filter);
      const n = order.length;
      if (desc.loading || desc.error || n === 0) {
        retry = dataState("State", { ...desc, emptyLabel: rows.length > 0 && filter ? desc.noMatchLabel || "No matches" : desc.emptyLabel ?? "" }, 6, rowH);
      } else {
        const r = kit.openList("Rows", { count: n, rowHeight: rowH, width: grow(), height: desc.height ? grow() : null, padding: xy(0, 0) }) as Api.ListRange;
        for (let v = r.first; v < r.last; v++) {
          const i = order[v]!;
          const sel = desc.selected === i;
          const resp = kit.listItem(v, sel) as Api.Response;
          try {
            // Each cell is its label alone (a box around each would double a row's elements: §15.1's
            // budget counts them), spaced by the row's gap where each box's padding stood.
            kit.row("Cells", { width: grow(), height: grow(), padding: xy(t.sm + t.xs, 0), gap: 2 * t.xs, cross: "center", background: desc.striped !== false && v % 2 === 1 ? token("control", 0.5) : null, part: `Table.row${flags(sel && "selected", v % 2 === 1 && "odd")}`, partClass: "slot" }, () => {
              cols.forEach((c) => {
                const k = colIndex(c);
                const end = c.numeric ?? false;
                const tone = c.toneKey ? String((rows[i] as Record<string, unknown>)[c.toneKey] ?? "") : "";
                const ink: Api.ThemeColor = tone === "up" || tone === "success" ? "success" : tone === "down" || tone === "danger" ? "danger" : k === 0 ? "ink" : "inkMuted";
                // Data in a fixed column: an ellipsis, with the whole value in a tooltip when it cuts.
                const v = cellText(rows[i], c);
                const w = colW.get(c);
                text(kit.idi("Cell", k), v, { step: "sm", nowrap: true, ellipsis: true, color: ink, font: end ? "mono" : "body", align: end ? "end" : "start", width: w ? px(Math.max(0, w - 2 * t.xs)) : grow() });
                if (v.length * charW > (w ?? room)) kit.tooltip(v);
                kit.semantics({ role: "cell" });
              });
            });
          } finally {
            kit.close();
          }
          kit.semantics({ role: "row", selected: sel });
          if (resp.clicked) clicked = i;
        }
        kit.closeList();
        kit.semantics({ role: "table", name: desc.label ?? "", value: `${n} rows` });
      }
    } finally {
      kit.close();
    }
    if (desc.footer !== false) {
      const n = tableOrder(rows, all_, sortBy, descending, filter).length;
      text("Count", fillCount(desc.countLabel || "{shown} of {total}", n, rows.length), { step: "caption", color: "inkMuted", nowrap: true });
    }
  } finally {
    kit.close();
  }
  if (clicked !== null) kit.feedback("select", -1);
  return { sortBy, descending, filter, clicked, retry, changed: sortBy !== (desc.sortBy ?? -1) || descending !== (desc.descending ?? false) || filter !== (desc.filter ?? "") };
}

function fillCount(s: string, shown: number, total: number): string {
  return s.replace("{shown}", groupNumber(shown, 0)).replace("{total}", groupNumber(total, 0));
}
