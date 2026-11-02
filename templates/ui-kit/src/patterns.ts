// The genre patterns' preview screens (docs/ui.md §11, U6d): each a game's
// screen built from one or more patterns over the kit's fixtures, with the
// game's side done as a game would do it (a move applied to the
// viewmodel, an item equipped, a pull resolved). `pat:<key>:<value>` sets
// a preview's state (the item carried, the hand fanned, the reveal step) so
// the playtests (tests/patterns*.og.ts) can draw each state.
//
// The previews' own state is the preview's, kept here between ticks: it is
// a gallery, never saved.

import { og } from "@origingame/engine";
import type { CardTarget, GridItem, HandCard, InventoryCarry, InventoryMove, PaperdollSlot, PullItem, RewardOption, Size, UiRect, World } from "@origingame/engine";

/** A fixture the preview changes: its readonly fields writable. */
type Mut<T> = { -readonly [K in keyof T]: T[K] };
type Item = Mut<GridItem>;
type Slot = Mut<PaperdollSlot>;

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

function phone(): boolean {
  const b = og.ui.frameViewport().breakpoint;
  return b === "phonePortrait" || b === "phoneLandscape";
}

/**
 * A pattern screen: a header (its title and a line under it), the body,
 * and the action bar at the foot; a scrolling column on a phone. Leaves
 * the kit's Gallery button its corner (bottom start on a desktop, the top
 * on a phone).
 */
function screen(id: string, title: string, sub: string, build: () => void, hints: Array<{ action: string; hint: string }> = []): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  // A 1080p desktop reads from further away: the header steps up (18 px is the floor there, docs/ui.md §15.2).
  const big = !small && vp.height >= 1000;
  // On a phone the kit's Gallery button takes the top end corner: the header leaves it room.
  const top = small ? t.space.sm : t.space.lg;
  const galleryW = small ? (og.ui.bounds("Back")?.width ?? 100) + t.space.sm : 0;
  // The action hints sit on the bottom edge (actionHints anchors itself), and grow with the text size.
  const bottom = small ? t.space.sm : Math.round(72 * Math.max(1, t.textScale ?? 1));
  og.ui.anchor(id, { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom), direction: "column", gap: t.space.md, padding: { left: small ? t.space.sm : t.space.xl, top, right: small ? t.space.sm : t.space.xl, bottom } }, () => {
    og.ui.column("Header", { width: grow, gap: 2, padding: { left: 0, top: 0, right: galleryW, bottom: 0 } }, () => {
      og.ui.label("Title", title, { step: small ? "xl2" : big ? "xl6" : "xl4", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
      if (sub) og.ui.label("Sub", sub, { step: small ? "sm" : big ? "xl" : "base", color: { token: "inkMuted" }, width: grow, maxLines: 2 });
    });
    og.ui.scroll("Body", { width: grow, height: grow, gap: t.space.md }, build);
    if (hints.length > 0 && !small) og.ui.actionHints("Hints", { hints });
  });
}

// ── state ───────────────────────────────────────────────────────────────

const BAG0: Item[] = [
  { key: "greatsword", name: "Ashbringer", x: 0, y: 0, w: 1, h: 3, rarity: "legendary", tag: "weapon", icon: "sword" },
  { key: "bow", name: "Windrunner Bow", x: 1, y: 0, w: 1, h: 3, rarity: "epic", tag: "weapon", charge: 0.18 },
  { key: "potions", name: "Healing Potion", x: 2, y: 0, count: 12, maxStack: 20, tag: "potion", rarity: "common" },
  { key: "mana", name: "Mana Potion", x: 3, y: 0, count: 4, maxStack: 20, tag: "potion", rarity: "uncommon", fresh: true },
  { key: "ring", name: "Signet of the Vale", x: 2, y: 1, tag: "ring", rarity: "rare", locked: true },
  { key: "helm", name: "Crown of Thorns", x: 4, y: 0, w: 2, h: 2, tag: "helmet", rarity: "rare" },
  { key: "plate", name: "Dragonscale Plate", x: 6, y: 0, w: 2, h: 3, tag: "chest", rarity: "epic", unusable: true },
  { key: "gem", name: "Flawless Ruby", x: 3, y: 1, tag: "gem", rarity: "mythic" },
  { key: "boots", name: "Boots of Haste", x: 4, y: 2, w: 2, h: 2, tag: "feet", rarity: "uncommon", rotated: false },
  { key: "scroll", name: "Town Portal", x: 2, y: 3, count: 3, maxStack: 10, tag: "scroll" },
];

const STASH0: Item[] = [{ key: "ore", name: "Iron Ore", x: 0, y: 0, count: 40, maxStack: 50, tag: "ore" }, { key: "amulet", name: "Amulet of Dawn", x: 1, y: 0, tag: "amulet", rarity: "legendary" }];

// Two columns flanking the figure (Diablo IV, Baldur's Gate 3): armour and jewellery at the end, the hands at the start.
const SLOTS0: Slot[] = [
  { key: "relic", label: "Relic", x: 0.11, y: 0.1, locked: true, lockedHint: "Unlocks at level 30" },
  { key: "hands", label: "Hands", accept: ["gloves"], x: 0.11, y: 0.32 },
  { key: "chest", label: "Chest", accept: ["chest"], x: 0.11, y: 0.54, item: { key: "robe", name: "Silk Robe", rarity: "uncommon", tag: "chest" } },
  { key: "main", label: "Main hand", accept: ["weapon"], x: 0.11, y: 0.81, aspect: 0.6, size: 0, item: { key: "dagger", name: "Rusty Dagger", rarity: "common", tag: "weapon", charge: 0.5 } },
  { key: "head", label: "Head", accept: ["helmet"], x: 0.89, y: 0.1 },
  { key: "neck", label: "Neck", accept: ["amulet"], x: 0.89, y: 0.3 },
  { key: "ring1", label: "Ring", accept: ["ring"], x: 0.89, y: 0.5 },
  { key: "off", label: "Off hand", accept: ["weapon", "shield"], x: 0.89, y: 0.71, aspect: 0.6 },
  { key: "feet", label: "Feet", accept: ["feet"], x: 0.89, y: 0.91 },
];

