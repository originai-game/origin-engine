import type { LaneKeysDesc } from "../types.ts";
// og.ui.laneKeys: B-rhythm-hud's lane keys under the hit line (osu!mania's
// and DJMAX's keycaps). The pack dresses them: nothing here draws a glow
// edge or a bevel of a particular look. The wave preview moved into the
// strategy kit (docs/ui.md §17).

import type * as Api from "@origingame/engine";
import { BLACK, a11y, absolute, anchorOf, artTile, box, chip, edge, frame, hudPlace, hueOf, keyId, kit, lit, mix, msToTicks, paintRgba, px, role, sel, shapeOf, solid, stepSize, stroke, text, token, ts, u, type Rgba } from "@origingame/engine/ui-parts/hud";

// ── wave preview ──────────────────────────────────────────────────────

// ── lane keys ─────────────────────────────────────────────────────────

export function laneKeys(id: string, desc: LaneKeysDesc = {}): void {
  const vp = frame().viewport;
  const band = vp.hud ?? { x: 0, y: 0, width: vp.width, height: vp.height };
  const lanes = desc.lanes ?? [];
  if (lanes.length === 0) return;
  const w = u(desc.width || 56);
  const h = Math.round(w * 0.72);
  const pressed = new Set(desc.pressed ?? []);
  const hit = new Set(desc.hit ?? []);
  const xs = lanes.map((l) => l.x ?? 0);
  const left = Math.min(...xs) - w / 2;
  const span = Math.max(...xs) + w / 2 - left;
  const r = Math.min(shapeOf("button").radius, u(8));
  // One group under the hit line spanning the lanes; each cap placed at its lane's x inside it.
  kit.anchor(`${id}Hud`, { anchor: "topLeft", margin: 0, offsetX: left - band.x, offsetY: (desc.y ?? 0) + u(6) - band.y, direction: "column" }, () => {
    box(id, { width: px(span), height: px(h + u(4)) }, undefined, () => {
      for (const l of lanes) {
        const k = l.key ?? "";
        const down = pressed.has(k);
        const c: Rgba = l.color ? paintRgba(l.color, role("primary")) : role("primary");
        // A hit flashes the cap in the lane's colour, fading over 220 ms.
        const e = edge(`${k}~hit`, hit.has(k) ? kit.tick() : kit.memory(`${id}~${k}~last`, -1));
        if (hit.has(k)) kit.remember(`${id}~${k}~last`, kit.tick());
        const flash = kit.memory(`${id}~${k}~last`, -1e9) >= 0 && e.since < msToTicks(220) ? 1 - e.since / msToTicks(220) : 0;
        box(keyId(k), {
          width: px(w), height: px(h), align: "center", cross: "center",
          part: sel("LaneKeys", "cap", down ? "pressed" : null, flash > 0 ? "hit" : null), partClass: "button",
        }, {
          shape: { kind: "rect", radii: [r, r, r, r] },
          fills: [solid(token("hud", 0.88)), ...(down ? [solid(lit(c, 0.32))] : []), ...(flash > 0 ? [solid(lit(mix(c, { r: 1, g: 1, b: 1, a: 1 }, 0.3), 0.55 * flash))] : [])],
          strokes: [stroke(down ? 2 : 1.5, lit(c, down ? 1 : 0.7), "inside")],
          effects: [{ kind: "dropShadow", offset: { x: 0, y: down ? 1 : 3 }, blur: down ? 3 : 6, spread: 0, color: token("shadow", 0.5) }],
        }, () => {
          const glyph = l.hotkey || (l.action ? kit.frameGlyph(l.action)?.text ?? "" : "");
          text("Glyph", glyph, { step: ts(20, true), font: "display", color: down ? lit(mix(c, { r: 1, g: 1, b: 1, a: 1 }, 0.55)) : token("ink"), nowrap: true });
        }, {
          ...absolute({ start: (l.x ?? 0) - w / 2 - left, top: down ? u(3) : 0 }),
          a11y: a11y("text", l.hotkey || l.action || k, { selected: down }),
        });
      }
    }, { a11y: a11y("group", "Lanes") });
  });
}
