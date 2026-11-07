// Key-value rows and stat-compare rows (docs/ui.md §2, Data): a
// character sheet's stats, an item's properties, and the comparison an
// equip screen shows (Attack 120 → 138, +18), the change carried by a
// mark and a sign as well as its colour (XAG 103).

import type * as Api from "@origingame/engine";
import { byText, clamp, fit, flags, frame, grow, kit, mark, paint, px, role, spring, still, text, token, xy } from "../containers/base.ts";
import { formatDelta, formatNumber } from "./format.ts";

const TONE: Record<string, Api.ThemeColor> = { success: "success", warn: "warn", danger: "danger", info: "info", primary: "primary", neutral: "ink", gold: "gold", health: "health", mana: "mana", xp: "xp" };

/**
 * Key-value rows: a label at the start, its value at the end (numbers in
 * the mono face so columns line up), an optional icon, a hint under the
 * label and a tone for the value; hairlines between rows unless `dividers`
 * is false, and striped rows when asked. On a phone at 200 % text a value
 * too long for its row wraps under its label instead of cutting.
 */
export function keyValueRows(id: string, desc: Api.KeyValueRowsDesc): void {
  const { t, phone } = frame();
  const rows = desc.rows ?? [];
  const dense = desc.dense ?? false;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), cross: "stretch", part: "KeyValue.frame", partClass: "panel" });
  try {
    if (desc.title) {
      kit.row("Title", { width: grow(), padding: { left: 0, top: 0, right: 0, bottom: t.xs } }, () => text("Text", desc.title ?? "", { step: "caption", color: "inkMuted", width: grow() }));
    }
    rows.forEach((r, k) => {
      const ink = TONE[r.tone ?? "neutral"] ?? "ink";
      const numeric = r.mono ?? /^[\s+\-−±]?[\d.,%:]+\s?\S{0,4}$/.test(r.value ?? "");
      kit.row(kit.idi("Row", k), {
        width: grow(),
        padding: xy(desc.inset ?? 0, dense ? t.xxs : t.xs),
        gap: t.sm,
        cross: "center",
        wrap: phone,
        background: desc.striped && k % 2 === 1 ? token("control", 0.45) : null,
        part: `KeyValue.row${flags(r.tone && `tone=${r.tone}`, desc.striped && k % 2 === 1 && "odd")}`,
        partClass: "slot",
      }, () => {
        if (r.icon) kit.icon("Icon", r.icon, { size: dense ? 16 : 18, tint: token("inkMuted"), label: "" });
        kit.column("Key", { width: grow(), gap: 0 }, () => {
          text("Label", r.label ?? "", { step: dense ? "caption" : "sm", color: "inkMuted", width: grow() });
          if (r.hint) text("Hint", r.hint, { step: "caption", color: "inkFaint", width: grow() });
        });
        text("Value", r.value ?? "", { step: dense ? "sm" : "base", color: ink, font: numeric ? "mono" : "body", align: "end" });
      });
      kit.semantics({ role: "listitem", name: `${r.label ?? ""}: ${r.value ?? ""}${r.hint ? `, ${r.hint}` : ""}` });
      if (k < rows.length - 1 && (desc.dividers ?? true) && !desc.striped) kit.divider(kit.idi("Divider", k), false);
    });
  } finally {
    kit.close();
  }
  kit.semantics({ role: "list", name: desc.title ?? desc.label ?? "" });
}

/**
 * Stat-compare rows: each stat's label, its current value, an arrow and
 * the value it would take, and the change as a signed delta in a chip
 * with an up or down mark (green when it is better: higher, or lower for
 * a stat where less is better), with a bar under the row that shows the
 * current value and the gain or loss on top of it when the stat has a
 * `max`. Rows with no change are quiet. The deltas count in on the ui
 * spring when the compared item changes.
 */