/** What an item of each tag would change against what is worn in its slot: the compare column's deltas. */
const COMPARE: Record<string, Partial<Record<string, number>>> = {
  helmet: { Armor: 48, Strength: 12, "Fire resistance": -6 },
  weapon: { Strength: 32, Cooldowns: -0.4 },
  chest: { Armor: 160, "Move speed": -5 },
  feet: { Armor: 22, "Move speed": 10 },
  ring: { Strength: 8, "Fire resistance": 4 },
};

const p = {
  bag: BAG0.map((i) => ({ ...i })),
  stash: STASH0.map((i) => ({ ...i })),
  slots: SLOTS0.map((s) => ({ ...s })),
  carry: null as InventoryCarry | null,
  selected: "helm",
  highlight: "",
  pick: null as { key: string; count: number } | null,
  menu: null as { key: string; at: { x: number; y: number } } | null,
  log: "",
};

// ── cards: a combat table and its hand ──────────────────────────────────

const HAND0: HandCard[] = [
  { key: "strike", name: "Strike", cost: 1, typeLine: "Attack", text: "Deal [b]6[/b] damage.", needsTarget: true, rarity: "common" },
  { key: "defend", name: "Defend", cost: 1, typeLine: "Skill", text: "Gain [b]5[/b] [color=info]Block[/color].", rarity: "common" },
  { key: "bash", name: "Bash", cost: 2, typeLine: "Attack", text: "Deal [b]8[/b] damage. Apply 2 [color=warn]Vulnerable[/color].", needsTarget: true, rarity: "rare" },
  { key: "anger", name: "Anger", cost: 0, typeLine: "Attack", text: "Deal [b]6[/b] damage. Add a copy to your discard.", needsTarget: true, rarity: "uncommon", glow: true },
  { key: "inflame", name: "Inflame", cost: 1, costDown: true, typeLine: "Power", text: "Gain [b]2[/b] [color=danger]Strength[/color].", rarity: "epic" },
  { key: "wound", name: "Wound", typeLine: "Status", text: "Unplayable.", playable: false },
];

/** An enemy as the game holds it: its health, the health its bar's ghost still shows (falling to it after a hit), and its intent. */
interface Foe { key: string; name: string; hp: number; max: number; taunt: boolean; ghost: number; hit: number; intent: "attack" | "defend"; intentValue: number }
const FOES0: Foe[] = [
  { key: "cultist", name: "Cultist", hp: 48, max: 50, taunt: false, ghost: 48, hit: 0, intent: "attack", intentValue: 6 },
  { key: "louse", name: "Red Louse", hp: 12, max: 15, taunt: true, ghost: 12, hit: 0, intent: "defend", intentValue: 5 },
];

const c = {
  hand: HAND0.map((x) => ({ ...x })) as Array<Mut<HandCard>>,
  foes: FOES0.map((x) => ({ ...x })),
  energy: 3,
  turn: true,
  inspect: "",
  log: "",
  /** The enemies wear real art (`pat:art:on`). */
  art: false,
  /** A foe that refused a card (behind a taunt) and how many ticks its flash has left. */
  refused: "",
  refusedT: 0,
};

/** A path in a `size` box, its points given as fractions of it. */
function marks(size: number, pts: Array<[number, number]>, filled: boolean): { kind: "path"; points: Array<{ x: number; y: number }>; closed: boolean; filled: boolean; width: number; cap: "round"; join: "round" } {
  return { kind: "path", points: pts.map(([x, y]) => ({ x: x * size, y: y * size })), closed: filled, filled, width: Math.max(1.5, size * 0.13), cap: "round", join: "round" };
}

/** The kit's intent and taunt symbols, drawn (a style's icon set may lack them). */
const SWORD: Array<[number, number]> = [[0.18, 0.82], [0.8, 0.2], [0.5, 0.5], [0.34, 0.5], [0.5, 0.66]];
const SHIELD: Array<[number, number]> = [[0.5, 0.04], [0.9, 0.18], [0.84, 0.6], [0.5, 0.96], [0.16, 0.6], [0.1, 0.18]];

/** The End turn action (E, the pad's Y), defined once in each world the table is shown in. */
const endTurnDefined = new WeakSet<object>();

