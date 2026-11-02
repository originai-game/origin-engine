// Rewards: pick one of N (docs/ui.md §11 genre patterns, Appendix D: pick
// one of N, reroll, banish). The rewards are cards dealt in face up; a
// press takes one, or selects it when the choice confirms or takes
// several. What is armed (banishing) is the component's own state
// (og.ui.memory); what the player took is answered for the game to apply.

import type * as Api from "@origingame/engine";
import {
  artSlot, btn, frame, glow, grow, keyHash, kit, lastInteraction, line, mix, over, paint, pips, px, rarityColor, rarityPips, role, shadow, st, still,
  inkOn, spring, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut, type Rgba,
} from "./core.ts";

/** A card's rules as words for a screen reader: the markup's tags left out. */
export function plainText(markup: string): string {
  return markup.replace(/\[\[/g, "\u0000").replace(/\[[^\]]*\]/g, "").replace(/\u0000/g, "[").replace(/\s+/g, " ").trim();
}

/** A small tag in a corner (New, Upgrade): ink on the highlight, outlined so it reads on any art; `up` leads it with an up arrow. */
export function cornerTag(id: string, s: string, tone: Api.ThemeColor | Rgba = "highlight", up = false): void {
  // A tier's own colour (a pull's New on a legendary card) takes the ink that reads on it.
  const fill = typeof tone === "string" ? role(tone) : tone;
  const ink = inkOn(fill);
  kit.open(id, { direction: "row", gap: u(3), cross: "center", padding: xy(u(8), u(3)), part: "Reward.tag", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: paint(fill) }], strokes: [line(1, token("shadow", 0.6), "outside")], effects: [shadow(2, 4, 0.45)] });
  if (up) symbol("Up", "arrowUp", u(12), ink);
  text("Text", s, { step: st(12), color: ink, nowrap: true, font: "display" });
  kit.close();
}

/** A reward's tag in its kind's ink: new in the tier's colour, upgrade in success with an up arrow, a note in the highlight. */
function rewardTag(o: Api.RewardOption): void {
  const kind = o.tagKind ?? "note";
  cornerTag("Tag", o.tag ?? "", kind === "new" ? rarityColor(o.rarity) ?? role("highlight") : kind === "upgrade" ? role("success") : "highlight", kind === "upgrade");
}

/**
 * A choice of N rewards (Slay the Spire, Hades, Vampire Survivors,
 * Balatro): see `fn rewardChoice` in crates/og-api/api/ui.ogt.
 */
