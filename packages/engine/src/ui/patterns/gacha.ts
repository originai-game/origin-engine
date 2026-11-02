// Gacha (docs/ui.md §11 genre patterns, Appendix B: gacha banner and pull
// results; pack opening): the banner the player pulls on, and the pull's
// reveal. Where a pull is in its reveal is the component's own state
// (og.ui.memory, so a snapshot carries it); the game only starts one (a new
// token) and hears what the player pressed.

import type * as Api from "@origingame/engine";
import {
  accent, artSlot, btn, clamp, fit, flags, pct, floating, frame, glow, grow, hex, keyHash, kit, line, mix, over, paint, priceTag, px, radial, rarityColor, rarityPips,
  role, rtl, scrimFill, shadow, spring, st, still, surface, symbol, text, timerChip, token, u, vgrad, vstate, xy, type Mut, type Rgba, type TimeWords,
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

  // Laid out to the screen's edges (a phone's full-bleed banner), the art runs under the status bar and the notch while
  // the words and controls keep inside the safe area, below the room the game keeps for its own chrome (topInset).
  kit.open(id, { direction: "column", width: grow(), height: grow(), part: "Gacha.frame", partClass: "panel", radius: t.panel });
  const self = kit.bounds(kit.lastPath());
  const sa = vp.safeArea;
  const bleed = {
    top: self ? Math.max(0, sa.top - self.y) : 0,
    left: self ? Math.max(0, sa.left - self.x) : 0,
    right: self ? Math.max(0, self.x + self.width - (vp.width - sa.right)) : 0,
    bottom: self ? Math.max(0, self.y + self.height - (vp.height - sa.bottom)) : 0,
  };
  const edgeToEdge = !!self && self.y <= 0.5 && self.x <= 0.5;
  const topInset = desc.topInset ?? 0;
  // The key art behind everything, and a scrim from the text's edge so the text reads on any art.
  // The key art: the game's texture, else the style's placeholder (a figure standing at the focal point on two hues);
  // the game's builder draws over it. The featured unit stands on the art side, the words on the other.
  const feat = desc.featured ?? [];
  const focal = desc.focal ?? (portrait ? { x: 0.5, y: 0.3 } : { x: 0.68, y: 0.36 });
  artSlot("Art", "Gacha.art", { texture: desc.image ?? null, kind: "banner", focal, rarity: feat[0]?.rarity, radius: edgeToEdge ? 0 : t.panel, width: grow(), height: grow() }, art);
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: -2, layoutMotion: spring("ui") });
  kit.open("Scrim", { width: grow(), height: grow(), part: "Gacha.scrim", partClass: "panel" });
  // In portrait the words stand under the splash's face: the shade rises from the foot and leaves the face clear.
  kit.setDraw({ shape: { kind: "rect" }, fills: portrait ? [scrimFill("bottom", 0.94), scrimFill("bottom", 0.5)] : [scrimFill("start", 0.9), scrimFill("bottom", 0.7)] });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: -1 });
  // Chrome floating on the art (the status bar, the game's buttons, the tabs): a short shade from the top edge.
  const chromeH = bleed.top + topInset;
  if (chromeH > 0 || (portrait && tabs.length > 1)) {
    kit.open("TopScrim", { width: grow(), height: px(chromeH + u(portrait ? 96 : 64)), part: "Gacha.topScrim", partClass: "panel" });
    kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, token("shadow", 0.72)], [0.45, token("shadow", 0.4)], [1, token("shadow", 0)]])] });
    kit.close();
    over({ insets: { start: 0, top: 0, end: 0 }, inert: true, z: -1 });
  }

  const padX = u(phone ? 12 : 28);
  const padY = u(phone ? 12 : 24);
  kit.open("Body", { direction: portrait ? "column" : "row", width: grow(), height: grow(), gap: u(portrait ? 14 : 20), padding: { left: padX + bleed.left, top: padY + chromeH, right: padX + bleed.right, bottom: padY + bleed.bottom } });
  // The banners: a rail at the start on a desktop; on a phone in portrait chips leading the words (under the
  // splash's face, which the art's height fixes in the top third).
  const tabsBlock = (): void => {
  if (tabs.length > 1) {
    kit.open("Tabs", { direction: portrait ? "row" : "column", wrap: portrait, gap: u(8), width: portrait ? grow() : null, part: "Gacha.tabs", partClass: "panel" });
    for (const b of tabs) {
      const on = b.key === desc.current;
      const r = surface(b.key ?? "", {
        part: "Gacha.tab",
        cls: "tab",
        flags: [on ? "current" : "", b.fresh ? "fresh" : ""],
        // The glass is the tab's own draw below: no surface fill under it.
        fill: null,
        selected: on,
        direction: portrait ? "column" : "row",
        // On a phone in portrait three tabs share the row; at large text each takes its own.
        width: portrait ? (big ? grow() : px(Math.floor((vp.width - vp.safeArea.left - vp.safeArea.right - u(24) - u(8) * (tabs.length - 1)) / tabs.length))) : px(u(short ? 160 : 200)),
        padding: xy(u(10), u(touch ? 12 : 8)),
        gap: u(10),
        radius: t.control,
        semantics: { role: "tab", name: b.name ?? b.key ?? "", selected: on },
      }, () => {
        // One selection grammar (docs/ui.md §12.1): every tab a chip of the same dark glass over the art, with no
        // outline; the current one a raised neutral chip (the strong line's tone laid into the glass, a light along
        // its top edge, a short shadow), its name in full ink where the others are muted. Told by tone, ink and lift,
        // not by a rim or a hue (a white rim reads as the primary, or as a debug box).
        const white = hex("#ffffff");
        kit.setDraw({
          shape: { kind: "rect", radii: [t.control, t.control, t.control, t.control] },
          fills: [
            // In portrait, over the bright splash, the step between the two is wider: the others a darker glass, the
            // current one a lit neutral chip.
            { kind: "solid", color: paint(mix(role("panel"), role("shadow"), portrait && !on ? 0.6 : 0.3), on ? (portrait ? 0.86 : 0.62) : portrait ? 0.58 : 0.46) },
            ...(on ? [{ kind: "solid", color: paint(role("lineStrong"), portrait ? 0.8 : 0.55) } as Api.Fill] : []),
            vgrad([[0, paint(white, on ? 0.24 : 0.12)], [0.12, paint(white, on ? 0.1 : 0.05)], [1, paint(white, 0.02)]]),
          ],
          strokes: [],
          effects: on
            ? [{ kind: "innerShadow", offset: { x: 0, y: 1 }, blur: 0, spread: 0, color: paint(white, 0.35) }, shadow(u(3), u(8), 0.55)]
            : [shadow(u(2), u(6), 0.3)],
        } as Api.DrawDesc);
        if (!phone && !short) artSlot("Thumb", "Gacha.thumb", { texture: b.image ?? null, kind: "banner", width: px(u(44)), height: px(u(44)), radius: t.small });
        kit.column("Text", { width: grow(), gap: 2, cross: portrait ? "center" : "start" }, () => {
          text("Name", b.name ?? "", { step: st(14), font: "display", maxLines: 2, width: grow(), align: portrait ? "center" : "start", color: on ? "ink" : "inkMuted" });
          if (b.endsIn !== undefined && b.endsIn !== null && !portrait && !short) text("Ends", countdownShort(b.endsIn, timeWords(desc)), { step: st(12), color: b.endsIn < 3600 ? "warn" : on ? "ink" : "inkMuted", nowrap: true, font: "mono" });
        });
        // On a phone the current tab also carries a short underline in the neutral primary.
        if (on && portrait) {
          kit.open("Rim", { width: px(u(28)), height: px(u(3)), radius: 999, background: token("primary"), part: "Gacha.tabRim", partClass: "badge" });
          kit.close();
          over({ insets: { centerX: true, bottom: u(4) }, inert: true });
        }
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
  };
  if (!portrait) tabsBlock();

  // The banner's words, at the start over the scrim.
  // Beside the art (a desktop, a phone on its side) the words and the purchase block under them are one column at the
  // start, centred down the frame; in portrait the words run from the top and the purchase block sits at the foot.
  const sideW = Math.min(u(480), Math.round(vp.width * 0.4));
  // In portrait the splash's face holds the top third and the words start under it, the purchase block straight after
  // them: the column stands on the frame's foot (no gap between the rate line and the pity meter).
  kit.open("Main", { direction: "column", width: portrait ? grow() : px(sideW), height: grow(), gap: u(portrait ? 8 : 18), align: portrait ? "end" : short ? "start" : "center" });
  // The words scroll when they outgrow the frame (a phone on its side, 200 % text); the purchase block never leaves it.
  if (portrait) tabsBlock();
  kit.openScroll("Info", { width: grow(), height: short || (portrait && big) ? grow() : fit(), gap: u(portrait ? 8 : 10), padding: xy(0, 0) });
  {
    if (desc.subtitle) text("Subtitle", desc.subtitle.toLocaleUpperCase(), { step: st(13), color: "highlight", font: "display", nowrap: true, ellipsis: true, width: grow() });
    text("Title", desc.title ?? "", { step: st(phone ? 26 : 48), heading: true, font: "display", width: grow(), maxLines: 2, ellipsis: true });
    if (desc.endsIn !== undefined && desc.endsIn !== null) {
      timerChip("Timer", desc.endsIn, "", timeWords(desc));
    }
    // A phone in portrait at large text keeps the words and the purchase: the featured units are on the art already,
    // and squeezed into what is left they showed as slivers.
    if (feat.length > 0 && !(portrait && big)) {
      kit.column("Featured", { width: grow(), gap: u(6), part: "Gacha.featured", partClass: "panel" }, () => {
        // Each unit's portrait carries its tier as light rising from its foot (about a third of its height), under its
        // name and stars; no box and no rim. Beside the art (a desktop) each unit is a quiet strip of dark glass with its
        // words at the portrait's side; in portrait the units stand side by side as portraits, their words laid on that
        // light, with nothing framing them.
        const tileH = u(short ? 52 : portrait ? 104 : 84);
        kit.open("Units", { direction: portrait ? "row" : "column", gap: u(portrait ? 12 : 6), width: grow() });
        feat.forEach((f, k) => {
          const ink = tierInk(f.rarity);
          const tierLight = (h: number, strength: number, shade = 0): void => {
            const ink = mix(tierInk(f.rarity), role("shadow"), shade);
            // Undressed (a style's badge dress would lay its own flat fill over the gradient): light rising from the
            // foot, quadratic, so it reads as glow on the art and not as a coloured block.
            kit.open("Tier", { width: grow(), height: px(Math.round(h)) });
            kit.setDraw({ shape: { kind: "rect", radii: [0, 0, t.small, t.small] }, fills: [vgrad([[0, paint(ink, 0)], [0.3, paint(ink, strength * 0.08)], [0.55, paint(ink, strength * 0.25)], [0.8, paint(ink, strength * 0.6)], [1, paint(ink, strength)]])] });
            kit.close();
          };
          kit.open(kit.idi("Unit", k), { direction: portrait ? "column" : "row", width: grow(), height: portrait ? null : px(tileH), gap: u(12), cross: portrait ? "stretch" : "center", padding: portrait ? xy(0, 0) : { left: 0, top: 0, right: u(10), bottom: 0 }, radius: t.small, part: `Gacha.unit${flags(`rarity=${f.rarity ?? "none"}`)}`, partClass: "badge" });
          if (!portrait) {
            kit.setDraw({
              shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] },
              fills: [{ kind: "solid", color: token("shadow", 0.55) }, vgrad([[0, token("sheen", 0.06)], [1, token("sheen", 0)]])],
              strokes: [],
            });
          }
          artSlot("Portrait", "Gacha.unitPortrait", { texture: f.image ?? null, kind: "portrait", rarity: f.rarity, focal: { x: 0.5, y: 0.3 }, width: portrait ? grow() : px(Math.round(tileH * 1.15)), height: px(tileH), radius: t.small }, () => {
            if (!portrait) {
              tierLight(tileH * 0.38, 0.55);
              over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true });
              return;
            }
            // In portrait the words stand on the light at the portrait's foot.
            // The tier's light over the lower 40 % of the portrait (55 % at the foot, none at its top), and the name and
            // stars on a dark contact shadow over it, never on the colour itself.
            // Legendary is the one hot spot: lower tiers rise to 45 %, a little toward their shade.
            tierLight(tileH * 0.4, rarityPips(f.rarity) >= 5 ? 0.55 : 0.45, rarityPips(f.rarity) >= 5 ? 0 : 0.18);
            over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true });
            kit.column("Plate", { width: grow(), gap: u(2), cross: "center", padding: { left: u(4), right: u(4), top: u(14), bottom: u(5) } }, () => {
              kit.setDraw({ shape: { kind: "rect", radii: [0, 0, t.small, t.small] }, fills: [vgrad([[0, token("shadow", 0)], [0.4, token("shadow", 0.42)], [1, token("shadow", 0.7)]])] });
              text("Name", f.name ?? "", { step: st(14), font: "display", nowrap: true, ellipsis: true, width: grow(), align: "center" });
              stars("Stars", f.stars ?? 0, u(11), mix(ink, hex("#ffffff"), 0.25));
            });
            over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true });
          });
          if (!portrait) {
            kit.column("Text", { width: grow(), gap: u(2), cross: "start" }, () => {
              text("Name", f.name ?? "", { step: st(17), font: "display", nowrap: true, ellipsis: true, width: grow(), align: "start" });
              if (f.title && !short) text("Title", f.title, { step: st(12), color: "inkMuted", maxLines: 1, ellipsis: true, width: grow() });
              stars("Stars", f.stars ?? 0, u(13), ink);
            });
          }
          kit.close();
          kit.semantics({ role: "text", name: [f.name ?? "", f.title ?? "", `${f.stars ?? 0} stars`].filter(Boolean).join(", ") });
        });
        kit.close();
      });
    }
    // The same at large text in portrait for the rate line: Details opens the rules in full (a scrolled sliver of it
    // read as broken text).
    if (desc.rateText && !(portrait && big)) {
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
        // The purse as the shop shows it: the currency's art and the numeral, no box round them (the scrim under the
        // column keeps them readable).
        kit.row("Purse", { gap: u(6), cross: "center", part: "Gacha.purse", partClass: "badge" }, () => {
          priceTag("Balance", { amount: balance, currency, size: "md", image: desc.currencyIcon ?? null });
        });
      });
      kit.open("Buttons", { direction: big ? "column" : "row", gap: u(10), cross: "stretch", width: grow() });
      {
        const pull = (key: string, label: string, n: number, cost: number): void => {
          const can = balance >= cost;
          // The ten-pull leads (gold, the wider); the single is glass beside it.
          // On a phone in portrait the two share the row, each its label over its price (side by side the two did not fit).
          const stack = portrait;
          const ink = n > 1 ? hex("#1d1406") : role("ink");
          const r = btn(key, stack ? "" : label, {
            variant: n > 1 ? "premium" : "glass",
            size: "lg",
            width: n > 1 || big || stack ? grow() : null,
            part: `Gacha.pull${flags(n > 1 ? "multi" : "single", !can && "short")}`,
            name: `${label}, ${cost} ${currency}${can ? "" : `, ${desc.shortLabel ?? "not enough"}`}`,
            before: () => {
              if (stack) {
                kit.column("Stack", { gap: u(2), cross: "center", padding: xy(0, u(6)) }, () => {
                  // On the premium gold the label is bold, as og.ui's buttons set it.
                  if (n > 1) kit.richText("Label", `[b]${label.replace(/\[/g, "[[")}[/b]`, { step: st(16), color: paint(ink), nowrap: true, font: "display" } as Api.RichTextDesc);
                  else text("Label", label, { step: st(16), color: ink, nowrap: true, font: "display" });
                  priceTag("Price", { amount: cost, currency, affordable: can, size: "sm", shortLabel: desc.shortLabel ?? "not enough", ink, image: desc.currencyIcon ?? null });
                });
                return;
              }
              // In the button's ink and face (dark on the premium gold), its gem too.
              priceTag("Price", { amount: cost, currency, affordable: can, size: "sm", shortLabel: desc.shortLabel ?? "not enough", ink, image: desc.currencyIcon ?? null });
              // The rule between price and label in the label's own ink (a light line on the gold reads as a grey smear).
              kit.open("Sep", { width: px(1), height: px(u(18)), background: n > 1 ? paint(hex("#1d1406"), 0.45) : token("line") });
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
        if ((desc.single ?? 0) > 0) pull("Single", desc.singleLabel ?? "Summon ×1", 1, desc.single!);
        if ((desc.multi ?? 0) > 0) pull("Multi", fill(desc.multiLabel ?? "Summon ×{n}", { n: multiN }), multiN, desc.multi!);
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
function countdownShort(seconds: number, words: TimeWords): string {
  const s = Math.max(0, Math.ceil(seconds));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  return d > 0 ? fill(words.days ?? "{d}d {h}h", { d, h }) : h > 0 ? fill(words.hours ?? "{h}h {m}m", { h, m }) : fill(words.minutes ?? "{m}m", { m });
}

/** The banner's words for a time left. */
const timeWords = (desc: Api.GachaBannerDesc): TimeWords => ({ days: desc.daysLabel, hours: desc.hoursLabel, minutes: desc.minutesLabel });

/** The pity meter: a thin bar toward the guarantee and how many pulls are left. */
function pityMeter(id: string, pity: number, max: number, label: string): void {
  const f = clamp(pity / max, 0, 1);
  const words = fill(label, { left: Math.max(0, max - pity), max, pity });
  // The label at the start over a well track the meter's whole width, the fill its share of it: a meter, not an
  // underline. Lit in the highlight once the guarantee is near.
  kit.column(id, { gap: u(5), cross: "stretch", width: grow(), part: "Gacha.pity", partClass: "badge" }, () => {
    kit.row("Words", { gap: u(8), cross: "center", width: grow(), align: "spaceBetween" }, () => {
      text("Label", words, { step: st(13), nowrap: true, ellipsis: true, color: f > 0.8 ? "highlight" : "ink" });
      text("Count", `${pity} / ${max}`, { step: st(12), nowrap: true, font: "mono", color: "inkMuted" });
    });
    kit.open("Track", { width: grow(), height: px(u(6)), radius: 999 });
    kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: paint(hex("#ffffff"), 0.12) }], strokes: [line(1, token("shadow", 0.6), "outside")] });
    kit.open("Fill", { width: pct(Math.max(0.03, f)), height: grow() });
    kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token(f > 0.8 ? "highlight" : "primary") }] });
    kit.close();
    kit.close();
  });
  kit.semantics({ role: "meter", name: words, valueMin: 0, valueMax: max, valueNow: pity });
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
      // The banner's scene behind the light, blurred and dimmed with a vignette: the pull happens in its world, and no
      // half of the screen is a void.
      if (desc.backdrop) {
        // Sized to the stage (an absolute element's grow takes only the flow's leftover).
        kit.open("Backdrop", { width: px(W), height: px(H), part: "Pull.backdrop", partClass: "panel" });
        // One layer: the scene, dimmed, with a vignette, blurred together.
        kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "image", texture: desc.backdrop, fit: "cover" }, { kind: "solid", color: token("shadow", 0.62) }, radial([[0, token("shadow", 0)], [0.6, token("shadow", 0.25)], [1, token("shadow", 0.8)]], 0.5, 0.5, 0.75, 0.75)], effects: [{ kind: "layerBlur", radius: u(14) }] } as Api.DrawDesc);
        kit.close();
        // A blurred layer keeps its place in the flow (an absolute one loses its insets): it fills the stage, and the
        // content is laid over it.
        kit.props({ inert: true });
      }
      kit.open("Content", { direction: "column", width: px(W), height: px(H), align: "center", cross: "center", padding: xy(u(16), u(16)) });
      if (phase === 1) charge(top, tick - t0, W, H);
      else if (phase === 2) {
        const it = items[clamp(idx, 0, n - 1)]!;
        reveal(it, idx, n, tick - t0, W, H, desc.tapLabel ?? "Tap to continue", desc.continueLabel ?? "Continue", desc.newLabel ?? "NEW");
      } else if (phase === 3) summary(items, desc, W, H, (again) => {
        if (again) out.again = true;
        else out.done = true;
      });
      kit.close();
      if (desc.backdrop) over({ insets: { start: 0, top: 0 } });
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
function reveal(it: Api.PullItem, idx: number, n: number, dt: number, W: number, H: number, tap: string, go: string, newLabel: string): void {
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
  // The light is one radial bloom centred on the card, about 1.6 times its height across and never wider than about
  // 40 % of the screen, falling smoothly to nothing (no disc's edge, no wash over half the screen): the banner's scene
  // reads round it.
  // About 1.6 times the card on each side of its centre: the card covers the bloom's core, so its light must reach
  // past the card's edges to be seen (capped so it never washes half the screen).
  const rayD = Math.round(Math.min(artH * (pipsN >= 5 ? 1.9 : pipsN >= 4 ? 1.7 : 1.5), W * 0.6));
  const centred = { start: Math.round((artW - rayD) / 2), top: Math.round((artH - rayD) / 2) };
  const g0 = pipsN >= 5 ? 0.42 : pipsN >= 4 ? 0.32 : 0.22;
  kit.open("Glow", { width: px(rayD), height: px(rayD), part: "Pull.glow", partClass: "badge" });
  // Quadratic from the card's edge (about 0.35 of the radius, under the card) to nothing: the light hugs the card and
  // its outer third is barely there.
  kit.setDraw({ shape: { kind: "ellipse" }, blend: "add", fills: [radial([[0, paint(mix(ink, hex("#ffffff"), 0.2), g0)], [0.35, paint(ink, g0)], [0.48, paint(ink, g0 * 0.64)], [0.61, paint(ink, g0 * 0.36)], [0.74, paint(ink, g0 * 0.16)], [0.87, paint(ink, g0 * 0.04)], [1, paint(ink, 0)]])] } as Api.DrawDesc);
  kit.close();
  over({ insets: centred, inert: true, overlaps: ["*"] });
  if (pipsN >= 3) {
    // Motes drifting up through the bloom, twinkling.
    const moteInk = mix(ink, hex("#ffffff"), 0.35);
    const motes = pipsN >= 5 ? 12 : pipsN >= 4 ? 8 : 5;
    const t = still() ? 0 : dt / 60;
    for (let m = 0; m < motes; m++) {
      const h = (keyHash(`${it.key ?? it.name ?? ""}~${m}`) % 1000) / 1000;
      // Each rises slowly from below the card and fades in and out over its climb, then starts again.
      const period = 5 + 3 * h;
      const life = ((t / period + h) % 1 + 1) % 1;
      const x = artW / 2 + (h - 0.5) * rayD * 0.62 + Math.sin((t + m) * 0.6) * u(10);
      const y = artH / 2 + rayD * (0.38 - 0.76 * life);
      const sz = Math.round(u(3 + ((m * 13) % 4)));
      const a = still() ? 0.6 : Math.sin(life * Math.PI);
      kit.open(kit.idi("Mote", m), { width: px(sz), height: px(sz), part: "Pull.mote", partClass: "badge" });
      kit.setDraw({ shape: { kind: "ellipse" }, blend: "add", fills: [{ kind: "solid", color: paint(moteInk, 0.9 * a) }], effects: [glow(sz * 2, paint(ink, 0.6 * a), 0)] } as Api.DrawDesc);
      kit.close();
      over({ insets: { start: Math.round(x - sz / 2), top: Math.round(y - sz / 2) }, inert: true, overlaps: ["*"] });
    }
  }
  kit.open("Card", { width: px(artW), height: px(artH), part: "Pull.card", partClass: "card" });
  kit.setDraw({ shape: { kind: "rect", radii: [u(14), u(14), u(14), u(14)] }, fills: [{ kind: "solid", color: token("well") }], strokes: [line(3, paint(ink)), line(1, token("shadow", 0.7), "outside")], effects: [glow(u(16), paint(ink, 0.5), 0), shadow(u(10), u(24), 0.6)] });
  artSlot("Art", "Pull.art", { texture: it.image ?? null, kind: "character", rarity: it.rarity, width: px(artW - 6), height: px(artH - 6), radius: u(12) });
  over({ insets: { start: 3, top: 3 }, inert: true });
  if (it.isNew) {
    // New in the tier's own ink, not a second accent beside it.
    cornerTag("New", newLabel, ink);
    over({ insets: { end: -u(8), top: -u(10) }, inert: true, z: 4 });
  }
  kit.close();
  kit.close();
  kit.column("Words", { gap: u(8), cross: landscape ? "start" : "center", width: landscape ? px(Math.min(u(440), Math.round(W * 0.36))) : grow() }, () => {
    // The tier in a word as well as in colour and stars (Legendary · Character): the word in the tier's ink, bold.
    const tierWord = it.rarityLabel || (it.rarity && it.rarity !== "none" ? it.rarity[0]!.toUpperCase() + it.rarity.slice(1) : "");
    if (tierWord || it.kind) {
      kit.row("Kind", { gap: u(8), cross: "center" }, () => {
        if (tierWord) kit.richText("Tier", `[b]${tierWord.toLocaleUpperCase().replace(/\[/g, "[[")}[/b]`, { step: st(15), color: paint(mix(ink, hex("#ffffff"), 0.15)), font: "display", nowrap: true } as Api.RichTextDesc);
        if (tierWord && it.kind) text("Dot", "·", { step: st(15), color: "inkMuted", nowrap: true });
        if (it.kind) text("Text", it.kind.toLocaleUpperCase(), { step: st(14), color: "inkMuted", font: "display", nowrap: true });
      });
    }
    // The name leads: larger than the stars under it.
    text("Name", it.name ?? "", { step: phone ? "displaySm" : "displayLg", heading: true, font: "display", maxLines: 2, width: grow(), align: landscape ? "start" : "center" });
    kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: u(-40) }], transition: spring("panel"), delayMs: 180 } });
    // The banner's grammar: the name large, the epithet a caption under it in the tier's ink, the stars under that.
    if (it.title) {
      text("Title", it.title, { step: st(phone ? 16 : 22), font: "display", color: mix(ink, hex("#ffffff"), 0.2), maxLines: 2, width: grow(), align: landscape ? "start" : "center" });
      kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: u(-30) }], transition: spring("panel"), delayMs: 240 } });
    }
    const ns = it.stars ?? 0;
    stars("Stars", ns, u(phone ? 16 : 22), ink, ns, Array.from({ length: ns }, (_, k) => 320 + k * 110));
    // Legendary and above: a burst from the stars as the last one lands (from the stars, not the art, so its rays
    // stay fine and clear of the art and its tag).
    if (rarityPips(it.rarity) >= 5 && !still()) kit.burst({ count: 12, color: paint(mix(ink, role("ink"), 0.25)), radius: u(phone ? 90 : 140), delayMs: 320 + ns * 110 });
    if (it.extra) text("Extra", it.extra, { step: st(15), color: "highlight", maxLines: 2, width: grow(), align: landscape ? "start" : "center" });
  });
  kit.close();
  kit.semantics({ role: "status", name: [it.name, it.title ?? "", `${it.stars ?? 0} stars`, it.isNew ? "new" : "", it.extra ?? ""].filter(Boolean).join(", ") });
  // How far along, and how to go on.
  kit.row("Hint", { gap: u(10), cross: "center", part: "Pull.hint", partClass: "badge", padding: xy(u(16), u(8)) }, () => {
    // A pill on the stage, no outline: a hint, not a control.
    kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.6) }] });
    text("Count", `${idx + 1} / ${n}`, { step: st(14), font: "mono", color: "inkMuted", nowrap: true });
    // The hint for the device in hand only: the confirm key or button in a keycap and Continue, else Tap to continue.
    const g = frame().touch ? null : (kit.frameGlyph("confirm") as Api.UiGlyph | null);
    if (g) {
      kit.open("Glyph", { direction: "row", align: "center", cross: "center", padding: xy(u(6), 0), height: px(u(22)), radius: u(4), part: "Pull.keycap", partClass: "badge" });
      kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("ink", 0.12) }], strokes: [line(1, token("ink", 0.5))] });
      text("Text", g.text, { step: st(12), font: "mono", nowrap: true });
      kit.close();
    }
    text("Text", g ? go : tap, { step: st(14), nowrap: true });
  });
  over({ insets: { centerX: true, bottom: u(18) }, inert: true });
  void dt;
}

