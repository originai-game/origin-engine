// The management ledger and the map's overlay bar (docs/ui.md §11 genre
// patterns, Appendix B: management dashboards, ledgers, budgets, graphs,
// overlays). Read against Cities: Skylines' budget and info views,
// Crusader Kings' and Stellaris' ledgers and Football Manager's finances:
// the headline figures with their change, the lines by section with their
// totals and funding sliders, sortable by name, amount or change, the net
// at the foot, the history graph and a section's breakdown beside them;
// and over a city's map, the row of overlays with the shown one's legend.
// The money, the rates and what an overlay draws on the map are the game's.

import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, clamp, eyebrow, flags, frame, glow, grouped, grow, hex, kit, line, mix, paint, parentOf, px, rememberRoom, role, rtl, safeHeight,
  safeWidth, shadow, st, surface, symbol, text, token, u, vgrad, xy, type Mut,
} from "./core.ts";
import { chevron } from "../containers/base.ts";
import { chart } from "../data/charts.ts";
import { stepSize } from "../feedback/core.ts";
import { formatNumber } from "../data/format.ts";
import { valueSlider } from "../controls/ranges.ts";

/** An empty column `w` wide (a row's place for a slider it does not have). */
function blank(id: string, w: number): void {
  kit.open(id, { width: px(w), height: px(0) });
  kit.close();
}

/** A line's change since last period, as a fraction; null when it had none. */
const changeOf = (l: Api.LedgerLine): number | null => ((l.previous ?? -1) > 0 ? ((l.amount ?? 0) - l.previous!) / l.previous! : null);

/** A signed percent with its arrow: ▲ 4.2% or ▼ 12%. */
function signedPct(f: number): string {
  const p = Math.abs(f * 100);
  return `${f >= 0 ? "+" : "\u2212"}${p >= 10 ? Math.round(p) : Math.round(p * 10) / 10}%`;
}

/** An up or down arrow and a signed change, in success or danger by whether it is good news. */
function delta(id: string, value: string, up: boolean, good: boolean, size: number): void {
  kit.row(id, { gap: Math.max(2, Math.round(size * 0.3)), cross: "center", part: `Ledger.delta${flags(up ? "up" : "down", good ? "good" : "bad")}`, partClass: "badge" }, () => {
    chevron("Arrow", up ? "up" : "down", Math.round(size * 0.7), role(good ? "success" : "danger"), 0, 2.5);
    text("Value", value, { step: st(size), font: "mono", nowrap: true, color: good ? "success" : "danger" });
  });
}

/** The ledger rows' heights, by the ledger's id and the row's key, from a tick each was seen whole. */
const rowHeights = new Map<string, number>();

