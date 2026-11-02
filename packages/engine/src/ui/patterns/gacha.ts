// Gacha (docs/ui.md §11 genre patterns, Appendix B: gacha banner and pull
// results; pack opening): the banner the player pulls on, and the pull's
// reveal. Where a pull is in its reveal is the component's own state
// (og.ui.memory, so a snapshot carries it); the game only starts one (a new
// token) and hears what the player pressed.

import type * as Api from "@origingame/engine";
import {
  accent, artSlot, btn, clamp, fit, flags, floating, frame, glow, grow, hex, keyHash, kit, line, mix, over, paint, priceTag, px, radial, rarityColor, rarityPips,
  role, rtl, scrimFill, shadow, spring, st, still, surface, symbol, text, timerChip, token, u, vgrad, vstate, xy, type Mut, type Rgba,
} from "./core.ts";
import { cornerTag, plainText } from "./rewards.ts";

const fill = (template: string, v: Record<string, string | number>): string => template.replace(/\{(\w+)\}/g, (m, k: string) => (k in v ? `${v[k]}` : m));

/** A row of `n` stars, `size` square: lit in `color`, drawn as shapes (a font may have no ★). `lit` stars of them shine; the rest are outlines. */
export function stars(id: string, n: number, size: number, color: Rgba, lit = n, delays: number[] = []): void {
  kit.open(id, { direction: "row", gap: Math.max(1, Math.round(size * 0.12)), cross: "center", part: "Gacha.stars", partClass: "badge" });
  for (let k = 0; k < n; k++) {
    const on = k < lit;
    symbol(kit.idi("Star", k), on ? "star" : "starOutline", size, on ? color : mix(color, role("inkFaint"), 0.6));
    kit.props({
      states: [{ name: "lit", states: [vstate("off"), vstate("on")], transitions: [] }],
      state: [{ group: "lit", state: on ? "on" : "off" }],
      ...(delays[k] !== undefined ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "scale", value: 2.2 }, { prop: "rotate", value: -0.6 }], transition: spring("emphasis"), delayMs: delays[k]! } } : {}),
    });
  }
  kit.close();
  kit.semantics({ role: "image", name: `${n} stars` });
}

/** The colour a pull's tier is felt in: the style's rarity colour, else the primary. */
const tierInk = (r: Api.Rarity | undefined): Rgba => rarityColor(r) ?? role("primary");

// ── the banner ──────────────────────────────────────────────────────────

/**
 * A gacha banner (Genshin Impact, Honkai: Star Rail, FGO): see
 * `fn gachaBanner` in crates/og-api/api/ui.ogt.
 */
