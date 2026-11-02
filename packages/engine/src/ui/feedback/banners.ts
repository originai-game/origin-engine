// og.ui.banner and og.ui.ribbon: announcements and ribbons (docs/ui.md §11
// "banners and ribbons", Appendix E "Map and fog reveals, region title
// cards", "Celebrations"). Read against Elden Ring's place names (a thin
// rule drawing out under a serif title that fades up and away), Bloons TD
// 6 and Fortnite's round bands (a band across the screen, a big title that
// slams in), Hades' victory and defeat plates, the corner ribbons of store
// cards and Candy Crush's "New best!" stamp.

import type * as Api from "@origingame/engine";
import {
  a11y, absolute, box, clamp01, close, drop, easeBack, easeOut, frame, groups, kit, leaf, lit, msToTicks, now, open, pct, px,
  radial, reduced, role, sel, solid, sound, stroke, text, toneRole, token, vgrad, xy, inkOn, ts, mix, stepSize,
} from "./core.ts";

const HOLD_MS: Record<Api.BannerKind, number> = { title: 3000, wave: 2400, victory: 3500, defeat: 3500, warning: 3000 };
const ENTER_MS: Record<Api.BannerKind, number> = { title: 900, wave: 280, victory: 420, defeat: 600, warning: 300 };
const EXIT_MS = 520;
const SOUND: Record<Api.BannerKind, string> = { title: "open", wave: "confirm", victory: "success", defeat: "error", warning: "warning" };

