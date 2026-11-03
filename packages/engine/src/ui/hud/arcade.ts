// The racing HUD's arcade look (the owner's decision of 2026-11-03: an
// option on the racing components, shipped with cartoon and kawaii, not a
// separate kit): og.ui.speedometer's arcade style, og.ui.itemSlot and
// og.ui.driftScore. Read against Mario Kart's item roulette and Need for
// Speed's and Forza Horizon's drift chains and nitro tubes. Every motion
// runs by the tick from its event's tick, so a still is deterministic.

import type * as Api from "@origingame/engine";
import {
  a11y, absolute, box, clamp01, easeOut, edge, ellipseShape, fixed, frame, glow, groups, hudPlace, anchorOf, kit, leaf, lit, msToTicks, now,
  pop, pulse, px, reduced, role, sel, solid, stroke, text, token, ts, tsFit, u, vgrad, bp, stepSize, apply, ticksToMs,
} from "./core.ts";

const TUBE_START = Math.PI * 1.1;
const TUBE_SWEEP = Math.PI * 0.8;

/** The arcade speedometer: a big bouncy speed under a fat curved nitro tube cut into whole segments, flaring when full. */
export function arcadeSpeedo(id: string, speed: number, unit: string, desc: Api.SpeedometerDesc, size: number): void {
  const n = Math.max(1, Math.min(8, desc.boostSegments || 3));
  const boost = desc.boost === null || desc.boost === undefined ? null : clamp01(desc.boost);
  const full = boost !== null && boost >= 1 - 1e-6;
  const h = Math.round(size * 0.72);
  const c = size / 2;
  const thick = Math.max(8, Math.round(size * 0.13));
  // The speed pops each time it crosses ten.
  const e = edge(`${id}~ten`, Math.floor(speed / 10));
  const k = pop(e.since, 0.08, 260);
  box(id, { width: px(size), height: px(h), part: sel("Speedometer", "dial", "arcade") }, undefined, (path) => {
    // The tube curves over the readout by design.
    const readout = `${path}/Readout`;
    if (boost !== null) {
      const gap = 0.05;
      const seg = (TUBE_SWEEP - gap * (n - 1)) / n;
      const flare = full && !reduced() ? 0.6 + 0.4 * pulse(3) : 1;
      for (let i = 0; i < n; i++) {
        const start = TUBE_START + i * (seg + gap);
        const filled = boost * n >= i + 1 - 1e-6;
        const charging = !filled && boost * n > i;
        leaf(kit.idi("Tube", i), { width: px(size), height: px(size), part: sel("Speedometer", "boost", "track") }, {
          shape: { kind: "arc", start, sweep: seg, thickness: thick, cap: "round" },
          fills: [solid(token("shadow", 0.7))],
          strokes: [stroke(2, token("lineStrong", 0.7), "outside")],
        }, fixed({ start: 0, top: thick / 2 }, { overlaps: [readout] }));
        if (filled || charging) {
          leaf(kit.idi("Nitro", i), { width: px(size - 4), height: px(size - 4), part: sel("Speedometer", "boost", filled ? "full" : null) }, {
            shape: { kind: "arc", start, sweep: filled ? seg : seg * (boost * n - i), thickness: thick - 4, cap: "round" },
            fills: [solid(filled ? lit(role("info"), flare) : lit(role("info"), 0.4))],
            effects: filled && full ? [glow(thick, lit(role("info"), 0.7 * flare))] : [],
          }, fixed({ start: 2, top: thick / 2 + 2 }, { overlaps: [readout] }));
        }
      }
    }
    const step = tsFit(56, size * 0.34, true);
    const sp = stepSize(step);
    box("Readout", { direction: "column", cross: "center", gap: 0, width: px(size) }, undefined, () => {
      text("Speed", String(Math.round(speed)), { step, font: "display", color: token("ink"), nowrap: true, outline: 2.4 },
        { transform: { scaleX: k, scaleY: k, skewX: -0.22, pivotY: 1 } });
      kit.row("Meta", { gap: u(6), cross: "center" }, () => {
        text("Unit", unit, { step: ts(11), font: "display", color: token("inkMuted"), nowrap: true, outline: 1.2 });
        if (desc.gear) text("Gear", desc.gear, { step: ts(14, true), font: "display", color: token("highlight"), nowrap: true, outline: 1.2 });
      });
    }, fixed({ start: 0, top: Math.round(c * 0.62 - sp.line * 0.15) }));
  }, {
    a11y: a11y("meter", "Speed", { value: `${Math.round(speed)} ${unit}${boost !== null ? `, nitro ${Math.round(boost * 100)} %` : ""}`, valueMin: 0, valueMax: desc.maxSpeed ?? 300, valueNow: speed }),
    ...groups({ nitro: full ? "full" : "charging" }, { nitro: ["charging", "full"] }),
  });
}