export function gachaBanner(id: string, desc: Api.GachaBannerDesc, art: () => void): Api.GachaBannerResult {
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  // A short frame (a phone on its side): the featured units drop their second line, the rail narrows.
  const short = vp.height - vp.safeArea.top - vp.safeArea.bottom < 480;
  const touch = frame().touch;
  // Large text (150 % and over) on a phone: rows that held two things hold one.
  const big = phone && (frame().theme.textScale ?? 1) >= 1.5;
  const out: Mut<Api.GachaBannerResult> = { pull: 0, short: 0, banner: null, details: false, history: false };
  const tabs = desc.banners ?? [];
  const balance = desc.balance ?? 0;
  const multiN = Math.max(1, desc.multiCount ?? 10);
  const currency = desc.currency ?? "gem";

  kit.open(id, { direction: "column", width: grow(), height: grow(), part: "Gacha.frame", partClass: "panel", radius: t.panel });
  // The key art behind everything, and a scrim from the text's edge so the text reads on any art.
  // The key art: the game's texture, else the style's placeholder (a figure standing at the focal point on two hues);
  // the game's builder draws over it. The featured unit stands on the art side, the words on the other.
  const feat = desc.featured ?? [];
  const focal = desc.focal ?? (portrait ? { x: 0.5, y: 0.3 } : { x: 0.68, y: 0.36 });
  artSlot("Art", "Gacha.art", { texture: desc.image ?? null, kind: "banner", focal, rarity: feat[0]?.rarity, radius: t.panel, width: grow(), height: grow() }, art);
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: -2, layoutMotion: spring("ui") });
  kit.open("Scrim", { width: grow(), height: grow(), part: "Gacha.scrim", partClass: "panel" });
  // In portrait the words stand under the splash's face: the shade rises from the foot and leaves the face clear.
  kit.setDraw({ shape: { kind: "rect" }, fills: portrait ? [scrimFill("bottom", 0.94), scrimFill("bottom", 0.5)] : [scrimFill("start", 0.9), scrimFill("bottom", 0.7)] });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: -1 });

  kit.open("Body", { direction: portrait ? "column" : "row", width: grow(), height: grow(), gap: u(20), padding: xy(u(phone ? 12 : 28), u(phone ? 12 : 24)) });
  // The banners: a rail at the start on a desktop, chips above on a phone.
  if (tabs.length > 1) {
    kit.open("Tabs", { direction: portrait ? "row" : "column", wrap: portrait, gap: u(8), width: portrait ? grow() : null, part: "Gacha.tabs", partClass: "panel" });
    for (const b of tabs) {
      const on = b.key === desc.current;
      const r = surface(b.key ?? "", {
        part: "Gacha.tab",
        cls: "tab",
        flags: [on ? "current" : "", b.fresh ? "fresh" : ""],
        fill: mix(role("panel"), role("shadow"), 0.2),
        selected: on,
        selectedFill: mix(role("panelRaised"), role("primary"), 0.22),
        direction: portrait ? "column" : "row",
        // On a phone in portrait three tabs share the row; at large text each takes its own.
        width: portrait ? (big ? grow() : px(Math.floor((vp.width - vp.safeArea.left - vp.safeArea.right - u(24) - u(8) * (tabs.length - 1)) / tabs.length))) : px(u(short ? 160 : 200)),
        padding: xy(u(10), u(touch ? 12 : 8)),
        gap: u(10),
        radius: t.control,
        semantics: { role: "tab", name: b.name ?? b.key ?? "", selected: on },
      }, () => {
        // The current banner: an accent rim with a glow, lifted on its shadow (a white outline reads as a debug box).
        kit.setDraw({ shape: { kind: "rect", radii: [t.control, t.control, t.control, t.control] }, fills: [{ kind: "solid", color: token(on ? "panelRaised" : "panel", 0.92) }], strokes: [line(on ? 2 : 1, on ? paint(accent()) : token("line", 0.6))], effects: on ? [glow(u(10), paint(accent(), 0.45)), shadow(u(4), u(10), 0.5)] : [] });
        if (!phone && !short) artSlot("Thumb", "Gacha.thumb", { texture: b.image ?? null, kind: "banner", width: px(u(44)), height: px(u(44)), radius: t.small });
        kit.column("Text", { width: grow(), gap: 2, cross: portrait ? "center" : "start" }, () => {
          text("Name", b.name ?? "", { step: st(14), font: "display", maxLines: 2, width: grow(), align: portrait ? "center" : "start" });
          if (b.endsIn !== undefined && b.endsIn !== null && !portrait && !short) text("Ends", countdownShort(b.endsIn), { step: st(12), color: b.endsIn < 3600 ? "warn" : "inkMuted", nowrap: true, font: "mono" });
        });
        if (b.fresh) {
          kit.open("Dot", { width: px(u(8)), height: px(u(8)) });
          kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("danger") }] });
          kit.close();
          over({ insets: { end: u(6), top: u(6) }, inert: true });
        }
      });
      if (r.clicked && !on) {
        out.banner = b.key ?? null;
        kit.feedback("tab", -1);
      }
    }
    kit.close();
    kit.semantics({ role: "tablist", name: desc.title ?? "" });
    // Bumpers (Q and E) step through the banners.
    const at = tabs.findIndex((b) => b.key === desc.current);
    const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
    if (step !== 0 && at >= 0) {
      out.banner = tabs[(at + step + tabs.length) % tabs.length]!.key ?? null;
      kit.feedback("tab", -1);
    }
  }

  // The banner's words, at the start over the scrim.
  // Beside the art (a desktop, a phone on its side) the words and the purchase block under them are one column at the
  // start, centred down the frame; in portrait the words run from the top and the purchase block sits at the foot.
  const sideW = Math.min(u(480), Math.round(vp.width * 0.4));
  // In portrait the splash's face holds the top third and the words start under it, the purchase block straight after
  // them: the column stands on the frame's foot (no gap between the rate line and the pity meter).
  kit.open("Main", { direction: "column", width: portrait ? grow() : px(sideW), height: grow(), gap: u(portrait ? 10 : 18), align: portrait ? "end" : short ? "start" : "center" });
  // The words scroll when they outgrow the frame (a phone on its side, 200 % text); the purchase block never leaves it.
  kit.openScroll("Info", { width: grow(), height: short || (portrait && big) ? grow() : fit(), gap: u(portrait ? 8 : 10), padding: xy(0, 0) });
  {
    if (desc.subtitle) text("Subtitle", desc.subtitle.toLocaleUpperCase(), { step: st(13), color: "highlight", font: "display", nowrap: true, ellipsis: true, width: grow() });
    text("Title", desc.title ?? "", { step: st(phone ? 26 : 48), heading: true, font: "display", width: grow(), maxLines: 2, ellipsis: true });
    if (desc.endsIn !== undefined && desc.endsIn !== null) {
      timerChip("Timer", desc.endsIn, "");
    }
    if (feat.length > 0) {
      kit.column("Featured", { width: grow(), gap: u(6), part: "Gacha.featured", partClass: "panel" }, () => {
        // Each unit a quiet strip of dark glass (Star Rail, Genshin): its portrait at the start carries the tier on its
        // rim, the stars under the title carry it too; the strip itself stays neutral so the splash leads the screen.
        // In portrait the units stand side by side, each a portrait over its name and stars.
        const tileH = u(short ? 52 : portrait ? 76 : 84);
        kit.open("Units", { direction: portrait ? "row" : "column", gap: u(portrait ? 8 : 6), width: grow() });
        feat.forEach((f, k) => {
          const ink = tierInk(f.rarity);
          kit.open(kit.idi("Unit", k), { direction: portrait ? "column" : "row", width: grow(), height: portrait ? null : px(tileH), gap: u(portrait ? 4 : 12), cross: portrait ? "stretch" : "center", padding: portrait ? xy(u(4), u(4)) : { left: 0, top: 0, right: u(10), bottom: 0 }, radius: t.small, part: `Gacha.unit${flags(`rarity=${f.rarity ?? "none"}`)}`, partClass: "panel" });
          kit.setDraw({
            shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] },
            fills: [{ kind: "solid", color: token("shadow", 0.55) }, vgrad([[0, token("sheen", 0.06)], [1, token("sheen", 0)]])],
            strokes: [line(1, token("sheen", 0.16))],
          });
          artSlot("Portrait", "Gacha.unitPortrait", { texture: f.image ?? null, kind: "portrait", rarity: f.rarity, focal: { x: 0.5, y: 0.35 }, width: portrait ? grow() : px(Math.round(tileH * 1.15)), height: px(portrait ? tileH : tileH), radius: t.small }, () => {
            // The tier's rim round the portrait.
            kit.open("Rim", { width: grow(), height: grow(), part: `Gacha.unitRim${flags(`rarity=${f.rarity ?? "none"}`)}`, partClass: "slot" });
            kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, strokes: [line(2, paint(ink)), line(4, paint(ink, 0.2), "outside")] });
            kit.close();
          });
          kit.column("Text", { width: grow(), gap: u(2), cross: portrait ? "center" : "start" }, () => {
            text("Name", f.name ?? "", { step: st(portrait ? 14 : 17), font: "display", nowrap: true, ellipsis: true, width: grow(), align: portrait ? "center" : "start" });
            if (f.title && !short && !portrait) text("Title", f.title, { step: st(12), color: "inkMuted", maxLines: 1, ellipsis: true, width: grow() });
            stars("Stars", f.stars ?? 0, u(portrait ? 11 : 13), ink);
          });
          kit.close();
        });
        kit.close();
      });
    }
    if (desc.rateText) {
      kit.open("Rate", { width: grow(), direction: "column", padding: xy(u(10), u(6)), part: "Gacha.rate", partClass: "panel" });
      kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
      kit.richText("Body", desc.rateText, { step: st(13), color: token("inkMuted"), width: grow() });
      kit.close();
    }
  }
  kit.close();

  // The purchase block: the pity and the purse over the pulls; details and history under them.
  kit.open("Foot", { direction: "column", width: grow(), gap: u(portrait ? 10 : 12), cross: "stretch" });
  {
    kit.column("Pulls", { gap: u(8), cross: "stretch", width: grow() }, () => {
      kit.row("Status", { gap: u(14), cross: "center", wrap: big, align: "spaceBetween", width: grow() }, () => {
        if ((desc.pityMax ?? 0) > 0) pityMeter("Pity", desc.pity ?? 0, desc.pityMax!, desc.pityLabel ?? "Guaranteed in {left}");
        kit.row("Purse", { gap: u(6), cross: "center", padding: xy(u(10), u(4)), radius: 999, background: token("shadow", 0.55), part: "Gacha.purse", partClass: "chip" }, () => {
          priceTag("Balance", { amount: balance, currency, size: "md", image: desc.currencyIcon ?? null });
        });
      });
      kit.open("Buttons", { direction: big ? "column" : "row", gap: u(10), cross: "stretch", width: grow() });
      {
        const pull = (key: string, label: string, n: number, cost: number): void => {
          const can = balance >= cost;
          // The ten-pull leads (gold, the wider); the single is glass beside it.
          const r = btn(key, label, {
            variant: n > 1 ? "premium" : "glass",
            size: "lg",
            width: n > 1 || big ? grow() : null,
            part: `Gacha.pull${flags(n > 1 ? "multi" : "single", !can && "short")}`,
            name: `${label}, ${cost} ${currency}${can ? "" : `, ${desc.shortLabel ?? "not enough"}`}`,
            before: () => {
              // In the button's ink and face (dark on the premium gold), its gem too.
              priceTag("Price", { amount: cost, currency, affordable: can, size: "sm", shortLabel: desc.shortLabel ?? "not enough", ink: n > 1 ? hex("#1d1406") : role("ink"), image: desc.currencyIcon ?? null });
              kit.open("Sep", { width: px(1), height: px(u(18)), background: token("line") });
              kit.close();
            },
          });
          kit.props({ states: [{ name: "purse", states: [vstate("afford"), vstate("short")], transitions: [] }], state: [{ group: "purse", state: can ? "afford" : "short" }] });
          if (r.clicked) {
            if (can) out.pull = n;
            else {
              out.short = n;
              kit.feedback("error", -1);
            }
          }
        };
        if ((desc.single ?? 0) > 0) pull("Single", desc.singleLabel ?? "Wish ×1", 1, desc.single!);
        if ((desc.multi ?? 0) > 0) pull("Multi", fill(desc.multiLabel ?? "Wish ×{n}", { n: multiN }), multiN, desc.multi!);
      }
      kit.close();
    });
    kit.row("Bar", { gap: u(8), cross: "center" }, () => {
      if (btn("Details", desc.detailsLabel ?? "Details", { variant: "quiet", size: "sm", sym: "eye", part: "Gacha.details" }).clicked) out.details = true;
      if (btn("History", desc.historyLabel ?? "History", { variant: "quiet", size: "sm", sym: "clock", part: "Gacha.history" }).clicked) out.history = true;
    });
  }
  kit.close();
  kit.close();
  kit.close();
  // The name plate: the first featured unit large on the art side, its title and stars under it (beside the art only;
  // in portrait its row leads the list).
  const lead = feat[0];
  if (lead && !portrait) {
    const ink = tierInk(lead.rarity);
    kit.column("Spotlight", { gap: u(6), cross: "end", width: px(Math.round(vp.width * 0.36)), part: `Gacha.spotlight${flags(`rarity=${lead.rarity ?? "none"}`)}`, partClass: "badge" }, () => {
      text("Name", lead.name ?? "", { step: st(short ? 36 : 64), heading: true, font: "display", align: "end", width: grow(), maxLines: 2 });
      if (lead.title) text("Title", lead.title, { step: st(short ? 14 : 18), color: mix(ink, role("ink"), 0.55), font: "display", align: "end", width: grow(), maxLines: 2 });
      stars("Stars", lead.stars ?? 0, u(short ? 18 : 24), ink);
    });
    over({ insets: { end: u(phone ? 16 : 48), bottom: u(phone ? 14 : 40) }, inert: true });
  }
  kit.close();
  return out;
}

