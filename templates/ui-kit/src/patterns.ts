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
import { storyCommand } from "./story.ts";
import { genreScene, genresCommand, resetGenres } from "./genres.ts";
import { genres2Command, resetGenres2 } from "./genres2.ts";
import { genres3Command, resetGenres3 } from "./genres3.ts";
import type { CardTarget, GridItem, HandCard, InventoryCarry, InventoryMove, PaperdollSlot, PullItem, RewardOption, Size, StoreOffer, UiRect, World } from "@origingame/engine";

/** A fixture the preview changes: its readonly fields writable. */
type Mut<T> = { -readonly [K in keyof T]: T[K] };
type Item = Mut<GridItem>;
type Slot = Mut<PaperdollSlot>;

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

export function phone(): boolean {
  const b = og.ui.frameViewport().breakpoint;
  return b === "phonePortrait" || b === "phoneLandscape";
}

/**
 * A pattern screen: a header (its title and a line under it), the body,
 * and the action bar at the foot; a scrolling column on a phone. Leaves
 * the kit's Gallery button its corner (bottom start on a desktop, the top
 * on a phone).
 */
function screen(id: string, title: string, sub: string, build: () => void, hints: Array<{ action: string; hint: string }> = [], scroll = true): void {
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
  // On a phone the body runs to the screen's foot, so its fade ends at the edge rather than over a band of scene.
  const bottom = small ? 0 : Math.round(72 * Math.max(1, t.textScale ?? 1));
  og.ui.anchor(id, { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - (small ? 0 : sa.bottom)), direction: "column", gap: t.space.md, padding: { left: small ? t.space.sm : t.space.xl, top, right: small ? t.space.sm : t.space.xl, bottom } }, () => {
    // At least the Gallery button's height on a phone, so the body starts under it (a pass's header ran up into it).
    const galleryH = small ? og.ui.bounds("Back")?.height ?? 0 : 0;
    og.ui.column("Header", { width: grow, height: { kind: "fit", min: galleryH }, gap: 2, align: "center", padding: { left: 0, top: 0, right: galleryW, bottom: 0 } }, () => {
      og.ui.label("Title", title, { step: small ? "xl2" : big ? "xl6" : "xl4", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
      if (sub) og.ui.label("Sub", sub, { step: small ? "sm" : big ? "xl" : "base", color: { token: "inkMuted" }, width: grow, maxLines: 2 });
    });
    // The body fades out at the foot where more lies below, so a row cut by the screen's edge reads as more to scroll.
    // A screen whose pattern scrolls itself (a battle pass's track) takes the room as it is, so its own view scrolls.
    if (scroll) og.ui.scrollArea("Body", { width: grow, height: grow, gap: t.space.md, fade: small ? 72 : 56, fadeInto: "shadow", padding: { left: 0, top: 0, right: 0, bottom: small ? sa.bottom : 0 } }, build);
    else og.ui.column("Body", { width: grow, height: grow, gap: t.space.md, padding: { left: 0, top: 0, right: 0, bottom: small ? sa.bottom : 0 } }, build);
    if (hints.length > 0 && !small) og.ui.actionHints("Hints", { hints });
  });
}

// ── the kit's art set ───────────────────────────────────────────────────

// Painted for the kit (templates/ui-kit/assets/textures/art, docs/ui.md §15.5.3): loaded once at the start and shown by
// every preview; `pat:art:off` takes it away so the previews show the style's placeholders (the path a game without its
// art yet takes).
const ART_FILES = [
  "cards/strike", "cards/defend", "cards/bash", "cards/anger", "cards/inflame", "cards/wound",
  "portraits/cultist", "portraits/red-louse", "portraits/aria", "portraits/kael", "portraits/mira", "portraits/aria-vell",
  "scenes/gacha-moonlit-tide", "scenes/paperdoll-aria-vell", "scenes/title", "scenes/storybook-vale", "scenes/golden-valley",
  "items/ashbringer", "items/elven-bow", "items/hunters-bow", "items/healing-draught", "items/mana-vial", "items/ember-ring", "items/crown",
  "items/arcane-staff", "items/boots", "items/iron-key", "items/sapphire-amulet", "items/mage-robe", "items/iron-sword", "items/oak-shield",
  "items/gauntlets", "items/knight-helm",
  "boons/thunder-strike", "boons/titan-heart", "boons/crown-of-ages", "boons/frost", "boons/gold", "boons/flame-blade", "boons/crystal",
  "boons/star", "boons/rune-shield", "boons/mana",
  "figures/mira", "figures/aria", "figures/kael",
  "portraits/wren", "portraits/vesper", "portraits/maren", "portraits/torvin", "portraits/brann", "scenes/gothic-halls", "maps/world",
  "songs/neon-moon", "songs/sakura-wind", "songs/stormbreak", "songs/desert-drum", "songs/crystal-echo", "songs/phoenix",
  "garage/coupe", "garage/turbo", "garage/wheels", "garage/suspension", "garage/exhaust", "garage/brakes", "garage/wing", "scenes/garage",
  "loadout/raven", "loadout/sidearm", "loadout/frag", "loadout/stim", "loadout/knife", "loadout/drone", "loadout/shield", "loadout/optic",
  "loadout/suppressor", "loadout/grip", "loadout/magazine", "loadout/stock", "loadout/laser",
  "cg/riverboat", "cg/storm-ferry", "cg/dawn-gate",
  "city/residential", "city/commercial", "city/industrial", "city/tourism", "city/police", "city/fire", "city/health", "city/education",
  "city/transit", "city/parks", "city/power", "city/water", "scenes/city",
  "rts/stop", "rts/patrol", "rts/build", "rts/rally", "rts/gather", "rts/repair", "rts/barracks", "rts/tower", "rts/farm", "rts/wall",
  "rts/forge", "rts/keep", "scenes/battlefield",
  "scenes/tactical", "photo/shot", "photo/vivid", "photo/noir", "photo/sepia", "photo/cool", "photo/frame-cinema", "photo/frame-polaroid", "photo/frame-film",
  "photo/thumb-cinema", "photo/thumb-polaroid", "photo/thumb-film",
] as const;
/** The cut-outs on transparent ground (PNG); the rest is painted edge to edge (JPEG). */
const CUTOUTS: ReadonlySet<string> = new Set(["garage/coupe", "loadout/raven", "photo/frame-cinema", "photo/frame-polaroid", "photo/frame-film"]);
export type ArtName = (typeof ART_FILES)[number];
const ART: Partial<Record<ArtName, number>> = {};
let artOn = true;

/** The world whose string tables the previews read (the kit has one). */
let words: World | null = null;

/**
 * A preview's words in the active locale (og.i18n, assets/strings): every string a pattern screen shows is the game's,
 * keyed, so a locale frame has no English chrome. Banner and character names stay proper nouns.
 */
export function L(key: string, args?: Record<string, string | number>): string {
  return words ? og.i18n.t(words, key, args) : key;
}

/** Load the kit's art set (the game's start), and keep the world whose strings the previews read. */
export function loadPatternArt(world: World): void {
  words = world;
  for (const f of ART_FILES) {
    // Figures and the other cut-outs are on transparent ground (PNG); the rest is painted edge to edge (JPEG).
    const r = og.load.texture(world, `textures/art/${f}.${f.startsWith("figures/") || CUTOUTS.has(f) ? "png" : "jpg"}`);
    if (r.ok) ART[f] = r.value as unknown as number;
  }
}

/** A texture of the art set, or null (art off, or not loaded): the art slot then draws the style's placeholder. */
export function art(name: ArtName | undefined): never | null {
  if (!artOn || !name) return null;
  const h = ART[name];
  return h ? (h as never) : null;
}

/**
 * A pattern screen's scene: a painted backdrop of the art set over the whole
 * surface (under the notch too), blurred and dimmed with a vignette so the
 * screen's art and text stand in front of it; without art, a dark wash.
 */
const SCENES: Record<string, [ArtName, { x: number; y: number }]> = {
  cards: ["scenes/title", { x: 0.62, y: 0.55 }],
  rewards: ["scenes/title", { x: 0.4, y: 0.5 }],
  gacha: ["scenes/gacha-moonlit-tide", { x: 0.72, y: 0.3 }],
  gear: ["scenes/paperdoll-aria-vell", { x: 0.5, y: 0.3 }],
  board: ["scenes/title", { x: 0.5, y: 0.6 }],
  generators: ["scenes/gacha-moonlit-tide", { x: 0.3, y: 0.5 }],
  store: ["scenes/paperdoll-aria-vell", { x: 0.5, y: 0.25 }],
  pass: ["scenes/title", { x: 0.6, y: 0.4 }],
  dialogue: ["scenes/storybook-vale", { x: 0.5, y: 0.45 }],
  vn: ["scenes/golden-valley", { x: 0.5, y: 0.5 }],
  quests: ["scenes/storybook-vale", { x: 0.35, y: 0.5 }],
  lobby: ["scenes/title", { x: 0.55, y: 0.45 }],
  select: ["scenes/gothic-halls", { x: 0.5, y: 0.4 }],
  draft: ["scenes/gothic-halls", { x: 0.5, y: 0.6 }],
  garage: ["scenes/garage", { x: 0.5, y: 0.55 }],
  ledger: ["scenes/city", { x: 0.5, y: 0.6 }],
  chat: ["scenes/gothic-halls", { x: 0.45, y: 0.5 }],
  loadout: ["scenes/tactical", { x: 0.5, y: 0.5 }],
};

/** The pattern screen's painted scene, built before anything over it (the kit's Gallery button among them). */
export function patternScene(screen: string): void {
  const sc = genreScene(screen) ?? SCENES[screen];
  if (sc) scene(sc[0], sc[1]);
}

function scene(name: ArtName, focal = { x: 0.5, y: 0.5 }): void {
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const tex = art(name);
  og.ui.anchor("Scene", { anchor: "topLeft", margin: 0, offsetX: -sa.left, offsetY: -sa.top, width: px(vp.width), height: px(vp.height) }, () => {
    og.ui.column("Picture", { width: grow, height: grow }, () => {});
    og.ui.setDraw({
      shape: { kind: "rect" },
      fills: tex ? [{ kind: "image", texture: tex, fit: "cover", focal }] : [{ kind: "solid", color: { token: "backdrop" } }],
      effects: tex ? [{ kind: "layerBlur", radius: 10 }] : [],
    } as never);
    og.ui.column("Shade", { width: grow, height: grow }, () => {});
    og.ui.setDraw({
      shape: { kind: "rect" },
      fills: [
        { kind: "solid", color: { token: "shadow", alpha: 0.55 } },
        { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.45 }, to: { x: 0.85, y: 0.85 }, stops: [{ offset: 0, color: { token: "shadow", alpha: 0 } }, { offset: 1, color: { token: "shadow", alpha: 0.75 } }] },
      ],
    } as never);
    og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true } as never);
  });
  og.ui.props({ inert: true } as never);
}