export function banner(id: string, desc: Api.BannerDesc = {}): Api.BannerResult {
  const f = frame();
  const tk = f.t;
  const kind: Api.BannerKind = desc.kind ?? "wave";
  const t = now();
  const settled = desc.shownAt === undefined;
  const age = settled ? msToTicks(ENTER_MS[kind]) + 1 : t - (desc.shownAt ?? 0);
  const enter = msToTicks(ENTER_MS[kind]);
  const hold = msToTicks(desc.durationMs || HOLD_MS[kind]);
  const exit = msToTicks(EXIT_MS);
  const total = enter + hold + exit;
  if (age < 0 || (!settled && age >= total)) return { visible: false, done: !settled && age === total, progress: age < 0 ? 0 : 1 };
  sound(SOUND[kind], age === 0);
  const phase = age < enter ? "entering" : settled || age < enter + hold ? "holding" : "leaving";
  const x = clamp01(age / enter);
  const out = phase === "leaving" ? clamp01((age - enter - hold) / exit) : 0;
  const still = reduced();
  const danger = role("danger");
  const gold = role("gold");
  const ink: Api.Paint = kind === "victory" ? lit(gold) : kind === "defeat" ? lit(danger) : kind === "warning" ? token("ink") : token("ink");

  // Over the rest of the UI, a pinned tooltip included (the top layer, built after it): an announcement is never covered by a card or a meter.
  kit.anchor(id, { anchor: desc.anchor ?? "center", width: pct(1), direction: "column", cross: "center", over: true, tooltip: true }, () => {
    const band = kind !== "title";
    const bandInk = kind === "victory" ? gold : kind === "defeat" || kind === "warning" ? danger : role("primary");
    open("Band", {
      direction: "column",
      cross: "center",
      width: pct(1),
      padding: xy(tk.xl, kind === "title" ? tk.lg : tk.md),
      gap: kind === "title" ? tk.sm : 2,
      part: sel("Banner", "band", kind),
      partClass: "hud",
    }, {
      fills: band
        // A dark scrim under the band's own fill, so its title reads at
        // 4.5:1 or better over a bright scene as well as a dark one.
        ? [vgrad([[0, token("shadow", 0)], [0.14, token("shadow", 0.82)], [0.86, token("shadow", 0.82)], [1, token("shadow", 0)]], 90), vgrad([[0, token("hud", 0)], [0.18, token("hud", 0.9)], [0.82, token("hud", 0.9)], [1, token("hud", 0)]], 90), vgrad([[0, lit(bandInk, 0.0)], [0.5, lit(bandInk, kind === "warning" ? 0.24 : 0.14)], [1, lit(bandInk, 0)]], 90)]
        : [radial([[0, token("shadow", 0.72)], [0.7, token("shadow", 0.4)], [1, token("shadow", 0)]], 0.5, 0.5, 0.5, 0.5)],
      strokes: [],
      effects: [],
    });
    // The band's edges: thin lines in its colour that draw out from the centre.
    if (band) {
      const w = still ? 1 : easeOut(x * 1.4);
      for (const [lid, top] of [["EdgeTop", true], ["EdgeBottom", false]] as const) {
        leaf(lid, { width: pct(w), height: px(2), part: sel("Banner", "rule", kind) }, {
          fills: [vgrad([[0, lit(bandInk, 0)], [0.5, lit(bandInk)], [1, lit(bandInk, 0)]], 90)],
        }, absolute(top ? { top: 0, centerX: true } : { bottom: 0, centerX: true }));
      }
    }
    if (kind === "warning") {
      // Danger stripes sweeping across, pulsing.
      const off = still ? 0 : (kit.seconds() * 40) % 28;
      leaf("Stripes", { width: pct(1), height: pct(1), part: sel("Banner", "stripes") }, {
        fills: [{
          kind: "gradient", type: "linear", spread: "repeat", from: { x: 0, y: 0 }, to: { x: 0.03, y: 0.06 },
          stops: [{ offset: 0, color: lit(danger, 0.0) }, { offset: 0.5, color: lit(danger, 0.0) }, { offset: 0.5, color: lit(danger, 0.22) }, { offset: 1, color: lit(danger, 0.22) }],
        }],
      }, absolute({ start: 0, top: 0 }, { transform: { x: off - 28 }, opacity: 0.6 + 0.4 * Math.cos(kit.seconds() * 6.28 * 1.5) ** 2 }));
    }
    if (desc.image) {
      leaf("Art", { width: pct(1), height: pct(1), part: sel("Banner", "art") }, {
        fills: [{ kind: "image", texture: desc.image, fit: "cover", tint: { token: "ink", alpha: 0.35 } }],
      }, absolute({ start: 0, top: 0 }));
    }
    if (desc.eyebrow) text("Eyebrow", desc.eyebrow.toUpperCase(), { step: ts(14), color: kind === "title" ? token("ink") : lit(mix(bandInk, { r: 1, g: 1, b: 1, a: 1 }, 0.35)), nowrap: true, outline: 1.2 });
    // The title: a title card fades up; a wave or a plate slams in from large and settles.
    const slam = kind !== "title" && !still;
    const s = phase === "entering" && slam ? 1 + 0.9 * (1 - easeBack(x, 2.2)) : 1;
    const rise = kind === "title" && !still ? (1 - easeOut(x)) * 14 : 0;
    const shake = slam && age >= enter && age < enter + msToTicks(160) ? Math.sin(age * 2.4) * 4 * (1 - (age - enter) / msToTicks(160)) : 0;
    text("Title", desc.title ?? "", {
      step: kind === "title" ? "displayMd" : "displayLg",
      font: "display",
      nowrap: true,
      align: "center",
      color: ink,
      outline: 2,
      ...(kind === "victory" ? { gradient: { type: "linear", from: { x: 0, y: 0 }, to: { x: 0, y: 1 }, stops: [{ offset: 0, color: lit({ ...gold, r: Math.min(1, gold.r + 0.2), g: Math.min(1, gold.g + 0.2), b: Math.min(1, gold.b + 0.15) }) }, { offset: 1, color: lit(gold) }] } } : {}),
    }, { transform: { scaleX: s, scaleY: s, y: rise, x: shake }, opacity: kind === "title" ? easeOut(x) : clamp01(x * 3) });
    if (kind === "title") {
      leaf("Rule", { width: px(Math.max(40, 360 * (still ? 1 : easeOut(clamp01((age - enter * 0.3) / enter))))), height: px(1.5), part: sel("Banner", "rule", kind) }, {
        fills: [vgrad([[0, token("ink", 0)], [0.5, token("ink", 0.85)], [1, token("ink", 0)]], 90)],
      });
    }
    if (desc.subtitle) text("Subtitle", desc.subtitle, { step: ts(16), color: token("ink"), nowrap: true, align: "center", outline: 1.2 }, { opacity: clamp01((age - enter * 0.5) / enter) });
    if (kind === "victory" && age === enter) kit.particles({ burst: 48, fire: true, colors: [lit(gold), token("primary"), token("highlight")], travelMin: 80, travelMax: 260, gravity: 120, spin: 3, shape: "square", size: { x: 8, y: 5 }, from: "area" });
    else if (kind === "victory") kit.particles({ burst: 48, fire: false, colors: [lit(gold)], from: "area" });
    close({
      opacity: 1 - out,
      transform: { y: -18 * out, scaleY: kind === "title" || still ? 1 : phase === "entering" ? clamp01(x * 2.5) : 1 },
      a11y: a11y(kind === "warning" || kind === "defeat" ? "alert" : "status", [desc.eyebrow, desc.title, desc.subtitle].filter((v) => v).join(". ")),
      ...groups({ phase }, { phase: ["entering", "holding", "leaving", "gone"] }),
    });
  });
  return { visible: true, done: false, progress: settled ? 0.5 : clamp01(age / total) };
}