/** Seconds as `2d 4h` or `3h 12m`, for a tab. */
function countdownShort(seconds: number): string {
  const s = Math.max(0, Math.ceil(seconds));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  return d > 0 ? `${d}d ${h}h` : h > 0 ? `${h}h ${m}m` : `${m}m`;
}

/** The pity meter: a thin bar toward the guarantee and how many pulls are left. */
function pityMeter(id: string, pity: number, max: number, label: string): void {
  const f = clamp(pity / max, 0, 1);
  const w = u(140);
  kit.column(id, { gap: u(4), cross: "end", padding: xy(u(10), u(6)), part: "Gacha.pity", partClass: "badge" }, () => {
    // On a dark pill, as the purse beside it: it reads on any key art.
    kit.setDraw({ shape: { kind: "rect", radii: [u(10), u(10), u(10), u(10)] }, fills: [{ kind: "solid", color: token("shadow", 0.6) }] });
    text("Label", fill(label, { left: Math.max(0, max - pity), max, pity }), { step: st(13), nowrap: true, color: f > 0.8 ? "highlight" : "ink" });
    kit.open("Track", { width: px(w), height: px(u(6)), radius: 999, background: token("shadow", 0.6) });
    kit.open("Fill", { width: px(Math.max(u(6), Math.round(w * f))), height: px(u(6)), radius: 999, background: token(f > 0.8 ? "highlight" : "primary") });
    kit.close();
    kit.close();
  });
  kit.semantics({ role: "meter", name: fill(label, { left: Math.max(0, max - pity), max, pity }), valueMin: 0, valueMax: max, valueNow: pity });
}