const ITEM_ART: Record<string, ArtName> = {
  greatsword: "items/ashbringer", bow: "items/elven-bow", potions: "items/healing-draught", mana: "items/mana-vial", ring: "items/ember-ring",
  helm: "items/crown", plate: "items/arcane-staff", gem: "boons/star", boots: "items/boots", scroll: "items/iron-key", gloves: "items/gauntlets",
  ore: "boons/crystal", amulet: "items/sapphire-amulet", shield: "items/oak-shield", robe: "items/mage-robe", dagger: "items/iron-sword",
};
/** An item with its art and its name in the active locale (a split stack keeps its source's name). */
const withArt = <T extends { key?: string; image?: unknown; name?: string }>(it: T): T => ({ ...it, name: L(`gear.item.${(it.key ?? "").replace(/-\d+$/, "")}`), image: art(ITEM_ART[(it.key ?? "").replace(/-\d+$/, "")]) });
/** The grids' own words. */
const gridWords = () => ({ lockedLabel: L("gear.locked"), unusableLabel: L("gear.unusable"), emptyLabel: L("gear.empty"), rotateLabel: L("gear.rotate") });

// ── state ───────────────────────────────────────────────────────────────

const BAG0: Item[] = [
  { key: "greatsword", name: "Ashbringer", x: 0, y: 0, w: 1, h: 3, rarity: "legendary", tag: "weapon", icon: "sword" },
  { key: "bow", name: "Windrunner Bow", x: 1, y: 0, w: 1, h: 3, rarity: "epic", tag: "weapon", charge: 0.18 },
  { key: "potions", name: "Healing Potion", x: 2, y: 0, count: 12, maxStack: 20, tag: "potion", rarity: "common" },
  { key: "mana", name: "Mana Potion", x: 3, y: 0, count: 4, maxStack: 20, tag: "potion", rarity: "uncommon", fresh: true },
  { key: "ring", name: "Signet of the Vale", x: 2, y: 1, tag: "ring", rarity: "rare", locked: true },
  { key: "helm", name: "Crown of Thorns", x: 4, y: 0, w: 2, h: 2, tag: "helmet", rarity: "rare" },
  { key: "plate", name: "Arcane Staff", x: 6, y: 0, w: 1, h: 3, tag: "weapon", rarity: "epic", unusable: true },
  { key: "gloves", name: "Gauntlets", x: 7, y: 0, tag: "gloves", rarity: "uncommon" },
  { key: "gem", name: "Star Shard", x: 3, y: 1, tag: "gem", rarity: "mythic" },
  { key: "boots", name: "Boots of Haste", x: 4, y: 2, w: 2, h: 2, tag: "feet", rarity: "uncommon", rotated: false },
  { key: "scroll", name: "Iron Key", x: 2, y: 3, count: 3, maxStack: 10, tag: "scroll" },
];

const STASH0: Item[] = [
  { key: "ore", name: "Raw Crystal", x: 0, y: 0, count: 40, maxStack: 50, tag: "ore" },
  { key: "amulet", name: "Sapphire Amulet", x: 1, y: 0, tag: "amulet", rarity: "legendary" },
  { key: "shield", name: "Oak Shield", x: 2, y: 0, tag: "shield", rarity: "common" },
];

// Two columns flanking the figure (Diablo IV, Baldur's Gate 3), five sockets each on one grid, every socket one size:
// the hands, body and weapon at the start; armour and jewellery at the end.
const ROWS = [0.1, 0.3, 0.5, 0.7, 0.9];
const SLOTS0: Slot[] = [
  { key: "relic", label: "Relic", x: 0.11, y: ROWS[0]!, locked: true, lockedHint: "Unlocks at level 30" },
  { key: "hands", label: "Hands", accept: ["gloves"], x: 0.11, y: ROWS[1]! },
  { key: "chest", label: "Chest", accept: ["chest"], x: 0.11, y: ROWS[2]!, item: { key: "robe", name: "Mage Robe", rarity: "uncommon", tag: "chest" } },
  { key: "main", label: "Main hand", accept: ["weapon"], x: 0.11, y: ROWS[3]!, item: { key: "dagger", name: "Iron Sword", rarity: "common", tag: "weapon", charge: 0.5 } },
  { key: "ring2", label: "Ring", accept: ["ring"], x: 0.11, y: ROWS[4]! },
  { key: "head", label: "Head", accept: ["helmet"], x: 0.89, y: ROWS[0]! },
  { key: "neck", label: "Neck", accept: ["amulet"], x: 0.89, y: ROWS[1]! },
  { key: "ring1", label: "Ring", accept: ["ring"], x: 0.89, y: ROWS[2]! },
  { key: "off", label: "Off hand", accept: ["weapon", "shield"], x: 0.89, y: ROWS[3]! },
  { key: "feet", label: "Feet", accept: ["feet"], x: 0.89, y: ROWS[4]! },
];

