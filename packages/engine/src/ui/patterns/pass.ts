// The battle pass (docs/ui.md §11 genre patterns, Appendix B: battle pass and
// season track): the season's header with the level and its XP, then the
// track of tiers, a free and a premium reward each. The game holds the
// level, the XP, what is claimed and whether the premium track is owned; a
// press answers what the player claimed (or asked to unlock) for the game
// to grant. Which tier the track last opened on, a claim's pop and a
// refusal's shake are the component's own state (og.ui.memory).

import type * as Api from "@origingame/engine";
import {
  artSlot, btn, clamp, countdown, pct, fitStep, recallPath, rememberPath, rtl, fill, frame, glow, grouped, grow, hex, keyHash, kit, line, mix, over, paint, px, radial, rarityColor, role,
  scrimFill, shadow, spring, st, still, surface, symbol, text, token, u, vgrad, vstate, xy, isPhone, type Mut, type Rgba, type TimeWords,
} from "./core.ts";
import { STAR_GOLD } from "./saga.ts";

/** A reward's claim state: past the level, ready, claimed, or premium while the track is locked. */
type Claim = "ahead" | "ready" | "claimed" | "locked";

function claimOf(t: Api.PassTier, premiumTrack: boolean, desc: Api.BattlePassDesc): Claim {
  const claimed = premiumTrack ? t.premiumClaimed : t.freeClaimed;
  if (claimed) return "claimed";
  if (premiumTrack && !desc.premium) return "locked";
  return (t.level ?? 0) <= (desc.level ?? 0) ? "ready" : "ahead";
}

/** Every reward ready to claim, free before premium within a tier, tiers in order. */
function readyClaims(desc: Api.BattlePassDesc): Api.PassClaim[] {
  const out: Api.PassClaim[] = [];
  for (const t of desc.tiers ?? []) {
    if (t.free && claimOf(t, false, desc) === "ready") out.push({ level: t.level ?? 0, premium: false });
    if (t.premium && claimOf(t, true, desc) === "ready") out.push({ level: t.level ?? 0, premium: true });
  }
  return out;
}

