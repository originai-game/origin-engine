// The shooter kit's scoreboard: read against Valorant's and Overwatch's tab
// scoreboards (team headers in their colour, tabular columns, the player's
// row lit, the dead dimmed). Moved from the engine's hud/scores.ts
// (docs/ui.md §17).

import { a11y, anchorOf, box, chip, hudRoom, frame, hudPlace, icon, inkOn, keyId, kit, leaf, lit, pct, px, role, sel, shapeOf, solid, stroke, teamColor, text, token, ts, u, vgrad, ut, stepSize } from "@origingame/engine/ui-parts/hud";
import type { ScoreColumn, ScoreboardDesc } from "../types.ts";

function pingLevel(ms: number): "good" | "fair" | "poor" {
  return ms > 200 ? "poor" : ms > 100 ? "fair" : "good";
}

export function scoreboard(id: string, desc: ScoreboardDesc = {}): void {
  const f = frame();
  const tk = f.t;
  const vp = f.viewport;
  const width = Math.min(desc.width || (f.phone ? hudRoom() : u(760)), hudRoom());
  const allCols = desc.columns ?? [];
  // Rows shrink to fit the safe area's height (a phone held sideways), to
  // no less than a touch row; past that the panel scrolls.
  const teams = desc.teams ?? [];
  const lines = teams.reduce((n, tm) => n + 1 + (tm.rows ?? []).length, 0);
  const roomH = vp.height - vp.safeArea.top - vp.safeArea.bottom - tk.gutter * 2;
  const chrome = (desc.title || desc.subtitle ? u(30) + tk.sm : 0) + tk.md * 2 + teams.length * tk.sm + lines * u(2);
  // Rows tighten to their text's line (and a little air) before the board has to scroll: a short
  // screen at a large text size (a phone held sideways at 200 %) still shows every row.
  const rowH = Math.max(Math.min(ut(22), stepSize(ts(13)).line + u(4)), Math.min(ut(f.phone ? 30 : 34), Math.floor((roomH - chrome) / Math.max(1, lines))));
  const scrolls = chrome + lines * rowH > roomH;
  // Each column is as wide as its widest value or label (measured at the
  // text size in effect, mono at 0.62 em), at least its design width. A
  // panel too narrow for them all (a phone at 200 %) drops columns from the
  // right, ping first, rather than cut every value to "6,…": the name keeps
  // its share and what shows reads whole.
  const monoEm = stepSize(ts(13)).size * 0.62;
  const need = (c: ScoreColumn, i: number): number => {
    const texts = [c.label ?? "", ...teams.flatMap((tm) => (tm.rows ?? []).map((r) => r.values?.[i] ?? ""))];
    return Math.max(u(c.width || 56), Math.ceil(Math.max(...texts.map((x) => x.length)) * monoEm) + u(4));
  };
  const nameMin = Math.max(f.phone ? u(84) : u(140), Math.ceil(stepSize(ts(13)).size * 6));
  const room = width - tk.md * 2 - tk.sm * 2 - u(3) - nameMin;
  let pingW = Math.max(u(40), Math.ceil(Math.max(...teams.map((tm) => (tm.score ?? "").length), 2) * stepSize(ts(20, true)).size * 0.62));
  let shownCols = allCols.length;
  const total = (n: number, ping: number): number => allCols.slice(0, n).reduce((w, c, i) => w + need(c, i) + tk.sm, 0) + ping + tk.sm * 2;
  while (shownCols > 1 && total(shownCols, pingW) > room) shownCols -= 1;
  const cols = allCols.slice(0, shownCols);
  const colW = (c: ScoreColumn | undefined) => (c ? need(c, allCols.indexOf(c)) : u(56));
  if (total(shownCols, pingW) > room) pingW = Math.max(0, room - total(shownCols, 0));
  // A scoreboard opens over the rest of the HUD, as a held Tab board does.
  hudPlace(id, anchorOf(desc, "center"), () => {
    box(id, {
      direction: "column", gap: tk.sm, width: px(width), padding: { left: tk.md, right: tk.md, top: tk.md, bottom: tk.md }, part: sel("Scoreboard", "panel"), partClass: "panel",
      ...(scrolls ? { height: px(roomH), scroll: true } : {}),
    }, {
      ...chip(shapeOf("panel").radius, true),
      // Opaque whatever the style's HUD alpha: it opens over the rest of the HUD, which must not show through.
      fills: [solid(lit({ ...role("hud"), a: 1 })), vgrad([[0, token("sheen", 0.1)], [0.4, token("sheen", 0)]])],
    }, () => {
      if (desc.title || desc.subtitle) {
        // Title and subtitle share a line only when both fit whole (display letters at 0.6 em, text at 0.55):
        // else the subtitle goes under the title, which wraps rather than stubs.
        const inner = width - tk.md * 2;
        const together = (desc.title ?? "").length * stepSize(ts(20)).size * 0.6 + tk.sm + (desc.subtitle ?? "").length * stepSize(ts(13)).size * 0.55 <= inner;
        box("Head", { gap: together ? tk.sm : u(2), cross: together ? "end" : "start", width: pct(1), direction: together ? "row" : "column" }, undefined, () => {
          text("Title", desc.title ?? "", { step: ts(20), font: "display", color: token("ink"), width: together ? { kind: "grow" } : pct(1), nowrap: together, maxLines: 2, ellipsis: true });
          if (desc.subtitle) text("Subtitle", desc.subtitle, { step: ts(13), color: token("inkMuted"), nowrap: true });
        });
      }
      (desc.teams ?? []).forEach((team, ti) => {
        const tc = teamColor(team.team ?? ti);
        box(kit.idi("Team", ti), { direction: "column", gap: u(2), width: pct(1), part: sel("Scoreboard", "team", ["team", team.team ?? ti]) }, undefined, () => {
          // The header: the team's colour band, its name and score, the column labels.
          box("Header", { direction: "row", gap: tk.sm, cross: "center", width: pct(1), height: px(rowH), padding: { left: tk.sm, right: tk.sm, top: 0, bottom: 0 }, part: sel("Scoreboard", "header") }, {
            shape: { kind: "rect", radii: [3, 3, 3, 3] },
            fills: [vgrad([[0, lit(tc, 0.85)], [1, lit(tc, 0.35)]], -90)],
          }, () => {
            text("Name", team.name ?? "", { step: ts(14), font: "display", color: lit(inkOn(tc)), width: { kind: "grow" }, nowrap: true, ellipsis: true });
            for (const c of cols) text(keyId(c.key ?? c.label ?? ""), c.label ?? "", { step: ts(11), font: "mono", color: lit(inkOn(tc), 0.85), nowrap: true, ellipsis: true, width: px(colW(c)), align: "end" });
            text("Score", team.score ?? "", { step: rowH < u(30) ? ts(16) : ts(20, true), font: "display", color: lit(inkOn(tc)), nowrap: true, width: px(pingW), align: "end" });
          });
          (team.rows ?? []).forEach((row, ri) => {
            const lvl = pingLevel(row.ping ?? 0);
            box(keyId(row.key ?? row.name ?? String(ri)), {
              direction: "row", gap: tk.sm, cross: "center", width: pct(1), height: px(rowH), padding: { left: tk.sm, right: tk.sm, top: 0, bottom: 0 },
              part: sel("Scoreboard", "row", row.local ? "local" : null, row.dead ? "dead" : null),
            }, {
              shape: { kind: "rect", radii: [2, 2, 2, 2] },
              fills: [solid(row.local ? token("highlight", 0.22) : ri % 2 === 0 ? token("shadow", 0.35) : token("shadow", 0.2))],
              strokes: row.local ? [stroke(1, token("highlight", 0.8))] : [],
            }, () => {
              leaf("Team", { width: px(u(3)), height: pct(0.7) }, { fills: [solid(lit(tc))] });
              if (row.avatar) {
                kit.image("Avatar", { texture: row.avatar, width: px(rowH - u(6)), height: px(rowH - u(6)), label: "" });
              } else if (row.icon) icon("Icon", row.icon, rowH - u(10), token("ink"));
              text("Name", row.name ?? "", { step: ts(13), font: row.local ? "display" : "body", color: row.dead ? token("inkFaint") : token("ink"), width: { kind: "grow" }, nowrap: true, ellipsis: true });
              if (row.mvp) icon("Mvp", "star", u(14), lit(role("gold")), "MVP");
              if (row.dead) icon("Dead", "skull", u(14), token("inkFaint"), "Dead");
              (row.values ?? []).slice(0, cols.length).forEach((v, i) => {
                text(kit.idi("Value", i), v, { step: ts(13), font: "mono", color: row.dead ? token("inkMuted") : token("ink"), nowrap: true, ellipsis: true, width: px(colW(cols[i])), align: "end" });
              });
              if ((row.ping ?? 0) > 0) {
                const ink = lvl === "poor" ? role("danger") : lvl === "fair" ? role("warn") : role("success");
                const bars = lvl === "poor" ? 1 : lvl === "fair" ? 2 : 3;
                box("Ping", { direction: "row", gap: 1, cross: "end", width: px(pingW), align: "end", part: sel("Scoreboard", "ping", lvl) }, undefined, () => {
                  for (let b = 0; b < 3; b++) leaf(kit.idi("Bar", b), { width: px(u(3)), height: px(u(4 + b * 3)) }, { fills: [solid(b < bars ? lit(ink) : token("inkFaint", 0.4))] });
                });
              } else leaf("PingSpace", { width: px(pingW), height: px(1) });
            }, {
              opacity: row.dead ? 0.7 : 1,
              a11y: a11y("row", row.name ?? "", { value: (row.values ?? []).join(", "), selected: !!row.local }),
            });
          });
        }, { a11y: a11y("group", team.name ?? "") });
      });
    }, { a11y: a11y("table", desc.title || "Scoreboard") });
  }, { over: true });
}