/** What an item of each tag would change against what is worn in its slot: the compare column's deltas. */
const COMPARE: Record<string, Partial<Record<string, number>>> = {
  helmet: { armor: 48, strength: 12, fire: -6 },
  weapon: { strength: 32, cooldowns: -0.4 },
  chest: { armor: 160, speed: -5 },
  feet: { armor: 22, speed: 10 },
  ring: { strength: 8, fire: 4 },
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
export function cardsScreen(world?: World): void {
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
    og.input.defineAction(world, { name: "endTurn", label: L("cards.endTurn"), bindings: [{ control: "KeyE" }, { control: "pad.y" }] });
  }
  // On a phone the kit's Gallery button takes the top end corner (the start one is the web shell's Back): the header leaves it room.
  const galleryW = small ? (og.ui.bounds("Back")?.width ?? 100) + t.space.sm : 0;
  // The table stops above the foot (its height of last tick): the enemies never stand behind the hand.
  const footH = og.ui.bounds("Foot")?.height ?? (small ? 260 : 320);
  const tablePath = landscape ? "Cards/Top" : "Cards/Table";
  // The enemies, as the game would project its units: their plates of last tick are the hand's targets.
  const targets: CardTarget[] = c.foes.map((f) => ({ key: f.key, name: L(`cards.foe.${f.key}`), rect: og.ui.bounds(`${tablePath}/Foes/${f.key}/Plate`) ?? { x: -1000, y: -1000, width: 0, height: 0 } as UiRect, valid: !f.taunt }));
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
  const chrome = Math.round((small ? 104 : big ? 128 : 108) * ts);
  // A phone in portrait gives the portraits the table's height: up to a quarter taller than wide, as a mobile card
  // game's enemies stand.
  const figMax = Math.round((plateW - 2 * frameW) * (portrait ? 1.7 : 1));
  // Where the table has no room for a portrait (a phone on its side, a phone at 200 % text) the plates stand on their
  // names alone rather than run under the hand's bar.
  const room = tableH > 0 ? tableH - chrome : portrait ? 150 : figMax;
  const figH = landscape || room < 64 ? 0 : Math.round(Math.min(figMax, room));
  const plate = (f0: Foe): void => {
    const f = { ...f0, name: L(`cards.foe.${f0.key}`) };
    const flash = c.refused === f.key && c.refusedT > 0;
    og.ui.column(f.key, { width: px(plateW), gap: t.space.xs, cross: "center" }, () => {
      // The intent above the plate: what it will do on its turn, its symbol and its number. A compact plate (no room
      // for a portrait: a phone at a large text size) stands without it above, so it fits the table.
      if (figH > 0 || !small) og.ui.row("Intent", { gap: t.space.xxs, cross: "center", padding: { left: t.space.sm, top: 2, right: t.space.sm, bottom: 2 }, radius: 999, background: { token: "shadow", alpha: 0.72 } as never }, () => {
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
          og.ui.artSlot("Figure", { kind: "portrait", texture: art(f.key === "cultist" ? "portraits/cultist" : "portraits/red-louse"), focal: { x: 0.5, y: 0.35 }, width: grow, height: px(figH), radius: ir, label: f.name });
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
          og.ui.semantics({ role: "meter", name: L("cards.health", { name: f.name }), value: `${f.hp} / ${f.max}` } as never);
          og.ui.label("HpText", `${f.hp} / ${f.max}`, { step: small ? "sm" : big ? "lg" : "base", font: "mono", nowrap: true });
          if (f.taunt) {
            // Behind a taunt: a shield chip, lit in danger ink while a card it refused shakes.
            og.ui.row("Taunt", { gap: t.space.xxs, cross: "center", align: "center", padding: { left: t.space.xs, top: 2, right: t.space.sm, bottom: 2 }, radius: 999, background: { token: flash ? "danger" : "well", alpha: flash ? 0.25 : 1 } as never }, () => {
              const sh = Math.round((small ? 14 : big ? 20 : 16) * ts);
              og.ui.column("Shield", { width: px(sh), height: px(sh) }, () => {});
              og.ui.setDraw({ shape: marks(sh, SHIELD, true), fills: [{ kind: "solid", color: { token: flash ? "danger" : "inkMuted" } }] } as never);
              if (figH === 0 && small) og.ui.semantics({ role: "image", name: L("cards.taunt") } as never);
              else og.ui.label("Note", L("cards.taunt"), { step: small ? "xs" : big ? "base" : "sm", color: { token: flash ? "danger" : "inkMuted" }, maxLines: 2 });
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
      og.ui.label("Label", L("cards.energy"), { step: big ? "lg" : "sm", color: { token: "inkMuted" }, nowrap: true });
    });
  };
  const endTurn = (): void => {
    // The screen's primary action, raised, half again the energy counter's width; E or the pad's Y on a desktop.
    const ew = og.ui.bounds(portrait ? "Foot/Bar/Energy" : "Foot/Energy")?.width ?? Math.round(120 * ts);
    // On a phone it shares the bar with the energy: never wider than the room the energy leaves it.
    const want = Math.round(Math.max(ew * 1.5, (small ? 132 : 168) * ts));
    const room = W - 2 * t.space.xs - 3 * t.space.sm - t.space.md - ew;
    const pressed = og.ui.heroButton("EndTurn", c.turn ? L("cards.endTurn") : L("cards.waiting"), { tone: "primary", size: "large", disabled: !c.turn, key: "E", width: px(portrait ? Math.max(96, Math.min(want, room)) : want) }).clicked;
    if (c.turn && (pressed || (world && og.input.actionPressed(world, "endTurn")))) c.turn = false;
  };
  og.ui.anchor("Cards", { anchor: "topLeft", margin: 0, width: px(W), height: px(Hs), direction: "column", gap: small ? t.space.sm : 0, padding: { left: small ? t.space.sm : t.space.xl, top: small ? t.space.sm : t.space.lg, right: small ? t.space.sm : t.space.xl, bottom: Math.round(footH + t.space.sm) } }, () => {
    og.ui.row("Top", { width: grow, gap: t.space.md, cross: "center", padding: { left: 0, top: 0, right: galleryW, bottom: 0 } }, () => {
      og.ui.column("Heading", { gap: 0, width: portrait ? grow : null }, () => {
        og.ui.label("Title", L("cards.title"), { step: small ? "xl2" : "xl4", heading: true, font: "display", nowrap: true });
        og.ui.label("Turn", c.turn ? L("cards.yourTurn") : L("cards.enemyTurn"), { step: small ? "sm" : big ? "xl" : "base", color: { token: c.turn ? "success" : "warn" }, nowrap: true });
      });
      if (landscape) {
        og.ui.spacer("Gap", grow);
        foes();
      }
    });
    // The enemies centred over the play area: standing on the table's foot on a desktop, filling the table's middle on
    // a phone (their portraits as tall as it allows).
    if (!landscape) og.ui.row("Table", { width: grow, height: grow, align: "center", cross: "end", gap: small ? t.space.md : t.space.xl3 }, foes);
  });
  // The hand along the foot with the energy at its start and End turn at its end, as card games lay them: on a desktop
  // and a phone on its side in the hand's own row; on a phone in portrait on a bar under it (Marvel Snap), so End turn
  // sits at the thumb and the energy where the player reads it while choosing a card.
  og.ui.anchor("Foot", { anchor: "bottom", margin: 0, width: px(W), direction: portrait ? "column" : "row", cross: portrait ? "stretch" : "center", gap: portrait ? t.space.xs : t.space.lg, padding: { left: small ? t.space.xs : t.space.xl, top: 0, right: small ? t.space.xs : t.space.xl, bottom: small ? t.space.xs : t.space.md } }, () => {
    if (!portrait) energy();
    og.ui.column("HandArea", { width: grow, cross: "stretch" }, () => hand(targets));
    // On a phone in portrait the bar docks under the hand, in the thumb's reach: a lifted card rises away from it.
    if (portrait) {
      og.ui.row("Bar", { width: grow, cross: "center", gap: t.space.md, padding: { left: t.space.sm, top: 0, right: t.space.sm, bottom: 0 } }, () => {
        energy();
        og.ui.spacer("Gap", grow);
        endTurn();
      });
    } else endTurn();
  });
}

/** The kit's hand: the cardHand over the table's viewmodel, and the game's side of a play. */
function hand(targets: CardTarget[]): void {
  {
    const r = og.ui.cardHand("Hand", {
      cards: c.hand.map((x) => {
        const base = (x.key ?? "").replace(/\d+$/, "");
        return { ...x, name: L(`cards.${base}.name`), typeLine: L(`cards.type.${base}`), text: L(`cards.${base}.text`), image: art(`cards/${base}` as ArtName), playable: x.playable !== false && (x.cost ?? 0) <= c.energy };
      }),
      targets,
      disabled: !c.turn,
      inspect: c.inspect,
      label: L("cards.hand"),
      targetPrompt: L("cards.target"),
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
  }
}

// ── rewards: a run's reward after a fight ───────────────────────────────

const REWARD_ART: Record<string, ArtName> = {
  thunder: "boons/thunder-strike", vigor: "boons/titan-heart", crown: "boons/crown-of-ages", frost: "boons/frost", gold: "boons/gold", blade: "boons/flame-blade",
};
const OFFER0: RewardOption[] = [
  { key: "thunder", title: "Thunder Strike", kind: "Gift of the Storm Warden", text: "Your attacks call down [b]bolts[/b] for [color=highlight]+30[/color] damage.", rarity: "rare", icon: "bolt" },
  { key: "vigor", title: "Ironbloom Heart", kind: "Gift of the Iron Matron", text: "[b]+25 %[/b] Max Health; heal [color=success]10[/color] on each room cleared.", rarity: "epic", tag: "Upgrade", tagKind: "upgrade", icon: "heart" },
  { key: "crown", title: "Crown of Ages", kind: "Relic", text: "Start each fight with [b]2[/b] extra [color=info]Energy[/color].", rarity: "legendary", tag: "New", tagKind: "new", locked: true, lockedHint: "Clear the second boss to unlock", icon: "crown" },
];
const OFFER1: RewardOption[] = [
  { key: "frost", title: "Frostbite", kind: "Gift of the Frost Weaver", text: "Hits [color=info]chill[/color] foes, slowing them by [b]20 %[/b].", rarity: "uncommon", icon: "shield" },
  { key: "gold", title: "Gilded Touch", kind: "Gift of the Coin Saint", text: "Gain [b]40[/b] gold now and [b]5[/b] per kill.", rarity: "rare", icon: "coin" },
  { key: "blade", title: "Duskward Edge", kind: "Weapon", text: "Your dash leaves a blade that deals [b]18[/b] damage.", rarity: "epic", tag: "Lv 3 → 4", icon: "sword" },
];

/** A reward with its art and its words in the active locale. */
function rewardWords(o: RewardOption): RewardOption {
  const k = o.key ?? "";
  return {
    ...o,
    title: L(`rewards.${k}.title`),
    kind: L(`rewards.${k}.kind`),
    text: L(`rewards.${k}.text`),
    ...(o.tag ? { tag: L(`rewards.${k}.tag`) } : {}),
    ...(o.lockedHint ? { lockedHint: L(`rewards.${k}.locked`) } : {}),
    image: art(REWARD_ART[k]),
  };
}

const r = {
  offer: OFFER0.map((o) => ({ ...o })),
  selected: [] as string[],
  rerolls: 2,
  banishes: 1,
  picks: 1,
  taken: "",
};

/** The reward after a fight (a roguelite's blessings, a deckbuilder's card reward): three rewards, Reroll, Banish and Skip. */
export function rewardsScreen(): void {
  const vp = og.ui.frameViewport();
  const two = r.picks > 1;
  const t = og.ui.frameTheme();
  const sa = vp.safeArea;
  const small = phone();
  const portrait = vp.breakpoint === "phonePortrait";
  // The choice is the screen: its title the display line centred over the cards, what to do as the caption under it,
  // the actions under the cards (docked at the foot on a phone in portrait). A phone keeps the Gallery's row clear; a
  // short frame or large text scrolls.
  const scrolls = !portrait || (t.textScale ?? 1) >= 1.5;
  const top = small ? Math.round((og.ui.bounds("Back")?.height ?? 44) + t.space.lg) : t.space.xl;
  og.ui.anchor("Rewards", { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom), direction: "column", align: scrolls && !small ? "center" : "start", padding: { left: small ? t.space.sm : t.space.xl, top, right: small ? t.space.sm : t.space.xl, bottom: small ? t.space.sm : Math.round(72 * Math.max(1, t.textScale ?? 1)) } }, () => {
    const body = (build: () => void): void => (scrolls ? og.ui.scrollArea("Body", { width: grow, height: small ? grow : null, gap: t.space.md, fade: small ? 40 : 56, fadeInto: "shadow", padding: { left: 0, top: 0, right: 0, bottom: 0 } }, build) : build());
    body(() => {
    const res = og.ui.rewardChoice("Choice", {
      options: r.offer.map((o) => rewardWords(o)),
      title: L("rewards.title"),
      subtitle: r.taken ? L("rewards.taken", { list: r.taken }) : two ? L("rewards.chooseTwo") : L("rewards.choose"),
      picks: r.picks,
      selected: r.selected,
      rerolls: r.rerolls,
      rerollCost: L("rewards.rerollCost"),
      banishes: r.banishes,
      skipLabel: L("rewards.skip"),
      rerollLabel: L("rewards.reroll"),
      banishLabel: L("rewards.banish"),
      confirmLabel: L("rewards.confirm"),
      cardWidth: vp.height >= 1000 ? 340 : 0,
    });
    r.selected = [...res.selected];
    if (res.picked.length > 0) {
      r.taken = res.picked.map((k) => L(`rewards.${k}.title`)).join(L("list.separator"));
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
    if (res.skipped) r.taken = L("rewards.skipped");
    });
    if (!small) og.ui.actionHints("Hints", { hints: [{ action: "confirm", hint: L("rewards.hint.take") }, { action: "back", hint: L("hint.back") }] });
  });
}

// ── gacha: a banner and its pull ────────────────────────────────────────

const PULL1: PullItem[] = [{ key: "aria", name: "Aria", kind: "Character", stars: 5, rarity: "legendary", isNew: true }];
const PULL10: PullItem[] = [
  { key: "p0", name: "Warden's Blade", kind: "Weapon", stars: 4, rarity: "epic" },
  { key: "p1", name: "Ember Fang", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p2", name: "Hunter's Sling", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p3", name: "Kael", kind: "Character", stars: 4, rarity: "epic", extra: "+2" },
  { key: "p4", name: "Oak Cudgel", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p5", name: "Aria", kind: "Character", stars: 5, rarity: "legendary", isNew: true },
  { key: "p6", name: "Night Spear", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p7", name: "Tome of Tides", kind: "Weapon", stars: 3, rarity: "rare" },
  { key: "p8", name: "Mira", kind: "Character", stars: 4, rarity: "epic", isNew: true },
  { key: "p9", name: "Plain Steel", kind: "Weapon", stars: 3, rarity: "rare" },
];

const g = {
  banner: "event",
  balance: 2400,
  pity: 74,
  token: 0,
  items: [] as PullItem[],
  log: "",
};

const BANNER_THUMB: Record<string, ArtName> = { event: "portraits/aria", weapon: "items/ashbringer", std: "portraits/kael" };
/** Which unit each pull of the fixtures draws. */
const PULL_UNIT: Record<string, string> = { aria: "aria", p0: "warden", p1: "fang", p2: "sling", p3: "kael", p4: "cudgel", p5: "aria", p6: "spear", p7: "tome", p8: "mira", p9: "steel" };
const PULL_ART: Record<string, ArtName> = {
  aria: "portraits/aria", kael: "portraits/kael", mira: "portraits/mira",
  warden: "items/iron-sword", fang: "boons/flame-blade", sling: "items/hunters-bow", cudgel: "items/arcane-staff",
  spear: "items/elven-bow", tome: "boons/rune-shield", steel: "items/knight-helm",
};
/** The characters, by name (a proper noun in every locale); their epithets are the locale's. */
const CHARACTER_NAME: Record<string, string> = { aria: "Aria", kael: "Kael", mira: "Mira" };
/** A pulled unit's art and words: its kind and the extra line in the active locale; a character keeps its name with
 * its epithet as the banner says it, a weapon is named in the locale. */
function pullWords(x: PullItem): PullItem {
  const u = PULL_UNIT[x.key ?? ""] ?? "";
  const who = CHARACTER_NAME[u];
  return {
    ...x,
    ...(who ? { name: who, title: L(`gacha.title.${u}`) } : { name: L(`gacha.unit.${u}`) }),
    kind: L(x.kind === "Character" ? "gacha.kind.character" : "gacha.kind.weapon"),
    rarityLabel: L(`gacha.rarity.${x.rarity ?? "rare"}`),
    ...(x.extra ? { extra: L("gacha.extra.shards", { n: 2 }) } : {}),
    image: art(PULL_ART[u]),
  };
}
const BANNERS = [
  { key: "event", name: "Moonlit Tide", title: "Moonlit Tide", subtitle: "Limited banner · Character", endsIn: 3 * 86400 + 5 * 3600, fresh: false },
  { key: "weapon", name: "Forged Oath", title: "Forged Oath", subtitle: "Limited banner · Weapon", endsIn: 50 * 60, fresh: true },
  { key: "std", name: "Open Road", title: "Open Road", subtitle: "Standard banner", endsIn: null, fresh: false },
] as const;

/** A gacha banner and its pull: the pull is the game's (it spends the purse and draws the results) and the reveal the pattern's. */
/** The event banner wears its key art and the featured units their portraits; the weapon banner the kit's backdrop;
 * the standard one draws the style's placeholder (the path a game without its art takes). */
/** The rate lines are the game's strings (og.i18n), so an RTL locale shows its own sentence. */
export function gachaScreen(backdrop = 0): void {
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  const b = BANNERS.find((x) => x.key === g.banner) ?? BANNERS[0];
  // On a phone the banner is the screen: its art runs under the status bar and the notch (the banner keeps its words
  // inside the safe area), the Gallery button floating on it over the tabs.
  const bleed = small && vp.breakpoint === "phonePortrait";
  og.ui.anchor("Gacha", { anchor: "topLeft", margin: 0, ...(bleed ? { offsetX: -sa.left, offsetY: -sa.top } : {}), width: px(bleed ? vp.width : vp.width - sa.left - sa.right), height: px(bleed ? vp.height : vp.height - sa.top - sa.bottom), direction: "column", padding: { left: 0, top: small && !bleed ? Math.round((og.ui.bounds("Back")?.height ?? 44) + og.ui.frameTheme().space.sm) : 0, right: 0, bottom: 0 } }, () => {
    const keyArt = b.key === "event" ? art("scenes/gacha-moonlit-tide") : null;
    const res = og.ui.gachaBanner("Banner", {
      banners: BANNERS.map((x) => ({ key: x.key, name: x.name, endsIn: x.endsIn, fresh: x.fresh, image: art(BANNER_THUMB[x.key]) })),
      current: b.key,
      title: b.title,
      subtitle: L(`gacha.banner.${b.key}`),
      image: keyArt ?? (b.key === "weapon" && backdrop && artOn ? (backdrop as never) : null),
      // Aria stands in the right 55 % of the key art; in portrait the crop keeps her face.
      ...(keyArt ? { focal: vp.breakpoint === "phonePortrait" ? { x: 0.74, y: 0.3 } : { x: 0.7, y: 0.36 } } : {}),
      featured: b.key === "std" ? [] : [
        { name: "Aria", title: L("gacha.title.aria"), stars: 5, rarity: "legendary", image: art("portraits/aria") },
        { name: "Kael", title: L("gacha.title.kael"), stars: 4, rarity: "epic", image: art("portraits/kael") },
        { name: "Mira", title: L("gacha.title.mira"), stars: 4, rarity: "epic", image: art("portraits/mira") },
      ],
      rateText: L(b.key === "std" ? "gacha.rate.standard" : "gacha.rate.event"),
      endsIn: b.endsIn,
      pity: g.pity,
      pityMax: 90,
      balance: g.balance,
      single: 160,
      multi: 1600,
      currency: "gem",
      currencyIcon: art("boons/crystal"),
      pityLabel: L("gacha.pity"),
      singleLabel: L("gacha.summon1"),
      multiLabel: L("gacha.summonN"),
      shortLabel: L("gacha.short"),
      detailsLabel: L("gacha.details"),
      historyLabel: L("gacha.history"),
      daysLabel: L("time.days"),
      hoursLabel: L("time.hours"),
      minutesLabel: L("time.minutes"),
      topInset: bleed ? Math.round((og.ui.bounds("Back")?.height ?? 44) + og.ui.frameTheme().space.xs) : 0,
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
  const seq = og.ui.pullSequence("Pull", { token: g.items.length > 0 ? g.token : 0, items: g.items.map(pullWords), againPrice: g.items.length * 160, affordable: g.balance >= g.items.length * 160, againLabel: L("gacha.summonN"), doneLabel: L("gacha.done"), skipLabel: L("gacha.skip"), tapLabel: L("gacha.tap"), continueLabel: L("gacha.continue"), newLabel: L("gacha.new"), currencyIcon: art("boons/crystal"), backdrop: b.key === "event" ? art("scenes/gacha-moonlit-tide") : null });
  if (seq.again && g.balance >= g.items.length * 160) {
    g.balance -= g.items.length * 160;
    g.token += 1;
  }
  if (seq.done) g.items = [];
}

/** `pat:<key>:<value>`: a pattern preview's state (pick:<item>, highlight:<tag>, reset). */
export function patternsCommand(world: World, args: readonly string[]): void {
  const [key, value = ""] = args;
  if ((key === "dlg" || key === "quest") && storyCommand(world, key, value, args[2] ?? "")) return;
  if (key && (genresCommand(key, value, args[2] ?? "") || genres2Command(key, value, args[2] ?? "") || genres3Command(key, value, args[2] ?? ""))) return;
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
  else if (key === "art") artOn = value !== "off";
  else if (key === "saga") s.current = value || "l21";
  else if (key === "score") b.score = Number(value) || 0;
  else if (key === "goal") b.have = value.split(",").map((x) => Number(x) || 0);
  else if (key === "purse") ig.energy = Number(value) || 0;
  else if (key === "gems") sto.gems = Number(value) || 0;
  else if (key === "storeConfirm") sto.confirm = value;
  else if (key === "premium") ps.premium = value !== "off";
  else if (key === "level") ps.level = Number(value) || 1;
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
    artOn = true;
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
    s.current = "l21";
    s.picked = "";
    b.score = 18450;
    b.moves = 12;
    b.have = [14, 4];
    ig.energy = 4.82e15;
    ig.mult = 0;
    ig.gens = GEN0.map((x) => ({ ...x }));
    sto = st0();
    ps = pass0();
    resetGenres();
    resetGenres2();
    resetGenres3();
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
function stat(key: string, value: string, unit: string, lowerIsBetter = false): { label: string; value: string; delta?: number; deltaText?: string; lowerIsBetter?: boolean } {
  const d = COMPARE[compareTag()]?.[key] ?? 0;
  return { label: L(`gear.stat.${key}`), value, ...(d ? { delta: d, deltaText: `${d > 0 ? "+" : "\u2212"}${Math.abs(d)}${unit}` } : {}), ...(lowerIsBetter ? { lowerIsBetter } : {}) };
}

export function gearScreen(): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const portrait = vp.breakpoint === "phonePortrait";
  // Touch has no Shift and no R (a carried item shows its Rotate button): its line says only what a finger does.
  screen("Gear", L("gear.title"), vp.coarsePointer ? L("gear.sub.touch") : L("gear.sub"), () => {
    const body = () => {
      const doll = og.ui.paperdoll("Doll", {
        slots: p.slots.map((x) => ({ ...x, label: L(`gear.slot.${(x.key ?? "").replace(/\d$/, "")}`), ...(x.lockedHint ? { lockedHint: L("gear.relicLocked", { level: 30 }) } : {}), ...(x.item ? { item: withArt(x.item) } : {}) })),
        title: "Aria Vell",
        subtitle: L("gear.class", { level: 42 }),
        rating: "812",
        ratingLabel: L("gear.power"),
        statsTitle: L("gear.attributes"),
        lockedLabel: L("gear.locked"),
        emptyLabel: L("gear.empty"),
        figureAspect: 0.5,
        // The item in hand, else the one selected, against what is worn where it would go.
        compare: !!COMPARE[compareTag()],
        stats: [
          stat("armor", "1,240", ""),
          stat("strength", "312", ""),
          stat("fire", "28%", "%"),
          stat("speed", "100%", "%"),
          stat("cooldowns", "8.2 s", " s", true),
        ],
        ...(p.carry ? { carry: p.carry } : {}),
      }, () => {
        // The figure: the game's character art in a body slot (the style's standing silhouette until it is set).
        og.ui.artSlot("Figure", { kind: "body", texture: art("scenes/paperdoll-aria-vell"), focal: { x: 0.5, y: 0.4 }, width: grow, height: grow, label: "Aria Vell" });
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
        og.ui.label("BagTitle", L("gear.backpack"), { step: "lg", font: "display", heading: true });
        // Eight columns fill a phone in portrait at the finger's 44 px floor, so the grids need no sideways scroll.
        const cell = portrait ? { cellSize: Math.max(44, Math.floor((vp.width - vp.safeArea.left - vp.safeArea.right - 32) / 8)) } : {};
        const pick = p.pick;
        p.pick = null;
        const bag = og.ui.inventoryGrid("Bag", { ...cell, columns: portrait ? 8 : 10, rows: portrait ? 7 : 6, items: p.bag.map(withArt), label: L("gear.backpack"), ...gridWords(), selected: p.selected, highlight: p.highlight, ...(p.carry && p.carry.source !== "Bag" ? { carry: p.carry } : {}), ...(pick ? { pick } : {}) });
        og.ui.label("StashTitle", L("gear.stash"), { step: "lg", font: "display", heading: true });
        const stash = og.ui.inventoryGrid("Stash", { ...cell, columns: portrait ? 8 : 10, rows: portrait ? 2 : 3, capacity: portrait ? 12 : 24, items: p.stash.map(withArt), label: L("gear.stash"), ...gridWords(), ...(p.carry && p.carry.source !== "Stash" ? { carry: p.carry } : {}) });
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
        entries: [{ id: "use", label: L("gear.menu.use") }, { id: "split", label: L("gear.menu.split") }, { id: "drop", label: L("gear.menu.drop") }],
      });
      if (m.chosen === "split" && p.menu) p.pick = { key: p.menu.key, count: 1 };
      if (m.chosen || m.dismissed) p.menu = null;
    };
    // Doll and grids side by side where both fit (a desktop), one above the other otherwise.
    if (vp.width >= 1100) og.ui.row("Split", { width: grow, gap: t.space.xl, cross: "start" }, body);
    else og.ui.column("Stack", { width: grow, gap: t.space.lg, cross: "start" }, body);
  }, [{ action: "confirm", hint: L("gear.hint.place") }, { action: "rotate", hint: L("gear.rotate") }, { action: "details", hint: L("gear.hint.menu") }, { action: "back", hint: L("gear.hint.putBack") }]);
}

// ── saga: a run's levels on a painted path ──────────────────────────────

/** The run: thirty levels, the first twenty cleared with their stars, the player on the twenty-first, a boss at 25. */
const SAGA0 = Array.from({ length: 30 }, (_, i) => ({
  key: `l${i + 1}`, label: `${i + 1}`, stars: i < 20 ? [3, 2, 3, 3, 1][i % 5]! : 0, locked: i > 20, boss: i === 24,
}));
const s = { current: "l21", picked: "" };

/** The saga map (Candy Crush Saga, Royal Match): the map is the screen, its title over its top under a scrim, Play at the foot. */
export function sagaScreen(): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  const big = !small && vp.height >= 1000;
  const lvl = (s.picked || s.current).slice(1);
  // The map runs under the notch and the status bar; its words keep to the safe area.
  og.ui.anchor("Saga", { anchor: "topLeft", margin: 0, offsetX: -sa.left, offsetY: -sa.top, width: px(vp.width), height: px(vp.height), direction: "column" }, () => {
    const r = og.ui.sagaMap("Map", {
      nodes: SAGA0.map((n) => ({ ...n, ...(n.boss ? { image: art("portraits/cultist") } : {}) })),
      current: s.current,
      avatar: art("portraits/aria-vell"),
      backdrop: art("scenes/title"),
      label: L("saga.title"),
      levelLabel: L("saga.level"),
      bossLabel: L("saga.boss"),
      starsLabel: L("saga.stars"),
      lockedLabel: L("gear.locked"),
      currentLabel: L("saga.here"),
      // On a phone Play is docked at the thumb: the path stops above it (last tick's place of the button and its margin).
      footInset: small ? Math.round((og.ui.bounds("SagaPlay")?.height ?? 64) + 2 * t.space.md) : 0,
    });
    if (r.picked) s.picked = r.picked;
  });
  // The world's name over the map's top, on a scrim that keeps it legible on any art; clear of the Gallery's corner.
  const galleryW = small ? (og.ui.bounds("Back")?.width ?? 100) + t.space.sm : 0;
  og.ui.anchor("SagaHead", { anchor: "topLeft", margin: 0, offsetX: -sa.left, offsetY: -sa.top, width: px(vp.width), direction: "column", padding: { left: sa.left + (small ? t.space.sm : t.space.xl), top: sa.top + (small ? t.space.sm : t.space.lg), right: sa.right + galleryW + t.space.sm, bottom: t.space.xl3 } }, () => {
    og.ui.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: { token: "shadow", alpha: 0.82 } }, { offset: 1, color: { token: "shadow", alpha: 0 } }] }] } as never);
    og.ui.label("Title", L("saga.world"), { step: small ? "xl2" : big ? "xl6" : "xl4", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
    og.ui.label("Sub", L("saga.sub", { stars: SAGA0.reduce((n, x) => n + x.stars, 0), max: 60 }), { step: small ? "sm" : big ? "xl" : "base", color: { token: "inkMuted" }, nowrap: true });
    og.ui.props({ inert: true } as never);
  });
  // Play the level picked (or the one the player stands on), the screen's one call to action at the thumb.
  og.ui.anchor("SagaPlay", { anchor: small ? "bottom" : "bottomRight", margin: small ? t.space.md : t.space.xl, direction: "column", width: small ? px(vp.width - sa.left - sa.right - 2 * t.space.md) : null }, () => {
    og.ui.heroButton("Play", L("saga.play", { level: lvl }), { tone: "primary", size: "large", key: small ? "" : "Enter", width: small ? grow : px(big ? 420 : 300) });
  });
}

// ── board: a match-3 level's goals and stars ────────────────────────────

const b = { score: 18450, moves: 12, have: [14, 4] as number[] };
const GEMS = ["#ff4d5e", "#3ea8ff", "#3ed46a", "#ffc93c", "#b05cff", "#e8f0ff"];

/** A match-3 level (Candy Crush, Royal Match): its goals over the board, the score toward three stars and the moves left beside it. */
export function boardScreen(): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  const portrait = vp.breakpoint === "phonePortrait";
  const big = !small && vp.height >= 1000;
  const goals = [
    // Blue gems are the board's own pieces: drawn as the board draws them, not the kit's (purple) crystal art.
    { key: "blue", label: L("board.goal.blue"), have: b.have[0]!, need: 20 },
    { key: "ice", label: L("board.goal.ice"), have: b.have[1]!, need: 6, image: art("boons/frost") },
  ];
  const H = vp.height - sa.top - sa.bottom;
  const W = vp.width - sa.left - sa.right;
  // The board: as large as the room left, eight by eight; its gems are the game's (drawn here as the kit's shapes).
  const boardPx = portrait ? Math.min(W - 2 * t.space.sm, H - 330) : Math.min(H - (big ? 210 : 150), W * 0.48);
  const cell = Math.floor(boardPx / 8);
  const panel = (): void => {
    og.ui.column("Panel", { gap: t.space.md, cross: "center", padding: { left: t.space.lg, top: t.space.lg, right: t.space.lg, bottom: t.space.lg }, radius: t.radius.panel, width: portrait ? grow : px(big ? 360 : 280), background: { token: "shadow", alpha: 0.62 } as never }, () => {
      og.ui.label("LevelCap", L("board.level").toLocaleUpperCase(), { step: big ? "lg" : "sm", color: { token: "inkMuted" }, font: "display", nowrap: true });
      og.ui.label("Level", "47", { step: big ? "xl6" : "xl4", font: "display", nowrap: true });
      og.ui.label("MovesCap", L("board.moves").toLocaleUpperCase(), { step: big ? "lg" : "sm", color: { token: "info" }, font: "display", nowrap: true });
      og.ui.label("Moves", `${b.moves}`, { step: big ? "xl6" : "xl4", font: "display", nowrap: true, color: { token: b.moves <= 5 ? "danger" : "ink" } });
      og.ui.starBar("Stars", { value: b.score, thresholds: [8000, 16000, 26000], label: L("board.score"), valueText: og.ui.formatNumber(b.score, { notation: "grouped" } as never), starsLabel: L("saga.stars") });
    });
  };
  og.ui.anchor("Board", { anchor: "topLeft", margin: 0, width: px(W), height: px(H), direction: portrait ? "column" : "row", align: "center", cross: "center", gap: portrait ? t.space.sm : t.space.xl, padding: { left: t.space.sm, top: portrait ? Math.round((og.ui.bounds("Back")?.height ?? 44) + t.space.md) : t.space.lg, right: t.space.sm, bottom: t.space.sm } }, () => {
    if (!portrait) panel();
    og.ui.column("Play", { gap: t.space.md, cross: "center" }, () => {
      og.ui.goalChips("Goals", { goals, label: L("board.goals"), progressLabel: L("board.progress"), doneLabel: L("board.done"), compact: portrait });
      if (portrait) og.ui.row("Strip", { gap: t.space.lg, cross: "center" }, () => {
        og.ui.column("Moves", { gap: 0, cross: "center" }, () => {
          og.ui.label("Cap", L("board.moves"), { step: "xs", color: { token: "info" }, nowrap: true });
          og.ui.label("Value", `${b.moves}`, { step: "xl3", font: "display", nowrap: true });
        });
        og.ui.starBar("Stars", { value: b.score, thresholds: [8000, 16000, 26000], label: L("board.score"), valueText: og.ui.formatNumber(b.score, { notation: "grouped" } as never), compact: true, starsLabel: L("saga.stars") });
      });
      og.ui.column("Grid", { width: px(cell * 8 + 12), height: px(cell * 8 + 12), padding: { left: 6, top: 6, right: 6, bottom: 6 }, radius: t.radius.panel, gap: 0 }, () => {
        og.ui.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { token: "shadow", alpha: 0.7 } }], strokes: [{ width: 1.5, align: "inside", paint: { kind: "solid", color: { token: "info", alpha: 0.55 } } }] } as never);
        for (let y = 0; y < 8; y++) {
          og.ui.row(`R${y}`, { gap: 0 }, () => {
            for (let x = 0; x < 8; x++) {
              const g = GEMS[(x * 7 + y * 5 + ((x * y) % 3)) % GEMS.length]!;
              og.ui.column(`C${x}`, { width: px(cell), height: px(cell), align: "center", cross: "center" }, () => {
                og.ui.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { token: "panel", alpha: (x + y) % 2 ? 0.28 : 0.16 } }] } as never);
                og.ui.column("Gem", { width: px(Math.round(cell * 0.72)), height: px(Math.round(cell * 0.72)) }, () => {});
                og.ui.setDraw({ shape: { kind: "polygon", sides: 4, rounding: 2 }, fills: [{ kind: "gradient", type: "linear", from: { x: 0.3, y: 0 }, to: { x: 0.7, y: 1 }, stops: [{ offset: 0, color: { color: hexColor(g, 1.35) } }, { offset: 1, color: { color: hexColor(g, 0.7) } }] }], effects: [{ kind: "outerGlow", blur: 6, spread: 0, color: { color: hexColor(g, 1), alpha: 0.35 } }] } as never);
              });
            }
          });
        }
      });
    });
    if (portrait) og.ui.props({} as never);
  });
}