// ── the pull ────────────────────────────────────────────────────────────

const CHARGE = 84;
const PHASES: Api.PullPhase[] = ["idle", "charge", "reveal", "summary"];

/**
 * A pull's reveal (Genshin's wish, Star Rail's warp, a pack opening): see
 * `fn pullSequence` in crates/og-api/api/ui.ogt.
 */
export function pullSequence(id: string, desc: Api.PullSequenceDesc): Api.PullSequenceResult {
  const { viewport: vp, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const items = desc.items ?? [];
  const n = items.length;
  const tick = kit.tick();
  const tok = desc.token ?? 0;
  const out: Mut<Api.PullSequenceResult> = { phase: "idle", index: 0, revealed: null, again: false, done: false };

  // Where the sequence is: a new token starts it from the charge.
  kit.open(id, { direction: "column", width: px(0), height: px(0) });
  let phase = kit.memory("~p.phase", 0);
  let idx = kit.memory("~p.idx", 0);
  let t0 = kit.memory("~p.t0", 0);
  const seen = kit.memory("~p.tok", 0);
  const shown = kit.memory("~p.shown", -1);
  if (tok !== 0 && tok !== seen) {
    phase = 1;
    idx = 0;
    t0 = tick;
    kit.feedback("open", -1);
  }
  if (tok === 0 || n === 0) phase = 0;
  const go = (p: number): void => {
    phase = p;
    t0 = tick;
  };
  const top = items.reduce<Api.PullItem | null>((b, x) => (!b || rarityPips(x.rarity) > rarityPips(b.rarity) ? x : b), null);
  const reduced = still();
  if (phase === 1 && (reduced || tick - t0 >= CHARGE)) go(2);

  const open = phase !== 0;
  let advance = false;
  let skip = false;
  const m = floating(`${id}Layer`, open, { scrim: true, locked: true, transition: "fade" }, () => ({ x: vp.safeArea.left, y: vp.safeArea.top }), () => {
    if (!open) return;
    // The layer sits inside the safe area; the scrim behind it covers the whole screen.
    const W = vp.width - vp.safeArea.left - vp.safeArea.right;
    const H = vp.height - vp.safeArea.top - vp.safeArea.bottom;
    const stage = surface("Stage", {
      part: `Pull.layer${flags(PHASES[phase] ?? "")}`,
      cls: "panel",
      fill: mix(role("well"), role("shadow"), 0.55),
      washBy: 0,
      direction: "column",
      width: px(W),
      height: px(H),
      align: "center",
      cross: "center",
      radius: 0,
      semantics: { role: "dialog", name: phase === 3 ? "Results" : top?.name ?? "" },
      props: { states: [{ name: "phase", states: [vstate("charge"), vstate("reveal"), vstate("summary")], transitions: [] }], state: [{ group: "phase", state: PHASES[phase] === "idle" ? "charge" : PHASES[phase]! }] },
    }, () => {
      // Edge to edge: the layer is the screen, so it draws no outline round it.
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: paint(mix(role("well"), role("shadow"), 0.55)) }] });
      kit.open("Content", { direction: "column", width: grow(), height: grow(), align: "center", cross: "center", padding: xy(u(16), u(16)) });
      if (phase === 1) charge(top, tick - t0, W, H);
      else if (phase === 2) {
        const it = items[clamp(idx, 0, n - 1)]!;
        reveal(it, idx, n, tick - t0, W, H, desc.tapLabel ?? "Tap to continue");
      } else if (phase === 3) summary(items, desc, W, H, (again) => {
        if (again) out.again = true;
        else out.done = true;
      });
      kit.close();
      // Skip, at the top end, until the summary.
      if (phase === 1 || phase === 2) {
        kit.row("Bar", { gap: u(8) }, () => {
          if (btn("Skip", desc.skipLabel ?? "Skip", { variant: "quiet", size: "sm", part: "Pull.skip" }).clicked) skip = true;
        });
        over({ insets: { end: u(16), top: u(12) }, z: 10 });
      }
    });
    if (stage.clicked && phase !== 3) advance = true;
  });
  void m;

  if (open) {
    if (kit.back()) {
      if (phase === 3) out.done = true;
      else skip = true;
    }
    if (skip && phase !== 3) {
      go(3);
      kit.feedback("confirm", -1);
    } else if (advance) {
      if (phase === 1) go(2);
      else if (phase === 2) {
        if (idx + 1 < n) {
          idx += 1;
          t0 = tick;
        } else go(3);
      }
    }
    if (out.done) {
      kit.feedback("close", -1);
      phase = 0;
    }
    if (phase === 2 && shown !== idx + keyHash(`${tok}`) * 64) {
      const it = items[idx]!;
      out.revealed = it.key ?? null;
      kit.feedback(rarityPips(it.rarity) >= 5 ? "success" : "reward", -1);
    }
  }
  kit.remember("~p.shown", phase === 2 ? idx + keyHash(`${tok}`) * 64 : -1);
  kit.remember("~p.phase", phase);
  kit.remember("~p.idx", idx);
  kit.remember("~p.t0", t0);
  kit.remember("~p.tok", tok);
  kit.close();
  out.phase = PHASES[phase] ?? "idle";
  out.index = idx;
  void phone;
  void portrait;
  return out;
}