/** The combat table: two enemies (the second behind a taunt, so it cannot be targeted), the energy, the piles, and the hand. */
/** `pat:art:on` gives the enemies the kit's backdrop as their portraits (the art path); else the style's placeholder. */
export function cardsScreen(backdrop = 0, world?: World): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  const portrait = vp.breakpoint === "phonePortrait";
  const landscape = vp.breakpoint === "phoneLandscape";
  const W = vp.width - sa.left - sa.right;
  const Hs = vp.height - sa.top - sa.bottom;
  const big = !small && vp.height >= 1000;
  const ts = Math.max(1, t.textScale ?? 1);
  if (world && !endTurnDefined.has(world)) {
    endTurnDefined.add(world);
    og.input.defineAction(world, { name: "endTurn", label: "End turn", bindings: [{ control: "KeyE" }, { control: "pad.y" }] });
  }
  // On a phone the kit's Gallery button takes the top end corner (the start one is the web shell's Back): the header leaves it room.
  const galleryW = small ? (og.ui.bounds("Back")?.width ?? 100) + t.space.sm : 0;
  // The table stops above the foot (its height of last tick): the enemies never stand behind the hand.
  const footH = og.ui.bounds("Foot")?.height ?? (small ? 260 : 320);
  const tablePath = landscape ? "Cards/Top" : "Cards/Table";
  // The enemies, as the game would project its units: their plates of last tick are the hand's targets.
  const targets: CardTarget[] = c.foes.map((f) => ({ key: f.key, name: f.name, rect: og.ui.bounds(`${tablePath}/Foes/${f.key}/Plate`) ?? { x: -1000, y: -1000, width: 0, height: 0 } as UiRect, valid: !f.taunt }));
  // A hit's ghost holds a moment, then falls to the health left.
  for (const f of c.foes) {
    if (f.ghost < f.hp) f.ghost = f.hp;
    if (f.ghost > f.hp) {
      if (f.hit > 0) f.hit -= 1;
      else f.ghost = Math.max(f.hp, f.ghost - Math.max(0.15, (f.ghost - f.hp) * 0.08));
    }
  }
  const pad = small ? t.space.xs : t.space.sm;
  const frameW = 3;
  // The plates: as wide as the table allows (two side by side on a phone in portrait), the portrait as tall as the
  // table's height leaves once the intent, the name and the health have theirs.
  const plateW = portrait ? Math.floor((W - 2 * t.space.sm - t.space.md) / 2) : landscape ? 150 : big ? 260 : 196;
  const tableH = og.ui.bounds(tablePath)?.height ?? 0;
  const chrome = Math.round((small ? 92 : big ? 128 : 108) * ts);
  const figMax = plateW - 2 * frameW;
  const figH = landscape ? 0 : Math.round(Math.max(56, Math.min(figMax, tableH > 0 ? tableH - chrome : portrait ? 150 : figMax)));
  const plate = (f: Foe): void => {
    const flash = c.refused === f.key && c.refusedT > 0;
    og.ui.column(f.key, { width: px(plateW), gap: t.space.xs, cross: "center" }, () => {
      // The intent above the plate: what it will do on its turn, its symbol and its number.
      og.ui.row("Intent", { gap: t.space.xxs, cross: "center", padding: { left: t.space.sm, top: 2, right: t.space.sm, bottom: 2 }, radius: 999, background: { token: "shadow", alpha: 0.72 } as never }, () => {
        const z = Math.round((small ? 16 : big ? 22 : 18) * ts);
        og.ui.column("Mark", { width: px(z), height: px(z) }, () => {});
        og.ui.setDraw({ shape: marks(z, f.intent === "attack" ? SWORD : SHIELD, f.intent !== "attack"), fills: [{ kind: "solid", color: { token: f.intent === "attack" ? "danger" : "info" } }] } as never);
        og.ui.label("Value", `${f.intentValue}`, { step: small ? "base" : big ? "xl" : "lg", font: "display", color: { token: f.intent === "attack" ? "danger" : "info" }, nowrap: true });
      });
      // The plate: a frame round the portrait, full-bleed, the name on a scrim over its foot; the health inset under it.
      og.ui.column("Plate", { width: grow, gap: 0, padding: { left: frameW, top: frameW, right: frameW, bottom: pad }, radius: t.radius.panel }, () => {
        og.ui.setDraw({
          shape: { kind: "rect", radii: [t.radius.panel, t.radius.panel, t.radius.panel, t.radius.panel] },
          fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: { token: "panelRaised" } }, { offset: 1, color: { token: "panel" } }] }],
          strokes: [{ width: flash ? 3 : 1.5, align: "inside", paint: { kind: "solid", color: { token: flash ? "danger" : f.taunt ? "lineStrong" : "line" } } }, { width: 1, align: "outside", paint: { kind: "solid", color: { token: "shadow", alpha: 0.8 } } }],
          effects: [{ kind: "dropShadow", offset: { x: 0, y: 6 }, blur: 16, spread: 0, color: { token: "shadow", alpha: 0.55 } }],
        } as never);
        const ir = Math.max(2, t.radius.panel - frameW);
        if (figH > 0) {
          og.ui.artSlot("Figure", { kind: "portrait", texture: c.art && backdrop ? (backdrop as never) : null, focal: { x: 0.5, y: 0.4 }, width: grow, height: px(figH), radius: ir, label: f.name });
          // The name over the portrait's foot, on a scrim that keeps it legible on any art.
          const nameH = Math.round((small ? 30 : big ? 44 : 36) * ts);
          og.ui.column("Name", { width: px(plateW - 2 * frameW), height: px(Math.min(figH, nameH + 14)), align: "end", cross: "center", padding: { left: t.space.xs, top: 0, right: t.space.xs, bottom: t.space.xxs } }, () => {
            og.ui.setDraw({ shape: { kind: "rect", radii: [0, 0, ir, ir] }, fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: { token: "shadow", alpha: 0 } }, { offset: 0.55, color: { token: "shadow", alpha: 0.72 } }, { offset: 1, color: { token: "shadow", alpha: 0.9 } }] }] } as never);
            og.ui.label("Text", f.name, { step: small ? "base" : big ? "xl2" : "lg", font: "display", maxLines: 2, ellipsis: false, width: grow, align: "center" });
          });
          og.ui.props({ position: "absolute", insets: { start: frameW, top: frameW + figH - Math.min(figH, nameH + 14) } } as never);
        } else og.ui.label("NameText", f.name, { step: "base", font: "display", maxLines: 2, width: grow, align: "center" });
        og.ui.column("Health", { width: grow, gap: 2, cross: "center", padding: { left: pad, top: pad, right: pad, bottom: 0 } }, () => {
          // The bar inset in a dark track with a 1 px edge; behind the health, the ghost of a recent hit falling to it.
          const barW = plateW - 2 * frameW - 2 * pad;
          const barH = Math.round((small ? 8 : 10) * Math.min(1.5, ts));
          og.ui.row("Bar", { width: px(barW), height: px(barH), radius: 3 }, () => {
            og.ui.setDraw({ shape: { kind: "rect", radii: [3, 3, 3, 3] }, fills: [{ kind: "solid", color: { token: "shadow", alpha: 0.85 } }], strokes: [{ width: 1, align: "outside", paint: { kind: "solid", color: { token: "shadow" } } }] } as never);
            const inner = barW - 2;
            og.ui.column("Ghost", { width: px(Math.max(0, Math.round((inner * f.ghost) / f.max))), height: px(barH - 2), radius: 2, background: { token: "warn", alpha: 0.85 } as never }, () => {});
            og.ui.props({ position: "absolute", insets: { start: 1, top: 1 } } as never);
            og.ui.column("Fill", { width: px(Math.max(0, Math.round((inner * f.hp) / f.max))), height: px(barH - 2), radius: 2 }, () => {});
            // An enemy's health in the danger red, a light along its top so it reads as a filled tube.
            og.ui.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: { token: "danger" } }, { kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: { token: "sheen", alpha: 0.45 } }, { offset: 0.5, color: { token: "sheen", alpha: 0 } }] }] } as never);
            og.ui.props({ position: "absolute", insets: { start: 1, top: 1 } } as never);
          });
          og.ui.semantics({ role: "meter", name: `${f.name} health`, value: `${f.hp} / ${f.max}` } as never);
          og.ui.label("HpText", `${f.hp} / ${f.max}`, { step: small ? "sm" : big ? "lg" : "base", font: "mono", nowrap: true });
          if (f.taunt) {
            // Behind a taunt: a shield chip, lit in danger ink while a card it refused shakes.
            og.ui.row("Taunt", { gap: t.space.xxs, cross: "center", align: "center", padding: { left: t.space.xs, top: 2, right: t.space.sm, bottom: 2 }, radius: 999, background: { token: flash ? "danger" : "well", alpha: flash ? 0.25 : 1 } as never }, () => {
              const sh = Math.round((small ? 14 : big ? 20 : 16) * ts);
              og.ui.column("Shield", { width: px(sh), height: px(sh) }, () => {});
              og.ui.setDraw({ shape: marks(sh, SHIELD, true), fills: [{ kind: "solid", color: { token: flash ? "danger" : "inkMuted" } }] } as never);
              og.ui.label("Note", "Behind a taunt", { step: small ? "xs" : big ? "base" : "sm", color: { token: flash ? "danger" : "inkMuted" }, maxLines: 2 });
            });
          }
        });
      });
    });
  };
  const foes = (): void => {
    og.ui.row("Foes", { gap: small ? t.space.md : t.space.xl, cross: "end" }, () => {
      for (const f of c.foes) plate(f);
    });
  };
  if (c.refusedT > 0) c.refusedT -= 1;
  const energy = (): void => {
    og.ui.row("Energy", { gap: t.space.xs, cross: "center", padding: { left: t.space.md, top: t.space.xs, right: t.space.md, bottom: t.space.xs }, radius: 999, background: { token: "panelRaised" } }, () => {
      og.ui.label("Value", `${c.energy} / 3`, { step: small ? "lg" : "xl2", font: "mono", color: { token: "highlight" }, nowrap: true });
      og.ui.label("Label", "Energy", { step: big ? "lg" : "sm", color: { token: "inkMuted" }, nowrap: true });
    });
  };
  const endTurn = (): void => {
    // The screen's primary action, raised, half again the energy counter's width; E or the pad's Y on a desktop.
    const ew = og.ui.bounds(portrait ? "Cards/Top/Energy" : "Foot/Bar/Energy")?.width ?? 120;
    const pressed = og.ui.heroButton("EndTurn", c.turn ? "End turn" : "Waiting…", { tone: "primary", size: "large", disabled: !c.turn, key: "E", width: px(Math.round(Math.max(ew * 1.5, (small ? 132 : 168) * ts))) }).clicked;
    if (c.turn && (pressed || (world && og.input.actionPressed(world, "endTurn")))) c.turn = false;
  };
  og.ui.anchor("Cards", { anchor: "topLeft", margin: 0, width: px(W), height: px(Hs), direction: "column", gap: small ? t.space.sm : 0, padding: { left: small ? t.space.sm : t.space.xl, top: small ? t.space.sm : t.space.lg, right: small ? t.space.sm : t.space.xl, bottom: Math.round(footH + t.space.sm) } }, () => {
    og.ui.row("Top", { width: grow, gap: t.space.md, cross: "center", padding: { left: 0, top: 0, right: galleryW, bottom: 0 } }, () => {
      og.ui.column("Heading", { gap: 0, width: portrait ? grow : null }, () => {
        og.ui.label("Title", "Combat", { step: small ? "xl2" : "xl4", heading: true, font: "display", nowrap: true });
        og.ui.label("Turn", c.turn ? "Your turn" : "Enemy turn", { step: small ? "sm" : big ? "xl" : "base", color: { token: c.turn ? "success" : "warn" }, nowrap: true });
      });
      // On a phone in portrait the energy stands in the header, by the title (under it at a large text size).
      if (portrait && ts <= 1.25) energy();
      if (landscape) {
        og.ui.spacer("Gap", grow);
        foes();
      }
    });
    if (portrait && ts > 1.25) og.ui.row("Status", { width: grow }, energy);
    // The enemies in the table's middle (a phone's upper half), standing on its foot on a desktop.
    if (!landscape) og.ui.row("Table", { width: grow, height: grow, align: portrait ? "center" : "end", cross: portrait ? "center" : "end", gap: small ? t.space.md : t.space.xl3, padding: { left: 0, top: 0, right: small ? 0 : t.space.xl3, bottom: 0 } }, foes);
  });
  // The hand along the foot. On a desktop and a phone on its side the energy and End turn stand on a bar over it; on a
  // phone in portrait End turn alone, at the thumb's end, over the band a lifted card rises into.
  og.ui.anchor("Foot", { anchor: "bottom", margin: 0, width: px(W), direction: "column", gap: t.space.xs, padding: { left: small ? t.space.xs : t.space.lg, top: 0, right: small ? t.space.xs : t.space.lg, bottom: small ? t.space.xs : t.space.md } }, () => {
    og.ui.row("Bar", { width: grow, cross: "center", gap: t.space.md, padding: { left: portrait ? 0 : 0, top: 0, right: portrait ? t.space.sm : 0, bottom: 0 } }, () => {
      if (!portrait) energy();
      og.ui.spacer("Gap", grow);
      endTurn();
    });
    if (portrait) og.ui.spacer("Lift", px(liftClear(W - 2 * t.space.xs, Hs, c.hand.length, ts)));
    const r = og.ui.cardHand("Hand", {
      cards: c.hand.map((x) => ({ ...x, playable: x.playable !== false && (x.cost ?? 0) <= c.energy })),
      targets,
      disabled: !c.turn,
      inspect: c.inspect,
      label: "Hand",
    });
    if (r.played) {
      const card = c.hand.find((x) => x.key === r.played!.key)!;
      c.energy -= card.cost ?? 0;
      c.hand.splice(c.hand.indexOf(card), 1);
      const foe = c.foes.find((f) => f.key === r.played!.target);
      if (foe) {
        foe.ghost = Math.max(foe.ghost, foe.hp);
        foe.hp = Math.max(0, foe.hp - 6);
        foe.hit = 30;
      }
      c.log = `${card.name}${foe ? ` → ${foe.name}` : ""}`;
    }
    // Refused on a target (a foe behind a taunt): that foe flashes while the card shakes.
    if (r.refused && r.aiming?.target) {
      c.refused = r.aiming.target;
      c.refusedT = 36;
    }
    if (r.inspect) c.inspect = r.inspect;
    if (r.dismissed) c.inspect = "";
  });
}

