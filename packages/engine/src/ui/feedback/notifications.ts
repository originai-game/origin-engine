// og.ui.notifications: the toast and notification stack (docs/ui.md §11,
// Appendix E "Toast and notification queues"). Read against Xbox's
// achievement toast (a medal well that pops, then the text), Steam's
// corner popups, Destiny 2's reward toasts and Fortnite's stacked item
// notices: cards anchored at a corner of the safe area, the newest
// nearest the edge, a draining time line, held by the pointer, thrown off
// by a swipe. The viewmodel is the game's: it appends items and drops the
// keys the stack reports, so the queue is world state.

import type * as Api from "@origingame/engine";
import {
  a11y, absolute, box, bp, clamp01, close, drop, frame, groups, glow, hudBandInset, icon, inner, keyId, kit, leaf, lit, msToTicks, now, open,
  pct, pop, px, radial, rarityColor, role, sel, shapeOf, solid, sound, stroke, text, toneRole, token, vgrad, xy, ts,
} from "./core.ts";

const VARIANT_MS: Record<Api.NotificationVariant, number> = { standard: 4000, achievement: 6000, reward: 5000, alert: 8000, social: 5000 };
const VARIANT_SOUND: Record<Api.NotificationVariant, string> = { standard: "notification", achievement: "reward", reward: "reward", alert: "warning", social: "notification" };
const TONE_MARK: Record<string, string> = { success: "check", warn: "warning", danger: "warning", info: "info", primary: "star", inverse: "bell", neutral: "bell" };

type Slide = "slideLeft" | "slideRight" | "slideUp" | "slideDown";

/** Which way cards enter for an anchor: from the edge they sit against. */
function slideFor(anchor: Api.Anchor): Slide {
  if (anchor.endsWith("Right") || anchor === "right") return "slideLeft";
  if (anchor.endsWith("Left") || anchor === "left") return "slideRight";
  return anchor.startsWith("bottom") ? "slideUp" : "slideDown";
}

export function notifications(id: string, desc: Api.NotificationsDesc = {}): Api.NotificationsResult {
  const f = frame();
  const b = bp();
  const { t: tk } = f;
  const out: { expired: string[]; dismissed: string[]; action: string | null; shown: string[] } = { expired: [], dismissed: [], action: null, shown: [] };
  const items = desc.items ?? [];
  const asked = desc.anchor ?? "topRight";
  const anchor: Api.Anchor = b.portrait && asked === "topRight" ? "top" : asked;
  const max = desc.max || (b.phone ? 2 : 3);
  const safe = f.viewport.safeArea;
  const width = desc.width || (b.portrait ? Math.max(240, f.viewport.width - safe.left - safe.right - tk.gutter * 2) : Math.min(380, f.viewport.width * 0.42));
  const slide = slideFor(anchor);
  const atBottom = anchor.startsWith("bottom");
  const t = now();
  const cross: Api.Cross = anchor.endsWith("Right") || anchor === "right" ? "end" : anchor.endsWith("Left") || anchor === "left" ? "start" : "center";

  // Inside the HUD's band on an ultrawide, as every HUD corner is.
  const band = hudBandInset();
  kit.anchor(id, { anchor, direction: "column", gap: tk.sm, cross, over: true, width: px(width), offsetX: cross === "start" ? band : cross === "end" ? -band : 0 }, () => {
    // A key posted again (a higher count, a later tick) is one card: the
    // last post wins. A post is done once it expired or was dismissed,
    // kept in the UI's memory while the game still lists it, so it never
    // shows twice; the oldest live posts show first, at most max.
    const byKey = new Map<string, Api.NotificationItem>();
    for (const it of items) if (it.key) byKey.set(it.key, it);
    const live: Array<{ it: Api.NotificationItem; mem: string }> = [];
    for (const it of byKey.values()) {
      const mem = `${it.key}@${it.at ?? 0}`;
      if (kit.memory(`done:${mem}`, 0) === 0) live.push({ it, mem });
    }
    const visible = live.slice(0, max);
    for (const { it, mem } of atBottom ? visible : [...visible].reverse()) {
      const key = it.key ?? "";
      const variant: Api.NotificationVariant = it.variant ?? "standard";
      const durTicks = msToTicks(it.durationMs || VARIANT_MS[variant]);
      // Its time runs from when it first shows; the pointer resting on it holds it.
      const start = kit.memory(`at:${mem}`, t);
      const age = t - start;
      if (!it.sticky && age >= durTicks) {
        out.expired.push(key);
        kit.remember(`done:${mem}`, 1);
        continue;
      }
      out.shown.push(key);
      if (age === 0) sound(VARIANT_SOUND[variant], true);
      const dismissedBefore = out.dismissed.length;
      const r = card(key, it, variant, age, it.sticky ? 1 : 1 - age / durTicks, slide, out);
      if (out.dismissed.length > dismissedBefore) kit.remember(`done:${mem}`, 1);
      else if (r.hovered && !it.sticky) kit.remember(`at:${mem}`, start + 1);
    }
  });
  if (atBottom) out.shown.reverse();
  return out;
}