/** The header: the season's art, its name and time left, the level in a badge, the XP bar, and the pass's actions. */
function header(desc: Api.BattlePassDesc, out: Mut<Api.BattlePassResult>, phone: boolean, words: TimeWords): void {
  const { t, theme } = frame();
  // Stacked on a phone and at large text (the actions under the season), so nothing runs past the edge.
  const small = phone || (theme.textScale ?? 1) > 1.4;
  const ready = readyClaims(desc);
  const frac = desc.xpNext && desc.xpNext > 0 ? clamp((desc.xp ?? 0) / desc.xpNext, 0, 1) : 0;
  kit.open("Header", { direction: small ? "column" : "row", width: grow(), gap: u(small ? 12 : 24), cross: small ? "stretch" : "center", padding: xy(u(small ? 14 : 24), u(small ? 14 : 18)), part: "Pass.header", partClass: "badge" });
  const r = t.panel;
  // The season's art behind it, shaded from the start so the words read; without art, the style's two hues.
  artSlot("Art", "Pass.art", { texture: desc.image ?? null, kind: "banner", focal: { x: 0.7, y: 0.35 }, width: grow(), height: grow(), radius: r });
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
  kit.open("Shade", { width: grow(), height: grow(), part: "Pass.shade", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [r, r, r, r] }, fills: [scrimFill(small ? "bottom" : "start", 0.9), { kind: "solid", color: token("shadow", 0.25) }], strokes: [line(1, paint(hex("#ffffff"), 0.08))], effects: [shadow(u(6), u(18), 0.45)] });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });

  // The level badge and the season's name with its XP.
  kit.open("Season", { direction: "row", width: small ? grow() : grow(), gap: u(small ? 12 : 18), cross: "center" });
  const lb = u(small ? 56 : 72);
  kit.open("Level", { direction: "column", width: px(lb), height: px(lb), align: "center", cross: "center", gap: 0, part: "Pass.level", partClass: "badge" });
  kit.setDraw({
    shape: { kind: "ellipse" },
    fills: [radial([[0, paint(mix(STAR_GOLD, hex("#ffffff"), 0.15))], [1, paint(mix(STAR_GOLD, hex("#7a4a00"), 0.45))]], 0.5, 0.35, 0.7, 0.7)],
    strokes: [line(u(3), paint(hex("#fff3c4"), 0.9)), line(1, token("shadow", 0.7), "outside")],
    effects: [glow(u(14), paint(STAR_GOLD, 0.45), 0), shadow(u(3), u(8), 0.5)],
  });
  text("N", `${desc.level ?? 0}`, { step: st(small ? 22 : 28), color: hex("#2a1800"), font: "display", nowrap: true });
  kit.close();
  kit.semantics({ role: "text", name: fill(desc.levelLabel ?? "Level {n}", { n: desc.level ?? 0 }) });
  kit.open("Words", { direction: "column", width: grow(), gap: u(6) });
  kit.open("TitleRow", { direction: "row", width: grow(), gap: u(10), cross: "center", wrap: true });
  text("Title", desc.title ?? "", { step: st(small ? 20 : 28), font: "display", heading: true, maxLines: 2, ...(small ? { width: grow() } : {}) });
  if (desc.endsIn !== undefined && desc.endsIn !== null) {
    kit.open("Timer", { direction: "row", gap: u(5), cross: "center", padding: xy(u(9), u(3)), part: `Pass.timer${desc.endsIn < 86400 ? "[urgent]" : ""}`, partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: desc.endsIn < 86400 ? token("danger", 0.9) : token("shadow", 0.6) }] });
    symbol("Clock", "clock", u(13), desc.endsIn < 86400 ? hex("#ffffff") : role("inkMuted"), { icon: "clock" });
    text("Time", countdown(desc.endsIn, words), { step: st(13), color: desc.endsIn < 86400 ? hex("#ffffff") : "ink", nowrap: true, font: "display" });
    kit.close();
    kit.semantics({ role: "timer", name: countdown(desc.endsIn, words) });
  }
  kit.close();
  // The XP toward the next level: a bar in the star gold, the numbers after it.
  kit.open("Xp", { direction: "row", width: grow(), gap: u(10), cross: "center", part: "Pass.xp", partClass: "badge" });
  kit.open("Track", { direction: "row", width: small ? grow() : px(u(280)), height: px(u(10)), radius: 999, part: "Pass.xpTrack", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.7) }], strokes: [line(1, paint(hex("#ffffff"), 0.12))] });
  // A share of the track (a phone's track grows with the header, so a fixed basis overfilled it).
  kit.open("Fill", { width: pct(Math.max(0.04, frac)), height: grow(), part: "Pass.xpFill", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [vgrad([[0, paint(mix(STAR_GOLD, hex("#ffffff"), 0.35))], [1, paint(STAR_GOLD)]])], effects: [glow(u(6), paint(STAR_GOLD, 0.5), 0)] });
  kit.props({ motion: spring("ui") });
  kit.close();
  kit.close();
  text("Value", fill(desc.xpLabel ?? "{xp} / {need} XP", { xp: grouped(desc.xp ?? 0), need: grouped(desc.xpNext ?? 0) }), { step: st(14), color: "inkMuted", nowrap: true, font: "display" });
  kit.close();
  kit.semantics({ role: "meter", name: fill(desc.levelLabel ?? "Level {n}", { n: desc.level ?? 0 }), value: fill(desc.xpLabel ?? "{xp} / {need} XP", { xp: grouped(desc.xp ?? 0), need: grouped(desc.xpNext ?? 0) }) });
  kit.close();
  kit.close();

  // The actions: Claim all while anything is ready, Unlock Premium until the track is owned.
  // A phone in portrait (and large text) stacks them, each the width of the header: side by side they ran off the screen.
  const stackActs = frame().viewport.breakpoint === "phonePortrait" || (theme.textScale ?? 1) > 1.4;
  kit.open("Actions", { direction: stackActs ? "column" : "row", gap: u(10), cross: stackActs ? "stretch" : "center", width: small ? grow() : null, align: small ? "stretch" : "end", part: "Pass.actions", partClass: "badge" });
  if (ready.length > 0) {
    if (btn("ClaimAll", fill(desc.claimAllLabel ?? "Claim all ({n})", { n: ready.length }), { variant: "primary", size: small ? "md" : "lg", part: "Pass.claimAll", width: small ? grow() : null }).clicked) {
      out.claimed = ready;
      kit.feedback("success", -1);
    }
  }
  if (!desc.premium) {
    const label = desc.unlockPriceText ? `${desc.unlockLabel ?? "Unlock Premium"} · ${desc.unlockPriceText}` : desc.unlockLabel ?? "Unlock Premium";
    if (btn("Unlock", label, { variant: "premium", size: small ? "md" : "lg", part: "Pass.unlock", sym: "crown", width: small ? grow() : null }).clicked) {
      out.unlock = true;
      kit.feedback("select", -1);
    }
  }
  kit.close();
  kit.close();
}