/**
 * How far a lifted card rises over the fan on a phone, as og.ui.cardHand
 * sizes it (a phone's 96 px card narrowed to fit the hand, lifted to 168 or
 * more and grown with the text): the band End turn keeps clear of.
 */
function liftClear(fanW: number, roomH: number, n: number, ts: number): number {
  const w0 = Math.min(96, fanW * 0.34, (roomH * 0.3) / 1.4);
  const w = Math.round(n > 1 ? Math.max(Math.min(w0, 80), Math.min(w0, (fanW * 0.9) / (1 + 0.62 * (n - 1)))) : w0);
  const ch = Math.round(w * 1.4);
  const liftW = Math.round(Math.min(Math.max(w * 1.45, 168) * Math.min(1.35, ts), fanW * 0.62, (roomH * 0.62) / 1.4));
  const fanH = ch + 8 + Math.round(Math.max(26, w * 0.22) * 0.3 * Math.min(1.5, ts)) + 14 + 6;
  return Math.max(0, Math.round(liftW * 1.4) + 12 - fanH);
}

// ── rewards: a run's reward after a fight ───────────────────────────────

const OFFER0: RewardOption[] = [
  { key: "thunder", title: "Thunder Strike", kind: "Boon of Zeus", text: "Your attacks call down [b]bolts[/b] for [color=highlight]+30[/color] damage.", rarity: "rare", icon: "bolt" },
  { key: "vigor", title: "Heart of the Titan", kind: "Boon of Ares", text: "[b]+25 %[/b] Max Health; heal [color=success]10[/color] on each room cleared.", rarity: "epic", tag: "Upgrade", icon: "heart" },
  { key: "crown", title: "Crown of Ages", kind: "Relic", text: "Start each fight with [b]2[/b] extra [color=info]Energy[/color].", rarity: "legendary", tag: "New", locked: true, lockedHint: "Clear the second boss to unlock", icon: "crown" },
];
const OFFER1: RewardOption[] = [
  { key: "frost", title: "Frostbite", kind: "Boon of Demeter", text: "Hits [color=info]chill[/color] foes, slowing them by [b]20 %[/b].", rarity: "uncommon", icon: "shield" },
  { key: "gold", title: "Midas Touch", kind: "Boon of Hermes", text: "Gain [b]40[/b] gold now and [b]5[/b] per kill.", rarity: "rare", icon: "coin" },
  { key: "blade", title: "Stygian Edge", kind: "Weapon", text: "Your dash leaves a blade that deals [b]18[/b] damage.", rarity: "epic", tag: "Lv 3 → 4", icon: "sword" },
];