export function itemSlot(id: string, desc: Api.ItemSlotDesc = {}): void {
  const size = Math.round(u(desc.size || (bp().phone ? 64 : 84)));
  const t = now();
  const rollT = msToTicks(desc.rollMs ?? 1500);
  const since = desc.rollingAt ? t - desc.rollingAt : -1;
  const rolling = since >= 0 && since < rollT && !reduced();
  const pool = desc.roulette && desc.roulette.length > 0 ? desc.roulette : desc.item ? [desc.item] : [];
  // Slowing: the roulette's place eases out over the spin, three times round its items.
  let shown: Api.Texture | null = desc.item ?? null;
  let uv = desc.uv ?? null;
  if (rolling && pool.length > 0) {
    shown = pool[Math.floor(easeOut(since / rollT) * pool.length * 3) % pool.length]!;
    uv = null;
  }
  const k = since >= rollT ? pop(since - rollT, 0.22, 360) : 1;
  const ring = Math.max(3, Math.round(size * 0.07));
  const slot = (sid: string, side: number, tex: Api.Texture | null, tuv: readonly number[] | null, part: string, scale = 1) =>
    box(sid, { width: px(side), height: px(side), align: "center", cross: "center", part }, {
      shape: ellipseShape(),
      fills: [vgrad([[0, token("hud", 0.75)], [1, token("hud", 0.95)]])],
      strokes: [stroke(ring, token("lineStrong")), stroke(1, token("shadow", 0.8), "outside")],
      effects: [{ kind: "dropShadow", offset: { x: 0, y: 2 }, blur: 8, spread: 0, color: token("shadow", 0.5) }],
    }, () => {
      if (tex) {
        const s = Math.round(side * 0.66);
        kit.image("Item", { texture: tex, ...(tuv ? { uv: [tuv[0]!, tuv[1]!, tuv[2]!, tuv[3]!] } : {}), width: px(s), height: px(s), label: "" });
        if (scale !== 1) apply({ transform: { scaleX: scale, scaleY: scale } });
      }
    });
  hudPlace(id, anchorOf(desc, "topLeft"), () => {
    box(id, { direction: "row", gap: u(6), cross: "start", part: sel("ItemSlot", "frame", !shown ? "empty" : rolling ? "rolling" : null) }, undefined, () => {
      slot("Ring", size, shown, uv, sel("ItemSlot", "item"), k);
      if (desc.queued) slot("Queued", Math.round(size * 0.55), desc.queued, desc.queuedUv ?? null, sel("ItemSlot", "queued"));
    }, { a11y: a11y("image", rolling ? "Rolling" : desc.label || (shown ? "Item" : "No item")) });
  });
}