/** A hex colour as the engine's colour, lightened (k > 1) or darkened (k < 1). */
function hexColor(h: string, k: number): { r: number; g: number; b: number } {
  const n = parseInt(h.slice(1), 16);
  const f = (v: number) => Math.min(1, Math.max(0, k > 1 ? v + (1 - v) * (k - 1) : v * k));
  return { r: f(((n >> 16) & 255) / 255), g: f(((n >> 8) & 255) / 255), b: f((n & 255) / 255) };
}

// ── generators: an idle game's main screen ──────────────────────────────

/** The generators as the game holds them: what each makes a second, what the next one costs, its level and how many are owned. */
const GEN0 = [
  { key: "solar", art: "boons/star", level: 125, owned: 348, rate: 1.24e6, cost: 12.4e6, secs: 1.6, cycle: 0.55 },
  { key: "fusion", art: "boons/flame-blade", level: 87, owned: 112, rate: 52.7e6, cost: 380e6, secs: 4, cycle: 0.3 },
  { key: "orbital", art: "boons/rune-shield", level: 64, owned: 36, rate: 2.31e9, cost: 5.6e9, secs: 0.12, cycle: 1 },
  { key: "drill", art: "boons/crystal", level: 28, owned: 8, rate: 97.4e9, cost: 210e9, secs: 12, cycle: 0.72 },
  { key: "dyson", art: "boons/gold", level: 12, owned: 2, rate: 4.12e12, cost: 6.8e12, secs: 30, cycle: 0.2 },
  { key: "forge", art: "boons/frost", level: 0, owned: 0, rate: 0, cost: 1.2e15, secs: 0, cycle: 0, locked: true },
] as const;
const ig = { energy: 4.82e15, mult: 0, gens: GEN0.map((x) => ({ ...x })), ascended: 0 };
// Idle letters past trillions (aa, ab …), three significant digits: what the genre writes.
const fmtIdle = (n: number) => og.ui.formatNumber(n, { notation: "letters" } as never);

