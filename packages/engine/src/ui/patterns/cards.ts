// Card hands (docs/ui.md §11 genre patterns, Appendix D: drag to play with
// a targeting arc). The hand fans along an arc; the card under the pointer
// or the focus lifts clear, straightens and is laid out larger so it reads
// whole. A card is played by dragging it above the play line, by a click
// and a second click, or by confirm with keys and pads; a card that needs a
// target is aimed with an arc that snaps to the target under it. What is
// held and aimed is the component's own state (og.ui.memory, so a snapshot
// carries it); the game only hears CardHandResult.played.

import type * as Api from "@origingame/engine";
import {
  artSlot, clamp, fitStep, flags, floating, floatingContent, frame, glow, grow, hex, inside, keyHash, keyOf, kit, lastInteraction, line, mix, over, overProps,
  paint, pips, pointer, px, rarityColor, recallPath, rememberPath, role, rtl, shadow, spring, st, stIn, still, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut,
} from "./core.ts";

// ── the hand's state ────────────────────────────────────────────────────

interface Hand {
  /** The card pressed (its key's hash) while the pointer is down, where and when. */
  held: number;
  px: number;
  py: number;
  pt: number;
  /** 1: the held card has moved past the drag threshold. */
  drag: number;
  /** The card a click picked (it stays lifted; a second click plays it). */
  sel: number;
  /** Aiming a targeted card without a drag: 1 by keys and pads, 2 by the pointer (after a second click). */
  aim: number;
  /** The aimed target by keys: its index in desc.targets. */
  tgt: number;
  /** The cursor of keys and pads: a card's index. */
  cur: number;
  /** A refusal: its tick and card. */
  rt: number;
  rk: number;
  /** The tick it was last built, plus one. */
  seen: number;
}

const KEYS: Array<keyof Hand> = ["held", "px", "py", "pt", "drag", "sel", "aim", "tgt", "cur", "rt", "rk", "seen"];

function readHand(): Hand {
  const h = {} as Hand;
  for (const k of KEYS) h[k] = kit.memory(`~c.${k}`, 0);
  // Not built last tick (the screen closed and came back): nothing is held, only the cursor stays.
  if (h.seen !== 0 && kit.tick() + 1 - h.seen > 1) for (const k of KEYS) if (k !== "cur") h[k] = 0;
  return h;
}

function writeHand(h: Hand): void {
  h.seen = kit.tick() + 1;
  for (const k of KEYS) kit.remember(`~c.${k}`, h[k]);
}