/** The charge: a light that gathers in the middle, its colour turning toward the pull's highest tier as it grows. */
function charge(top: Api.PullItem | null, dt: number, W: number, H: number): void {
  const k = still() ? 1 : clamp(dt / CHARGE, 0, 1);
  const ease = 1 - (1 - k) ** 3;
  const target = tierInk(top?.rarity);
  // The first third is the primary's light; it turns to the tier's after it, so the tier is felt before it shows.
  const ink = mix(role("primary"), target, clamp((k - 0.3) / 0.4, 0, 1));
  const d = Math.round(Math.min(W, H) * (0.12 + 0.5 * ease));
  kit.open("Charge", { width: px(d), height: px(d), align: "center", cross: "center", part: `Pull.charge${flags(`rarity=${top?.rarity ?? "none"}`)}`, partClass: "badge" });
  kit.setDraw({ shape: { kind: "ellipse" }, fills: [radial([[0, paint({ r: 1, g: 1, b: 1, a: 1 }, 0.95)], [0.18, paint(ink, 0.9)], [0.6, paint(ink, 0.35)], [1, paint(ink, 0)]])], effects: [glow(Math.round(d * 0.25), paint(ink, 0.6), 2)] });
  kit.close();
  kit.semantics({ role: "status", name: "" });
}