export function ribbon(id: string, desc: Api.UiRibbonDesc = {}): void {
  const tk = frame().t;
  const corner: Api.Anchor = desc.corner ?? "topRight";
  const tone: Api.Tone = desc.tone ?? "primary";
  const fill = role(toneRole(tone, "primary"));
  const dark = { r: fill.r * 0.62, g: fill.g * 0.62, b: fill.b * 0.62, a: 1 };
  const ink = lit(inkOn(fill));
  const parent = kit.openPath();
  const t = now();
  const at = desc.shownAt;
  const age = at === undefined || at === null ? 1e6 : t - at;
  const land = msToTicks(260);
  if (age < 0) return;
  const landing = age < land;
  sound("confirm", age === land);
  const name = desc.name || desc.text || "";
  const g = groups({ slap: age < 0 ? "waiting" : landing ? "landing" : "settled" }, { slap: ["waiting", "landing", "settled"] });
  if (corner === "topLeft" || corner === "topRight" || corner === "bottomLeft" || corner === "bottomRight") {
    // A diagonal band across the corner (the container clips it).
    const right = corner.endsWith("Right");
    const top = corner.startsWith("top");
    const rot = (right === top ? 1 : -1) * (Math.PI / 4);
    // The container clips the band to the chord across its corner: a longer
    // text (pseudo-locale, 200 %) moves the band out from the corner and
    // lengthens it until the chord holds the text.
    const label = (desc.text ?? "").toUpperCase();
    const st = stepSize(ts(13));
    const reach = Math.max(70, (label.length * st.size * 0.7 + 24) / Math.SQRT2);
    const bandW = Math.max(150, Math.ceil(reach * Math.SQRT2 + 40));
    const bandH = st.line + 6;
    box(id, { width: px(bandW), height: fitH(), align: "center", cross: "center", padding: xy(0, 3), part: sel("Ribbon", "band", corner, tone), partClass: "badge" }, {
      fills: [vgrad([[0, lit(fill)], [1, lit(dark)]])],
      strokes: [stroke(1, token("sheen", 0.4), "inside"), stroke(1, lit(dark), "outside", { dash: 3, gap: 2 })],
      effects: [drop(2, 4, token("shadow", 0.45))],
    }, () => text("Text", label, { step: ts(13), font: "display", color: ink, nowrap: true }), {
      position: "absolute",
      insets: { ...(top ? { top: Math.round(reach / 2 - bandH / 2) } : { bottom: Math.round(reach / 2 - bandH / 2) }), ...(right ? { end: Math.round(reach / 2 - bandW / 2) } : { start: Math.round(reach / 2 - bandW / 2) }) },
      transform: { rotate: rot },
      z: 4,
      overlaps: parent ? [parent] : [],
      a11y: a11y("status", name),
      ...g,
    });
    return;
  }
  if (corner === "center") {
    // A stamp slapped across the middle: lands from large and turned, squashes, settles.
    const x = clamp01(age / land);
    const s = landing && !reduced() ? 2.4 - 1.4 * easeOut(x) : 1;
    const squash = !landing && age < land + msToTicks(140) && !reduced() ? 1 - 0.08 * Math.sin(((age - land) / msToTicks(140)) * Math.PI) : 1;
    box(id, { padding: xy(tk.lg, tk.sm), align: "center", cross: "center", part: sel("Ribbon", "band", "stamp", tone), partClass: "badge" }, {
      shape: { kind: "rect", radii: [6, 6, 6, 6] },
      fills: [solid(lit(fill, 0.16))],
      strokes: [stroke(3, lit(fill), "inside"), stroke(1.5, lit(fill, 0.8), "outside")],
      effects: [drop(4, 10, token("shadow", 0.4))],
    }, () => {
      text("Text", (desc.text ?? "").toUpperCase(), { step: ts(22), font: "display", color: lit(fill), nowrap: true, outline: 1 });
      if (age === land) kit.particles({ burst: 18, fire: true, colors: [lit(fill, 0.8), token("inkMuted")], travelMin: 20, travelMax: 70, size: { x: 5, y: 5 }, shape: "circle", from: "edges" });
      else kit.particles({ burst: 18, fire: false, colors: [lit(fill)], from: "edges" });
    }, {
      position: "absolute",
      insets: { centerX: true, centerY: true },
      transform: { rotate: -0.18, scaleX: s * squash, scaleY: s / squash },
      opacity: landing ? clamp01(age / (land * 0.5)) : 1,
      z: 5,
      overlaps: parent ? [parent] : [],
      a11y: a11y("status", name),
      ...g,
    });
    return;
  }
  // A banner across the top or bottom, its ends folded behind it.
  box(id, { direction: "row", cross: "center", part: sel("Ribbon", "band", corner, tone), partClass: "badge" }, undefined, () => {
    leaf("TailStart", { width: px(18), height: px(24), part: sel("Ribbon", "fold") }, { shape: { kind: "rect", radii: [0, 0, 0, 0], corner: "bevel" }, fills: [solid(lit(dark))] }, { transform: { y: 6 } });
    box("Band", { padding: xy(tk.md, 3), cross: "center", part: sel("Ribbon", "band", tone) }, {
      fills: [vgrad([[0, lit(fill)], [1, lit({ r: fill.r * 0.85, g: fill.g * 0.85, b: fill.b * 0.85, a: 1 })]])],
      strokes: [stroke(1, token("sheen", 0.35), "inside")],
      effects: [drop(2, 4, token("shadow", 0.4))],
    }, () => text("Text", desc.text ?? "", { step: ts(14), font: "display", color: ink, nowrap: true }), { z: 1 });
    leaf("TailEnd", { width: px(18), height: px(24), part: sel("Ribbon", "fold") }, { fills: [solid(lit(dark))] }, { transform: { y: 6 } });
  }, {
    position: "absolute",
    insets: { centerX: true, ...(corner === "bottom" ? { bottom: -10 } : { top: -10 }) },
    z: 4,
    overlaps: parent ? [parent] : [],
    a11y: a11y("status", name),
    ...g,
  });
}

function fitH(): Api.Size {
  return { kind: "fit", min: 22 };
}
