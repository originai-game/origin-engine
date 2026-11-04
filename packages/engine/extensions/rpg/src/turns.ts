// The RPG kit's turn order: read against the turn orders of Final Fantasy X
// and Baldur's Gate 3 (portraits in the order they act, the current one
// larger). Moved from the engine's hud/scores.ts (docs/ui.md §17).

import type * as Api from "@origingame/engine";
import { a11y, absolute, anchorOf, box, clamp01, edge, fillDraw, frame, glow, hudPlace, icon, keyId, kit, leaf, lit, mix, now, pct, pulse, px, role, sel, shapeOf, solid, stroke, teamColor, text, token, trackDraw, ts, u, vgrad, type Rgba, apply, tsFit, stepSize } from "@origingame/engine/ui-parts/hud";
import type { TurnOrderDesc, TurnUnit } from "../types.ts";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

export function turnOrder(id: string, desc: TurnOrderDesc = {}): void {
  const tk = frame().t;
  const size = u(desc.size || 52);
  const units = (desc.units ?? []).slice(0, desc.max || 8);
  if (desc.style === "list") {
    turnList(id, desc, units);
    return;
  }
  const row = (desc.direction ?? "row") === "row";
  hudPlace(id, anchorOf(desc, "top"), () => {
    box(id, { direction: row ? "row" : "column", gap: tk.sm, cross: row ? "end" : "start", part: sel("TurnOrder", "frame") }, undefined, () => {
      units.forEach((un, i) => {
        const cur = !!un.current;
        const s = cur ? Math.round(size * 1.3) : size;
        const enemy = (un.team ?? 0) !== 0;
        const ink = enemy ? (un.team === 1 ? role("danger") : teamColor(un.team ?? 1)) : role("primary");
        // The order slides on layout: each unit keeps its key, and the frame animates its place.
        box(keyId(un.key ?? un.name ?? String(i)), {
          direction: "column", gap: u(3), cross: "center",
          part: sel("TurnOrder", "unit", cur ? "current" : null, ["team", un.team ?? 0], un.dead ? "dead" : null),
        }, undefined, () => {
          box("Portrait", { width: px(s), height: px(s), align: "center", cross: "center", part: sel("TurnOrder", "portrait") }, {
            // Enemies are notched at the corner, so a side is not told by colour alone.
            shape: { kind: "rect", radii: enemy ? [s * 0.3, 4, 4, 4] : [4, 4, 4, 4], corner: enemy ? "bevel" : "round" },
            fills: [vgrad([[0, lit(mix(ink, WHITE, 0.2), 0.9)], [1, lit(mix(ink, { r: 0, g: 0, b: 0, a: 1 }, 0.55))]])],
            strokes: [stroke(cur ? 3 : 1.5, cur ? lit(role("highlight")) : lit(ink)), stroke(1, token("shadow", 0.9), "outside")],
            effects: cur ? [glow(8 + 4 * pulse(1), lit(role("highlight"), 0.55))] : [],
            clip: "shape",
          }, () => {
            if (un.portrait) {
              kit.image("Art", { texture: un.portrait, width: px(s), height: px(s), label: "" });
              apply(absolute({ start: 0, top: 0 }));
            } else text("Initials", (un.name ?? "?").slice(0, 2).toUpperCase(), { step: tsFit(cur ? 20 : 15, s * 0.42, true), font: "display", color: token("ink"), nowrap: true, outline: 1 });
            if (un.dead) {
              leaf("Cross", { width: px(s), height: px(s) }, {
                shape: { kind: "path", points: [{ x: s * 0.2, y: s * 0.2 }, { x: s * 0.8, y: s * 0.8 }], width: 3, cap: "round" },
                fills: [solid(lit(role("danger")))],
              }, absolute({ start: 0, top: 0 }));
            }
            if (un.delayed) box("Delay", { width: px(u(14)), height: px(u(14)), part: sel("TurnOrder", "delay") }, undefined, () => icon("Mark", "arrow-down", u(14), lit(role("warn")), "Delayed"), absolute({ end: 1, top: 1 }));
          });
          if (un.atb !== null && un.atb !== undefined) {
            box("Atb", { width: px(s), height: px(u(4)), part: sel("TurnOrder", "atb") }, trackDraw(2), () => {
              leaf("Fill", { width: pct(clamp01(un.atb ?? 0)), height: pct(1) }, fillDraw((un.atb ?? 0) >= 1 ? role("gold") : role("info"), 2));
            });
          }
          if (cur) {
            box("Now", { padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, part: sel("TurnOrder", "now") }, { shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [solid(token("highlight"))] }, () => {
              text("Text", desc.nowLabel ?? "Now", { step: ts(11), font: "display", color: token("inkInverse"), nowrap: true });
            });
          }
        }, {
          opacity: un.dead ? 0.45 : 1,
          a11y: a11y("image", un.name ?? "", { value: cur ? desc.nowLabel ?? "Now" : `${i + 1}` }),
        });
      });
    }, { a11y: a11y("list", "Turn order") });
  });
}