/** One result large: its art, then its name, kind and stars landing one by one, a burst from legendary. */
function reveal(it: Api.PullItem, idx: number, n: number, dt: number, W: number, H: number, tap: string): void {
  const { phone } = frame();
  const ink = tierInk(it.rarity);
  const landscape = W > H;
  const artH = Math.round(Math.min(H * (landscape ? 0.62 : 0.5), u(560)));
  const artW = Math.round(artH * 0.72);
  kit.open("Reveal", { direction: landscape ? "row" : "column", gap: u(phone ? 16 : 40), align: "center", cross: "center", part: `Pull.reveal${flags(`rarity=${it.rarity ?? "none"}`)}`, partClass: "panel" });
  kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "scale", value: 0.7 }, { prop: "rotate", value: -0.08 }], transition: spring("emphasis") } });
  // Behind the card, in its tier's colour: a glow and rays turning slowly (still under reduced motion), wider and
  // brighter the higher the tier, so the result is felt before it is read.
  const pipsN = rarityPips(it.rarity);
  kit.open("Stage", { width: px(artW), height: px(artH) });
  const rayD = Math.round(artH * (pipsN >= 5 ? 2.1 : pipsN >= 4 ? 1.8 : 1.45));
  kit.open("Glow", { width: px(rayD), height: px(rayD), part: "Pull.glow", partClass: "badge" });
  kit.setDraw({ shape: { kind: "ellipse" }, fills: [radial([[0, paint(ink, pipsN >= 4 ? 0.7 : 0.45)], [0.45, paint(ink, 0.22)], [1, paint(ink, 0)]])] });
  kit.close();
  over({ insets: { start: Math.round((artW - rayD) / 2), top: Math.round((artH - rayD) / 2) }, inert: true, overlaps: ["*"] });
  if (pipsN >= 3) {
    kit.open("Rays", { width: px(rayD), height: px(rayD), part: "Pull.rays", partClass: "badge" });
    kit.setDraw({ shape: { kind: "polygon", sides: pipsN >= 5 ? 18 : 12, inner: 0.1, rotation: 0 }, fills: [{ kind: "solid", color: paint(mix(ink, role("ink"), 0.2), pipsN >= 5 ? 0.3 : 0.18) }] });
    kit.close();
    over({ insets: { start: Math.round((artW - rayD) / 2), top: Math.round((artH - rayD) / 2) }, inert: true, overlaps: ["*"], transform: { rotate: still() ? 0 : (dt / 60) * 0.12 } });
  }
  kit.open("Card", { width: px(artW), height: px(artH), part: "Pull.card", partClass: "card" });
  kit.setDraw({ shape: { kind: "rect", radii: [u(14), u(14), u(14), u(14)] }, fills: [{ kind: "solid", color: token("well") }], strokes: [line(3, paint(ink)), line(1, token("shadow", 0.7), "outside")], effects: [glow(u(28), paint(ink, 0.7), 4), shadow(u(10), u(24), 0.6)] });
  artSlot("Art", "Pull.art", { texture: it.image ?? null, kind: "character", rarity: it.rarity, width: px(artW - 6), height: px(artH - 6), radius: u(12) });
  over({ insets: { start: 3, top: 3 }, inert: true });
  if (it.isNew) {
    // New in the tier's own ink, not a second accent beside it.
    cornerTag("New", "NEW", ink);
    over({ insets: { end: -u(8), top: -u(10) }, inert: true, z: 4 });
  }
  kit.close();
  kit.close();
  kit.column("Words", { gap: u(8), cross: landscape ? "start" : "center", width: landscape ? px(Math.min(u(440), Math.round(W * 0.36))) : grow() }, () => {
    if (it.kind) text("Kind", it.kind.toLocaleUpperCase(), { step: st(14), color: mix(ink, role("ink"), 0.4), font: "display", nowrap: true });
    // The name leads: larger than the stars under it.
    text("Name", it.name ?? "", { step: st(phone ? 34 : 56), heading: true, font: "display", maxLines: 2, width: grow(), align: landscape ? "start" : "center" });
    kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: u(-40) }], transition: spring("panel"), delayMs: 180 } });
    const ns = it.stars ?? 0;
    stars("Stars", ns, u(phone ? 18 : 24), ink, ns, Array.from({ length: ns }, (_, k) => 320 + k * 110));
    // Legendary and above: a burst from the stars as the last one lands (from the stars, not the art, so its rays
    // stay fine and clear of the art and its tag).
    if (rarityPips(it.rarity) >= 5 && !still()) kit.burst({ count: 12, color: paint(mix(ink, role("ink"), 0.25)), radius: u(phone ? 90 : 140), delayMs: 320 + ns * 110 });
    if (it.extra) text("Extra", it.extra, { step: st(15), color: "highlight", maxLines: 2, width: grow(), align: landscape ? "start" : "center" });
  });
  kit.close();
  kit.semantics({ role: "status", name: [it.name, `${it.stars ?? 0} stars`, it.isNew ? "new" : "", it.extra ?? ""].filter(Boolean).join(", ") });
  // How far along, and how to go on.
  kit.row("Hint", { gap: u(10), cross: "center", part: "Pull.hint", partClass: "badge", padding: xy(u(16), u(8)) }, () => {
    // A pill on the stage, no outline: a hint, not a control.
    kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.6) }] });
    text("Count", `${idx + 1} / ${n}`, { step: st(14), font: "mono", color: "inkMuted", nowrap: true });
    const g = kit.frameGlyph("confirm") as Api.UiGlyph | null;
    if (g && !frame().touch) text("Glyph", g.text, { step: st(13), font: "mono", nowrap: true });
    text("Text", tap, { step: st(14), nowrap: true });
  });
  over({ insets: { centerX: true, bottom: u(18) }, inert: true });
  void dt;
}