const r = {
  offer: OFFER0.map((o) => ({ ...o })),
  selected: [] as string[],
  rerolls: 2,
  banishes: 1,
  picks: 1,
  taken: "",
};

/** The reward after a fight (Hades' boons, Slay the Spire's card reward): three rewards, Reroll, Banish and Skip. */
export function rewardsScreen(): void {
  const vp = og.ui.frameViewport();
  const two = r.picks > 1;
  screen("Rewards", "Chamber cleared", r.taken ? `Taken: ${r.taken}` : two ? "Choose two boons, then confirm" : "Choose a boon", () => {
    const res = og.ui.rewardChoice("Choice", {
      options: r.offer,
      title: "",
      picks: r.picks,
      selected: r.selected,
      rerolls: r.rerolls,
      rerollCost: "50 gold",
      banishes: r.banishes,
      skipLabel: "Skip · take 25 gold",
      cardWidth: vp.height >= 1000 ? 340 : 0,
    });
    r.selected = [...res.selected];
    if (res.picked.length > 0) {
      r.taken = res.picked.map((k) => r.offer.find((o) => o.key === k)?.title ?? k).join(", ");
      r.selected = [];
    }
    if (res.rerolled && r.rerolls > 0) {
      r.rerolls -= 1;
      r.offer = (r.offer[0]?.key === "thunder" ? OFFER1 : OFFER0).map((o) => ({ ...o }));
      r.selected = [];
    }
    if (res.banished && r.banishes > 0) {
      r.banishes -= 1;
      r.offer = r.offer.filter((o) => o.key !== res.banished);
    }
    if (res.skipped) r.taken = "25 gold";
  }, [{ action: "confirm", hint: "Take" }, { action: "back", hint: "Back" }]);
}