/**
 * The turn order as a list (a tactics game's initiative): rows of portrait
 * and name read top to bottom with no numbers. The current unit's row takes
 * the selected row's rule (the list's controlStrong, lit by the highlight's
 * rim and glow, as the strip rings its current one); a stripe on the start
 * edge, a mark wide, carries an enemy's side and is empty for the player's,
 * so every portrait starts on one line. The row is a flow row, so it
 * mirrors right to left with the stripe.
 */
function turnList(id: string, desc: TurnOrderDesc, units: readonly TurnUnit[]): void {
  const tk = frame().t;
  const s = u(desc.size || 52);
  const pw = Math.round(s * 1.5);
  const stripe = u(3);
  const pad = tk.xs;
  const nameStep = ts(15);
  const rowH = Math.max(s, stepSize(nameStep).line + (units.some((un) => un.atb !== null && un.atb !== undefined) ? u(8) : 0)) + pad * 2;
  const r = Math.min(shapeOf("button").radius, rowH / 4);
  const hi = role("highlight");
  hudPlace(id, anchorOf(desc, "top"), () => {
    box(id, { direction: "column", gap: u(2), width: { kind: "grow" }, cross: "stretch", part: sel("TurnOrder", "frame", "list") }, undefined, () => {
      units.forEach((un, i) => {
        const cur = !!un.current;
        const team = un.team ?? 0;
        const enemy = team !== 0;
        const ink = enemy ? (team === 1 ? role("danger") : teamColor(team)) : role("primary");
        const said = [un.name ?? "", enemy ? desc.hostileLabel ?? "Hostile" : "", cur ? desc.nowLabel ?? "Now" : ""].filter(Boolean).join(", ");
        box(keyId(un.key ?? un.name ?? String(i)), {
          direction: "row", gap: tk.sm, cross: "center", width: { kind: "grow" }, height: px(rowH),
          padding: { left: 0, right: tk.sm, top: pad, bottom: pad },
          part: sel("TurnOrder", "row", cur ? "current" : null, ["team", team], un.dead ? "dead" : null),
        }, cur ? {
          shape: { kind: "rect", radii: [r, r, r, r] },
          fills: [solid(token("controlStrong"))],
          strokes: [stroke(2, lit(hi))],
          effects: [glow(8, lit(hi, 0.45))],
        } : undefined, () => {
          // The side's stripe: empty on the player's side, so the portraits line up.
          leaf("Stripe", { width: px(stripe), height: { kind: "grow" }, part: sel("TurnOrder", "stripe", ["team", team]) },
            enemy ? { shape: { kind: "rect", radii: [stripe / 2, stripe / 2, stripe / 2, stripe / 2] }, fills: [solid(lit(ink))] } : undefined);
          box("Portrait", { width: px(pw), height: px(s), align: "center", cross: "center", part: sel("TurnOrder", "portrait") }, {
            shape: { kind: "rect", radii: [4, 4, 4, 4] },
            fills: [solid(token("well")), ...(un.portrait ? [{ kind: "image", texture: un.portrait, fit: "contain" } as Api.Fill] : [])],
            strokes: [stroke(1, token("line"))],
            clip: "shape",
          }, () => {
            if (!un.portrait) text("Initials", (un.name ?? "?").slice(0, 2).toUpperCase(), { step: tsFit(15, s * 0.42, true), font: "display", color: token("ink"), nowrap: true });
            if (un.dead) {
              leaf("Cross", { width: px(pw), height: px(s) }, {
                shape: { kind: "path", points: [{ x: pw * 0.3, y: s * 0.2 }, { x: pw * 0.7, y: s * 0.8 }], width: 3, cap: "round" },
                fills: [solid(lit(role("danger")))],
              }, absolute({ start: 0, top: 0 }));
            }
          });
          box("Text", { direction: "column", gap: u(3), width: { kind: "grow" } }, undefined, () => {
            text("Name", un.name ?? "", { step: nameStep, font: "display", color: token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
            if (un.atb !== null && un.atb !== undefined) {
              box("Atb", { width: { kind: "grow" }, height: px(u(4)), part: sel("TurnOrder", "atb") }, trackDraw(2), () => {
                leaf("Fill", { width: pct(clamp01(un.atb ?? 0)), height: pct(1) }, fillDraw((un.atb ?? 0) >= 1 ? role("gold") : role("info"), 2));
              });
            }
          });
          if (un.delayed) box("Delay", { width: px(u(14)), height: px(u(14)), part: sel("TurnOrder", "delay") }, undefined, () => icon("Mark", "arrow-down", u(14), lit(role("warn")), "Delayed"));
        }, {
          opacity: un.dead ? 0.45 : 1,
          a11y: a11y("listitem", said, { value: `${i + 1}`, selected: cur }),
        });
      });
    }, { a11y: a11y("list", "Turn order") });
  });
}