/** An idle game's main screen (AdVenture Capitalist, Cookie Clicker): the energy over everything, the generators with the buy amount, the prestige. */
export function generatorsScreen(): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const small = phone();
  const wide = vp.width >= 1100;
  const n = [1, 10, 100][ig.mult]!;
  screen("Gens", L("idle.title"), L("idle.sub"), () => {
    // The energy, the screen's number, counting as it changes.
    og.ui.column("Energy", { width: grow, cross: "center", gap: 0, padding: { left: t.space.md, top: t.space.sm, right: t.space.md, bottom: t.space.sm }, radius: t.radius.panel, background: { token: "shadow", alpha: 0.55 } as never }, () => {
      og.ui.label("Cap", L("idle.energy").toLocaleUpperCase(), { step: small ? "sm" : "base", color: { token: "inkMuted" }, font: "display", nowrap: true });
      og.ui.numberCounter("Value", ig.energy, { format: { notation: "letters" }, step: small ? "xl4" : "xl6", font: "display", delta: false } as never);
      og.ui.label("Rate", L("idle.rate", { rate: fmtIdle(ig.gens.reduce((s0, g0) => s0 + g0.rate, 0)) }), { step: small ? "base" : "lg", color: { token: "info" }, font: "display", nowrap: true });
    });
    const list = (): void => {
      og.ui.column("Generators", { width: grow, gap: t.space.sm }, () => {
        og.ui.row("Head", { width: grow, cross: "center", gap: t.space.sm }, () => {
          og.ui.label("Title", L("idle.generators").toLocaleUpperCase(), { step: small ? "base" : "lg", font: "display", heading: true, nowrap: true, width: grow });
          const m = og.ui.segmented("Mult", { items: [{ label: "×1" }, { label: "×10" }, { label: "×100" }], selected: ig.mult, size: "small", label: L("idle.buyAmount") } as never);
          if (m.changed) ig.mult = m.selected;
        });
        for (const g0 of ig.gens) {
          const locked = "locked" in g0 && g0.locked;
          const cost = g0.cost * (n === 1 ? 1 : n === 10 ? 11.2 : 140);
          const r = og.ui.idleGenerator(g0.key, {
            name: L(`idle.gen.${g0.key}`),
            image: art(g0.art as ArtName),
            level: g0.level,
            ownedText: og.ui.formatNumber(g0.owned, { notation: "grouped" } as never),
            rateText: L("idle.perSec", { rate: fmtIdle(g0.rate) }),
            cycle: g0.cycle,
            cycleSeconds: g0.secs,
            costText: fmtIdle(cost),
            currencyIcon: art("boons/crystal"),
            affordable: !locked && ig.energy >= cost,
            buyCount: n,
            buyLabel: L("idle.buy"),
            levelLabel: L("idle.level"),
            ownedLabel: L("idle.owned"),
            locked,
            lockedHint: L("idle.unlock", { cost: fmtIdle(g0.cost) }),
          });
          if (r.bought) {
            ig.energy -= cost;
            g0.owned += n;
          }
        }
      });
    };
    const prestige = (): void => {
      const p0 = og.ui.prestigePanel("Prestige", {
        title: L("idle.prestige"),
        lead: L("idle.ascendFor"),
        rewardText: L("idle.shards", { n: 128 }),
        points: [L("idle.point.reset"), L("idle.point.keep"), L("idle.point.stronger")],
        image: art("boons/star"),
        actionLabel: L("idle.ascend"),
        available: ig.energy >= 1e15,
        lockedHint: L("idle.unlock", { cost: fmtIdle(1e15) }),
      });
      if (p0.confirmed) {
        ig.ascended += 1;
        ig.energy = 0;
      }
    };
    if (wide) og.ui.row("Split", { width: grow, gap: t.space.xl, cross: "start" }, () => {
      og.ui.column("Left", { width: grow, gap: t.space.md }, list);
      og.ui.column("Right", { width: px(vp.height >= 1000 ? 560 : 420), gap: t.space.md }, prestige);
    });
    else {
      list();
      prestige();
    }
  }, [{ action: "confirm", hint: L("idle.hint.buy") }, { action: "back", hint: L("hint.back") }]);
}