// ── gacha: a banner and its pull ────────────────────────────────────────

const PULL1: PullItem[] = [{ key: "aria", name: "Aria, Tidecaller", kind: "Character", stars: 5, rarity: "legendary", isNew: true }];
const PULL10: PullItem[] = [
  { key: "p0", name: "Favonius Sword", kind: "Weapon", stars: 4, rarity: "epic" },
  { key: "p1", name: "Iron Sting", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p2", name: "Slingshot", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p3", name: "Kael, the Ember", kind: "Character", stars: 4, rarity: "epic", extra: "+2 Starglitter" },
  { key: "p4", name: "Debate Club", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p5", name: "Aria, Tidecaller", kind: "Character", stars: 5, rarity: "legendary", isNew: true },
  { key: "p6", name: "Black Tassel", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p7", name: "Thrilling Tales", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p8", name: "Mira of the Grove", kind: "Character", stars: 4, rarity: "epic", isNew: true },
  { key: "p9", name: "Cool Steel", kind: "Weapon", stars: 3, rarity: "rare" },
];

const g = {
  banner: "event",
  balance: 2400,
  pity: 74,
  token: 0,
  items: [] as PullItem[],
  log: "",
};

const BANNERS = [
  { key: "event", name: "Moonlit Tide", title: "Moonlit Tide", subtitle: "Event wish · Character", endsIn: 3 * 86400 + 5 * 3600, fresh: false },
  { key: "weapon", name: "Epitome", title: "Epitome Invocation", subtitle: "Event wish · Weapon", endsIn: 50 * 60, fresh: true },
  { key: "std", name: "Wanderlust", title: "Wanderlust Invocation", subtitle: "Standard wish", endsIn: null, fresh: false },
] as const;

/** A gacha banner and its pull: the pull is the game's (it spends the purse and draws the results) and the reveal the pattern's. */
/** The weapon banner wears the kit's backdrop (the art path); the others draw the style's placeholder. */
/** The rate lines are the game's strings (og.i18n), so an RTL locale shows its own sentence. */
export function gachaScreen(backdrop = 0, rate: { event: string; standard: string } = { event: "", standard: "" }): void {
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  const b = BANNERS.find((x) => x.key === g.banner) ?? BANNERS[0];
  og.ui.anchor("Gacha", { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom), direction: "column", padding: { left: 0, top: small ? Math.round((og.ui.bounds("Back")?.height ?? 44) + og.ui.frameTheme().space.sm) : 0, right: 0, bottom: small ? 0 : 52 } }, () => {
    const res = og.ui.gachaBanner("Banner", {
      banners: BANNERS.map((x) => ({ key: x.key, name: x.name, endsIn: x.endsIn, fresh: x.fresh })),
      current: b.key,
      title: b.title,
      subtitle: b.subtitle,
      image: b.key === "weapon" && backdrop ? (backdrop as never) : null,
      featured: b.key === "std" ? [] : [{ name: "Aria", title: "Tidecaller of the Moon", stars: 5, rarity: "legendary" }, { name: "Kael", title: "The Ember", stars: 4, rarity: "epic" }, { name: "Mira", title: "Of the Grove", stars: 4, rarity: "epic" }],
      rateText: b.key === "std" ? rate.standard : rate.event,
      endsIn: b.endsIn,
      pity: g.pity,
      pityMax: 90,
      balance: g.balance,
      single: 160,
      multi: 1600,
      currency: "gem",
    }, () => {});
    if (res.banner) g.banner = res.banner;
    if (res.pull > 0 && g.balance >= res.pull * 160) {
      g.balance -= res.pull * 160;
      g.pity = (g.pity + res.pull) % 90;
      g.items = (res.pull === 1 ? PULL1 : PULL10).map((x) => ({ ...x }));
      g.token += 1;
    }
    if (res.short) g.log = `short ${res.short}`;
  });
  const seq = og.ui.pullSequence("Pull", { token: g.items.length > 0 ? g.token : 0, items: g.items, againPrice: g.items.length * 160, affordable: g.balance >= g.items.length * 160, againLabel: "Wish ×{n}" });
  if (seq.again && g.balance >= g.items.length * 160) {
    g.balance -= g.items.length * 160;
    g.token += 1;
  }
  if (seq.done) g.items = [];
}