/** A card's rules as words for a screen reader: the markup's tags left out. */
function plain(markup: string): string {
  return markup.replace(/\[\[/g, "\u0000").replace(/\[[^\]]*\]/g, "").replace(/\u0000/g, "[").replace(/\s+/g, " ").trim();
}

// ── a card's face ───────────────────────────────────────────────────────

export interface FaceOpts {
  playable: boolean;
  ready: boolean;
  /** Draw the rules (a card wide enough to read them). */
  rules: boolean;
  /** Lifted, held or shown large: the name and rules at their reading size. */
  big?: boolean;
  /** It has the focus of keys and pads: the ring is the card's (the hand draws none). */
  focused?: boolean;
  /** How much of its width shows from its start edge (the next card in the fan covers the rest); unset: all of it. */
  shows?: number;
  /** How much of its width a lifted card covers from its start edge; unset: none. */
  hides?: number;
}

/** A name's width in em, by its letters' rough widths (narrow i, l, f; wide m, w; capitals). */
function emWidth(s: string): number {
  let em = 0;
  for (const ch of s) em += /[ilIjft.,'!|:; ]/.test(ch) ? 0.3 : /[mwMW]/.test(ch) ? 0.85 : /[A-Z]/.test(ch) ? 0.64 : ch.charCodeAt(0) > 0x2e80 ? 1 : 0.54;
  return em;
}

/**
 * A card's face in the open element, `cw` × `ch`: the frame in its tier's
 * colour, the cost gem, the art, the name banner, the type line with its
 * pips, and the rules as rich text. Sizes follow the card's width.
 */
export function cardFace(c: Api.HandCard, cw: number, ch: number, o: FaceOpts): void {
  // The tier is a glow rising from the card's foot (none for common); the frame keeps one neutral stroke, so a tier
  // never reads as a selection.
  const tier = c.rarity === "common" ? null : rarityColor(c.rarity);
  const edge = role("lineStrong");
  const r = Math.max(6, Math.round(cw * 0.07));
  const glowing = o.ready ? role("success") : c.glow && o.playable ? role("primary") : null;
  kit.setDraw({
    shape: { kind: "rect", radii: [r, r, r, r] },
    fills: [
      vgrad([[0, paint(mix(role("panelRaised"), edge, 0.14))], [1, paint(mix(role("panel"), edge, 0.06))]]),
      ...(tier ? [vgrad([[0.5, paint(tier, 0)], [1, paint(tier, 0.42)]])] : []),
    ],
    strokes: [
      line(Math.max(2, Math.round(cw / 75)), paint(mix(edge, role("ink"), 0.12))),
      // Focused by keys or a pad: a ring in the focus ink just outside the card.
      ...(o.focused ? [line(u(3), token("focus"), "outside")] : [line(1, token("shadow", 0.5), "outside")]),
    ],
    effects: [shadow(Math.round(cw * 0.04), Math.round(cw * 0.1), 0.5), ...(glowing ? [glow(Math.round(cw * 0.1), paint(glowing, 0.85), 2)] : [])],
  });
  const pad = Math.max(4, Math.round(cw * 0.045));
  const artH = Math.round(ch * (o.rules ? 0.4 : 0.56));
  // Without its rules (a covered card at rest) the art takes the card down to its name banner: no empty half.
  artSlot("Art", "Card.art", { texture: c.image ?? null, kind: "card", rarity: c.rarity, radius: Math.max(3, r - pad), width: px(cw - pad * 2), height: o.rules ? px(artH) : grow() });
  // The banner: the name on a band of the frame's edge colour, darkened so the ink reads on any tier.
  const band = mix(role("well"), edge, 0.3);
  kit.open("Name", { width: grow(), direction: "row", align: "center", cross: "center", padding: xy(Math.round(pad * 1.2), Math.max(2, Math.round(pad * 0.6))), radius: Math.max(3, Math.round(r * 0.5)), background: paint(band), part: "Card.name", partClass: "badge" });
  // The name whole: up to two lines, its step fitted to the banner where one word is wider than it (a narrow fan at large text).
  const nameW = cw - pad * 2 - Math.round(pad * 2.4);
  const base = o.big ? 18 : cw >= u(180) ? 15 : 13;
  const sizeOf = (step: Api.TypeStepName) => frame().theme.typeRamp[step]?.size ?? 13;
  const widthAt = (step: Api.TypeStepName) => emWidth(c.name ?? "") * sizeOf(step);
  // Overlapped in the fan, every card still shows its name, the one beside a lifted card included: in the strip
  // that shows (from what a lifted card covers at its start to where the next card covers its end), its step
  // fitted down to it (to 10 px at the reference scale), always drawn: at worst its tail runs under the next card.
  const hides = Math.max(0, Math.min(o.hides ?? 0, cw - pad * 4));
  const strip = (o.shows ?? cw) - hides - pad * 2 - Math.round(pad * 1.2);
  const fitted = fitStep(c.name ?? "", nameW, base);
  const covered = hides > 0 || (o.shows !== undefined && cw / 2 + widthAt(fitted) / 2 > o.shows - pad);
  let nameStep = fitted;
  if (covered) {
    // The largest step down to the floor whose width fits the strip.
    nameStep = fitStep(c.name ?? "", strip, base, 10);
    for (let px = base; px >= 10; px--) {
      if (widthAt(st(px)) <= strip) {
        nameStep = sizeOf(st(px)) >= sizeOf(nameStep) ? st(px) : nameStep;
        break;
      }
    }
  }
  if (hides > 0) {
    kit.open("Hidden", { width: px(Math.max(0, hides - Math.round(pad * 1.2))), height: px(1) });
    kit.close();
  }
  text("Text", c.name ?? "", { step: nameStep, maxLines: covered ? 1 : 2, nowrap: covered, width: grow(), align: covered ? "start" : "center", font: "display", heading: true });
  kit.close();
  if (o.rules) {
    kit.row("Type", { width: grow(), gap: u(6), cross: "center", align: "center", part: "Card.type", partClass: "panel" }, () => {
      if (c.typeLine) text("Text", c.typeLine, { step: st(12), color: "inkMuted", nowrap: true, ellipsis: true });
      pips("Pips", c.rarity, Math.max(5, Math.round(cw * 0.035)), 6);
    });
    if (c.text) {
      kit.open("Rules", { width: grow(), height: grow(), direction: "column", align: "center", cross: "center", padding: xy(Math.round(pad * 0.6), 0), part: "Card.text", partClass: "panel" });
      // Lifted, the rules read at the body size (16 px at the reference scale); at rest they are a glance.
      kit.richText("Body", c.text, { step: st(o.big ? 16 : cw >= u(180) ? 14 : 12), color: token("ink"), align: "center", width: grow() });
      kit.close();
    }
  }
  // The cost: a gem in the top start corner. Its number is in the ink unless something changed it: danger when it is raised or
  // out of reach this turn, success when it is lowered (with a mark saying so, not only a colour). A card that cannot be played
  // at all (a status, a curse) has no number: its gem carries a cross.
  const hasCost = c.cost !== undefined && c.cost !== null;
  if (hasCost || !o.playable) {
    // It grows with the player's text size (to half again), so its number stays inside it.
    const g = Math.round(Math.max(26, cw * 0.22) * Math.min(1.5, Math.max(1, frame().theme.textScale ?? 1)));
    const never = !hasCost;
    const tone: Api.ThemeColor = never || !o.playable || c.costUp ? "danger" : c.costDown ? "success" : "ink";
    kit.open("Cost", { width: px(g), height: px(g), align: "center", cross: "center", part: `Card.cost${flags(c.costUp && "up", c.costDown && "down", hasCost && !o.playable && "unaffordable", never && "unplayable")}`, partClass: "badge" });
    // A gem in the energy colour (the highlight, as the energy counter wears it), lit from its top start with a light rim;
    // out of reach this turn it dulls to a dark stone so its danger number reads; a card never playable wears a danger gem.
    // A changed cost darkens its gem a step, so the green or red number on it holds its contrast.
    const gem = never ? role("danger") : !o.playable ? mix(role("lineStrong"), role("well"), 0.5) : c.costUp || c.costDown ? mix(role("highlight"), role("well"), 0.45) : role("highlight");
    const white = hex("#ffffff");
    kit.setDraw({
      shape: { kind: "ellipse" },
      fills: [{ kind: "gradient", type: "radial", from: { x: 0.36, y: 0.3 }, to: { x: 0.62, y: 0.62 }, stops: [{ offset: 0, color: paint(mix(gem, white, 0.3)) }, { offset: 0.45, color: paint(mix(gem, role("well"), 0.38)) }, { offset: 1, color: paint(mix(gem, role("well"), 0.62)) }] }],
      strokes: [line(Math.max(1.5, g * 0.06), paint(mix(gem, white, 0.55))), line(1, token("shadow", 0.75), "outside")],
      effects: [shadow(2, 5, 0.6)],
    });
    if (never) symbol("Never", "cross", Math.round(g * 0.5), role("ink"));
    else text("Value", `${c.cost}`, { step: stIn(g >= u(40) ? 22 : 16, Math.round(g * 0.62)), color: tone, font: "display", nowrap: true });
    kit.close();
    // Seated inside the card's corner, so the frame's line never runs through it.
    const seat = Math.max(3, Math.round(pad * 0.5));
    over({ insets: { start: seat, top: seat }, inert: true, z: 5 });
    // Changed this turn: a chevron on the gem's end edge, down when lowered and up when raised.
    if (hasCost && (c.costDown || c.costUp)) {
      const m = Math.round(g * 0.46);
      kit.open("Changed", { width: px(m), height: px(m), align: "center", cross: "center", part: `Card.costMark${flags(c.costDown ? "down" : "up")}`, partClass: "badge" });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token(c.costDown ? "success" : "danger") }], strokes: [line(1, token("shadow", 0.7), "outside")] });
      kit.open("Chevron", { width: px(Math.round(m * 0.6)), height: px(Math.round(m * 0.6)) });
      const q = Math.round(m * 0.6);
      const pts = c.costDown ? [{ x: q * 0.15, y: q * 0.32 }, { x: q * 0.5, y: q * 0.7 }, { x: q * 0.85, y: q * 0.32 }] : [{ x: q * 0.15, y: q * 0.68 }, { x: q * 0.5, y: q * 0.3 }, { x: q * 0.85, y: q * 0.68 }];
      kit.setDraw({ shape: { kind: "path", points: pts, width: Math.max(1.5, q * 0.18), cap: "round", join: "round" }, fills: [{ kind: "solid", color: token("onPrimary") }] });
      kit.close();
      kit.close();
      // On the gem's lower end edge, clear of its number.
      over({ insets: { start: seat + Math.round(g * 0.8), top: seat + Math.round(g * 0.68) }, inert: true, z: 6 });
    }
  }
  // Out of reach: the card under a shade (still readable), never faded to nothing.
  if (!o.playable) {
    kit.open("Shade", { width: px(cw), height: px(ch), part: "Card.shade", partClass: "panel" });
    kit.setDraw({ shape: { kind: "rect", radii: [r, r, r, r] }, fills: [{ kind: "solid", color: token("shadow", 0.5) }] });
    kit.close();
    over({ insets: { start: 0, top: 0 }, inert: true, z: 4 });
  }
}