export function ledger(id: string, desc: Api.LedgerDesc): Api.LedgerResult {
  const out: Mut<Api.LedgerResult> = { tab: null, sort: null, descending: false, funding: null, selected: null };
  const { t, phone } = frame();
  const portrait = frame().viewport.breakpoint === "phonePortrait";
  const W = kit.memory(`~le.w.${id}`, 0) || firstRoom().w;
  const H = kit.memory(`~le.h.${id}`, 0) || firstRoom().h;
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const cur = desc.currency ?? "";
  const money = (v: number): string => `${v < 0 ? "\u2212" : ""}${cur}${grouped(Math.abs(v))}`;
  const sections = desc.sections ?? [];
  const lines = desc.lines ?? [];
  const sort = desc.sort ?? "";
  const down = !!desc.descending;
  const fmin = desc.fundingMin ?? 50;
  const fmax = desc.fundingMax ?? 150;
  const total = (s: Api.LedgerSection): number => lines.filter((l) => l.section === s.key).reduce((n, l) => n + (l.amount ?? 0), 0);
  const prevTotal = (s: Api.LedgerSection): number => lines.filter((l) => l.section === s.key).reduce((n, l) => n + Math.max(0, l.previous ?? 0), 0);
  const net = sections.reduce((n, s) => n + (s.expense ? -1 : 1) * total(s), 0);

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 10 : 16), part: "Ledger.frame", partClass: "panel" });

  // The head: the title, then the tabs (a lit chip for the one shown); the tab actions step them.
  const tabs = desc.tabs ?? [];
  if (desc.title || tabs.length > 0) {
    kit.row("Head", { width: grow(), gap: u(16), cross: "center", wrap: portrait }, () => {
      if (desc.title) text("Title", desc.title, { step: phone ? "xl3" : "xl5", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow() });
      else kit.spacer("Gap", grow());
      if (tabs.length > 0) {
        kit.row("Tabs", { gap: u(4), cross: "center", padding: xy(u(4), u(4)), part: "Ledger.tabs", partClass: "panel" }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.5) }], strokes: [line(1, token("ink", 0.08))] });
          for (const tb of tabs) {
            const k = tb.key ?? "";
            const on = k === desc.tab;
            const r = surface(k, {
              part: `Ledger.tab${flags(on && "selected")}`,
              cls: "tab",
              fill: on ? mix(role("panelRaised"), role("ink"), 0.12) : null,
              selected: on,
              selectedFill: mix(role("panelRaised"), role("ink"), 0.12),
              padding: xy(u(phone ? 12 : 16), u(phone ? 6 : 8)),
              radius: 999,
              semantics: { role: "tab", name: tb.label ?? k, selected: on },
            }, () => text("Text", tb.label ?? k, { step: st(phone ? 13 : 15), font: "display", nowrap: true, color: on ? "ink" : "inkMuted" }));
            if (r.clicked && !on) {
              out.tab = k;
              kit.feedback("tab", -1);
            }
          }
        });
        kit.semantics({ role: "tablist", name: desc.title || "Ledger" });
        const at = tabs.findIndex((x) => x.key === desc.tab);
        const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
        if (step !== 0 && at >= 0) {
          out.tab = tabs[(at + step + tabs.length) % tabs.length]!.key ?? null;
          kit.feedback("tab", -1);
        }
      }
    });
  }

  // The headline figures: a tile each, its art, caption, value large and its change.
  const figures = (): void => {
    const list = desc.figures ?? [];
    if (list.length === 0) return;
    const gap = u(phone ? 8 : 12);
    const cols = portrait ? 2 : list.length;
    const tileW = Math.floor((W - gap * (cols - 1)) / cols);
    kit.row("Figures", { width: grow(), gap, wrap: portrait }, () => {
      for (const f of list) {
        const k = f.key ?? "";
        const d = f.delta ?? 0;
        const good = (d > 0) !== !!f.lowerIsBetter;
        const fmt = (v: number): string => (f.format === "percent" || f.format === "compact" ? formatNumber(v, f.format, f.format === "percent" ? 1 : 0, "") : grouped(v));
        const value = `${fmt(f.value ?? 0)}${f.unit ? ` ${f.unit}` : ""}`;
        kit.row(k, { width: portrait ? px(tileW) : grow(), gap: u(phone ? 8 : 12), cross: "center", padding: xy(u(phone ? 10 : 16), u(phone ? 10 : 14)), part: "Ledger.figure", partClass: "panel" }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }, vgrad([[0, token("sheen", 0.07)], [1, token("sheen", 0)]])], strokes: [line(1, token("ink", 0.1))], effects: [shadow(u(4), u(14), 0.4)] });
          if (f.image !== undefined && f.image !== null) {
            const z = u(phone ? 32 : 48);
            artSlot("Art", "Ledger.figureArt", { texture: f.image, kind: "item", width: px(z), height: px(z), radius: Math.max(4, t.small) });
          }
          kit.column("Words", { width: grow(), gap: u(2) }, () => {
            eyebrow("Label", f.label ?? k);
            text("Value", value, { step: st(phone ? 20 : 30), font: "mono", nowrap: true, ellipsis: true, width: grow() });
            if (d !== 0) delta("Delta", f.format === "percent" ? `${d > 0 ? "+" : "\u2212"}${formatNumber(Math.abs(d), "percent", 1, "")}` : `${d > 0 ? "+" : "\u2212"}${fmt(Math.abs(d))}`, d > 0, good, phone ? 12 : 14);
          });
        });
        kit.semantics({ role: "status", name: `${f.label ?? k}: ${value}${d !== 0 ? `, ${d > 0 ? "up" : "down"} ${fmt(Math.abs(d))}` : ""}` });
      }
    });
  };

  // The ledger's columns: the amount, the change and the budget at fixed widths, the name taking the rest.
  const colAmount = Math.round(u(phone ? 84 : 120) * Math.min(1.5, ts));
  const colChange = Math.round(u(phone ? 64 : 90) * Math.min(1.5, ts));
  const colFunding = Math.round(u(210) * Math.min(1.3, ts));
  // Short of room for its text (a laptop at 200 %), the desktop stacks like a phone and the page scrolls, so the
  // ledger keeps whole rows rather than one line between the figures and the net.
  const stacked = portrait || H < u(520) * ts;
  const funded = lines.some((l) => (l.funding ?? -1) >= 0);
  const inlineFunding = !portrait && funded;

  const heading = (key: string, label: string, w: Api.Size, end: boolean): void => {
    const on = sort === key;
    const r = surface(key, {
      part: `Ledger.heading${flags(on && "sorted")}`,
      cls: "listItem",
      fill: null,
      washBy: 0.1,
      width: w,
      gap: u(4),
      cross: "center",
      align: end ? "end" : "start",
      padding: xy(u(6), u(6)),
      radius: Math.max(4, t.small),
      semantics: { role: "columnheader", name: `${label}${on ? (down ? ", sorted high to low" : ", sorted low to high") : ""}`, selected: on },
    }, () => {
      text("Text", label.toLocaleUpperCase(), { step: "caption", font: "display", nowrap: true, ellipsis: true, color: on ? "ink" : "inkMuted" });
      if (on) chevron("Sort", down ? "down" : "up", u(10), role("ink"), 0, 2);
    });
    if (r.clicked) {
      out.sort = key;
      out.descending = on ? !down : key !== "name";
      kit.feedback("select", -1);
    }
  };

  const sorted = (s: Api.LedgerSection): Api.LedgerLine[] => {
    const ls = lines.filter((l) => l.section === s.key);
    if (!sort) return ls;
    const k = (l: Api.LedgerLine): number | string => (sort === "name" ? (l.name ?? l.key ?? "") : sort === "amount" ? l.amount ?? 0 : changeOf(l) ?? -Infinity);
    const out2 = [...ls].sort((a, b) => {
      const x = k(a);
      const y = k(b);
      const c = typeof x === "string" && typeof y === "string" ? x.localeCompare(y) : (x as number) - (y as number);
      return down ? -c : c;
    });
    return out2;
  };

  const lineRow = (l: Api.LedgerLine, s: Api.LedgerSection): void => {
    const k = l.key ?? "";
    const ch = changeOf(l);
    const good = ch !== null && (ch > 0) !== !!s.expense;
    const f = l.funding ?? -1;
    const warn = !!l.warning;
    const sliderRow = (w: Api.Size): void => {
      kit.row("Budget", { width: w, gap: u(8), cross: "center" }, () => {
        if (l.locked) symbol("Lock", "lock", u(14), role("inkMuted"), { icon: "lock" });
        const r = valueSlider("Funding", f, { min: fmin, max: fmax, step: desc.fundingStep ?? 5, showValue: false, disabled: !!l.locked, width: grow(), label: `${desc.fundingLabel ?? "Budget"} ${l.name ?? k}`, origin: 100 });
        if (r.changed && !l.locked) out.funding = { key: k, value: r.value };
        text("Percent", `${Math.round(f)}%`, { step: st(phone ? 13 : 15), font: "mono", nowrap: true, color: f < 100 ? "warn" : f > 100 ? "info" : "inkMuted" });
      });
    };
    const r = surface(k, {
      part: `Ledger.line${flags(warn && "warning")}`,
      cls: "listItem",
      fill: null,
      washBy: 0.08,
      direction: "column",
      width: grow(),
      gap: u(6),
      padding: xy(u(8), u(phone ? 6 : 8)),
      radius: Math.max(4, t.small),
      semantics: { role: "row", name: [l.name ?? k, money(l.amount ?? 0), ch !== null ? signedPct(ch) : "", f >= 0 ? `${desc.fundingLabel ?? "Budget"} ${Math.round(f)}%` : "", l.locked ? desc.lockedLabel ?? "Locked" : "", l.warning ?? ""].filter(Boolean).join(", ") },
    }, () => {
      kit.row("Main", { width: grow(), gap: u(phone ? 8 : 12), cross: "center" }, () => {
        const z = u(phone ? 32 : 40);
        artSlot("Art", "Ledger.lineArt", { texture: l.image ?? null, kind: "item", width: px(z), height: px(z), radius: Math.max(4, t.small) });
        kit.column("Name", { width: grow(), gap: 0 }, () => {
          text("Text", l.name ?? k, { step: st(phone ? 14 : 16), nowrap: true, ellipsis: true, width: grow() });
          if (warn) kit.row("Warning", { width: grow(), gap: u(4), cross: "center" }, () => {
            symbol("Mark", "bolt", u(12), role("warn"));
            text("Text", l.warning!, { step: st(12), color: "warn", nowrap: true, ellipsis: true, width: grow() });
          });
        });
        text("Amount", money(l.amount ?? 0), { step: st(phone ? 14 : 16), font: "mono", nowrap: true, align: "end", width: px(colAmount) });
        kit.row("Change", { width: px(colChange), align: "end", cross: "center" }, () => {
          if (ch !== null && Math.abs(ch) >= 0.0005) delta("Delta", signedPct(ch), ch > 0, good, phone ? 12 : 13);
          else text("None", "\u2014", { step: st(13), color: "inkFaint", nowrap: true });
        });
        if (inlineFunding) {
          if (f >= 0) sliderRow(px(colFunding));
          else blank("NoBudget", colFunding);
        }
      });
      if (!inlineFunding && f >= 0) sliderRow(grow());
    });
    if (r.clicked) {
      out.selected = k;
      kit.feedback("select", -1);
    }
  };

  const sectionHead = (s: Api.LedgerSection): void => {
    const tot = total(s);
    const prev = prevTotal(s);
    const ch = prev > 0 ? (tot - prev) / prev : null;
    kit.row(s.key ?? "", { width: grow(), gap: u(12), cross: "center", padding: xy(u(8), u(phone ? 6 : 8)), part: "Ledger.section", partClass: "listItem" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [{ kind: "solid", color: token("ink", 0.05) }] });
      text("Name", (s.name ?? s.key ?? "").toLocaleUpperCase(), { step: st(phone ? 13 : 15), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow() });
      text("Total", `${s.expense ? "\u2212" : ""}${money(tot)}`, { step: st(phone ? 14 : 16), font: "mono", nowrap: true, align: "end", width: px(colAmount), color: s.expense ? "danger" : "success" });
      kit.row("Change", { width: px(colChange), align: "end", cross: "center" }, () => {
        if (ch !== null && Math.abs(ch) >= 0.0005) delta("Delta", signedPct(ch), ch > 0, (ch > 0) !== !!s.expense, phone ? 12 : 13);
      });
      if (inlineFunding) blank("NoBudget", colFunding);
    });
    kit.semantics({ role: "rowheader", name: `${s.name ?? s.key ?? ""}, ${desc.totalLabel ?? "Total"} ${money(tot)}` });
  };

  const header = (): void => {
    kit.row("Header", { width: grow(), gap: u(12), cross: "center", padding: xy(u(8), 0), part: "Ledger.header", partClass: "panel" }, () => {
      blank("Art", u(phone ? 32 : 40));
      heading("name", desc.nameLabel ?? "Name", grow(), false);
      heading("amount", desc.amountLabel ?? "Amount", px(colAmount), true);
      heading("change", desc.changeLabel ?? "Change", px(colChange), true);
      if (inlineFunding) kit.row("Funding", { width: px(colFunding), padding: xy(u(6), 0) }, () => eyebrow("Text", desc.fundingLabel ?? "Budget"));
    });
  };

  const netRow = (): void => {
    const good = net >= 0;
    kit.row("Net", { width: grow(), gap: u(12), cross: "center", padding: xy(u(16), u(phone ? 10 : 12)), part: `Ledger.net${flags(good ? "good" : "bad")}`, partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token(good ? "success" : "danger", 0.12) }], strokes: [line(1, token(good ? "success" : "danger", 0.45))] });
      text("Caption", (desc.netLabel ?? "Net").toLocaleUpperCase(), { step: st(phone ? 14 : 16), font: "display", heading: true, nowrap: true, width: grow() });
      text("Value", `${good ? "+" : ""}${money(net)}`, { step: st(phone ? 18 : 22), font: "mono", nowrap: true, color: good ? "success" : "danger" });
    });
    kit.semantics({ role: "status", name: `${desc.netLabel ?? "Net"} ${good ? "+" : ""}${money(net)}` });
  };

  const ledgerList = (scroll: boolean): void => {
    if (scroll) {
      // Whole rows only: the list is cut at the last row that fits the room above Net (from last tick's layout), so no
      // row stands half under it; the rest scroll.
      // Rows is the table's next child after its header (built just before, so lastPath names it).
      const rowsPath = `${parentOf(kit.lastPath())}/Rows`;
      kit.open("Rows", { direction: "column", width: grow(), height: grow() });
      const room = kit.bounds(rowsPath);
      const prev = kit.memory(`~le.rows.${id}`, 0);
      // og.ui.bounds gives the part in view, so each row's height is kept from a tick it was seen whole (ending clear of
      // the room's edge); the list then holds the rows from the top whose heights add up within the room.
      if (room && room.height > 0) {
        const keys = sections.flatMap((x) => [x.key ?? "", ...sorted(x).map((l) => l.key ?? "")]);
        for (const k of keys) {
          const b = kit.bounds(`${rowsPath}/List/${k}`);
          if (b && b.y >= room.y - 0.5 && b.y + b.height < room.y + room.height - 0.5) rowHeights.set(`${id}/${k}`, b.height);
        }
        let fit = 0;
        for (const k of keys) {
          const h = rowHeights.get(`${id}/${k}`);
          const end = fit + (fit > 0 ? u(2) : 0) + (h ?? Infinity);
          if (end > room.height + 0.5) break;
          fit = end;
        }
        kit.remember(`~le.rows.${id}`, fit > 0 ? Math.ceil(fit) : prev);
      } else kit.remember(`~le.rows.${id}`, prev);
      kit.openScroll("List", { width: grow(), height: prev > 0 && room && room.height > 0 ? px(Math.min(prev, room.height)) : grow(), gap: u(2), padding: xy(0, 0), fade: 28, fadeInto: "shadow" } as never);
    } else kit.open("List", { direction: "column", width: grow(), gap: u(2) });
    for (const s of sections) {
      sectionHead(s);
      for (const l of sorted(s)) lineRow(l, s);
    }
    kit.close();
    kit.semantics({ role: "table", name: desc.title || "Ledger" });
    if (scroll) kit.close();
  };

  // The graphs: the history over the periods, and a section's lines as a donut.
  // `room`, the scrolling column the graphs stand in (by path): the donut's legend keeps the lines that fit under the
  // history in it, the largest first, and folds the rest into one Other line, so its card ends inside the column.
  const graphs = (w: Api.Size, chartH: number, room = ""): void => {
    const hist = desc.history ?? [];
    const bd = sections.find((s) => s.key === desc.breakdown) ?? null;
    if (hist.length === 0 && !bd) return;
    kit.column("Graphs", { width: w, gap: u(12) }, () => {
      const card = (key: string, title: string, build: () => void): void => {
        kit.column(key, { width: grow(), gap: u(8), padding: xy(u(14), u(12)), part: "Ledger.graph", partClass: "panel" }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }], strokes: [line(1, token("ink", 0.1))], effects: [shadow(u(4), u(14), 0.4)] });
          eyebrow("Title", title);
          build();
        });
      };
      if (hist.length > 0) {
        card("History", desc.historyLabel ?? "History", () => {
          chart("Chart", { kind: desc.graph ?? "line", series: hist, categories: desc.periods ?? [], height: chartH, smooth: true, legend: true, format: "compact", label: desc.historyLabel ?? "History" });
        });
      }
      if (bd) {
        let ls = lines.filter((l) => l.section === bd.key).map((l) => ({ name: l.name ?? l.key ?? "", amount: l.amount ?? 0 }));
        const col = room ? kit.bounds(room) : null;
        const hist0 = room ? kit.bounds(`${room}/Graphs/History`) : null;
        const row0 = room ? kit.bounds(`${room}/Graphs/Breakdown/Chart/Body/Legend/Row#0`) : null;
        // The card's own room: the column less the history's card and the gap, less the title and padding.
        let donutH = Math.round(chartH * 0.9);
        if (col && col.height > 0) {
          const head = u(12) * 2 + u(8) + stepSize("caption").line;
          const left = col.height - (hist0 ? hist0.height + u(12) : 0) - head - 1;
          donutH = Math.max(u(96), Math.min(donutH, Math.floor(left)));
          const rowH = row0 && row0.height > 0 ? Math.max(row0.height, u(36)) : u(36);
          const fit = Math.max(3, Math.floor(left / rowH));
          if (ls.length > fit) {
            const by = [...ls].sort((a, b) => b.amount - a.amount);
            ls = [...by.slice(0, fit - 1), { name: desc.otherLabel ?? "Other", amount: by.slice(fit - 1).reduce((n, l) => n + l.amount, 0) }];
          }
        }
        card("Breakdown", `${desc.breakdownLabel ?? "Breakdown"} · ${bd.name ?? ""}`, () => {
          chart("Chart", { kind: "donut", series: [{ label: bd.name ?? "", values: ls.map((l) => l.amount) }], categories: ls.map((l) => l.name), height: donutH, totalLabel: desc.totalLabel ?? "Total", format: "compact", label: `${desc.breakdownLabel ?? "Breakdown"} ${bd.name ?? ""}` });
        });
      }
    });
  };

  if (stacked) {
    kit.openScroll("Body", { width: grow(), height: grow(), gap: u(10), padding: xy(0, 0), fade: 36, fadeInto: "shadow" } as never);
    figures();
    kit.column("Ledger", { width: grow(), gap: u(4), part: "Ledger.table", partClass: "panel" }, () => {
      header();
      ledgerList(false);
    });
    graphs(grow(), Math.round(clamp(H * 0.22, 140, u(220)) * Math.min(1.8, ts)));
    kit.close();
    netRow();
  } else {
    figures();
    kit.row("Body", { width: grow(), height: grow(), gap: u(16) }, () => {
      kit.column("Ledger", { width: grow(), height: grow(), gap: u(6), padding: xy(u(12), u(12)), part: "Ledger.table", partClass: "panel" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.86) }], strokes: [line(1, token("ink", 0.08))] });
        header();
        ledgerList(true);
        netRow();
      });
      const gw = Math.round(Math.min(u(480), W * 0.32) * Math.min(1.15, ts));
      kit.openScroll("Side", { width: px(gw), height: grow(), gap: 0, padding: xy(0, 0), fade: 28, fadeInto: "shadow" } as never);
      graphs(grow(), Math.round(clamp(H * 0.22, 130, u(200))), `${parentOf(kit.lastPath())}/Side`);
      kit.close();
    });
  }
  kit.close();
  rememberRoom("le", id);
  return out;
}