/** `pat:<key>:<value>`: a pattern preview's state (pick:<item>, highlight:<tag>, reset). */
export function patternsCommand(_world: World, args: readonly string[]): void {
  const [key, value = ""] = args;
  if (key === "pick") p.pick = { key: value, count: 0 };
  else if (key === "highlight") p.highlight = value;
  else if (key === "select") p.selected = value;
  else if (key === "inspect") c.inspect = value;
  else if (key === "turn") c.turn = value !== "off";
  else if (key === "energy") c.energy = Number(value) || 0;
  else if (key === "picks") {
    r.picks = Number(value) || 1;
    r.selected = [];
  } else if (key === "rselect") r.selected = value ? value.split(",") : [];
  else if (key === "art") c.art = value === "on";
  else if (key === "balance") g.balance = Number(value) || 0;
  else if (key === "banner") g.banner = value;
  else if (key === "pull") {
    // A pull resolved by the game: pull:1 or pull:10 starts its reveal.
    g.items = (value === "1" ? PULL1 : PULL10).map((x) => ({ ...x }));
    g.token += 1;
  } else if (key === "draw") {
    // A card drawn: it flies in from the start edge.
    const k = `${value || "strike"}${c.hand.length}`;
    c.hand.push({ ...(HAND0.find((x) => x.key === (value || "strike")) ?? HAND0[0]!), key: k, fresh: true });
  } else if (key === "reset") {
    c.hand = HAND0.map((x) => ({ ...x }));
    c.foes = FOES0.map((x) => ({ ...x }));
    c.energy = 3;
    c.turn = true;
    c.inspect = "";
    c.refused = "";
    c.refusedT = 0;
    c.art = false;
    p.bag = BAG0.map((i) => ({ ...i }));
    p.slots = SLOTS0.map((s) => ({ ...s }));
    p.stash = STASH0.map((i) => ({ ...i }));
    p.carry = null;
    p.pick = null;
    p.selected = "helm";
    p.highlight = "";
    p.menu = null;
    r.offer = OFFER0.map((o) => ({ ...o }));
    r.selected = [];
    r.rerolls = 2;
    r.banishes = 1;
    r.picks = 1;
    r.taken = "";
    g.banner = "event";
    g.balance = 2400;
    g.pity = 74;
    g.items = [];
  }
}

/** The game's side of a move: what an inventory does with InventoryGridResult.move. */
function applyMove(list: Item[], from: Item[] | null, m: InventoryMove, fromSlot: Slot | null): void {
  const src = (fromSlot ? fromSlot.item ?? null : (from ?? list).find((i) => i.key === m.key) ?? null) as Item | null;
  if (!src) return;
  const moved: Item = { ...src, x: m.x, y: m.y, rotated: m.rotated, count: m.count };
  if (m.mergeInto) {
    const into = list.find((i) => i.key === m.mergeInto)!;
    const room = (into.maxStack ?? 1) - (into.count ?? 1);
    const n = Math.min(room, m.count);
    into.count = (into.count ?? 1) + n;
    if (fromSlot) fromSlot.item = null;
    else if ((src.count ?? 1) - n <= 0) (from ?? list).splice((from ?? list).indexOf(src), 1);
    else src.count = (src.count ?? 1) - n;
    return;
  }
  if (m.swapWith) {
    const other = list.find((i) => i.key === m.swapWith)!;
    if (fromSlot) {
      fromSlot.item = { ...other };
      list.splice(list.indexOf(other), 1);
    } else {
      Object.assign(other, { x: src.x, y: src.y });
      if (from && from !== list) {
        list.splice(list.indexOf(other), 1);
        from.push(other);
      }
    }
  }
  if (fromSlot) {
    fromSlot.item = m.swapWith ? (fromSlot.item ?? null) : null;
    list.push(moved);
  } else if (m.split) {
    src.count = (src.count ?? 1) - m.count;
    list.push({ ...moved, key: `${src.key}-${og.ui.tick()}` });
  } else if (from && from !== list) {
    from.splice(from.indexOf(src), 1);
    list.push(moved);
  } else Object.assign(src, { x: m.x, y: m.y, rotated: m.rotated });
}

// ── gear: paperdoll, backpack and stash ─────────────────────────────────

/** The tag the compare column reads: the item in hand, else the selected one. */
function compareTag(): string {
  if (p.carry) return p.carry.tag ?? "";
  const it = [...p.bag, ...p.stash].find((i) => i.key === p.selected);
  return it?.tag ?? "";
}

/** A line of the attributes with its delta against the worn item, in the stat's unit; a lower cooldown is better. */
function stat(label: string, value: string, unit: string, lowerIsBetter = false): { label: string; value: string; delta?: number; deltaText?: string; lowerIsBetter?: boolean } {
  const d = COMPARE[compareTag()]?.[label] ?? 0;
  return { label, value, ...(d ? { delta: d, deltaText: `${d > 0 ? "+" : "\u2212"}${Math.abs(d)}${unit}` } : {}), ...(lowerIsBetter ? { lowerIsBetter } : {}) };
}

