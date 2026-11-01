// The calendar (docs/ui.md §11, Data): a month of days for daily
// rewards, events, seasons and booking a match. Dates are `YYYY-MM-DD`
// strings; the game passes today in (the engine reads no clock inside
// the tick), and the names of the months and weekdays in its language.

import type * as Api from "@origingame/engine";
import { chevron, clamp, flags, frame, grow, kit, mark, paint, px, role, spring, still, surface, text, token } from "../containers/base.ts";

const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const WEEKDAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

const pad2 = (n: number) => (n < 10 ? `0${n}` : `${n}`);
export const isoDate = (y: number, m: number, d: number): string => `${y}-${pad2(m)}-${pad2(d)}`;
function parse(s: string | undefined): { y: number; m: number; d: number } | null {
  const r = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s ?? "");
  return r ? { y: Number(r[1]), m: Number(r[2]), d: Number(r[3]) } : null;
}
const daysIn = (y: number, m: number) => new Date(Date.UTC(y, m, 0)).getUTCDate();
const weekday = (y: number, m: number, d: number) => new Date(Date.UTC(y, m - 1, d)).getUTCDay();

/**
 * A month grid: the month's name between previous and next (the tabPrev
 * and tabNext actions, LB and RB, turn the month while `bumpers` is set),
 * the weekday initials from `weekStart`, and six weeks of days. Today is
 * ringed, the selected day filled, a range shaded between its ends; days
 * outside the month are faint, days before `min` or after `max` (or
 * listed in `disabled`) are inert. A day can carry marks (dots in their
 * tones, up to three) or a claimed tick for a login calendar. Keys and
 * pads walk the days; the month slides in from the side it was turned to.
 */