function card(
  key: string,
  it: Api.NotificationItem,
  variant: Api.NotificationVariant,
  age: number,
  life: number,
  slide: Slide,
  out: { dismissed: string[]; action: string | null },
): Api.Response {
  const f = frame();
  const { t: tk } = f;
  const b = bp();
  const tone: Api.Tone = it.tone ?? (variant === "alert" ? "danger" : variant === "achievement" ? "primary" : "neutral");
  const accentRole = variant === "achievement" ? "gold" : toneRole(tone, "primary");
  const accent = role(accentRole);
  const rare = rarityColor(it.rarity);
  const shape = shapeOf("card");
  const radius = Math.min(shape.radius, 14);
  const id = keyId(key);
  const name = [it.eyebrow, it.title, it.body].filter((s) => s).join(". ");
  const well = b.phone ? 40 : 48;

  const path = open(id, {
    direction: "row",
    width: pct(1),
    padding: { left: tk.md + 4, top: tk.md, right: tk.md, bottom: tk.md + 3 },
    gap: tk.md,
    cross: "center",
    interactive: true,
    part: sel("Notification", "card", variant, tone, it.rarity && it.rarity !== "none" ? ["rarity", it.rarity] : null),
    partClass: "card",
  }, {
    shape: { kind: "rect", radii: [radius, radius, radius, radius], corner: shape.corner },
    fills: [vgrad([[0, token("panelRaised", 0.97)], [1, token("panel", 0.97)]])],
    strokes: [stroke(1, variant === "achievement" ? lit(accent, 0.55) : token("line"), "inside")],
    effects: [drop(10, 28, token("shadow", 0.5)), drop(2, 6, token("shadow", 0.3)), inner(1, 0, token("sheen", 0.35))],
    clip: "shape",
  });
  // The accent edge on the start side, in the tone.
  leaf("Accent", { width: px(4), height: pct(1) }, {
    fills: [vgrad([[0, lit(accent)], [1, lit(accent, 0.75)]])],
  }, absolute({ start: 0, top: 0, bottom: 0 }));

  // The icon well: an art slot, the variant's mark, or the tone's.
  const wellPop = variant === "achievement" ? pop(age, 0.28, 520) : 1;
  const wellInk = variant === "achievement" ? role("gold") : rare ?? accent;
  box("Well", {
    width: px(well),
    height: px(well),
    align: "center",
    cross: "center",
    part: sel("Notification", "well", variant, it.rarity && it.rarity !== "none" ? ["rarity", it.rarity] : null),
    partClass: variant === "social" ? "avatar" : "slot",
  }, {
    shape: variant === "social" ? { kind: "ellipse" } : { kind: "rect", radii: [radius - 4, radius - 4, radius - 4, radius - 4], corner: shape.corner },
    fills: [radial([[0, lit(wellInk, 0.42)], [1, lit(wellInk, 0.14)]], 0.5, 0.35, 0.75, 0.75)],
    strokes: [stroke(1.5, lit(wellInk, 0.8))],
    effects: rare || variant === "achievement" ? [glow(12, lit(wellInk, 0.55))] : [],
    clip: "shape",
  }, () => {
    if (it.image) kit.image("Art", { texture: it.image, width: px(well), height: px(well), label: "" });
    else icon("Icon", it.icon || (variant === "achievement" ? "trophy" : variant === "reward" ? "gift" : variant === "social" ? "user" : TONE_MARK[tone] ?? "bell"), Math.round(well * 0.55), lit(variant === "achievement" ? role("gold") : wellInk));
  }, { transform: { scaleX: wellPop, scaleY: wellPop } });

  // The words.
  kit.column("Text", { width: { kind: "grow" }, gap: 2 }, () => {
    if (it.eyebrow) text("Eyebrow", it.eyebrow.toUpperCase(), { step: ts(13), color: lit(variant === "achievement" ? role("gold") : accent), nowrap: true, ellipsis: true, width: { kind: "grow" } });
    kit.row("Head", { gap: tk.xs, cross: "center", width: { kind: "grow" } }, () => {
      text("Title", it.title ?? "", { step: ts(16), font: "display", nowrap: true, ellipsis: true, width: { kind: "grow" } });
      if ((it.count ?? 0) > 1) {
        box("Count", { padding: xy(tk.xs + 2, 1), part: sel("Notification", "count"), partClass: "badge" }, {
          shape: { kind: "rect", radii: [99, 99, 99, 99] },
          fills: [solid(lit(accent, 0.22))],
          strokes: [stroke(1, lit(accent, 0.6))],
        }, () => text("N", `×${it.count}`, { step: ts(13), font: "mono", nowrap: true, color: lit(accent) }));
      }
    });
    if (it.body) text("Body", it.body, { step: ts(14), color: token("inkMuted"), maxLines: 2, ellipsis: true, width: { kind: "grow" } });
    if (it.progress !== null && it.progress !== undefined) {
      kit.row("Progress", { gap: tk.sm, cross: "center", width: { kind: "grow" }, padding: { left: 0, top: 4, right: 0, bottom: 0 } }, () => {
        box("Track", { width: { kind: "grow" }, height: px(6), part: sel("Notification", "progress"), partClass: "bar" }, {
          shape: { kind: "rect", radii: [3, 3, 3, 3] },
          fills: [solid(token("controlStrong"))],
          clip: "shape",
        }, () => {
          leaf("Fill", { width: pct(clamp01(it.progress ?? 0)), height: pct(1), part: sel("Notification", "progressFill"), partClass: "bar" }, {
            shape: { kind: "rect", radii: [3, 3, 3, 3] },
            fills: [vgrad([[0, lit(accent)], [1, lit(accent, 0.8)]], 90)],
          });
        });
        if (it.progressText) text("ProgressText", it.progressText, { step: ts(13), font: "mono", color: token("inkMuted"), nowrap: true });
      });
    }
  });
  if (it.points) text("Points", it.points, { step: ts(16), font: "mono", color: lit(variant === "achievement" ? role("gold") : accent), nowrap: true });
  if (it.action) {
    const a = kit.button("Action", it.action, { variant: "primary", size: "small" }) as Api.Response;
    if (a.clicked) out.action = key;
  }
  // The time line drains along the bottom edge.
  if (!it.sticky) {
    leaf("Life", { width: pct(clamp01(life)), height: px(3), part: sel("Notification", "life"), partClass: "bar" }, {
      fills: [solid(lit(accent, 0.85))],
    }, absolute({ start: 0, bottom: 0 }));
  }
  const entering = age < msToTicks(240);
  const r = close({
    enter: { kind: slide, transition: { kind: "token", token: "enter" } },
    exit: { kind: slide, transition: { kind: "token", token: "exit" } },
    layoutMotion: { kind: "token", token: "panel" },
    drag: { axis: slide === "slideLeft" || slide === "slideRight" ? "x" : "y", returnHome: true, dismiss: {}, threshold: 8 },
    a11y: a11y(variant === "alert" ? "alert" : "status", name),
    ...groups({ life: entering ? "entering" : life < 0.15 ? "leaving" : "shown", tone }, { life: ["entering", "shown", "leaving"], tone: ["neutral", "success", "warn", "danger", "info", "primary", "inverse"] }),
  });
  if (kit.gesture(path).dismissed || (r.clicked && out.action !== key)) out.dismissed.push(key);
  return r;
}