export function gearScreen(): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const portrait = vp.breakpoint === "phonePortrait";
  screen("Gear", "Inventory", "Drag to move · Shift splits a stack · R turns", () => {
    const body = () => {
      const doll = og.ui.paperdoll("Doll", {
        slots: p.slots,
        title: "Aria Vell",
        subtitle: "Level 42 Paladin",
        rating: "812",
        ratingLabel: "Item power",
        statsTitle: "Attributes",
        figureAspect: 0.5,
        // The item in hand, else the one selected, against what is worn where it would go.
        compare: !!COMPARE[compareTag()],
        stats: [
          stat("Armor", "1,240", ""),
          stat("Strength", "312", ""),
          stat("Fire resistance", "28%", "%"),
          stat("Move speed", "100%", "%"),
          stat("Cooldowns", "8.2 s", " s", true),
        ],
        ...(p.carry ? { carry: p.carry } : {}),
      }, () => {
        // The figure: the game's character art in a body slot (the style's standing silhouette until it is set).
        og.ui.artSlot("Figure", { kind: "body", width: grow, height: grow, label: "Aria Vell" });
      });
      if (doll.equip) {
        const slot = p.slots.find((s) => s.key === doll.equip!.slot)!;
        const fromBag = p.bag.find((i) => i.key === doll.equip!.key);
        const fromStash = p.stash.find((i) => i.key === doll.equip!.key);
        const fromSlot = p.slots.find((s) => s.key === doll.equip!.fromSlot);
        const was = slot.item ?? null;
        const item = fromBag ?? fromStash ?? fromSlot?.item ?? null;
        if (item) {
          if (fromBag) p.bag.splice(p.bag.indexOf(fromBag), 1);
          if (fromStash) p.stash.splice(p.stash.indexOf(fromStash), 1);
          if (fromSlot) fromSlot.item = was && was.tag && (fromSlot.accept ?? []).includes(was.tag) ? was : null;
          slot.item = { ...item, x: 0, y: 0, rotated: false };
          if (was && !fromSlot) (fromStash ? p.stash : p.bag).push({ ...was, x: item.x ?? 0, y: item.y ?? 0, w: 1, h: 1 });
        }
        p.carry = null;
      }
      og.ui.column("Bags", { gap: t.space.sm }, () => {
        og.ui.label("BagTitle", "Backpack", { step: "lg", font: "display", heading: true });
        // Eight columns fill a phone in portrait at the finger's 44 px floor, so the grids need no sideways scroll.
        const cell = portrait ? { cellSize: Math.max(44, Math.floor((vp.width - vp.safeArea.left - vp.safeArea.right - 32) / 8)) } : {};
        const pick = p.pick;
        p.pick = null;
        const bag = og.ui.inventoryGrid("Bag", { ...cell, columns: portrait ? 8 : 10, rows: portrait ? 7 : 6, items: p.bag, label: "Backpack", selected: p.selected, highlight: p.highlight, ...(p.carry && p.carry.source !== "Bag" ? { carry: p.carry } : {}), ...(pick ? { pick } : {}) });
        og.ui.label("StashTitle", "Stash", { step: "lg", font: "display", heading: true });
        const stash = og.ui.inventoryGrid("Stash", { ...cell, columns: portrait ? 8 : 10, rows: portrait ? 2 : 3, capacity: portrait ? 12 : 24, items: p.stash, label: "Stash", ...(p.carry && p.carry.source !== "Stash" ? { carry: p.carry } : {}) });
        for (const [r, list, other] of [[bag, p.bag, p.stash], [stash, p.stash, p.bag]] as const) {
          if (r.move) {
            const fromSlot = r.move.fromSlot ? p.slots.find((s) => s.key === r.move!.fromSlot) ?? null : null;
            applyMove(list, r.move.from === (list === p.bag ? "Bag" : "Stash") ? list : fromSlot ? null : other, r.move, fromSlot);
            p.carry = null;
          }
          if (r.selected) p.selected = r.selected;
          if (r.quick) {
            const it = list.find((i) => i.key === r.quick)!;
            list.splice(list.indexOf(it), 1);
            other.push({ ...it, x: other.length % 10, y: Math.floor(other.length / 10) });
          }
          if (r.menu) p.menu = { key: r.menu, at: r.menuAt };
        }
        // What is in the hand, handed to the other places next tick; one let go (released) stays one tick more,
        // so the grid or paperdoll under the pointer (built before this one, or after) takes it, then it is gone.
        p.carry = bag.carried ?? stash.carried ?? doll.carried ?? null;
      });
      // The item's menu: a long press or the details action.
      const m = og.ui.contextMenu("ItemMenu", !!p.menu, p.menu?.at ?? { x: 0, y: 0 }, {
        entries: [{ id: "use", label: "Use" }, { id: "split", label: "Split stack" }, { id: "drop", label: "Drop" }],
      });
      if (m.chosen === "split" && p.menu) p.pick = { key: p.menu.key, count: 1 };
      if (m.chosen || m.dismissed) p.menu = null;
    };
    // Doll and grids side by side where both fit (a desktop), one above the other otherwise.
    if (vp.width >= 1100) og.ui.row("Split", { width: grow, gap: t.space.xl, cross: "start" }, body);
    else og.ui.column("Stack", { width: grow, gap: t.space.lg, cross: "start" }, body);
  }, [{ action: "confirm", hint: "Pick up / place" }, { action: "rotate", hint: "Rotate" }, { action: "details", hint: "Item menu" }, { action: "back", hint: "Put back" }]);
}