/** Every result at once, the highest tiers first, then Again and Done. */
function summary(items: readonly Api.PullItem[], desc: Api.PullSequenceDesc, W: number, H: number, done: (again: boolean) => void): void {
  const newLabel = desc.newLabel ?? "NEW";
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
    kit.setDraw({ shape: { kind: "rect", radii: [u(10), u(10), u(10), u(10)] }, fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 1 }, to: { x: 0.5, y: 0 }, stops: [{ offset: 0, color: paint(mix(role("panelRaised"), ink, 0.24)) }, { offset: 0.4, color: paint(mix(role("panelRaised"), ink, 0.04)) }, { offset: 1, color: token("panel") }] }], strokes: [line(1, token("shadow", 0.6), "outside")], ...(rarityPips(it.rarity) >= 5 ? { effects: [glow(u(12), paint(ink, 0.6), 1)] } : {}) });
    artSlot("Art", "Pull.art", { texture: it.image ?? null, kind: "character", rarity: it.rarity, width: portrait ? px(artH) : grow(), height: px(artH), radius: u(6) });
    kit.column("Text", { width: grow(), gap: 2, cross: portrait ? "start" : "center" }, () => {
      text("Name", it.name ?? "", { step: st(14), font: "display", maxLines: 2, width: grow(), align: portrait ? "start" : "center" });
      stars("Stars", it.stars ?? 0, u(12), ink);
      if (it.extra) text("Extra", it.extra, { step: st(12), color: "highlight", maxLines: 2, width: grow(), align: portrait ? "start" : "center" });
    });
    if (it.isNew) {
      cornerTag("New", newLabel, ink);
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