// ── store: a store front with a featured rotation and a daily one ───────

type Offer = Mut<StoreOffer>;
/** The store as the game holds it: the purse, each offer's price and what the player has bought or owns. */
const STORE0 = {
  gems: 2400,
  coins: 3250,
  featured: [
    { key: "tide", art: "scenes/gacha-moonlit-tide", focal: { x: 0.72, y: 0.3 }, rarity: "legendary", price: 2800, original: 4200, currency: "gems", featured: true, badge: "limited", limit: 1, endsIn: 3 * 3600 + 1200, contents: ["aria", "lantern", "glitter", "frame"] },
    { key: "ashbringer", art: "items/ashbringer", rarity: "legendary", price: 1800, currency: "gems", badge: "new", badgeKind: "new" },
    { key: "crown", art: "items/crown", rarity: "epic", price: 1200, currency: "gems", owned: true },
  ],
  daily: [
    { key: "potions", art: "items/healing-draught", rarity: "common", price: 250, currency: "coins", limit: 3, bought: 1 },
    { key: "mana", art: "items/mana-vial", rarity: "uncommon", price: 300, currency: "coins" },
    { key: "ember", art: "items/ember-ring", rarity: "rare", price: 1500, currency: "coins", badge: "new", badgeKind: "new" },
    { key: "amulet", art: "items/sapphire-amulet", rarity: "epic", price: 4000, currency: "coins" },
    { key: "key", art: "items/iron-key", rarity: "uncommon", price: 600, currency: "coins", limit: 1, bought: 1 },
  ],
  packs: [
    { key: "gems1", art: "boons/crystal", rarity: "rare", priceText: "$4.99", amount: 980 },
    { key: "gems2", art: "boons/crystal", rarity: "epic", priceText: "$14.99", amount: 3280, badge: "popular" },
    { key: "gems3", art: "boons/gold", rarity: "legendary", priceText: "$29.99", amount: 6480, badge: "best" },
  ],
} as const;
const st0 = () => ({ gems: STORE0.gems as number, coins: STORE0.coins as number, bought: {} as Record<string, number>, owned: new Set<string>(["crown"]), selected: "", confirm: "" });
let sto = st0();

