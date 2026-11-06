import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, clamp, eyebrow, fit, flags, frame, glow, grouped, grow, hex, kit, line, mix, paint, parentOf, px, rememberRoom, role, rtl, safeHeight,
  safeWidth, shadow, st, surface, symbol, text, token, u, vgrad, xy, type Mut,
} from "./core.ts";
import { chevron } from "../containers/base.ts";
import { chart } from "../data/charts.ts";
import { stepSize } from "../feedback/core.ts";
import { formatNumber } from "../data/format.ts";
import { valueSlider } from "../controls/ranges.ts";

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
      text("Name", (shown.name ?? shown.key ?? ""), { uppercase: true, step: st(phone ? 13 : 15), font: "display", heading: true, nowrap: true });
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