// ── the hand ────────────────────────────────────────────────────────────

interface Place {
  x: number;
  y: number;
  w: number;
  h: number;
  rot: number;
  z: number;
  mode: "rest" | "lifted" | "held" | "aiming";
}

/**
 * A hand of cards (Slay the Spire, Hearthstone, Marvel Snap, Balatro):
 * see `fn cardHand` in crates/og-api/api/ui.ogt.
 */
export function cardHand(id: string, desc: Api.CardHandDesc): Api.CardHandResult {
  const { viewport: vp, touch, phone } = frame();
  const cards = desc.cards ?? [];
  const targets = desc.targets ?? [];
  const keys = cards.map((c) => c.key ?? "");
  const n = cards.length;
  const p = pointer();
  const tick = kit.tick();
  const out: Mut<Api.CardHandResult> = { played: null, aiming: null, hovered: null, inspect: null, dismissed: false, refused: null };
  const sa = vp.safeArea;

  kit.open(id, { direction: "column", width: grow(), cross: "center", part: "CardHand.frame", partClass: "panel" });
  const h = readHand();
  const fanRect = kit.bounds(recallPath(`${id}~fan`));
  const fanW = fanRect?.width ?? vp.width - sa.left - sa.right;

  // Sizes: a card at rest, and lifted (laid out larger, so its rules read at their own size).
  const roomH = vp.height - sa.top - sa.bottom;
  // A card's width at rest: its own, kept to the room, and narrowed (to 80 at least) until each card in the fan shows
  // more than half of itself, so the middle of every card is its own to press.
  const w0 = Math.min(desc.cardWidth && desc.cardWidth > 0 ? desc.cardWidth : phone ? 96 : u(150), fanW * 0.34, (roomH * 0.3) / 1.4);
  const w = Math.round(n > 1 ? Math.max(Math.min(w0, 80), Math.min(w0, (fanW * 0.9) / (1 + 0.62 * (n - 1)))) : w0);
  const ch = Math.round(w * 1.4);
  // Lifted, a card is laid out large enough that its rules read at the body size; larger again at large text.
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const liftW = Math.round(Math.min(Math.max(w * 1.45, phone ? 168 : u(240)) * Math.min(1.35, ts), fanW * 0.62, (roomH * 0.62) / 1.4));
  const liftH = Math.round(liftW * 1.4);
  // The fan's turn: a card's outermost corners swing out and down by it, so the box makes room for them.
  const per = (n > 1 ? Math.min(desc.spread && desc.spread > 0 ? desc.spread : 3, 36 / (n - 1)) : 0) * (Math.PI / 180);
  const maxRot = per * Math.max(0, (n - 1) / 2);
  const swingX = Math.ceil(ch * Math.sin(maxRot));
  const swingY = Math.ceil((w / 2) * Math.sin(maxRot));
  // Room over the cards for the cost gem standing out of their corner.
  const topRoom = u(8) + Math.round(Math.max(26, w * 0.22) * 0.3 * Math.min(1.5, Math.max(1, frame().theme.textScale ?? 1)));
  const H = ch + topRoom + u(14) + swingY + 2;
  const playY = vp.height - sa.bottom - roomH * (desc.playLine && desc.playLine > 0 ? desc.playLine : 0.36);

  // Which card is which.
  const held = keyOf(keys, h.held);
  const sel = keyOf(keys, h.sel);
  if (h.held && !held) h.held = 0;
  if (h.sel && !sel) (h.sel = 0), (h.aim = 0);
  h.cur = clamp(h.cur, 0, Math.max(0, n - 1));
  const card = (k: string | null): Api.HandCard | null => (k ? cards[keys.indexOf(k)] ?? null : null);
  const can = (c: Api.HandCard | null): boolean => !!c && !desc.disabled && (c.playable ?? true);
  const local = (x: number, y: number) => (fanRect ? { x: rtl() ? fanRect.x + fanRect.width - x : x - fanRect.x, y: y - fanRect.y } : { x, y });
  const targetAt = (x: number, y: number): number => targets.findIndex((t) => inside(t.rect ?? null, x, y));
  const refuse = (k: string): void => {
    h.rt = tick;
    h.rk = keyHash(k);
    out.refused = k;
    kit.feedback("error", -1);
  };
  const play = (k: string, target: string | null): void => {
    out.played = { key: k, target };
    h.held = h.drag = h.sel = h.aim = 0;
    kit.feedback("confirm", -1);
  };
  // Start what a card asks for: play it, or aim it (by keys, or by the pointer after a second click).
  const activate = (k: string, by: 1 | 2): void => {
    const c = card(k);
    if (!can(c)) return refuse(k);
    if (c!.needsTarget && targets.length > 0) {
      h.sel = keyHash(k);
      h.aim = by;
      h.tgt = Math.max(0, targets.findIndex((t) => t.valid ?? true));
      kit.feedback("select", -1);
    } else play(k, null);
  };

  // ── the pointer: a press held becomes a drag; a release ends it ──
  if (held && p.down && !h.drag) {
    if (Math.hypot(p.x - h.px, p.y - h.py) > (touch ? 10 : 5)) {
      if (!can(card(held))) {
        refuse(held);
        h.held = 0;
      } else {
        h.drag = 1;
        h.sel = 0;
        h.aim = 0;
        kit.feedback("select", -1);
      }
    }
  }
  const dragging = held && h.drag ? card(held) : null;
  const dragAiming = !!dragging && !!dragging.needsTarget && targets.length > 0 && !!fanRect && p.y < fanRect.y;
  if (held && p.released) {
    if (dragging) {
      if (dragAiming) {
        // Let go on a target it cannot take (behind a taunt): refused, it shakes; let go on nothing: it goes back.
        const t = targetAt(p.x, p.y);
        if (t >= 0 && (targets[t]!.valid ?? true)) play(held, targets[t]!.key ?? "");
        else if (t >= 0) refuse(held);
        else kit.feedback("cancel", -1);
      } else if (!dragging.needsTarget && p.y < playY) play(held, null);
    } else if (h.sel === h.held) activate(held, 2);
    else h.sel = h.held;
    h.held = 0;
    h.drag = 0;
  }
  // Aiming by the pointer: the next press plays on a valid target or stops aiming.
  const pAiming = h.aim === 2 && sel ? sel : null;
  if (pAiming && p.pressed) {
    const t = targetAt(p.x, p.y);
    if (t >= 0 && (targets[t]!.valid ?? true)) play(pAiming, targets[t]!.key ?? "");
    else if (t >= 0) {
      refuse(pAiming);
      h.aim = 0;
    } else {
      h.aim = 0;
      kit.feedback("cancel", -1);
    }
  }

  // ── which card lifts ──
  let hovered: string | null = null;
  const fanIx = lastInteraction(`${id}~fan`);
  // The fan's ring is up (keys or a pad put the focus on it, Response.ring of last tick): read in the hand's scope, where it is remembered.
  const focusVisible = (kit.memory(`${id}~fan~ix`, 0) & 8) !== 0;
  // The fan takes the focus and the arrows; its focus is drawn on the focused card (ownRing), not around the hand.
  kit.open("Fan", { width: grow(), height: px(H), interactive: true, focusable: n > 0, adjustable: true, ownRing: true, part: `CardHand.fan${flags(!!desc.disabled && "disabled")}`, partClass: "panel" });
  for (const k of keys) if (lastInteraction(k).hovered) hovered = k;
  // The cursor's card lifts while keys or a pad drive the hand (a finger's press also gives the fan the focus, and must
  // not leave the cursor's card standing over the screen); the ring shows then too, as a browser's focus-visible.
  const focusCard = fanIx.focused && focusVisible && n > 0 ? keys[h.cur]! : null;
  const heldNow = keyOf(keys, h.held);
  const selNow = keyOf(keys, h.sel);
  const lifted = heldNow ?? selNow ?? (touch ? null : hovered) ?? focusCard;
  out.hovered = hovered ?? focusCard;

  // ── the fan's places ──
  const step = n > 1 ? Math.min(w * 0.82, (fanW - 2 * swingX - w) / (n - 1)) : 0;
  const x0 = (fanW - (w + step * (n - 1))) / 2;
  const L = lifted ? keys.indexOf(lifted) : -1;
  const push = L >= 0 ? clamp((liftW - step) * 0.45, 0, w * 0.6) : 0;
  const mid = (n - 1) / 2;
  // Parted around a lifted card, each side moves aside as one, and closes up (to half a step at least) only where it
  // meets the fan's edge, so the cards beside the lifted one keep their strip and their names.
  const lo = Math.min(swingX, x0);
  const hi = Math.max(fanW - swingX - w, x0 + (n - 1) * step);
  const xAt = (i: number): number => {
    if (L < 0 || i === L) return x0 + i * step;
    if (i > L) {
      const m = n - 1 - L;
      const first = Math.min(x0 + (L + 1) * step + push, hi - (m - 1) * step * 0.5);
      const s = m > 1 ? Math.min(step, (hi - first) / (m - 1)) : 0;
      return clamp(first + (i - L - 1) * s, lo, hi);
    }
    const last = Math.max(x0 + (L - 1) * step - push, lo + (L - 1) * step * 0.5);
    const s = L > 1 ? Math.min(step, (last - lo) / (L - 1)) : 0;
    return clamp(last - (L - 1 - i) * s, lo, hi);
  };
  const places: Place[] = cards.map((_, i) => {
    const off = i - mid;
    const dip = mid > 0 ? Math.round(u(14) * (off / mid) ** 2) : 0;
    return { x: xAt(i), y: topRoom + dip, w, h: ch, rot: off * per, z: i, mode: "rest" };
  });
  if (L >= 0) {
    const pl = places[L]!;
    const cx = x0 + L * step + w / 2;
    const k = keys[L]!;
    if (dragging && k === held && !dragAiming) {
      // Held by the pointer: it follows, at its rest size, a little turned the way it was.
      // Kept inside the safe area: a card dragged to the screen's edge stops there whole.
      const q = local(p.x, p.y);
      const fr = fanRect!;
      const startRoom = rtl() ? vp.width - sa.right - (fr.x + fr.width) : fr.x - sa.left;
      const endRoom = rtl() ? fr.x - sa.left : vp.width - sa.right - (fr.x + fr.width);
      const x = clamp(q.x - w / 2, -startRoom, fanW + endRoom - w);
      // Its top stops a little over the play line's height above it: ready, it never rises over the screen's header.
      const ceiling = Math.max(sa.top, playY - ch * 1.1) - fr.y;
      const y = clamp(q.y - ch * 0.55, ceiling, vp.height - sa.bottom - fr.y - ch);
      Object.assign(pl, { x, y, rot: 0, z: 100, mode: "held" });
    } else {
      const aiming = (dragging && k === held && dragAiming) || (k === selNow && h.aim > 0);
      Object.assign(pl, { x: clamp(cx - liftW / 2, 0, Math.max(0, fanW - liftW)), y: H - liftH - u(8), w: liftW, h: liftH, rot: 0, z: 100, mode: aiming ? "aiming" : "lifted" });
    }
  }

  // What of a resting card shows: its end under the next card or a lifted card after it, its start under a lifted
  // card before it.
  const up = L >= 0 && places[L]!.mode !== "held" ? places[L]! : null;
  const cover = (i: number): { shows?: number; hides?: number } => {
    const pl = places[i]!;
    const ends = [places[i + 1]?.mode === "rest" ? places[i + 1]!.x : Infinity, up && L > i ? up.x : Infinity];
    const end = Math.min(...ends);
    const hides = up && L < i ? Math.max(0, up.x + up.w - pl.x) : 0;
    return { ...(end < Infinity ? { shows: Math.max(0, end - pl.x) } : {}), ...(hides > 0 ? { hides } : {}) };
  };

  // ── the cards ──
  for (let i = 0; i < n; i++) {
    const c = cards[i]!;
    const k = keys[i]!;
    const pl = places[i]!;
    const playable = can(c) || (!!desc.disabled && (c.playable ?? true));
    const ready = pl.mode === "held" && !c.needsTarget && p.y < playY;
    const shaking = h.rt > 0 && tick - h.rt < 18 && h.rk === keyHash(k);
    const shake = shaking && !still() ? Math.sin((tick - h.rt) * 1.6) * (18 - (tick - h.rt)) * 0.6 : 0;
    const following = pl.mode === "held";
    const r = surface(k, {
      part: "Card.frame",
      cls: "card",
      flags: [`rarity=${c.rarity ?? "none"}`, pl.mode === "rest" ? "" : pl.mode === "held" ? "dragging" : pl.mode, playable ? "" : "unplayable", c.glow ? "glow" : "", ready ? "ready" : ""],
      fill: null,
      washBy: 0.05,
      focusable: false,
      direction: "column",
      width: px(pl.w),
      height: px(pl.h),
      padding: xy(Math.max(4, Math.round(pl.w * 0.045)), Math.max(4, Math.round(pl.w * 0.045))),
      gap: Math.max(3, Math.round(pl.w * 0.03)),
      cross: "stretch",
      radius: Math.max(6, Math.round(pl.w * 0.07)),
      layer: {
        insets: { start: Math.round(pl.x), top: Math.round(pl.y) },
        z: pl.z,
        transform: { rotate: pl.rot, x: shake, pivotX: 0.5, pivotY: 1 },
        // Moving between places on the ui spring; under the pointer it follows exactly.
        ...(following ? {} : { layoutMotion: spring("ui"), motion: spring("ui") }),
        ...(pl.mode === "rest" ? {} : { overlaps: ["*"] }),
        // The long press (inspect) is og-ui's recogniser: its timing and slop, shared with every element.
        gestures: { longPress: true },
        ...(c.fresh ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: -u(260) }, { prop: "y", value: u(40) }, { prop: "rotate", value: -0.5 }], transition: spring("panel") } } : {}),
        exit: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: -u(120) }, { prop: "scale", value: 1.08 }], transition: { kind: "tween", durationMs: 220, ease: "exit" } },
        states: [
          { name: "lift", states: [vstate("rest"), vstate("lifted"), vstate("held"), vstate("aiming")], transitions: [] },
          { name: "play", states: [vstate("playable"), vstate("unplayable")], transitions: [] },
        ],
        state: [{ group: "lift", state: pl.mode }, { group: "play", state: playable ? "playable" : "unplayable" }],
      },
    }, () => cardFace(c, pl.w, pl.h, {
      playable,
      ready,
      // At rest at large text the rules are left to the lifted card (they would not fit the rest card).
      // One rule for every card: its name at rest, its rules on lift (a covered card's lines would be cut at the next
      // card's edge, and one card reading differently from its neighbours looks broken).
      rules: pl.w >= u(140) && pl.mode !== "rest",
      big: pl.mode !== "rest" && pl.w >= u(200),
      focused: fanIx.focused && focusVisible && i === h.cur,
      ...(pl.mode === "rest" ? cover(i) : {}),
    }));
    kit.semantics({
      role: "option",
      name: [c.name ?? k, c.cost !== undefined && c.cost !== null ? `${c.cost}` : "", c.typeLine ?? ""].filter(Boolean).join(", "),
      description: plain(c.text ?? ""),
      selected: k === lifted,
      disabled: !playable,
    });
    if (r.longPressed && h.held === keyHash(k) && !h.drag) {
      out.inspect = k;
      h.held = 0;
    }
    if (r.pressed && p.pressed && !pAiming) {
      h.held = keyHash(k);
      h.px = p.x;
      h.py = p.y;
      h.pt = tick;
      h.drag = 0;
      h.cur = i;
    }
  }
  // The hover sound as a card lifts under the pointer.
  const hv = hovered ? keyHash(hovered) : 0;
  if (hv && hv !== kit.memory("~c.hover", 0)) kit.feedback("hover", -1);
  kit.remember("~c.hover", hv);

  // ── the play line, while a card that plays on release is held ──
  if (dragging && !dragging.needsTarget && fanRect) {
    const above = p.y < playY;
    kit.open("PlayLine", { width: px(fanW), height: px(u(3)), part: `CardHand.playLine${flags(above && "ready")}`, partClass: "bar" });
    kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "gradient", type: "linear", from: { x: 0, y: 0.5 }, to: { x: 1, y: 0.5 }, stops: [{ offset: 0, color: paint(role(above ? "success" : "primary"), 0) }, { offset: 0.5, color: paint(role(above ? "success" : "primary"), 0.9) }, { offset: 1, color: paint(role(above ? "success" : "primary"), 0) }] }], effects: [glow(u(8), paint(role(above ? "success" : "primary"), 0.6))] });
    kit.close();
    kit.props(overProps({ insets: { start: 0, top: Math.round(playY - fanRect.y) }, inert: true, z: 90, overlaps: ["*"] }));
  }

  // The arc: from the aimed card's top to the pointer or the target the keys are on.
  const aimKey = dragAiming ? held : h.aim > 0 ? selNow : null;
  if (aimKey && fanRect) {
    const L2 = keys.indexOf(aimKey);
    const pl = places[L2]!;
    const sx = rtl() ? fanRect.x + fanRect.width - (pl.x + pl.w / 2) : fanRect.x + pl.x + pl.w / 2;
    const sy = fanRect.y + pl.y + u(10);
    let tx = p.x;
    let ty = p.y;
    let ti = targetAt(p.x, p.y);
    if (h.aim === 1) {
      h.tgt = clamp(h.tgt, 0, Math.max(0, targets.length - 1));
      ti = targets.length > 0 ? h.tgt : -1;
      const tr = ti >= 0 ? targets[ti]!.rect ?? null : null;
      if (tr) {
        tx = tr.x + tr.width / 2;
        ty = tr.y + tr.height / 2;
      }
    }
    const tgt = ti >= 0 ? targets[ti]! : null;
    const verdict = tgt ? ((tgt.valid ?? true) ? "valid" : "invalid") : "none";
    arc(sx, sy, tx, ty, verdict, fanRect, tgt, h.aim === 1 ? desc.targetPrompt ?? "Choose a target" : "");
    out.aiming = { key: aimKey, target: tgt ? tgt.key ?? null : null, x: tx, y: ty };
  }
  const fan = kit.close() as Api.Response;
  const fanPath = kit.lastPath();
  rememberPath(`${id}~fan`, fanPath);
  kit.remember(`${id}~fan~ix`, (fan.hovered ? 1 : 0) | (fan.pressed ? 2 : 0) | (fan.focused ? 4 : 0) | (fan.ring ? 8 : 0));
  kit.props({
    states: [{ name: "interaction", states: [vstate("normal"), vstate("hover"), vstate("pressed"), vstate("focused"), vstate("disabled")], transitions: [] }],
  });
  kit.semantics({ role: "listbox", name: desc.label ?? "Hand", value: n > 0 ? cards[h.cur]?.name ?? "" : "" });

  // ── keys and pads: Left and Right walk the hand (it claims them while focused; Up and Down leave it) ──
  if (fan.focused && n > 0) {
    const adjust = rtl() ? -fan.adjust : fan.adjust;
    if (h.aim === 1 && selNow) {
      // Left and Right step through the targets; confirm plays on a valid one; Back stops aiming.
      const step2 = (fan.adjust || 0) + (fan.adjustY || 0);
      if (step2 !== 0 && targets.length > 0) {
        h.tgt = (((h.tgt + Math.sign(rtl() ? -step2 : step2)) % targets.length) + targets.length) % targets.length;
        kit.feedback("focus", -1);
      }
      if (kit.back()) {
        h.aim = 0;
        h.sel = 0;
        kit.feedback("cancel", -1);
      } else if (fan.clicked && !p.released) {
        const t = targets[h.tgt];
        if (t && (t.valid ?? true)) play(selNow, t.key ?? "");
        else refuse(selNow);
      }
    } else {
      if (adjust !== 0) {
        h.cur = clamp(h.cur + adjust, 0, n - 1);
        h.sel = 0;
        kit.feedback("focus", -1);
      }
      if (fan.clicked && !p.released) activate(keys[h.cur]!, 1);
      if (kit.action("details")) out.inspect = keys[h.cur]!;
    }
  }
  // A press on nothing of the hand lets a picked card go.
  if (p.pressed && !h.held && h.sel && h.aim !== 1 && !pAiming && !inside(kit.bounds(fanPath), p.x, p.y)) h.sel = 0;

  // ── a card shown large ──
  const big = desc.inspect ? card(desc.inspect) : null;
  const bw = Math.round(Math.min(phone ? 280 : u(340), (vp.width - sa.left - sa.right) * 0.8, ((roomH - u(48)) / 1.4)));
  // Standing on the hand (just above the fan), so the header and its actions stay clear while there is room.
  // A card taller than the room above the hand (large text on a phone) slides down over the hand, which the scrim
  // shades, before it rises over the header's band (the top 30 % of the room).
  const above = (s: { width: number; height: number }): number => {
    const floor = fanRect ? fanRect.y - u(8) : (vp.height + s.height) / 2;
    const y = Math.min((vp.height - s.height) / 2, floor - s.height);
    const header = sa.top + roomH * 0.3;
    return Math.max(sa.top + u(8), y >= header ? y : Math.min(header, vp.height - sa.bottom - u(8) - s.height));
  };
  const m = floating(`${id}Inspect`, !!big, { scrim: true }, (s) => ({ x: (vp.width - s.width) / 2, y: above(s) }), (at) => {
    if (!big) return;
    floatingContent({ direction: "column", part: "CardHand.inspect", partClass: "panel" }, () => {
      // At large text it grows with its rules (its art gives up room first), never past the safe area.
      const grown = ts >= 1.3;
      kit.open("Card", { width: px(bw), height: grown ? null : px(Math.round(bw * 1.4)), direction: "column", cross: "stretch", padding: all4(Math.round(bw * 0.045)), gap: Math.round(bw * 0.03), part: `Card.frame${flags(`rarity=${big.rarity ?? "none"}`, "inspect")}`, partClass: "card" });
      // Grown, the art gives up most of its room to the rules.
      cardFace(big, bw, Math.round(bw * (grown ? 0.7 : 1.4)), { playable: can(big) || !!desc.disabled, ready: false, rules: true, big: true });
      kit.close();
      kit.semantics({ role: "dialog", name: big.name ?? "", description: plain(big.text ?? "") });
    });
    void at;
  });
  if (m.dismissed) out.dismissed = true;

  writeHand(h);
  kit.close();
  return out;
}