/** The store (Fortnite's item shop, Genshin's): the purse, the featured rotation, the daily one and the gem packs; a press asks to confirm, and the game charges it. */
export function storeScreen(): void {
  const small = phone();
  const offer = (o: (typeof STORE0.featured)[number] | (typeof STORE0.daily)[number]): Offer => {
    const purse = o.currency === "gems" ? sto.gems : sto.coins;
    const x: Offer = {
      key: o.key,
      name: L(`store.item.${o.key}`),
      kind: L(`store.kind.${o.key}`),
      image: art(o.art as ArtName),
      rarity: o.rarity,
      price: o.price,
      currency: o.currency,
      original: "original" in o ? o.original : 0,
      limit: "limit" in o ? o.limit : 0,
      bought: ("bought" in o ? o.bought : 0) + (sto.bought[o.key] ?? 0),
      owned: sto.owned.has(o.key),
      affordable: purse >= o.price,
      featured: "featured" in o && o.featured,
      badge: "badge" in o ? L(`store.badge.${o.badge}`) : "",
      badgeKind: "badgeKind" in o ? o.badgeKind : "note",
    };
    if ("focal" in o) x.focal = o.focal;
    if ("endsIn" in o) x.endsIn = o.endsIn;
    if ("contents" in o) x.contents = o.contents.map((c) => L(`store.contents.${c}`));
    return x;
  };
  screen("Shop", L("store.title"), L("store.sub"), () => {
    const r = og.ui.storeFront("Store", {
      wallet: [
        { key: "gems", name: L("store.gems"), amount: sto.gems, image: art("boons/crystal"), topUp: true },
        { key: "coins", name: L("store.coins"), amount: sto.coins, image: art("boons/gold"), topUp: true },
      ],
      sections: [
        { key: "featured", title: L("store.featured"), endsIn: 2 * 86400 + 4 * 3600, offers: STORE0.featured.map(offer) },
        { key: "daily", title: L("store.daily"), endsIn: 7 * 3600 + 720, offers: STORE0.daily.map(offer) },
        {
          key: "packs",
          title: L("store.packs"),
          offers: STORE0.packs.map((o) => ({ key: o.key, name: L("store.gemPack", { n: og.ui.formatNumber(o.amount, { notation: "grouped" } as never) }), kind: L(`store.kind.${o.key}`), image: art(o.art as ArtName), rarity: o.rarity, priceText: o.priceText, ...("badge" in o ? { badge: L(`store.badge.${o.badge}`) } : {}) })),
        },
      ],
      selected: sto.selected,
      columns: small ? 2 : 0,
      label: L("store.title"),
      freeLabel: L("store.free"),
      ownedLabel: L("store.owned"),
      soldOutLabel: L("store.soldOut"),
      leftLabel: L("store.left"),
      saveLabel: L("store.save"),
      refreshLabel: L("store.refresh"),
      shortLabel: L("gacha.short"),
      topUpLabel: L("store.topUp"),
      daysLabel: L("time.days"),
      hoursLabel: L("time.hours"),
    });
    if (r.chosen) {
      sto.selected = r.chosen;
      sto.confirm = r.chosen;
    }
  }, [{ action: "confirm", hint: L("store.hint.buy") }, { action: "back", hint: L("hint.back") }]);
  // The game's confirm: what it costs, then the charge.
  const o = [...STORE0.featured, ...STORE0.daily].find((x) => x.key === sto.confirm);
  const c = og.ui.confirm("Buy", !!o, {
    title: o ? L(`store.item.${o.key}`) : "",
    message: o ? L("store.confirm", { price: og.ui.formatNumber(o.price, { notation: "grouped" } as never), currency: L(o.currency === "gems" ? "store.gems" : "store.coins") }) : "",
    confirmLabel: L("store.buy"),
    cancelLabel: L("store.cancel"),
  });
  if (o && c.confirmed) {
    if (o.currency === "gems") sto.gems -= o.price;
    else sto.coins -= o.price;
    sto.bought[o.key] = (sto.bought[o.key] ?? 0) + 1;
    if (!("limit" in o)) sto.owned.add(o.key);
    sto.confirm = "";
    sto.selected = "";
  } else if (c.cancelled) {
    sto.confirm = "";
    sto.selected = "";
  }
}