/** Every result at once, the highest tiers first, then Again and Done. */
function summary(items: readonly Api.PullItem[], desc: Api.PullSequenceDesc, W: number, H: number, done: (again: boolean) => void): void {
  const { viewport: vp, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const order = items.map((x, i) => ({ x, i })).sort((a, b) => rarityPips(b.x.rarity) - rarityPips(a.x.rarity) || a.i - b.i).map((e) => e.x);
  const n = order.length;
  // A phone in portrait: two columns, one at large text (a long word must fit beside the art).
  const cols = portrait ? ((frame().theme.textScale ?? 1) >= 1.5 ? 1 : 2) : Math.min(5, n);
  const gap = u(phone ? 8 : 14);
  const room = W - u(32);
  const tw = Math.floor(Math.min(u(200), (room - gap * (cols - 1)) / cols));
  // The art's height: a portrait card's on a desktop, as much as two rows leave on a short screen; a square at the start on a phone in portrait.
  const rows = Math.ceil(n / cols);
  const textH = u(phone ? 58 : 76) * Math.max(1, frame().theme.textScale ?? 1);
  const artH = portrait ? u(56) : Math.max(u(40), Math.floor(Math.min((tw - u(16)) * 1.1, (H - u(phone ? 110 : 160)) / rows - gap - textH - u(16))));
  kit.open("Summary", { direction: "column", gap: u(phone ? 10 : 18), cross: "center", width: grow(), height: grow(), align: "center", part: "Pull.summary", partClass: "panel" });
  // The grid scrolls when the results outgrow the screen (a phone at large text); its bar stays.
  // (A grid that only just fills the room is not tall: it stays centred, not pinned to the top.)
  const tall = rows * (artH * (portrait ? 0 : 1) + Math.max(portrait ? artH : 0, textH) + u(16)) + (rows - 1) * gap + u(20) > H - u(phone ? 110 : 160) + u(4);
  // Its top padding keeps a tile's NEW tag (it stands above the tile) inside the scroll's clip.
  kit.openScroll("Grid", { width: px(cols * tw + (cols - 1) * gap + u(12)), height: tall ? grow() : null, padding: { left: u(6), right: u(6), top: u(14), bottom: u(10) }, gap: 0 });
  kit.open("Tiles", { direction: "row", wrap: true, gap, width: px(cols * tw + (cols - 1) * gap), align: "center" });
  order.forEach((it, k) => {
    const ink = tierInk(it.rarity);
    kit.open(it.key ?? kit.idi("Tile", k), { width: px(tw), direction: portrait ? "row" : "column", gap: u(8), padding: xy(u(8), u(8)), cross: portrait ? "center" : "stretch", part: `Pull.tile${flags(`rarity=${it.rarity ?? "none"}`, it.isNew && "new")}`, partClass: "card" });
    kit.setDraw({ shape: { kind: "rect", radii: [u(10), u(10), u(10), u(10)] }, fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 1 }, to: { x: 0.5, y: 0 }, stops: [{ offset: 0, color: paint(mix(role("panelRaised"), ink, 0.4)) }, { offset: 1, color: paint(mix(role("panel"), ink, 0.08)) }] }], strokes: [line(2, paint(ink)), line(1, token("shadow", 0.6), "outside")], ...(rarityPips(it.rarity) >= 5 ? { effects: [glow(u(12), paint(ink, 0.6), 1)] } : {}) });
    artSlot("Art", "Pull.art", { texture: it.image ?? null, kind: "character", rarity: it.rarity, width: portrait ? px(artH) : grow(), height: px(artH), radius: u(6) });
    kit.column("Text", { width: grow(), gap: 2, cross: portrait ? "start" : "center" }, () => {
      text("Name", it.name ?? "", { step: st(14), font: "display", maxLines: 2, width: grow(), align: portrait ? "start" : "center" });
      stars("Stars", it.stars ?? 0, u(12), ink);
      if (it.extra) text("Extra", it.extra, { step: st(12), color: "highlight", maxLines: 2, width: grow(), align: portrait ? "start" : "center" });
    });
    if (it.isNew) {
      cornerTag("New", "NEW", ink);
      // A row tile (a phone in portrait) wears it over its art at the start, clear of its name.
      over({ insets: portrait ? { start: -u(4), top: -u(8) } : { end: -u(4), top: -u(8) }, inert: true, z: 3 });
    }
    kit.close();
    kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: u(30) }, { prop: "scale", value: 0.85 }], transition: spring("emphasis"), delayMs: 60 * k } });
    kit.semantics({ role: "listitem", name: [it.name, `${it.stars ?? 0} stars`, it.isNew ? "new" : "", it.extra ?? ""].filter(Boolean).join(", ") });
  });
  kit.close();
  kit.semantics({ role: "list", name: "Results" });
  kit.close();
  // It wraps (Done under Again) when the two outgrow the screen: a phone at large text.
  kit.row("Bar", { gap: u(12), cross: "center", align: "center", wrap: true, width: px(W - u(32)) }, () => {
    if (desc.againLabel) {
      const can = desc.affordable ?? true;
      const r = btn("Again", fill(desc.againLabel, { n }), {
        variant: "secondary",
        size: "lg",
        part: `Pull.again${flags(!can && "short")}`,
        before: () => priceTag("Price", { amount: desc.againPrice ?? 0, currency: desc.currency ?? "gem", affordable: can, size: "sm", image: desc.currencyIcon ?? null }),
      });
      if (r.clicked) {
        if (can) done(true);
        else kit.feedback("error", -1);
      }
    }
    if (btn("Done", desc.doneLabel ?? "Done", { variant: "primary", size: "lg", part: "Pull.done" }).clicked) done(false);
  });
  kit.close();
  void plainText;
}