const all4 = (v: number): Api.Insets => ({ left: v, top: v, right: v, bottom: v });

/**
 * The targeting arc from (sx, sy) to (tx, ty), screen CSS px: a curve that
 * rises from the card and bends to the tip, in the verdict's ink with a
 * dark outline so it reads on any scene, a reticle at the tip, a ring
 * around the target under it, and (aiming by keys) a chip naming it.
 */
function arc(sx: number, sy: number, tx: number, ty: number, verdict: "none" | "valid" | "invalid", fan: Api.UiRect, tgt: Api.CardTarget | null, prompt: string): void {
  const tone = role(verdict === "valid" ? "success" : verdict === "invalid" ? "danger" : "primary");
  // A quadratic Bézier with its control point above the start, sampled.
  const cx = sx + (tx - sx) * 0.1;
  const cy = Math.min(sy, ty) - Math.max(u(40), Math.abs(tx - sx) * 0.35);
  const R = u(18);
  const pts: Api.Vector2[] = [];
  const N = 28;
  for (let k = 0; k <= N; k++) {
    const t = k / N;
    const x = (1 - t) ** 2 * sx + 2 * (1 - t) * t * cx + t * t * tx;
    const y = (1 - t) ** 2 * sy + 2 * (1 - t) * t * cy + t * t * ty;
    if (Math.hypot(x - tx, y - ty) < R * 0.9) break;
    pts.push({ x, y });
  }
  if (pts.length < 2) pts.push({ x: tx, y: ty - R });
  const minX = Math.min(...pts.map((q) => q.x)) - u(8);
  const minY = Math.min(...pts.map((q) => q.y)) - u(8);
  const maxX = Math.max(...pts.map((q) => q.x)) + u(8);
  const maxY = Math.max(...pts.map((q) => q.y)) + u(8);
  // Placed in the fan's box from its start edge; its points are the screen's, so a mirrored layout keeps them where they are.
  const startOf = (x0: number, x1: number) => (rtl() ? fan.x + fan.width - x1 : x0 - fan.x);
  kit.open("Arc", { width: px(maxX - minX), height: px(maxY - minY), part: `CardHand.arc${flags(verdict)}`, partClass: "bar" });
  kit.setDraw({
    shape: { kind: "path", points: pts.map((q) => ({ x: q.x - minX, y: q.y - minY })), width: u(6), cap: "round", join: "round" },
    fills: [{ kind: "solid", color: paint(tone) }],
    effects: [{ kind: "dropShadow", offset: { x: 0, y: 0 }, blur: 1, spread: u(2), color: token("shadow", 0.8) }, glow(u(10), paint(tone, 0.55))],
  });
  kit.close();
  kit.props(overProps({ insets: { start: Math.round(startOf(minX, maxX)), top: Math.round(minY - fan.y) }, inert: true, z: 120, overlaps: ["*"], states: [{ name: "verdict", states: [vstate("none"), vstate("valid"), vstate("invalid")], transitions: [] }], state: [{ group: "verdict", state: verdict }] }));
  kit.semantics({ role: "status", name: [verdict === "none" ? "" : verdict, tgt?.name ?? ""].filter(Boolean).join(": ") });
  // The reticle: a ring and a dot at the tip.
  kit.open("Reticle", { width: px(R * 2), height: px(R * 2), align: "center", cross: "center", part: `CardHand.reticle${flags(verdict)}`, partClass: "badge" });
  kit.setDraw({ shape: { kind: "ellipse" }, strokes: [line(u(3), paint(tone)), line(u(2), token("shadow", 0.8), "outside")], effects: [glow(u(8), paint(tone, 0.5))] });
  kit.open("Dot", { width: px(u(6)), height: px(u(6)) });
  kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: paint(tone) }], strokes: [line(1, token("shadow", 0.8), "outside")] });
  kit.close();
  kit.close();
  kit.props(overProps({ insets: { start: Math.round(startOf(tx - R, tx + R)), top: Math.round(ty - R - fan.y) }, inert: true, z: 121, overlaps: ["*"] }));
  // The target under the tip, ringed.
  const tr = tgt?.rect ?? null;
  if (tr) {
    const o = u(6);
    kit.open("TargetRing", { width: px(tr.width + o * 2), height: px(tr.height + o * 2), part: `CardHand.target${flags(verdict)}`, partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect", radii: [u(10), u(10), u(10), u(10)] }, strokes: [line(u(3), paint(tone)), line(u(2), token("shadow", 0.7), "outside")], effects: [glow(u(12), paint(tone, 0.5))] });
    kit.close();
    kit.props(overProps({ insets: { start: Math.round(startOf(tr.x - o, tr.x + tr.width + o)), top: Math.round(tr.y - o - fan.y) }, inert: true, z: 119, overlaps: ["*"] }));
  }
  // Aiming by keys: what the target is, above it (or the prompt while none is valid).
  if (prompt) {
    // Centred over the tip by its own width of last tick.
    const was = kit.bounds(recallPath("~c.prompt"));
    const cw = was?.width ?? u(120);
    kit.open("Prompt", { direction: "row", padding: xy(u(12), u(6)), part: `CardHand.prompt${flags(verdict)}`, partClass: "chip" });
    kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("panelRaised") }], strokes: [line(2, paint(tone))], effects: [shadow(u(3), u(8), 0.5)] });
    text("Text", tgt ? tgt.name ?? "" : prompt, { step: st(13), nowrap: true, font: "display" });
    kit.close();
    rememberPath("~c.prompt", kit.lastPath());
    const v = frame().viewport;
    const x0 = clamp(tx - cw / 2, v.safeArea.left + u(4), v.width - v.safeArea.right - cw - u(4));
    const top = (tr ? tr.y : ty - R) - u(44);
    kit.props(overProps({ insets: { start: Math.round(startOf(x0, x0 + cw)), top: Math.round(Math.max(frame().viewport.safeArea.top + u(4), top) - fan.y) }, inert: true, z: 122, overlaps: ["*"] }));
  }
}