// ── pass: a season's battle pass ────────────────────────────────────────

const PASS_FREE = ["boons/gold", "boons/crystal", "items/healing-draught", "boons/gold", "items/mana-vial"] as const;
const PASS_PREM = ["items/ember-ring", "boons/crystal", "items/sapphire-amulet", "items/boots", "items/arcane-staff", "items/gauntlets"] as const;
const PASS_RARITY = ["common", "uncommon", "rare", "epic"] as const;
/** The season: thirty tiers, a milestone every tenth (the season's characters), the player at level 12. */
const pass0 = () => ({
  level: 12,
  xp: 640,
  premium: false,
  claimed: new Set<string>([...Array.from({ length: 10 }, (_, i) => `${i + 1}/f`).filter((k) => k !== "8/f")]),
});
let ps = pass0();

/** A battle pass (Fortnite, Brawl Stars): the season's header, then its track, a free and a premium reward each tier. */
export function passScreen(): void {
  const tiers = Array.from({ length: 30 }, (_, i) => {
    const lv = i + 1;
    const mile = lv % 10 === 0;
    const fa = PASS_FREE[i % PASS_FREE.length]!;
    const pa = PASS_PREM[i % PASS_PREM.length]!;
    return {
      level: lv,
      milestone: mile,
      free: i % 3 === 2 && !mile ? null : { key: `f${lv}`, name: L(`pass.reward.${mile ? "chest" : fa.split("/")[1]}`), amountText: fa.startsWith("boons/") && !mile ? `×${fa === "boons/gold" ? 200 : 50}` : "", image: art(mile ? "items/crown" : fa), rarity: mile ? "epic" : PASS_RARITY[i % 2] },
      premium: { key: `p${lv}`, name: L(`pass.reward.${mile ? (lv === 10 ? "aria" : lv === 20 ? "kael" : "mira") : pa.split("/")[1]}`), image: art(mile ? (lv === 10 ? "portraits/aria" : lv === 20 ? "portraits/kael" : "portraits/mira") : pa), rarity: mile ? ("legendary" as const) : PASS_RARITY[(i % 3) + 1]! },
      freeClaimed: ps.claimed.has(`${lv}/f`),
      premiumClaimed: ps.claimed.has(`${lv}/p`),
    };
  });
  screen("Pass", L("pass.title"), "", () => {
    const r = og.ui.battlePass("Pass", {
      title: L("pass.season"),
      image: art("scenes/gacha-moonlit-tide"),
      endsIn: 18 * 86400 + 6 * 3600,
      level: ps.level,
      xp: ps.xp,
      xpNext: 1000,
      premium: ps.premium,
      tiers,
      unlockPriceText: "$9.99",
      label: L("pass.title"),
      freeLabel: L("pass.free"),
      premiumLabel: L("pass.premium"),
      levelLabel: L("pass.level"),
      xpLabel: L("pass.xp"),
      claimLabel: L("pass.claim"),
      claimAllLabel: L("pass.claimAll"),
      claimedLabel: L("pass.claimed"),
      unlockLabel: L("pass.unlock"),
      lockedLabel: L("pass.premium"),
      reachLabel: L("pass.reach"),
      daysLabel: L("time.days"),
      hoursLabel: L("time.hours"),
    });
    for (const c0 of r.claimed) ps.claimed.add(`${c0.level}/${c0.premium ? "p" : "f"}`);
    if (r.unlock) ps.premium = true;
  }, [{ action: "confirm", hint: L("pass.claim") }, { action: "back", hint: L("hint.back") }], false);
}