export function rewardChoice(id: string, desc: Api.RewardChoiceDesc): Api.RewardChoiceResult {
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const opts = desc.options ?? [];
  const n = opts.length;
  const picks = Math.max(1, desc.picks ?? 1);
  const confirming = !!desc.confirm || picks > 1;
  const selected = new Set(desc.selected ?? []);
  const out: Mut<Api.RewardChoiceResult> = { picked: [], selected: [...selected], rerolled: false, banished: null, skipped: false, hovered: null, refused: null };
  const tick = kit.tick();
  const ts = Math.max(1, frame().theme.textScale ?? 1);

  // In portrait it takes the height it is given, its actions docked at its foot in the thumb's reach.
  kit.open(id, { direction: "column", width: grow(), height: portrait ? grow() : null, gap: u(20), cross: "center", part: "Reward.frame", partClass: "panel" });
  // Armed banishing, a refusal's shake, and the deal (when this set of rewards was first shown).
  const armed = kit.memory("~r.armed", 0) === 1 && (desc.banishes ?? -1) > 0;
  const setKey = keyHash(opts.map((o) => o.key ?? "").join("|"));
  let dealt = kit.memory("~r.dealt", 0);
  const fresh = kit.memory("~r.set", 0) !== setKey;
  if (fresh) {
    dealt = tick;
    kit.feedback("open", -1);
  }
  kit.remember("~r.set", setKey);
  kit.remember("~r.dealt", dealt);
  const rt = kit.memory("~r.rt", 0);
  const rk = kit.memory("~r.rk", 0);
  let nextArmed = armed;

  if (desc.title || desc.subtitle) {
    kit.column("Header", { width: grow(), gap: u(4), cross: "center" }, () => {
      // The screen's display line, centred over the choices (Hades' boon, Slay the Spire's reward); what to do under it.
      if (desc.title) text("Title", desc.title, { step: phone ? "xl3" : "displayMd", heading: true, font: "display", align: "center", width: grow(), maxLines: 2 });
      if (desc.subtitle) text("Subtitle", desc.subtitle, { step: st(phone ? 14 : 18), color: "inkMuted", align: "center", width: grow(), maxLines: 2 });
    });
  }

  // Sizes: side by side, each as wide as asked and as the row allows; a phone in portrait stacks compact rows.
  const room = vp.width - vp.safeArea.left - vp.safeArea.right - u(48);
  const gap = u(phone ? 12 : 24);
  const cw = portrait ? room : Math.floor(Math.min(desc.cardWidth && desc.cardWidth > 0 ? desc.cardWidth : u(260), (room - gap * Math.max(0, n - 1)) / Math.max(1, n)));
  const compact = portrait || vp.height - vp.safeArea.top - vp.safeArea.bottom < 480;

  // Its top padding keeps a card's corner tag (it stands above the card) and a hover's lift inside any clip around it.
  kit.open("Options", { direction: portrait ? "column" : "row", gap, cross: "stretch", align: "center", width: portrait ? grow() : null, padding: { left: 0, right: 0, top: u(22), bottom: 0 }, part: "Reward.options", partClass: "panel" });
  opts.forEach((o, i) => {
    const key = o.key ?? `r${i}`;
    const tier = rarityColor(o.rarity);
    const edge = tier ?? role("lineStrong");
    const sel = selected.has(key);
    const ix = lastInteraction(key);
    const shaking = rt > 0 && tick - rt < 18 && rk === keyHash(key) && !still();
    const shake = shaking ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.6 : 0;
    const lift = (ix.hovered || ix.focused) && !o.locked ? -u(10) : 0;
    const high = rarityPips(o.rarity) >= 4;
    // Said after the card closes: surface() would say a pressable card enabled, and a locked one is pressed to shake.
    const sem: Api.SemanticsDesc = {
      role: confirming && picks > 1 ? "checkbox" : "radio",
      name: [o.title ?? key, o.kind ?? "", o.rarity && o.rarity !== "none" ? o.rarity : "", o.tag ?? ""].filter(Boolean).join(", "),
      description: [plainText(o.text ?? ""), o.locked ? o.lockedHint ?? "" : ""].filter(Boolean).join(". "),
      checked: sel,
      disabled: !!o.locked,
    };
    const r = surface(key, {
      part: "Reward.card",
      cls: "card",
      flags: [`rarity=${o.rarity ?? "none"}`, sel ? "selected" : "", armed ? "banishing" : "", o.locked ? "locked" : ""],
      fill: mix(role("panelRaised"), edge, 0.1),
      wash: edge,
      washBy: 0.18,
      direction: compact ? "row" : "column",
      width: portrait ? grow() : px(cw),
      padding: xy(u(compact ? 12 : 16), u(compact ? 12 : 16)),
      gap: u(compact ? 12 : 10),
      cross: compact ? "center" : "stretch",
      radius: Math.max(8, t.panel),
      selected: sel,
      selectedFill: mix(role("panelRaised"), role("focus"), 0.16),
      props: {
        transform: { y: lift, x: shake },
        motion: spring("ui"),
        // Dealt in one after another, turning face up; when one is taken it rises while the others fall away.
        ...(fresh || tick - dealt < 2 ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: u(60) }, { prop: "scaleX", value: 0.15 }], transition: spring("emphasis"), delayMs: 110 * i } } : {}),
        exit: ix.hovered || ix.focused || sel
          ? { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: -u(70) }, { prop: "scale", value: 1.12 }], transition: { kind: "tween", durationMs: 320, ease: "exit" } }
          : { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: u(80) }], transition: { kind: "tween", durationMs: 240, ease: "exit" } },
        // What a press on it would do: take it, banish it (armed, the cross drawn on it) or nothing (locked, shaded).
        states: [{ name: "fate", states: [vstate("offered"), vstate("banishing"), vstate("locked")], transitions: [] }],
        state: [{ group: "fate", state: o.locked ? "locked" : armed ? "banishing" : "offered" }],
      },
    }, () => {
      // The frame: a gradient in the tier's colour, an edge in it, a glow from epic.
      kit.setDraw({
        shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] },
        fills: [vgrad([[0, paint(mix(role("panelRaised"), edge, 0.16))], [1, paint(mix(role("panel"), edge, 0.05))]])],
        // Epic and above lit: a card side by side an even glow round it; a stacked row (a phone) a rim of the tier's light
        // just outside it over a lift shadow, so no glow pools in the gap under it.
        strokes: [line(sel ? 3 : 2, paint(sel ? role("focus") : mix(edge, role("ink"), 0.1))), ...(high && !o.locked && compact ? [line(u(3), paint(edge, 0.4), "outside")] : [line(1, token("shadow", 0.5), "outside")])],
        effects: [shadow(u(compact ? 3 : 6), u(compact ? 10 : 16), 0.45), ...(high && !o.locked && !compact ? [glow(u(14), paint(edge, 0.5), 0)] : [])],
      });
      const artW = compact ? Math.round(Math.min(u(88), cw * 0.26) * Math.min(1.4, ts)) : cw - u(32);
      artSlot("Art", "Reward.art", { texture: o.image ?? null, kind: "item", icon: o.icon, rarity: o.rarity, width: px(artW), height: px(compact ? artW : Math.round(artW * 0.62)), radius: Math.max(4, t.small) }, () => {
        // Locked: the art drained of its colour and down to about a third of its value; the lock line says why.
        if (!o.locked) return;
        kit.open("Dim", { width: grow(), height: grow(), part: "Reward.artLocked", partClass: "badge" });
        kit.setDraw({ shape: { kind: "rect", radii: [Math.max(4, t.small), Math.max(4, t.small), Math.max(4, t.small), Math.max(4, t.small)] }, fills: [{ kind: "solid", color: paint(mix(role("well"), role("inkMuted"), 0.25), 0.62) }, { kind: "solid", color: token("shadow", 0.42) }] });
        kit.close();
      });
      kit.column("Text", { width: grow(), gap: u(4), cross: compact ? "start" : "center" }, () => {
        // A compact row's tag leads its words (standing on the corner it would cover them at large text).
        if (o.tag && compact) rewardTag(o);
        if (o.kind) text("Kind", o.kind.toLocaleUpperCase(), { step: st(12), color: tier ? mix(edge, role("ink"), 0.45) : "inkMuted", nowrap: true, ellipsis: true, width: grow(), align: compact ? "start" : "center", font: "display" });
        text("Title", o.title ?? "", { step: st(compact ? 17 : 20), heading: true, font: "display", width: grow(), align: compact ? "start" : "center", maxLines: 2, ellipsis: true });
        // Diamonds big enough to count (about 8 px drawn).
        pips("Pips", o.rarity, u(11), 6);
        if (o.text) {
          kit.open("Effect", { width: grow(), direction: "column", part: "Reward.text", partClass: "panel" });
          kit.richText("Body", o.text, { step: st(14), color: token("ink"), align: compact ? "start" : "center", width: grow() });
          kit.close();
        }
        if (o.locked && o.lockedHint) {
          kit.row("Locked", { gap: u(6), cross: "center", width: grow(), align: compact ? "start" : "center" }, () => {
            symbol("Lock", "lock", u(14), role("warn"), { icon: "lock" });
            text("Hint", o.lockedHint!, { step: st(13), color: "warn", maxLines: 2, width: grow() });
          });
        }
      });
      if (o.tag && !compact) {
        rewardTag(o);
        over({ insets: { end: -u(6), top: -u(10) }, inert: true, z: 6 });
      }
      // Selected: a tick in a disc in the focus ink at the top start corner.
      if (sel) {
        kit.open("Check", { width: px(u(30)), height: px(u(30)), align: "center", cross: "center", part: "Reward.check", partClass: "badge" });
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("focus") }], strokes: [line(2, token("panelRaised"), "outside")] });
        symbol("Mark", "check", u(18), role("onPrimary"));
        kit.close();
        over({ insets: { start: -u(10), top: -u(10) }, inert: true, z: 7 });
      }
      // Banishing armed: every card shows a cross in danger ink, over a shade.
      if (armed && !o.locked) {
        kit.open("Banish", { width: px(u(44)), height: px(u(44)), align: "center", cross: "center", part: "Reward.banishMark", partClass: "badge" });
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("danger") }], strokes: [line(2, token("shadow", 0.7), "outside")], effects: [shadow(2, 6, 0.5)] });
        symbol("Mark", "cross", u(22), role("onPrimary"));
        kit.close();
        // On the art: a compact row's art is at its start, a card's at its top.
        over({ insets: compact ? { start: u(12) + Math.round(artW / 2) - u(22), centerY: true } : { centerX: true, top: u(40) }, inert: true, z: 8 });
      }
      if (o.locked) {
        kit.open("Shade", { width: grow(), height: grow() });
        kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("shadow", 0.42) }] });
        kit.close();
        over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 5 });
      }
    });
    kit.semantics(sem);
    if (ix.hovered || ix.focused) out.hovered = key;
    if (r.clicked) {
      if (o.locked) {
        out.refused = key;
        kit.feedback("error", -1);
      } else if (armed) {
        out.banished = key;
        nextArmed = false;
        kit.feedback("confirm", -1);
      } else if (!confirming) {
        out.picked = [key];
        kit.feedback("confirm", -1);
      } else {
        const s = new Set(out.selected);
        if (s.has(key)) s.delete(key);
        else {
          if (picks === 1) s.clear();
          if (s.size < picks) s.add(key);
        }
        out.selected = opts.map((x) => x.key ?? "").filter((k) => s.has(k));
        kit.feedback("select", -1);
      }
    }
  });
  kit.close();
  kit.semantics({ role: picks > 1 ? "group" : "radiogroup", name: desc.title ?? "" });
  // A refusal's shake, kept where it is read (memory is keyed under the open element: written inside Options, it was
  // never found and a locked reward did not shake).
  if (out.refused) {
    kit.remember("~r.rt", tick);
    kit.remember("~r.rk", keyHash(out.refused));
  }

  // The actions under the cards.
  const showReroll = (desc.rerolls ?? -1) >= 0;
  const showBanish = (desc.banishes ?? -1) >= 0;
  if (showReroll || showBanish || desc.skipLabel || confirming) {
    if (portrait) kit.spacer("Dock", grow());
    kit.row("Actions", { gap: u(12), cross: "center", align: "center", wrap: true, width: portrait ? grow() : null, part: "Reward.actions", partClass: "panel" }, () => {
      if (showReroll) {
        const left = desc.rerolls ?? 0;
        const label = `${desc.rerollLabel ?? "Reroll"} (${left})${desc.rerollCost ? ` · ${desc.rerollCost}` : ""}`;
        if (btn("Reroll", label, { sym: "rotate", disabled: left <= 0, part: "Reward.reroll" }).clicked) {
          out.rerolled = true;
          nextArmed = false;
        }
      }
      if (showBanish) {
        const left = desc.banishes ?? 0;
        const b = btn("Banish", `${desc.banishLabel ?? "Banish"} (${left})`, { sym: "cross", disabled: left <= 0, selected: armed, part: `Reward.banish${armed ? "[armed]" : ""}`, variant: armed ? "danger" : "secondary" });
        if (b.clicked) {
          nextArmed = !armed;
          kit.feedback("toggle", -1);
        }
      }
      if (desc.skipLabel && btn("Skip", desc.skipLabel, { variant: "quiet", part: "Reward.skip" }).clicked) out.skipped = true;
      if (confirming) {
        const ready = out.selected.length > 0 && out.selected.length <= picks;
        const label = picks > 1 ? `${desc.confirmLabel ?? "Confirm"} (${out.selected.length}/${picks})` : desc.confirmLabel ?? "Confirm";
        if (btn("Confirm", label, { variant: "primary", disabled: !ready, part: "Reward.confirm" }).clicked && ready) {
          out.picked = [...out.selected];
          kit.feedback("confirm", -1);
        }
      }
    });
  }
  if (armed && kit.back()) {
    nextArmed = false;
    kit.feedback("cancel", -1);
  }
  kit.remember("~r.armed", nextArmed ? 1 : 0);
  kit.close();
  return out;
}
