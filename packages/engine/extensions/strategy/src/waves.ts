// The strategy kit's wave preview: B-td-build's column of waves to come
// (Kingdom Rush's and Bloons TD's). The pack dresses it: nothing here draws
// a glow edge or a bevel of a particular look. Moved from the engine's
// hud/genres.ts (docs/ui.md §17).

import { BLACK, a11y, absolute, anchorOf, artTile, box, chip, edge, frame, hudPlace, hueOf, keyId, kit, lit, px, role, sel, shapeOf, solid, stepSize, stroke, text, token, ts, u } from "@origingame/engine/ui-parts/hud";
import type { WavePreviewDesc } from "../types.ts";

export function wavePreview(id: string, desc: WavePreviewDesc = {}): void {
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