/**
 * A battle pass (Fortnite, Apex Legends, Brawl Stars): see `fn battlePass`
 * in crates/og-api/api/ui.ogt.
 */
export function battlePass(id: string, desc: Api.BattlePassDesc): Api.BattlePassResult {
  const { t, viewport: vp } = frame();
  const small = isPhone();
  const vertical = vp.breakpoint === "phonePortrait";
  const out: Mut<Api.BattlePassResult> = { claimed: [], unlock: false, refused: null, hovered: null };
  const words: TimeWords = { days: desc.daysLabel, hours: desc.hoursLabel };
  const tick = kit.tick();
  const moving = !still();
  const tiers = desc.tiers ?? [];
  const level = desc.level ?? 0;
  const frac = desc.xpNext && desc.xpNext > 0 ? clamp((desc.xp ?? 0) / desc.xpNext, 0, 1) : 0;

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(small ? 12 : 18), part: "Pass.frame", partClass: "badge" });
  header(desc, out, small, words);
  const rt = kit.memory("~bp.rt", 0);
  const rk = kit.memory("~bp.rk", 0);
  let refusedKey = "";

  // The lanes' names at the start of the track (over it in portrait): premium in gold with its crown (a lock while
  // the track is not owned), free in the ink.
  // The cells take the height the track is given (as laid out last frame): a premium cell a little taller than a free
  // one, a column a little narrower than its premium cell is tall; stacked, two cells share the row's width.
  const lineH = u(small ? 44 : 52);
  const room = kit.bounds(recallPath(`${id}~view`));
  const avail = room && !vertical ? room.height - u(20) : u(330);
  const premH = vertical ? u(124) : Math.round(clamp((avail - lineH) * 0.54, u(110), u(210)));
  const freeH = vertical ? u(124) : Math.round(clamp(avail - lineH - premH, u(96), u(180)));
  const colW = vertical ? u(104) : Math.round(clamp(premH * 0.84, u(104), u(170)));
  const laneW = Math.round(u(104) * Math.min(1.5, frame().theme.textScale ?? 1));
  const laneTitle = (key: string, premium: boolean, w: Api.Size, h: Api.Size, room: number): void => {
    kit.open(key, { direction: vertical ? "row" : "column", width: w, height: h, gap: u(6), align: "center", cross: "center", padding: xy(u(6), u(6)), part: `Pass.lane${premium ? "[premium]" : ""}`, partClass: "badge" });
    // Both lanes dark glass; the premium one told by its gold crown (a lock until it is owned), its gold ink and a
    // gold light along its top edge.
    kit.setDraw({
      shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] },
      fills: [{ kind: "solid", color: token("shadow", 0.58) }, vgrad([[0, paint(premium ? STAR_GOLD : hex("#ffffff"), premium ? 0.16 : 0.08)], [0.35, paint(hex("#ffffff"), 0.02)], [1, paint(hex("#ffffff"), 0)]])],
      effects: [{ kind: "innerShadow", offset: { x: 0, y: 1 }, blur: 0, spread: 0, color: paint(premium ? STAR_GOLD : hex("#ffffff"), premium ? 0.55 : 0.18) }],
    } as Api.DrawDesc);
    symbol("Mark", premium ? (desc.premium ? "crown" : "lock") : "star", u(small ? 16 : 20), premium ? STAR_GOLD : role("inkMuted"));
    const name = premium ? desc.premiumLabel ?? "Premium" : desc.freeLabel ?? "Free";
    text("Name", name, { step: fitStep(name, room, small ? 13 : 14), color: premium ? mix(STAR_GOLD, hex("#ffffff"), 0.45) : "ink", nowrap: true, font: "display" });
    kit.close();
  };

  kit.open("Track", { direction: vertical ? "column" : "row", width: grow(), height: grow(), gap: u(10), part: "Pass.track", partClass: "badge" });
  if (vertical) {
    kit.open("Lanes", { direction: "row", width: grow(), gap: u(8), cross: "center" });
    laneTitle("Free", false, grow(), px(u(36)), Math.round(vp.width / 2) - u(60));
    kit.open("Gap", { width: px(lineH) });
    kit.close();
    laneTitle("Premium", true, grow(), px(u(36)), Math.round(vp.width / 2) - u(60));
    kit.close();
  } else {
    kit.open("Lanes", { direction: "column", width: px(laneW), gap: 0, padding: { left: 0, top: u(10), right: 0, bottom: 0 } });
    laneTitle("Premium", true, grow(), px(premH), laneW - u(12));
    kit.open("Gap", { height: px(lineH) });
    kit.close();
    laneTitle("Free", false, grow(), px(freeH), laneW - u(12));
    kit.close();
  }

  kit.openScroll("View", { width: grow(), height: grow(), padding: vertical ? { left: 0, top: u(6), right: 0, bottom: u(24) } : { left: u(6), top: u(10), right: u(24), bottom: u(6) }, gap: 0, axes: vertical ? "vertical" : "horizontal" });
  kit.open("Tiers", { direction: vertical ? "column" : "row", width: vertical ? grow() : null, gap: 0, part: "Pass.tiers", partClass: "badge" });
  // Only the tiers near the view are built (a season has a hundred): one more than half a view away is an empty box of its
  // size, so the track keeps its length and scrolls the same. Where each tier lies follows from the player's tier's
  // cell as it was drawn last frame (a pressable part is found even scrolled out) and the tiers' known sizes.
  const viewBox = kit.bounds(recallPath(`${id}~view`));
  const anchor = kit.bounds(recallPath(`${id}~cur`));
  const sizeOf = (tr: Api.PassTier): number => (vertical ? Math.max(premH, freeH) + u(12) : tr.milestone ? Math.round(colW * 1.22) : colW);
  const along: number[] = [];
  let acc = 0;
  for (const tr of tiers) {
    along.push(acc + sizeOf(tr) / 2);
    acc += sizeOf(tr);
  }
  const curIdx = tiers.findIndex((tr) => (tr.level ?? 0) === level);
  let curPath = "";
  let curCell = "";
  tiers.forEach((tr, i) => {
    const lv = tr.level ?? 0;
    const reached = lv <= level;
    const current = lv === level;
    const progress = current ? "current" : reached ? "reached" : "ahead";
    const big = !!tr.milestone;
    const cw = big ? Math.round(colW * 1.22) : colW;
    const tierW = vertical ? grow() : px(cw);
    const tierH = px(vertical ? Math.max(premH, freeH) + u(12) : premH + lineH + freeH);
    let far = false;
    if (viewBox && anchor && curIdx >= 0 && !current) {
      const c = (vertical ? anchor.y + anchor.height / 2 : anchor.x + anchor.width / 2) + (along[i]! - along[curIdx]!) * (rtl() && !vertical ? -1 : 1);
      const half = sizeOf(tr) / 2;
      const v0 = vertical ? viewBox.y : viewBox.x;
      const len = vertical ? viewBox.height : viewBox.width;
      far = c + half < v0 - len / 2 || c - half > v0 + len * 1.5;
    }
    if (far) {
      kit.open(`L${lv}`, { width: tierW, height: tierH });
      kit.close();
      return;
    }
    kit.open(`L${lv}`, { direction: vertical ? "row" : "column", width: tierW, height: tierH, gap: 0, cross: "center", part: `Pass.tier${current ? "[current]" : reached ? "[reached]" : ""}${big ? "[milestone]" : ""}`, partClass: "badge" });
    kit.props({ states: [{ name: "progress", states: [vstate("ahead"), vstate("reached"), vstate("current")], transitions: [] }], state: [{ group: "progress", state: progress }] });
    if (current) {
      // The player's tier stands out of the track: a soft pool of the level's gold behind it on a darker ground (a pale
      // pane with a white rim read as a grey slab over the scene).
      kit.open("Here", { width: grow(), height: grow(), part: "Pass.here", partClass: "badge" });
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("shadow", 0.32) }, radial([[0, paint(STAR_GOLD, 0.16)], [0.55, paint(STAR_GOLD, 0.05)], [1, paint(STAR_GOLD, 0)]], 0.5, 0.5, 0.7, 0.7)], strokes: [] });
      kit.close();
      over({ insets: { start: u(2), top: 0, end: u(2), bottom: 0 }, inert: true });
    }
    // A reward cell: its art in its tier's rim, its count, and its claim state drawn over it.
    const cell = (track: "Free" | "Premium", rw: Api.PassReward | null | undefined, h: number): void => {
      const premiumTrack = track === "Premium";
      const w = vertical ? grow() : px(cw - u(14));
      if (!rw) {
        kit.open(track, { width: w, height: px(h) });
        kit.close();
        return;
      }
      const state = claimOf(tr, premiumTrack, desc);
      const key = `${lv}/${track}`;
      const tier = rarityColor(rw.rarity);
      const rim = tier ?? (premiumTrack ? mix(STAR_GOLD, role("panel"), 0.3) : role("lineStrong"));
      const popAt = kit.memory(`~bp.pop.${key}`, 0);
      const popping = moving && popAt > 0 && tick - popAt < 10;
      const shaking = moving && rt > 0 && tick - rt < 18 && rk === keyHash(key);
      const shake = shaking ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0;
      const breathe = state === "ready" && moving ? 0.5 + 0.5 * Math.sin(tick * 0.1) : state === "ready" ? 0.6 : 0;
      const stateWords = state === "claimed" ? desc.claimedLabel ?? "Claimed" : state === "locked" ? desc.lockedLabel ?? "Premium" : state === "ahead" ? fill(desc.reachLabel ?? "Reach level {n}", { n: lv }) : desc.claimLabel ?? "Claim";
      const r = surface(track, {
        part: "Pass.reward",
        cls: "badge",
        flags: [premiumTrack ? "premium" : "", state === "ready" ? "ready" : "", state === "claimed" ? "claimed" : "", state === "locked" ? "locked" : "", `rarity=${rw.rarity ?? "none"}`],
        fill: null,
        direction: "column",
        width: w,
        height: px(h),
        padding: xy(0, 0),
        radius: t.small,
        lift: state === "ready" ? -u(4) : 0,
        pressScale: 0.95,
        semantics: { role: "button", name: [rw.name ?? key, rw.amountText ?? "", premiumTrack ? desc.premiumLabel ?? "Premium" : desc.freeLabel ?? "Free", fill(desc.levelLabel ?? "Level {n}", { n: lv }), stateWords].filter(Boolean).join(", ") },
        props: {
          transform: { x: shake, scaleX: popping ? 1.12 : 1, scaleY: popping ? 1.12 : 1 },
          motion: spring("emphasis"),
          states: [{ name: "claim", states: [vstate("ahead"), vstate("ready"), vstate("claimed"), vstate("locked")], transitions: [] }],
          state: [{ group: "claim", state }],
        },
      }, () => {
        const radii: [number, number, number, number] = [t.small, t.small, t.small, t.small];
        // Ready: a glow in the star gold that breathes, and a lit rim; else no rim, the tier rising from the foot.
        kit.setDraw({
          shape: { kind: "rect", radii },
          fills: [vgrad([[0, paint(mix(role("panel"), rim, 0.3))], [1, paint(mix(role("panel"), rim, 0.12))]])],
          strokes: state === "ready" ? [line(2.5, paint(mix(STAR_GOLD, hex("#ffffff"), 0.3)))] : [line(1, token("shadow", 0.55), "outside")],
          effects: [shadow(u(3), u(8), 0.45), ...(state === "ready" ? [glow(u(10 + 8 * breathe), paint(STAR_GOLD, 0.35 + 0.35 * breathe), 0)] : [])],
        });
        artSlot("Art", "Pass.art", { texture: rw.image ?? null, kind: "item", rarity: rw.rarity, width: grow(), height: grow(), radius: radii }, () => {
          // The tier as light rising from the tile's foot, behind the words (no box round the tile).
          if (tier) {
            kit.open("Tier", { width: grow(), height: pct(0.42), part: "Pass.tier", partClass: "badge" });
            kit.setDraw({ shape: { kind: "rect", radii: [0, 0, radii[2], radii[3]] }, fills: [vgrad([[0, paint(tier, 0)], [0.45, paint(tier, 0.22)], [1, paint(tier, 0.6)]])] });
            kit.close();
            over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true });
          }
          if (state === "claimed" || state === "locked" || state === "ahead") {
            kit.open("Dim", { width: grow(), height: grow(), part: "Pass.dim", partClass: "badge" });
            kit.setDraw({ shape: { kind: "rect", radii }, fills: [{ kind: "solid", color: paint(mix(role("well"), role("inkMuted"), 0.15), state === "ahead" ? 0.2 : 0.62) }] });
            kit.close();
          }
        });
        over({ insets: { start: u(3), top: u(3), end: u(3), bottom: u(3) }, inert: true });
        kit.open("Body", { direction: "column", width: grow(), height: grow(), padding: xy(u(6), u(6)), cross: "center" });
        kit.open("Top", { direction: "row", width: grow() });
        kit.open("Gap", { width: grow() });
        kit.close();
        if (rw.amountText) {
          kit.open("Amount", { direction: "row", padding: xy(u(6), u(1)), part: "Pass.amount", partClass: "badge" });
          kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.7) }] });
          text("Text", rw.amountText, { step: st(12), color: "ink", nowrap: true, font: "display" });
          kit.close();
        }
        kit.close();
        kit.open("Mid", { width: grow(), height: grow(), align: "center", cross: "center" });
        if (state === "claimed") {
          kit.open("Check", { width: px(u(34)), height: px(u(34)), align: "center", cross: "center", part: "Pass.check", partClass: "badge" });
          kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("success") }], strokes: [line(2, paint(hex("#ffffff"), 0.9))], effects: [shadow(2, 6, 0.5)] });
          symbol("Tick", "check", u(18), hex("#ffffff"), { weight: u(3) });
          kit.close();
        } else if (state === "locked") {
          kit.open("Lock", { width: px(u(34)), height: px(u(34)), align: "center", cross: "center", part: "Pass.lock", partClass: "badge" });
          kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("shadow", 0.75) }], strokes: [line(1.5, paint(STAR_GOLD, 0.8))] });
          symbol("Icon", "lock", u(16), STAR_GOLD);
          kit.close();
        }
        kit.close();
        if (state === "ready") {
          kit.open("Claim", { direction: "row", padding: xy(u(10), u(2)), part: "Pass.claim", partClass: "badge" });
          kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [vgrad([[0, paint(mix(STAR_GOLD, hex("#ffffff"), 0.3))], [1, paint(STAR_GOLD)]])], effects: [shadow(2, 5, 0.5)] });
          kit.richText("Label", `[b]${(desc.claimLabel ?? "Claim").replace(/\[/g, "[[")}[/b]`, { step: st(13), color: paint(hex("#2a1800")), nowrap: true, font: "display" } as Api.RichTextDesc);
          kit.close();
        }
        kit.close();
      });
      if (current && !curCell) curCell = curPath = kit.lastPath();
      if (r.hovered || r.focused) out.hovered = lv;
      if (r.clicked) {
        if (state === "ready") {
          out.claimed = [...out.claimed, { level: lv, premium: premiumTrack }];
          kit.remember(`~bp.pop.${key}`, tick);
          kit.feedback("select", -1);
        } else if (state !== "claimed") {
          out.refused = { level: lv, premium: premiumTrack };
          refusedKey = key;
          kit.feedback("error", -1);
        }
      }
    };
    // The level line: lit up to the player's level (the current tier's far half by its XP), the level in a badge.
    const levelLine = (): void => {
      const lit = lv < level ? 1 : current ? 0.5 + 0.5 * frac : 0;
      kit.open("Line", { direction: vertical ? "column" : "row", width: vertical ? px(lineH) : grow(), height: vertical ? grow() : px(lineH), align: "center", cross: "center", part: `Pass.line${reached ? "[lit]" : ""}`, partClass: "badge" });
      kit.open("Rail", { direction: vertical ? "column" : "row", width: vertical ? px(u(6)) : grow(), height: vertical ? grow() : px(u(6)), part: "Pass.rail", partClass: "badge" });
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", 0.65) }] });
      kit.open("Lit", { width: vertical ? grow() : px(Math.round(cw * lit)), height: vertical ? px(Math.round((Math.max(premH, freeH) + u(12)) * lit)) : grow(), part: "Pass.railLit", partClass: "badge" });
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: paint(STAR_GOLD) }], effects: [glow(u(6), paint(STAR_GOLD, 0.6), 0)] });
      kit.close();
      kit.close();
      over({ insets: vertical ? { centerX: true, top: 0, bottom: 0 } : { start: 0, end: 0, centerY: true }, inert: true });
      const bs = u(big ? 40 : current ? 38 : 32);
      kit.open("Badge", { width: px(bs), height: px(bs), align: "center", cross: "center", part: `Pass.levelBadge${big ? "[milestone]" : ""}`, partClass: "badge" });
      kit.setDraw({
        shape: big ? { kind: "polygon", sides: 6, rotation: Math.PI / 6, rounding: u(3) } as Api.DrawShape : { kind: "ellipse" },
        fills: [{ kind: "solid", color: reached ? paint(big ? STAR_GOLD : mix(STAR_GOLD, hex("#ffffff"), 0.15)) : paint(mix(role("panelRaised"), role("ink"), 0.08)) }],
        strokes: [line(current ? 3 : 1.5, paint(current ? hex("#ffffff") : reached ? hex("#fff3c4") : role("lineStrong"), current ? 0.95 : 0.8))],
        effects: [shadow(2, 6, 0.5), ...(current ? [glow(u(10), paint(STAR_GOLD, 0.6), 0)] : [])],
      });
      text("N", `${lv}`, { step: st(big ? 16 : 14), color: reached ? hex("#2a1800") : "inkMuted", nowrap: true, font: "display" });
      kit.close();
      kit.close();
    };
    if (vertical) {
      cell("Free", tr.free, freeH);
      levelLine();
      cell("Premium", tr.premium, premH);
    } else {
      cell("Premium", tr.premium, premH);
      levelLine();
      cell("Free", tr.free, freeH);
    }
    kit.close();
    kit.semantics({ role: "listitem", name: fill(desc.levelLabel ?? "Level {n}", { n: lv }) });
  });
  kit.close();
  rememberPath(`${id}~cur`, curCell);
  kit.semantics({ role: "list", name: desc.label ?? "Battle pass" });
  // Open on the player's tier, and follow it when the level changes; asked until it shows, then left to the player.
  const want = keyHash(`${level}`) + 1;
  if (curPath && kit.memory("~bp.shown", 0) !== want) {
    kit.scrollIntoView(curPath, true);
    if (kit.bounds(curPath)) kit.remember("~bp.shown", want);
  }
  kit.close();
  rememberPath(`${id}~view`, kit.lastPath());
  kit.close();
  if (refusedKey) {
    kit.remember("~bp.rt", tick);
    kit.remember("~bp.rk", keyHash(refusedKey));
  }
  kit.close();
  kit.semantics({ role: "region", name: desc.label ?? "Battle pass" });
  return out;
}