// ── the overlay bar ─────────────────────────────────────────────────────

export function overlayBar(id: string, desc: Api.OverlayBarDesc): Api.OverlayBarResult {
  const out: Mut<Api.OverlayBarResult> = { selected: null };
  const { t, phone } = frame();
  const list = desc.overlays ?? [];
  const shown = list.find((o) => o.key === desc.selected) ?? null;
  kit.open(id, { direction: "column", gap: u(8), cross: "center", part: "Overlay.frame", partClass: "panel" });

  // The legend of the shown overlay: its name, the ramp low to high, the two words at its ends.
  if (shown) {
    const ramp = (shown.ramp ?? []).map((c) => paint(hex(c)));
    kit.column("Legend", { gap: u(6), padding: xy(u(14), u(10)), cross: "stretch", part: "Overlay.legend", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.92) }], strokes: [line(1, token("ink", 0.12))], effects: [shadow(u(4), u(16), 0.45)] });
      text("Name", (shown.name ?? shown.key ?? "").toLocaleUpperCase(), { step: st(phone ? 13 : 15), font: "display", heading: true, nowrap: true });
      kit.open("Ramp", { width: px(u(phone ? 220 : 280)), height: px(u(10)), part: "Overlay.ramp", partClass: "badge" });
      if (ramp.length > 0) {
        const n = Math.max(1, ramp.length - 1);
        const from = rtl() ? { x: 1, y: 0.5 } : { x: 0, y: 0.5 };
        const to = rtl() ? { x: 0, y: 0.5 } : { x: 1, y: 0.5 };
        kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [ramp.length === 1 ? { kind: "solid", color: ramp[0]! } : { kind: "gradient", type: "linear", from, to, stops: ramp.map((c, i) => ({ offset: i / n, color: c })) }], strokes: [line(1, token("ink", 0.2))] } as Api.DrawDesc);
      }
      kit.close();
      kit.row("Ends", { width: grow(), gap: u(12) }, () => {
        text("Low", shown.low ?? "", { step: st(12), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
        text("High", shown.high ?? "", { step: st(12), color: "inkMuted", nowrap: true, ellipsis: true, width: grow(), align: "end" });
      });
    });
    kit.semantics({ role: "status", name: `${shown.name ?? ""}: ${shown.low ?? ""} to ${shown.high ?? ""}` });
  }

  // The bar: a round button per overlay, its art; the shown one lit and named.
  const z = Math.max(44, u(phone ? 48 : 56));
  kit.openScroll("Bar", { gap: u(8), padding: xy(u(8), u(8)), axes: "horizontal", part: "Overlay.bar", partClass: "panel" } as never);
  kit.open("Row", { direction: "row", gap: u(8), cross: "center" });
  for (const o of list) {
    const k = o.key ?? "";
    const on = !!shown && shown.key === k;
    const r = surface(k, {
      part: `Overlay.button${flags(on && "selected")}`,
      cls: "chip",
      fill: on ? { ...role("primary"), a: 0.18 } : { ...role("shadow"), a: 0.6 },
      washBy: 0.14,
      gap: u(8),
      cross: "center",
      height: px(z),
      padding: { left: u(4), right: on ? u(16) : u(4), top: u(4), bottom: u(4) },
      radius: 999,
      semantics: { role: "button", name: o.name ?? k, checked: on },
    }, () => {
      if (on) kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("primary", 0.18) }], strokes: [line(u(2), token("primary"))], effects: [glow(u(10), token("primary", 0.45), 0)] });
      const a = z - u(8);
      artSlot("Art", "Overlay.art", { texture: o.image ?? null, kind: "item", width: px(a), height: px(a), radius: 999 });
      if (on) text("Name", o.name ?? k, { step: st(phone ? 13 : 15), font: "display", nowrap: true });
    });
    if (r.clicked) {
      out.selected = on ? "" : k;
      kit.feedback("toggle", -1);
    }
  }
  kit.close();
  kit.close();
  kit.semantics({ role: "toolbar", name: desc.label ?? "Overlays" });
  kit.close();
  return out;
}