export function statCompare(id: string, desc: Api.StatCompareDesc): void {
  const { t, phone } = frame();
  const rows = desc.rows ?? [];
  // The value columns grow with the text; where they and a readable label do not fit one line
  // (a narrow panel, a large text size) each row puts its label above its values.
  // Each column as wide as its widest value at this text size (mono digits are 0.6 em), so the
  // columns line up row to row and take no more than they need.
  const ramp = frame().theme.typeRamp as Record<string, Api.TypeStep>;
  const sm = ramp.sm?.size ?? 14;
  const cap = ramp.caption?.size ?? 13;
  const fmtRow = (r: Api.StatCompareRow, v: number) => formatNumber(v, r.format, r.digits ?? 0, r.unit ?? "");
  const chars = Math.max(3, ...rows.flatMap((r) => [fmtRow(r, r.current ?? 0).length, fmtRow(r, r.next ?? r.current ?? 0).length]));
  const deltaChars = Math.max(2, ...rows.map((r) => formatDelta((r.next ?? r.current ?? 0) - (r.current ?? 0), r.format, r.digits ?? 0, "").length));
  // A column heading wraps between its words, never inside one: the column holds its longest word.
  const headWord = Math.max(0, ...[desc.currentLabel ?? "", desc.nextLabel ?? ""].flatMap((l) => l.split(/\s+/).map((w) => w.length)));
  const valueCol = Math.ceil(chars * sm * 0.62) + 6;
  const headCol = Math.ceil(Math.max(chars * sm * 0.62, headWord * cap * 0.58)) + 6;
  const dcol = Math.ceil(deltaChars * cap * 0.62) + 8 + 2 + 12 + 4;
  const room = kit.memory(`~w:${id}`, 0);
  // Stacked, the headings are one line above the rows (“Equipped → Moonblade”) and the columns
  // only as wide as their values.
  const labelW = Math.min(byText(96), Math.max(0, ...rows.map((r) => (r.label ?? "").length)) * sm * 0.55 + (rows.some((r) => r.icon) ? 18 + t.sm : 0));
  const stacked = room > 0 && room < 2 * headCol + 16 + dcol + 3 * t.sm + labelW + t.sm;
  const col = stacked ? valueCol : headCol;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: 0, cross: "stretch", part: "StatCompare.frame", partClass: "panel" });
  try {
    if (stacked && (desc.currentLabel || desc.nextLabel)) {
      const arrow = kit.frameDirection() === "rtl" ? "←" : "→";
      text("Header", [desc.currentLabel, desc.nextLabel].filter((l) => l).join(` ${arrow} `), { step: "caption", color: "inkFaint", width: grow() });
    } else if (desc.currentLabel || desc.nextLabel) {
      kit.row("Header", { width: grow(), gap: t.sm, padding: xy(0, t.xxs), cross: "end" }, () => {
        kit.open("Gap", { width: grow() });
        kit.close();
        // Each column's heading wraps in its column, never into its neighbour's.
        text("Current", desc.currentLabel ?? "", { step: "caption", color: "inkFaint", align: "end", width: px(col) });
        // Empty columns as wide as the rows' arrow and delta (a spacer is square: it made the header tall).
        kit.open("Arrow", { width: px(16), height: px(0) });
        kit.close();
        text("Next", desc.nextLabel ?? "", { step: "caption", color: "inkFaint", align: "end", width: px(col) });
        kit.open("Delta", { width: px(dcol), height: px(0) });
        kit.close();
      });
    }
    rows.forEach((r, k) => {
      const cur = r.current ?? 0;
      const next = r.next ?? cur;
      const fmtOf = (v: number) => formatNumber(v, r.format, r.digits ?? 0, r.unit ?? "");
      const rawDelta = next - cur;
      const same = Math.abs(rawDelta) < 1e-9;
      const better = same ? 0 : (rawDelta > 0) === (r.lowerIsBetter !== true) ? 1 : -1;
      const ink: Api.ThemeColor = better > 0 ? "success" : better < 0 ? "danger" : "inkFaint";
      // The delta springs to its new value (a count), unless motion is reduced.
      const shownDelta = still() ? rawDelta : kit.animate(kit.idi("~delta", k), rawDelta, spring("ui"));
      kit.open(kit.idi("Row", k), { direction: "column", width: grow(), gap: t.xxs, padding: xy(0, t.xs), part: `StatCompare.row${flags(better > 0 ? "better" : better < 0 ? "worse" : "same")}`, partClass: "slot" });
      const label = () => {
        if (r.icon) kit.icon("Icon", r.icon, { size: 18, tint: token("inkMuted"), label: "" });
        text("Label", r.label ?? "", { step: "sm", color: "inkMuted", width: grow() });
      };
      if (stacked) kit.row("Name", { width: grow(), gap: t.sm, cross: "center" }, label);
      kit.row("Line", { width: grow(), gap: t.sm, cross: "center" }, () => {
        if (stacked) {
          kit.open("Gap", { width: grow(), height: px(0) });
          kit.close();
        } else label();
        text("Current", fmtOf(cur), { step: "sm", font: "mono", color: same ? "ink" : "inkMuted", nowrap: true, align: "end", width: px(col) });
        kit.open("Arrow", { width: px(16), height: px(16), align: "center", cross: "center" });
        if (!same) {
          kit.setDraw({ shape: { kind: "path", points: [{ x: 3, y: 8 }, { x: 13, y: 8 }, { x: 9, y: 4 }, { x: 13, y: 8 }, { x: 9, y: 12 }], width: 1.5, cap: "round", join: "round" }, fills: [{ kind: "solid", color: token("inkFaint") }] });
        }
        kit.close();
        if (kit.frameDirection() === "rtl") kit.props({ transform: { scaleX: -1 } });
        text("Next", same ? "" : fmtOf(next), { step: "sm", font: "mono", color: ink === "inkFaint" ? "ink" : ink, nowrap: true, align: "end", width: px(col) });
        kit.open("Delta", { direction: "row", width: px(dcol), height: fit(24), gap: 2, align: "end", cross: "center" });
        if (!same) {
          kit.open("Chip", { direction: "row", gap: 2, padding: xy(6, 0), height: fit(22), cross: "center", radius: 11, background: paint(role(ink), 0.16), part: `StatCompare.delta${flags(better > 0 ? "better" : "worse")}`, partClass: "chip" });
          // Up or down, as a triangle: the direction of the change, whatever its colour.
          kit.open("Mark", { width: px(8), height: px(8) });
          kit.setDraw({ shape: { kind: "polygon", sides: 3, rotation: rawDelta > 0 ? 0 : Math.PI, rounding: 1 }, fills: [{ kind: "solid", color: token(ink) }] });
          kit.close();
          text("Text", formatDelta(Math.abs(shownDelta) < Math.abs(rawDelta) * 0.02 ? 0 : shownDelta, r.format, r.digits ?? 0, ""), { step: "caption", font: "mono", color: ink, nowrap: true });
          kit.close();
        } else mark("Same", "minus", 12, role("inkFaint"));
        kit.close();
      });
      if (r.max && r.max > 0) {
        const max = r.max;
        const a = clamp(Math.min(cur, next) / max, 0, 1);
        const b = clamp(Math.max(cur, next) / max, 0, 1);
        kit.open("Bar", { width: grow(), height: px(4), radius: 2, background: token("control"), part: "StatCompare.track", partClass: "bar" });
        kit.close();
        const bar = kit.lastPath();
        const bw = kit.bounds(bar)?.width ?? 0;
        if (bw > 0) {
          kit.open("Fill", { width: px(Math.max(0, bw * a)), height: px(4), radius: 2, background: token("inkMuted"), part: "StatCompare.fill", partClass: "bar" });
          kit.close();
          kit.props({ position: "absolute", insets: { start: 0, top: 0 }, overlaps: [bar], motion: spring("ui"), layoutMotion: spring("ui") });
          if (!same) {
            kit.open("Change", { width: px(Math.max(2, bw * (b - a))), height: px(4), radius: 2, background: token(ink), part: `StatCompare.change${flags(better > 0 ? "better" : "worse")}`, partClass: "bar" });
            kit.close();
            kit.props({ position: "absolute", insets: { start: bw * a, top: 0 }, overlaps: [bar], layoutMotion: spring("ui"), opacity: better > 0 ? 1 : 0.85 });
          }
        }
      }
      kit.close();
      kit.semantics({
        role: "listitem",
        name: same ? `${r.label ?? ""}: ${fmtOf(cur)}, no change` : `${r.label ?? ""}: ${fmtOf(cur)} to ${fmtOf(next)}, ${formatDelta(rawDelta, r.format, r.digits ?? 0, r.unit ?? "")}, ${better > 0 ? "better" : "worse"}`,
      });
      if (k < rows.length - 1 && (desc.dividers ?? false)) kit.divider(kit.idi("Divider", k), false);
    });
  } finally {
    kit.close();
  }
  kit.remember(`~w:${id}`, kit.bounds(kit.lastPath())?.width ?? 0);
  kit.semantics({ role: "list", name: desc.label ?? "" });
}
