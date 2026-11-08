// og.ui.banner: announcements (docs/ui.md §2 "banners and ribbons"; the
// ribbon is a component document in src/ui/components), Appendix E "Map and fog reveals, region title
// cards", "Celebrations"). Read against Elden Ring's place names (a thin
// rule drawing out under a serif title that fades up and away), Bloons TD
// 6 and Fortnite's round bands (a band across the screen, a big title that
// slams in), Hades' victory and defeat plates, the corner ribbons of store
// cards and Candy Crush's "New best!" stamp.

import type * as Api from "@origingame/engine";
import {
  a11y, absolute, clamp01, close, easeBack, easeOut, frame, groups, kit, leaf, lit, msToTicks, now, open, pct, px, radial, reduced, role, sel, sound, text, token, vgrad, xy, ts, mix, claimAnnouncement,
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
  // Open tooltip cards step under the announcement and dim while it shows, so no text reads through it.
  claimAnnouncement();
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
    if (desc.eyebrow) text("Eyebrow", desc.eyebrow, { uppercase: true, step: ts(14), color: kind === "title" ? token("ink") : lit(mix(bandInk, { r: 1, g: 1, b: 1, a: 1 }, 0.35)), nowrap: true, outline: 1.2 });
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

function fitH(): Api.Size {
  return { kind: "fit", min: 22 };
}