export function driftScore(id: string, desc: Api.DriftScoreDesc = {}): void {
  const f = frame();
  const t = now();
  const win = msToTicks(800);
  const banked = (desc.bankedAt ?? 0) > 0 && t - desc.bankedAt! >= 0 && t - desc.bankedAt! < win;
  const failed = (desc.failedAt ?? 0) > 0 && t - desc.failedAt! >= 0 && t - desc.failedAt! < win;
  const score = Math.max(0, desc.score ?? 0);
  const mult = desc.multiplier ?? 1;
  // A banked or broken chain is over once its 0.8 s are; a new chain sets both ticks back to 0.
  const over = (desc.bankedAt ?? 0) > 0 || (desc.failedAt ?? 0) > 0;
  const chain = failed ? "failed" : banked ? "banked" : score > 0 && !over ? "running" : "idle";
  if (chain === "idle") return;
  const em = edge(`${id}~mult`, Math.round(mult * 10));
  const mk = pop(em.since, 0.3, 300);
  const age = failed ? t - desc.failedAt! : banked ? t - desc.bankedAt! : 0;
  const x = clamp01(ticksToMs(age) / 800);
  const still = reduced();
  const fade = chain === "running" ? 1 : 1 - (still ? 0 : easeOut(x));
  const shake = failed && !still ? Math.sin(age * 1.9) * u(6) * (1 - x) : 0;
  const ink = failed ? lit(role("danger")) : token("highlight");
  const build = () => {
    box(id, { direction: "column", cross: "center", gap: 0, part: sel("DriftScore", "score", chain === "running" ? null : chain) }, undefined, () => {
      text("Label", desc.label ?? "Drift", { step: ts(12, true), font: "display", color: token("inkMuted"), nowrap: true, outline: 1.2 });
      // The score's slant leans its top toward the badge: the row keeps that much room.
      kit.row("Chain", { gap: u(6) + Math.ceil(stepSize(ts(40, true)).line * 0.22), cross: "center" }, () => {
        text("Score", Math.round(score).toLocaleString("en-US"), { step: ts(40, true), font: "display", color: ink, nowrap: true, outline: 2.4 },
          { transform: { skewX: -0.22, pivotY: 1, x: shake, y: banked && !still ? -u(16) * easeOut(x) : 0 }, opacity: fade });
        if (mult > 1) {
          box("Multiplier", { padding: { left: u(6), right: u(6), top: u(1), bottom: u(2) }, part: sel("DriftScore", "multiplier") }, {
            shape: { kind: "rect", radii: [u(6), u(6), u(6), u(6)] },
            fills: [solid(failed ? lit(role("danger")) : token("primary"))],
            strokes: [stroke(1.5, token("shadow", 0.8), "outside")],
          }, () => text("Value", `×${mult % 1 === 0 ? mult : mult.toFixed(1)}`, { step: ts(18, true), font: "display", color: token("onPrimary"), nowrap: true }),
          { transform: { scaleX: mk, scaleY: mk }, opacity: fade });
        }
      });
      if (banked && desc.total !== undefined) {
        const shownTotal = Math.round(desc.total - score + score * (still ? 1 : easeOut(x)));
        text("Total", shownTotal.toLocaleString("en-US"), { step: ts(18, true), font: "display", color: token("ink"), nowrap: true, outline: 1.6 });
      }
    }, {
      a11y: a11y("status", desc.label ?? "Drift", { value: `${Math.round(score)}${mult > 1 ? ` ×${mult}` : ""}` }),
      ...groups({ chain }, { chain: ["idle", "running", "banked", "failed"] }),
      inert: true,
    });
  };
  if (desc.at) {
    // Over a screen point: centred on it by last tick's width, kept on screen.
    const vp = f.viewport;
    const w = kit.bounds(`${id}Hud/${id}`)?.width ?? 0;
    const h = kit.bounds(`${id}Hud/${id}`)?.height ?? 0;
    const left = Math.max(vp.safeArea.left, Math.min(vp.width - vp.safeArea.right - w, desc.at.x - w / 2));
    const top = Math.max(vp.safeArea.top, Math.min(vp.height - vp.safeArea.bottom - h, desc.at.y - h));
    kit.anchor(`${id}Hud`, { anchor: "topLeft", margin: 0, offsetX: left - vp.safeArea.left, offsetY: top - vp.safeArea.top, over: true }, build);
  } else hudPlace(id, anchorOf(desc, "top"), build);
}
