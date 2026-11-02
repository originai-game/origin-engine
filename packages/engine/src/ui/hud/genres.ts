// og.ui.wavePreview and og.ui.laneKeys: the small genre rows of the genre
// sheets. B-td-build's wave preview (Kingdom Rush's and Bloons TD's column
// of waves to come) and B-rhythm-hud's lane keys under the hit line
// (osu!mania's and DJMAX's keycaps). The pack dresses them: nothing here
// draws a glow edge or a bevel of a particular look.

import type * as Api from "@origingame/engine";
import { BLACK, a11y, absolute, anchorOf, artTile, box, chip, edge, frame, hudPlace, hueOf, keyId, kit, lit, mix, msToTicks, paintRgba, px, role, sel, shapeOf, solid, stepSize, stroke, text, token, ts, u, type Rgba } from "./core.ts";

// ── wave preview ──────────────────────────────────────────────────────

export function wavePreview(id: string, desc: Api.WavePreviewDesc = {}): void {
  const tk = frame().t;
  const waves = desc.waves ?? [];
  const max = desc.max || 5;
  const shown = waves.length > max ? waves.slice(0, max - 1) : waves;
  const more = waves.length - shown.length;
  const side = u(44);
  const numStep = ts(15, true);
  const numW = Math.ceil(stepSize(numStep).size * 1.6);
  const r = Math.min(shapeOf("slot").radius, u(6));
  hudPlace(id, anchorOf(desc, "right"), (inAnchor) => {
    box(id, {
      direction: "column", gap: u(4), cross: "stretch",
      padding: inAnchor ? { left: tk.sm, right: tk.sm, top: tk.sm, bottom: tk.sm } : { left: 0, right: 0, top: 0, bottom: 0 },
      part: sel("WavePreview", "frame"), partClass: "hud",
    }, inAnchor ? chip() : undefined, () => {
      text("Title", desc.title ?? "Wave preview", { step: ts(11, true), font: "display", color: token("inkMuted"), nowrap: true });
      for (const w of shown) {
        const cur = (desc.current ?? 0) > 0 && w.number === desc.current;
        // A row: its number, then its enemy at the tile's size with how many come at the tile's foot.
        box(keyId(w.key ?? String(w.number ?? 0)), {
          direction: "row", gap: tk.sm, cross: "center", padding: { left: u(4), right: u(4), top: u(3), bottom: u(3) },
          part: sel("WavePreview", "wave", cur ? "current" : null, w.boss ? "boss" : null),
        }, {
          shape: { kind: "rect", radii: [r, r, r, r] },
          fills: [solid(token("well", cur ? 0.7 : 0.35))],
          // The selected-tile rule (docs/ui.md §15.5): one mark, a highlight line inside the edge.
          strokes: cur ? [stroke(2, token("highlight"), "inside")] : [],
        }, () => {
          text("Number", String(w.number ?? ""), { step: numStep, font: "display", color: token(cur ? "ink" : "inkMuted"), nowrap: true, align: "center", width: px(numW) });
          box("Tile", { width: px(side), height: px(side), part: sel("WavePreview", "art") }, {
            shape: { kind: "rect", radii: [r, r, r, r] },
            fills: [solid(lit(BLACK, 0.85))],
            strokes: [stroke(1, w.boss ? lit(role("danger")) : token("lineStrong", 0.4), w.boss ? "inside" : "outside")],
            clip: "shape",
          }, () => {
            artTile("Art", w.image, w.icon ?? "", side, hueOf(w.icon || w.name || ""), absolute({ start: 0, top: 0 }));
          });
          if ((w.count ?? 0) > 0) text("Count", `×${w.count}`, { step: ts(13), font: "mono", color: token("ink"), nowrap: true });
        }, {
          a11y: a11y("text", `${w.number ?? ""} ${w.name ?? ""}`.trim(), { value: (w.count ?? 0) > 0 ? `${w.count}` : "", selected: cur }),
        });
        if (w.name) kit.tooltip(w.name);
      }
      if (more > 0) {
        box("More", { align: "center", cross: "center", part: sel("WavePreview", "more") }, undefined, () => {
          text("Dots", "···", { step: numStep, font: "display", color: token("inkMuted"), nowrap: true });
        }, { a11y: a11y("text", `${more} more`) });
      }
    }, { a11y: a11y("list", desc.title ?? "Wave preview") });
  });
}

// ── lane keys ─────────────────────────────────────────────────────────

export function laneKeys(id: string, desc: Api.LaneKeysDesc = {}): void {
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