export function calendar(id: string, desc: Api.CalendarDesc): Api.CalendarResult {
  const { t, phone } = frame();
  const today = parse(desc.today);
  let year = desc.year || today?.y || 2026;
  let month = clamp(desc.month || today?.m || 1, 1, 12);
  const before = year * 12 + month;
  const months = desc.monthNames && desc.monthNames.length === 12 ? desc.monthNames : MONTHS;
  const days = desc.weekdayNames && desc.weekdayNames.length === 7 ? desc.weekdayNames : WEEKDAYS;
  const short = desc.weekdayShort && desc.weekdayShort.length === 7 ? desc.weekdayShort : days.map((d) => d.slice(0, phone ? 1 : 2));
  const start = clamp(desc.weekStart ?? 1, 0, 6);
  const min = desc.min ?? "";
  const max = desc.max ?? "";
  const off = new Set(desc.disabled ?? []);
  const marks = new Map<string, Api.CalendarMark[]>();
  for (const m of desc.marks ?? []) {
    const l = marks.get(m.date ?? "") ?? [];
    l.push(m);
    marks.set(m.date ?? "", l);
  }
  const turn = (by: number) => {
    const k = year * 12 + (month - 1) + by;
    year = Math.floor(k / 12);
    month = (k % 12) + 1;
  };
  const canTurn = (by: number) => {
    const k = year * 12 + (month - 1) + by;
    const y = Math.floor(k / 12);
    const m = (k % 12) + 1;
    if (by < 0 && min && isoDate(y, m, daysIn(y, m)) < min) return false;
    if (by > 0 && max && isoDate(y, m, 1) > max) return false;
    return true;
  };
  if (desc.bumpers) {
    if (kit.action("tabPrev") && canTurn(-1)) turn(-1);
    if (kit.action("tabNext") && canTurn(1)) turn(1);
  }
  let selected: string | null = null;
  const cell = phone ? 44 : desc.cellSize ?? 44;
  kit.open(id, { direction: "column", width: desc.width ?? fitWidth(cell, t.xxs), gap: t.sm, cross: "stretch", part: "Calendar.frame", partClass: "panel" });
  try {
    kit.row("Header", { width: grow(), gap: t.xs, cross: "center" }, () => {
      const prev = surface("Prev", { part: "Calendar.step", cls: "button", flags: ["round"], fill: null, disabled: !canTurn(-1), width: px(44), height: px(44), align: "center", cross: "center", radius: 22, semantics: { role: "button", name: desc.prevLabel || "Previous month" } }, () => chevron("Chevron", "back", 16, role(canTurn(-1) ? "ink" : "inkFaint")));
      if (prev.clicked) turn(-1);
      text("Month", `${months[month - 1]} ${year}`, { step: "base", heading: true, align: "center", width: grow(), nowrap: true, ellipsis: true });
      kit.semantics({ role: "heading", name: `${months[month - 1]} ${year}` });
      const next = surface("Next", { part: "Calendar.step", cls: "button", flags: ["round"], fill: null, disabled: !canTurn(1), width: px(44), height: px(44), align: "center", cross: "center", radius: 22, semantics: { role: "button", name: desc.nextLabel || "Next month" } }, () => chevron("Chevron", "forward", 16, role(canTurn(1) ? "ink" : "inkFaint")));
      if (next.clicked) turn(1);
    });
    kit.row("Weekdays", { width: grow(), gap: t.xxs }, () => {
      for (let k = 0; k < 7; k++) {
        const wd = (start + k) % 7;
        text(kit.idi("Day", k), short[wd] ?? "", { step: "caption", color: wd === 0 || wd === 6 ? "inkFaint" : "inkMuted", align: "center", width: grow(), nowrap: true });
      }
    });
    // The month itself, keyed by its month so a turn plays the slide.
    const key = year * 12 + month;
    const dir = Math.sign(key - Math.round(kit.memory("~shown", key)));
    kit.remember("~shown", key);
    const first = weekday(year, month, 1);
    const lead = (first - start + 7) % 7;
    const n = daysIn(year, month);
    const prevN = daysIn(month === 1 ? year - 1 : year, month === 1 ? 12 : month - 1);
    const lo = desc.rangeStart ?? "";
    const hi = desc.rangeEnd ?? "";
    kit.open(`Month~${key}`, { direction: "column", width: grow(), gap: t.xxs, cross: "stretch" });
    for (let w = 0; w < 6; w++) {
      kit.row(kit.idi("Week", w), { width: grow(), gap: t.xxs }, () => {
        for (let k = 0; k < 7; k++) {
          const at = w * 7 + k - lead + 1;
          const inMonth = at >= 1 && at <= n;
          let y = year;
          let m = month;
          let d = at;
          if (at < 1) (m = month === 1 ? 12 : month - 1, y = month === 1 ? year - 1 : year, d = prevN + at);
          else if (at > n) (m = month === 12 ? 1 : month + 1, y = month === 12 ? year + 1 : year, d = at - n);
          const date = isoDate(y, m, d);
          const isToday = today !== null && today.y === y && today.m === m && today.d === d;
          const isSel = desc.selected === date;
          const inRange = lo && hi && date >= lo && date <= hi;
          const edge = date === lo || date === hi;
          const blocked = off.has(date) || (min !== "" && date < min) || (max !== "" && date > max);
          const ms = marks.get(date) ?? [];
          const claimed = ms.some((x) => x.kind === "claimed");
          const wd = (start + k) % 7;
          const filled = isSel || edge;
          const name = `${days[wd]}, ${months[m - 1]} ${d}, ${y}${isToday ? `, ${desc.todayLabel || "today"}` : ""}${claimed ? `, ${desc.claimedLabel || "claimed"}` : ""}${ms.filter((x) => x.label).map((x) => `, ${x.label}`).join("")}`;
          if (!inMonth && !(desc.showOutside ?? true)) {
            kit.open(kit.idi("Cell", k), { width: grow(), height: px(cell) });
            kit.close();
            continue;
          }
          const r = surface(kit.idi("Cell", k), {
            part: "Calendar.day",
            cls: "chip",
            flags: [isSel ? "selected" : "", isToday ? "today" : "", inRange ? "inRange" : "", !inMonth ? "outside" : "", claimed ? "claimed" : "", wd === 0 || wd === 6 ? "weekend" : ""],
            selected: filled,
            fill: inRange ? { ...role("primary"), a: 0.18 } : null,
            selectedFill: role("primary"),
            washBy: 0.12,
            disabled: blocked || !inMonth,
            focusable: inMonth && !blocked,
            direction: "column",
            width: grow(),
            height: px(cell),
            gap: 1,
            align: "center",
            cross: "center",
            radius: inRange && !filled ? t.small : cell / 2,
            semantics: { role: "gridcell", name, selected: isSel },
          }, () => {
            text("Number", `${d}`, { step: "sm", font: "mono", nowrap: true, align: "center", color: filled ? "onPrimary" : !inMonth || blocked ? "inkFaint" : isToday ? "primary" : "ink" });
            if (claimed) mark("Claimed", "tick", 12, role(filled ? "onPrimary" : "success"), 2);
            else if (ms.length > 0) {
              kit.row("Marks", { gap: 2, height: px(5), cross: "center" }, () => {
                ms.slice(0, 3).forEach((x, j) => {
                  kit.open(kit.idi("Mark", j), { width: px(5), height: px(5), radius: 2.5, background: filled ? token("onPrimary") : paint(role(toneInk(x.tone))) });
                  kit.close();
                });
              });
            }
            if (isToday && !filled) {
              kit.open("Today", { width: grow(), height: grow() });
              kit.setDraw({ shape: { kind: "ellipse" }, strokes: [{ width: 1.5, align: "inside", paint: { kind: "solid", color: token("primary") } }] });
              kit.close();
              kit.props({ position: "absolute", insets: { start: (cell - Math.min(cell, 40)) / 2, top: (cell - Math.min(cell, 40)) / 2 }, inert: true });
            }
          });
          if (r.clicked && inMonth && !blocked) selected = date;
        }
      });
      kit.semantics({ role: "row" });
    }
    kit.close();
    kit.semantics({ role: "grid", name: `${months[month - 1]} ${year}` });
    if (dir !== 0 && !still()) {
      kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: 28 * dir * (kit.frameDirection() === "rtl" ? -1 : 1) }], transition: spring("panel") }, exit: { values: [{ prop: "opacity", value: 0 }], transition: { kind: "tween", durationMs: 90, ease: "exit" } } });
    }
  } finally {
    kit.close();
  }
  const changed = year * 12 + month !== before;
  if (selected !== null) kit.feedback("select", -1);
  else if (changed) kit.feedback("tab", -1);
  return { selected, year, month, changed };
}

function toneInk(tone: string | undefined): Api.ThemeColor {
  return ({ success: "success", warn: "warn", danger: "danger", info: "info", primary: "primary", gold: "gold", neutral: "inkMuted" } as Record<string, Api.ThemeColor>)[tone ?? "primary"] ?? "primary";
}

function fitWidth(cell: number, gap: number): Api.Size {
  return { kind: "fit", min: cell * 7 + gap * 6 };
}

