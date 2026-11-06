// Ashen Crown's screens on og.ui and the style in assets/styles/ashen (the
// engine's pixel pack in ember and ash): the title and the pause menu (the
// game's screen documents, assets/ui/main-menu.ogt and pause.ogt), the map
// between fights, the battle's HUD, the rewards and the fire, the settings
// dialog and the run's summary. Screens change through a dissolve (a fade
// with reduced motion). Every string is a key of assets/strings/*.ogt.
//
// The battle HUD: the heroes' unit frames (the rpg kit: health, block,
// energy, statuses) top left, the turn order (the rpg kit) top centre, the
// map's step and Pause top right; over each foe its intent (what it will
// do, with a tooltip) and under it its health, block and statuses; along
// the bottom each local hero's hand (og.ui.cardHand: a fan whose card under
// the pointer lifts, a drag or two taps to play, an arc to the foe for a
// card that needs one) with its energy, its piles and End turn. Two on one
// device: a hand each, side by side. Keys and pads walk the hand from the
// world (src/run.ts: the focused card glows, the focused foe is ringed).
//
// A button or card that changes the run asks for it through its seat's
// input (ask: the play bit with a code in aimX, src/components.ts's Ask), so
// a room acts on it on one tick. The Ui system also sets the buses and
// keeps the settings and the record in og.platform.

import { og, defineSystem } from "@origingame/engine";
import type { HandCard, RewardOption, StatusEffect, Texture, World } from "@origingame/engine";
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";
import type { TurnUnit } from "@origingame/engine/extensions/rpg/index.ts";
import { CARDS, costOf, defOf, isUpgraded, numbers, UPGRADED } from "./cards.ts";
import { Ask, Foe, Game, Hero, HEROES, Phase, Side } from "./components.ts";
import type { FoeV, GameV, HeroV } from "./components.ts";
import { prefs } from "./device.ts";
import { FOES, FOE_SLOTS, Intent, MAP, Node } from "./foes.ts";
import { cardArt, inRoom, moveOf, must, tempersOf } from "./run.ts";
import { LOCAL, localSeats, ONLINE, press, SEATS, useInput, useTouchControls } from "./seats.ts";
import { TOP, toScreen, unitSize } from "./stage.ts";
import { damageOf } from "./run.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24 } as const;
const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

/** The portraits of the units (loaded in start), by name. */
export const portraits: Record<string, number> = {};

// ── asking the world, through a seat's input ───────────────────────────

/** What each local seat asks this tick: aimX and aimY with its play bit (src/run.ts reads them). */
const asks = [{ x: 0, y: 0, on: false }, { x: 0, y: 0, on: false }];
function ask(code: number, local = 0, y = 0): void {
  asks[local] = { x: code, y, on: true };
  press("play", local);
}
useInput((_world, local, input) => {
  const a = asks[local];
  if (!a?.on) return;
  input.aimX = a.x;
  input.aimY = a.y;
  asks[local] = { x: 0, y: 0, on: false };
});
// The HUD is the touch controls: cards, foes and End turn are tapped where they stand.
useTouchControls(() => ({ x: 0, y: 0, lx: 0, ly: 0, buttons: 0 }));

/** The local index (src/seats.ts's press) of a seat this device drives, or -1. */
function localOf(world: World, seat: number): number {
  if (LOCAL) return seat;
  return localSeats(world)[0] === seat ? 0 : -1;
}

/** The seats whose hands and choices this device shows. */
function mySeats(world: World, g: GameV): number[] {
  const seats = LOCAL ? [0, 1] : [localSeats(world)[0] ?? 0];
  return seats.filter((s) => s < g.party);
}

// ── this device's screen state ──────────────────────────────────────────

let settingsOpen = false;
/** Keys or a pad walk the hands (a d-pad press since the pointer last pressed): the focused card glows, the focused foe is ringed. */
let keysDrive = false;
const inspect = ["", ""];

// ── the platform: settings and the record ──────────────────────────────

function applyPrefs(world: World): void {
  const audio = world.resource(og.Audio);
  if (!audio.ok) return;
  const a = audio.value;
  if (a.master !== prefs.master) a.master = prefs.master;
  if (a.music !== prefs.music) a.music = prefs.music;
  if (a.sfx !== prefs.effects) a.sfx = prefs.effects;
  if (a.ui !== prefs.effects) a.ui = prefs.effects;
}

export function setLanguage(world: World, index: number): void {
  prefs.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[prefs.language]!.code);
}

/** The run record ({runs, wins}) written at the revision it was read at; a room keeps none. */
function saveRecord(world: World, g: GameV): void {
  if (inRoom(world)) return;
  const r = og.platform.storeSet(world, { key: "record", value: { runs: g.runs, wins: g.wins }, revision: g.recordRevision });
  if (r.ok) g.recordWrite = r.value;
}

function platform(world: World, g: GameV): void {
  if (world.tick === 0) {
    prefs.loadRequest = og.platform.loadSettings(world);
    const r = og.platform.storeGet(world, "record");
    g.recordRequest = r.ok ? r.value : 0;
    return;
  }
  if (prefs.loadRequest !== 0) {
    const r = og.platform.result(world, prefs.loadRequest);
    if (r !== null) {
      prefs.loadRequest = 0;
      const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
      if (s && typeof s === "object") {
        if (typeof s.masterVolume === "number") prefs.master = s.masterVolume;
        if (typeof s.musicVolume === "number") prefs.music = s.musicVolume;
        if (typeof s.sfxVolume === "number") prefs.effects = s.sfxVolume;
        if (typeof s.reducedMotion === "boolean") prefs.reducedMotion = s.reducedMotion;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, at);
      }
    }
  }
  if (inRoom(world)) return;
  if (g.recordRequest !== 0) {
    const r = og.platform.result(world, g.recordRequest);
    if (r !== null) {
      g.recordRequest = 0;
      const d = (r.ok ? r.data : null) as { value?: { runs?: number; wins?: number } | null; revision?: number } | null;
      g.runs = Math.max(g.runs, d?.value?.runs ?? 0);
      g.wins = Math.max(g.wins, d?.value?.wins ?? 0);
      if (typeof d?.revision === "number") g.recordRevision = d.revision;
    }
  }
  if (g.recordWrite !== 0) {
    const r = og.platform.result(world, g.recordWrite);
    if (r !== null) {
      g.recordWrite = 0;
      if (r.ok) g.recordRevision = (r.data as { revision: number }).revision;
      else if (r.error?.code === "conflict") {
        // Another device wrote the record first: read it again (the larger counts win) and write.
        const again = og.platform.storeGet(world, "record");
        g.recordRequest = again.ok ? again.value : 0;
      }
    }
  }
}

// ── reading the world ──────────────────────────────────────────────────

function heroOf(world: World, seat: number): HeroV | null {
  const e = world.find(seat === 0 ? "hero" : `hero-${seat}`);
  if (e === null) return null;
  const r = world.get(e, Hero);
  return r.ok ? r.value : null;
}

function foeOf(world: World, slot: number): FoeV | null {
  const e = world.find(`foe-${slot}`);
  if (e === null) return null;
  const r = world.get(e, Foe);
  return r.ok ? r.value : null;
}

function spot(world: World, name: string): { x: number; y: number } {
  const e = world.find(name);
  const t = e === null ? null : world.get(e, og.Transform);
  return t && t.ok ? { x: t.value.position.x, y: t.value.position.y } : { x: 0, y: 0 };
}

const heroWord = (world: World, seat: number) => say(world, `hero.${HEROES[seat]}`);

/** A card's words: its name (with + upgraded), its type line, its rules with its numbers. */
function cardWords(world: World, code: number): { name: string; type: string; text: string } {
  const d = defOf(code);
  const { a, b } = numbers(code);
  const name = say(world, `card.${d.id}`) + (isUpgraded(code) ? "+" : "");
  return { name, type: say(world, `kind.${d.kind}`), text: say(world, `card.${d.id}.text`, { a, b }) };
}

function rarity(code: number): "common" | "uncommon" | "rare" {
  return defOf(code).rarity;
}

function effects(world: World, v: { strength: number; weak: number; vulnerable: number; poison?: number; block: number; kindle?: number }): StatusEffect[] {
  const out: StatusEffect[] = [];
  if (v.strength > 0) out.push({ key: "strength", icon: "sword", name: say(world, "status.strength"), stacks: v.strength });
  if ((v.kindle ?? 0) > 0) out.push({ key: "kindle", icon: "fire", name: say(world, "status.kindle"), stacks: v.kindle ?? 0 });
  if (v.weak > 0) out.push({ key: "weak", icon: "drop", name: say(world, "status.weak"), stacks: v.weak, debuff: true });
  if (v.vulnerable > 0) out.push({ key: "vulnerable", icon: "bolt", name: say(world, "status.vulnerable"), stacks: v.vulnerable, debuff: true });
  if ((v.poison ?? 0) > 0) out.push({ key: "poison", icon: "skull", name: say(world, "status.poison"), stacks: v.poison ?? 0, debuff: true, type: "poison" });
  return out;
}

// ── the battle HUD ──────────────────────────────────────────────────────

/** What a foe's intent says: its icon, its number, its words. */
function intentOf(world: World, g: GameV, v: FoeV, seats: (HeroV | null)[]): { icon: string; value: string; tip: string; tone: "danger" | "info" | "warn" | "success" } {
  const m = moveOf(v);
  const target = seats[v.target] ?? null;
  const per = (h: HeroV | null) => damageOf(m.dmg ?? 0, v.strength, v.weak, h?.vulnerable ?? 0);
  const hits = m.hits ?? 1;
  const who = g.party > 1 && v.target >= 0 ? heroWord(world, v.target) : "";
  switch (m.kind) {
    case Intent.Attack:
      return { icon: "sword", value: hits > 1 ? `${per(target)}×${hits}` : `${per(target)}`, tip: say(world, who ? "intent.attackAt" : "intent.attack", { n: hits > 1 ? `${per(target)}×${hits}` : per(target), who }), tone: "danger" };
    case Intent.AttackAll:
      return { icon: "bomb", value: `${per(null)}`, tip: say(world, "intent.attackAll", { n: per(null) }), tone: "danger" };
    case Intent.AttackBlock:
      return { icon: "sword", value: `${per(target)}+${m.block}`, tip: say(world, "intent.attackBlock", { n: per(target), b: m.block ?? 0, who }), tone: "danger" };
    case Intent.Block:
      return { icon: "shield", value: `${m.block}`, tip: say(world, m.strength ? "intent.blockBuff" : "intent.block", { b: m.block ?? 0, s: m.strength ?? 0 }), tone: "info" };
    case Intent.Buff:
      return { icon: "plus", value: `${m.strength}`, tip: say(world, "intent.buff", { s: m.strength ?? 0 }), tone: "warn" };
    default:
      return { icon: "drop", value: "", tip: say(world, "intent.debuff", { who: who || heroWord(world, Math.max(0, v.target)) }), tone: "success" };
  }
}

/** Over each foe its intent, under it its name, health, block and statuses; a ring round the one the keys are on. */
/** Each foe's body on the screen this frame (CSS px), from its world anchor: the hands' drop targets. */
const bodies = new Map<number, { x: number; y: number; width: number; height: number }>();

function foeTags(world: World, g: GameV, seats: (HeroV | null)[], focus: Map<number, number[]>): void {
  bodies.clear();
  for (let slot = 0; slot < FOE_SLOTS; slot++) {
    const v = foeOf(world, slot);
    if (!v || !v.active || v.hp <= 0) continue;
    const at = spot(world, `foe-${slot}`);
    const size = unitSize(v.kind);
    if (g.side === Side.Heroes && !g.cleared) {
      const it = intentOf(world, g, v, seats);
      og.ui.openWorldAnchor(`Intent${slot}`, { target: { point: { x: at.x, y: at.y + size + 0.15, z: 0 } }, pivot: "bottom", direction: "row", gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xxs, right: S.sm, bottom: S.xxs }, background: { token: "hud" }, radius: 0 });
      og.ui.icon("Icon", it.icon, { size: 22, tint: { token: it.tone } });
      if (it.value) og.ui.label("Value", it.value, { step: "lg", font: "mono", nowrap: true, color: { token: it.tone === "danger" ? "danger" : "ink" } });
      og.ui.close();
      og.ui.tooltip(it.tip);
    }
    og.ui.openWorldAnchor(`Tag${slot}`, { target: { point: { x: at.x, y: at.y - 0.15, z: 0 } }, pivot: "top", direction: "column", gap: S.xxs, cross: "center" });
    // On a phone the foes stand too close for their names: the name is the tag's tooltip there.
    const narrow = og.ui.frameViewport().width < 600;
    if (!narrow) og.ui.label("Name", say(world, `foe.${FOES[v.kind]!.id}`), { step: "sm", nowrap: true, color: { token: FOES[v.kind]!.boss || FOES[v.kind]!.elite ? "gold" : "ink" } });
    og.ui.row("Bars", { gap: S.xs, cross: "center" }, () => {
      if (v.block > 0) {
        og.ui.icon("Shield", "shield", { size: 18, tint: { token: "info" } });
        og.ui.label("Block", `${v.block}`, { font: "mono", nowrap: true, color: { token: "info" } });
      }
      og.ui.meter("Health", v.hp, v.hpMax, { kind: "health", label: `${v.hp}/${v.hpMax}`, width: px(narrow ? 64 : size >= 4 ? 140 : 96) });
      if (narrow) og.ui.tooltip(say(world, `foe.${FOES[v.kind]!.id}`));
    });
    const fx = effects(world, v);
    if (fx.length > 0) og.ui.row("Status", { gap: S.xs }, () => fx.forEach((f) => {
      og.ui.row(f.key ?? "", { gap: S.xxs, cross: "center", padding: { left: S.xs, top: 0, right: S.xs, bottom: 0 }, background: { token: "hud" } }, () => {
        og.ui.icon("Icon", f.icon ?? "", { size: 16, tint: { token: f.type === "poison" ? "success" : f.debuff ? "danger" : "warn" } });
        og.ui.label("Stacks", `${f.stacks}`, { step: "sm", font: "mono", nowrap: true });
      });
      og.ui.tooltip(f.name ?? "");
    }));
    og.ui.close();
    // The body's box, by a world anchor (it follows the camera as the tags do): where a dragged
    // card lands on this foe, and, for each local seat whose keys are on it, a ring in the
    // seat's ink (a second seat's round the first's).
    const k = (toScreen(world, at.x + size / 2, at.y).x - toScreen(world, at.x, at.y).x) * 0.78;
    const box = og.ui.openWorldAnchor(`Body${slot}`, { target: { point: { x: at.x, y: at.y + size / 2, z: 0 } }, pivot: "center" });
    const rings = focus.get(slot) ?? [];
    const ring = (seat: number | undefined) => {
      if (seat !== undefined) og.ui.setDraw({ shape: { kind: "rect", radii: [4, 4, 4, 4] }, strokes: [{ width: 3, align: "inside", paint: { kind: "solid", color: { token: seat === 1 ? "info" : "focus" } } }] });
    };
    og.ui.column("Ring", { width: px(2 * k + 12), height: px(2 * k + 12), padding: { left: 6, top: 6, right: 6, bottom: 6 } }, () => {
      ring(rings[1]);
      og.ui.column("Inner", { width: grow, height: grow }, () => ring(rings[0]));
    });
    og.ui.props({ inert: true });
    og.ui.close();
    bodies.set(slot, { x: box.x - k, y: box.y - k, width: 2 * k, height: 2 * k });
  }
}

/** Under each hero its block and health. */
function heroTags(world: World, g: GameV): void {
  for (let seat = 0; seat < g.party; seat++) {
    const v = heroOf(world, seat);
    if (!v) continue;
    const at = spot(world, seat === 0 ? "hero" : `hero-${seat}`);
    og.ui.openWorldAnchor(`HeroTag${seat}`, { target: { point: { x: at.x, y: at.y - 0.15, z: 0 } }, pivot: "top", direction: "row", gap: S.xs, cross: "center" });
    if (v.block > 0) {
      og.ui.icon("Shield", "shield", { size: 18, tint: { token: "info" } });
      og.ui.label("Block", `${v.block}`, { font: "mono", nowrap: true, color: { token: "info" } });
    }
    og.ui.meter("Health", v.hp, v.hpMax, { kind: "health", label: `${v.hp}/${v.hpMax}`, width: px(96) });
    og.ui.close();
  }
}

/** A seat's hand, its energy, its piles and End turn; answers nothing (it asks through the seat's input). */
function hand(world: World, g: GameV, seat: number, v: HeroV, playing: boolean): void {
  const local = localOf(world, seat);
  const phone = og.ui.frameViewport().width < 600;
  const canAct = playing && v.hp > 0 && !v.ended;
  const keysOn = keysDrive;
  const focusKey = v.hand.length > 0 ? `${seat}-${v.hand[Math.min(v.focusCard, v.hand.length - 1)]}` : "";
  const cards: HandCard[] = v.hand.map((i) => {
    const code = v.deck[i]!;
    const w = cardWords(world, code);
    return {
      key: `${seat}-${i}`,
      name: w.name,
      cost: costOf(code),
      typeLine: w.type,
      text: w.text,
      image: (cardArt[code & 255] ?? null) as Texture | null,
      rarity: rarity(code),
      playable: canAct && costOf(code) <= v.energy,
      needsTarget: defOf(code).target,
      glow: canAct && keysOn && `${seat}-${i}` === focusKey,
      fresh: world.tick - v.freshAt <= 2 && v.fresh.includes(i),
    };
  });
  const targets: { key: string; name: string; rect: { x: number; y: number; width: number; height: number }; valid: boolean }[] = [];
  for (let slot = 0; slot < FOE_SLOTS; slot++) {
    const f = foeOf(world, slot);
    if (!f || !f.active || f.hp <= 0) continue;
    const rect = bodies.get(slot);
    if (rect) targets.push({ key: `${slot}`, name: say(world, `foe.${FOES[f.kind]!.id}`), rect, valid: true });
  }
  og.ui.column(`Seat${seat}`, { width: grow, gap: S.xs, cross: "stretch" }, () => {
    og.ui.row("Bar", { width: grow, gap: S.sm, cross: "center", padding: { left: S.sm, top: 0, right: S.sm, bottom: 0 } }, () => {
      og.ui.orbMeter("Energy", { value: v.energy, max: 3 + v.kindle, kind: "primary", size: phone ? 44 : 56, showValue: true, label: say(world, "hud.energy") });
      og.ui.column("Piles", { gap: 0 }, () => {
        og.ui.label("Draw", say(world, "hud.draw", { n: v.draw.length }), { step: "sm", nowrap: true, color: { token: "inkMuted" } });
        og.ui.label("Discard", say(world, "hud.discard", { n: v.discard.length }), { step: "sm", nowrap: true, color: { token: "inkMuted" } });
      });
      if (g.party > 1) og.ui.label("Who", heroWord(world, seat), { step: "lg", nowrap: true, color: { token: seat === 0 ? "primary" : "info" } });
      og.ui.spacer("Gap", grow);
      const label = v.ended && v.hp > 0 ? say(world, "hud.waiting") : say(world, "hud.end");
      if (og.ui.heroButton("End", label, { tone: "primary", disabled: !canAct, size: phone ? "medium" : "large", key: og.input.device(world) === "keyboardMouse" ? (LOCAL && seat === 1 ? "Bksp" : "R") : "" }).clicked && canAct && local >= 0) press("end", local);
    });
    const r = og.ui.cardHand(`Hand${seat}`, {
      cards,
      targets,
      disabled: !canAct,
      inspect: inspect[seat] ?? "",
      label: say(world, "hud.hand"),
      targetPrompt: say(world, "hud.target"),
      // From above the table needs the room: a smaller hand.
      ...(LOCAL ? { cardWidth: phone ? 84 : TOP ? 112 : 128 } : TOP && !phone ? { cardWidth: 124 } : {}),
    });
    if (r.inspect) inspect[seat] = r.inspect;
    if (r.dismissed) inspect[seat] = "";
    if (r.played && local >= 0) {
      const deckIndex = Number(r.played.key.split("-")[1]);
      ask(Ask.Card + deckIndex, local, r.played.target === null ? 0 : Number(r.played.target) + 1);
    }
  });
}

function battleHud(world: World, g: GameV): void {
  og.ui.setNavigation(false);
  if (og.ui.framePointer().pressed) keysDrive = false;
  const seats = Array.from({ length: SEATS }, (_, s) => heroOf(world, s));
  const playing = g.side === Side.Heroes && !g.cleared && !g.paused;
  // Top left: the heroes' frames (the rpg kit).
  og.ui.anchor("Party", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    for (let s = 0; s < g.party; s++) {
      const v = seats[s];
      if (!v) continue;
      rpg.unitFrame(`Frame${s}`, {
        name: heroWord(world, s),
        portrait: (portraits[HEROES[s]!] ?? null) as Texture | null,
        health: v.hp,
        healthMax: v.hpMax,
        shield: v.block,
        resource: v.energy,
        resourceMax: 3 + v.kindle,
        resourceKind: "mana",
        effects: effects(world, v),
        dead: v.hp <= 0,
        size: og.ui.frameViewport().width < 600 ? "small" : "medium",
        combat: true,
      });
    }
  });
  // Top centre: who acts, in order (the rpg kit).
  const units: TurnUnit[] = [];
  for (let s = 0; s < g.party; s++) {
    const v = seats[s];
    if (v) units.push({ key: `h${s}`, name: heroWord(world, s), portrait: (portraits[HEROES[s]!] ?? null) as Texture | null, team: 0, current: g.side === Side.Heroes && !v.ended && v.hp > 0, dead: v.hp <= 0 });
  }
  for (let slot = 0; slot < FOE_SLOTS; slot++) {
    const f = foeOf(world, slot);
    if (f && f.active) units.push({ key: `f${slot}`, name: say(world, `foe.${FOES[f.kind]!.id}`), portrait: (portraits[FOES[f.kind]!.id] ?? null) as Texture | null, team: 1, current: g.side === Side.Foes && g.acting === slot, dead: f.hp <= 0 });
  }
  if (og.ui.frameViewport().width >= 600) rpg.turnOrder("Turns", { units, size: 40, nowLabel: say(world, "hud.now"), hostileLabel: say(world, "hud.hostile") });
  og.ui.anchor("Corner", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    og.ui.row("Step", { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, background: { token: "hud" } }, () => {
      og.ui.icon("Icon", "map", { size: 20 });
      og.ui.label("Label", say(world, "hud.step", { n: g.step + 1, total: MAP.length }), { nowrap: true });
    });
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) press("pause");
  });
  // The foe the keys are on, for each local hero whose focused card needs one.
  const focus = new Map<number, number[]>();
  for (const s of mySeats(world, g)) {
    const v = seats[s];
    if (v && v.lastDir !== 0) keysDrive = true;
    if (!v || !playing || v.ended || v.hand.length === 0 || !keysDrive) continue;
    const code = v.deck[v.hand[Math.min(v.focusCard, v.hand.length - 1)]!]!;
    if (defOf(code).target) focus.set(v.focusFoe, [...(focus.get(v.focusFoe) ?? []), s]);
  }
  if (!TOP) heroTags(world, g);
  foeTags(world, g, seats, focus);
  // The turn's banner.
  const since = world.tick - g.bannerAt;
  og.ui.anchor("Banner", { anchor: "center", offsetY: -80 }, () => {
    og.ui.transition("Turn", since < 70 && g.banner > 0 && !g.cleared, { kind: prefs.reducedMotion ? "fade" : "pop" }, () => {
      og.ui.row("Plate", { padding: { left: S.xl, top: S.sm, right: S.xl, bottom: S.sm }, background: { token: "hud" } }, () => {
        og.ui.label("Text", say(world, g.banner === 1 ? (g.party > 1 ? "banner.yours2" : "banner.yours") : "banner.foes"), { step: "displaySm", font: "display", nowrap: true, color: { token: g.banner === 1 ? "primary" : "danger" } });
      });
    });
  });
  // Along the bottom, each local hero's hand.
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  // Two on a portrait phone: the hands one over the other (the Hunter's above), each the screen's width.
  const stacked = vp.width < vp.height;
  og.ui.anchor("Hands", { anchor: "bottom", direction: stacked ? "column" : "row", gap: stacked ? S.xs : S.md, cross: stacked ? "stretch" : "end", width: px(vp.width - sa.left - sa.right) }, () => {
    for (const s of stacked ? mySeats(world, g).reverse() : mySeats(world, g)) {
      const v = seats[s];
      if (v) hand(world, g, s, v, playing);
    }
  });
}

// ── between fights ──────────────────────────────────────────────────────

const NODE_ICON = ["sword", "skull", "fire", "crown"] as const;

function nodeName(world: World, kind: number, foes: readonly number[]): string {
  if (kind === Node.Rest) return say(world, "map.rest");
  const names = foes.map((f) => say(world, `foe.${FOES[f]!.id}`)).join(", ");
  return say(world, kind === Node.Boss ? "map.boss" : kind === Node.Elite ? "map.elite" : "map.fight", { foes: names });
}

function mapScreen(world: World, g: GameV): void {
  og.ui.setNavigation(true);
  const phone = og.ui.frameViewport().width < 600;
  og.ui.anchor("Map", { anchor: "center" }, () => {
    og.ui.column("Panel", { width: px(Math.min(560, og.ui.frameViewport().width - 32)), gap: S.lg, padding: { left: S.xl, top: S.xl, right: S.xl, bottom: S.xl }, background: { token: "hud" }, frame: "panel", cross: "center" }, () => {
      og.ui.label("Title", say(world, "map.title"), { step: phone ? "xl2" : "displaySm", font: "display", nowrap: true });
      og.ui.row("Road", { gap: phone ? S.xs : S.sm, cross: "center" }, () => {
        MAP.forEach((st, i) => {
          og.ui.column(og.ui.idi("Step", i), { gap: S.xxs, cross: "center" }, () => {
            st.options.forEach((o, k) => {
              const done = i < g.step;
              const here = i === g.step;
              og.ui.icon(og.ui.idi("Node", k), done ? "check" : NODE_ICON[o.kind]!, { size: here ? (phone ? 34 : 44) : phone ? 24 : 32, tint: { token: here ? "primary" : done ? "success" : "inkMuted" } });
            });
          });
          if (i < MAP.length - 1) og.ui.label(og.ui.idi("Link", i), "·", { color: { token: "inkFaint" } });
        });
      });
      og.ui.row("Party", { gap: S.lg }, () => {
        for (let s = 0; s < g.party; s++) {
          const v = heroOf(world, s);
          if (!v) continue;
          og.ui.row(og.ui.idi("Hero", s), { gap: S.xs, cross: "center" }, () => {
            og.ui.icon("Heart", "heart", { size: 20, tint: { token: "danger" } });
            og.ui.label("Hp", `${heroWord(world, s)} ${v.hp}/${v.hpMax}`, { nowrap: true });
            og.ui.label("Deck", say(world, "map.deck", { n: v.deck.length }), { nowrap: true, color: { token: "inkMuted" } });
          });
        }
      });
      og.ui.label("Next", say(world, MAP[g.step]!.options.length > 1 ? "map.choose" : "map.next"), { color: { token: "inkMuted" } });
      og.ui.column("Paths", { width: grow, gap: S.sm }, () => {
        MAP[g.step]!.options.forEach((o, k) => {
          if (og.ui.heroButton(og.ui.idi("Path", k), nodeName(world, o.kind, o.foes), { tone: k === 0 ? "primary" : "secondary", size: "large", width: grow }).clicked) ask(Ask.Path + k);
        });
      });
    });
  });
}

function rewardOption(world: World, code: number, key: string, tag = ""): RewardOption {
  const w = cardWords(world, code);
  return { key, title: w.name, kind: w.type, text: w.text, image: (cardArt[code & 255] ?? null) as Texture | null, rarity: rarity(code), tag, ...(tag ? { tagKind: "upgrade" as const } : {}) };
}

/** The first local seat that has not chosen yet, or -1 (every local seat has: the others are awaited). */
function chooser(world: World, g: GameV): number {
  for (const s of mySeats(world, g)) if ((g.decided & (1 << s)) === 0) return s;
  return -1;
}

function waiting(world: World): void {
  og.ui.anchor("Waiting", { anchor: "center" }, () => {
    og.ui.row("Plate", { gap: S.sm, cross: "center", padding: { left: S.xl, top: S.md, right: S.xl, bottom: S.md }, background: { token: "hud" } }, () => {
      og.ui.spinner("Spin", 20);
      og.ui.label("Text", say(world, "reward.waiting"), { nowrap: true });
    });
  });
}

function rewardScreen(world: World, g: GameV): void {
  og.ui.setNavigation(true);
  const seat = chooser(world, g);
  if (seat < 0) return waiting(world);
  const local = localOf(world, seat);
  const offer = seat === 0 ? g.rewards0 : g.rewards1;
  const r = og.ui.rewardChoice(`Reward${seat}`, {
    options: offer.map((code, i) => rewardOption(world, code, `r${i}`)),
    title: say(world, "reward.title"),
    subtitle: say(world, g.party > 1 ? "reward.subtitleFor" : "reward.subtitle", { who: heroWord(world, seat) }),
    skipLabel: say(world, "reward.skip"),
  });
  const picked = r.picked[0];
  if (picked) ask(Ask.Reward + Number(picked.slice(1)), local);
  else if (r.skipped) ask(Ask.Skip, local);
}

function restScreen(world: World, g: GameV): void {
  og.ui.setNavigation(true);
  const seat = chooser(world, g);
  if (seat < 0) return waiting(world);
  const local = localOf(world, seat);
  const v = heroOf(world, seat);
  if (!v) return;
  const tempers = tempersOf(g, seat);
  if (tempers.length === 0) {
    const heal = Math.min(v.hpMax - v.hp, Math.round(v.hpMax * 0.3));
    const r = og.ui.rewardChoice(`Rest${seat}`, {
      options: [
        { key: "heal", title: say(world, "rest.heal"), kind: say(world, "rest.fire"), text: say(world, "rest.healText", { n: heal }), icon: "heart", rarity: "uncommon" },
        { key: "temper", title: say(world, "rest.temper"), kind: say(world, "rest.fire"), text: say(world, "rest.temperText"), icon: "fire", rarity: "rare" },
      ],
      title: say(world, "rest.title"),
      subtitle: say(world, g.party > 1 ? "rest.subtitleFor" : "rest.subtitle", { who: heroWord(world, seat), hp: v.hp, max: v.hpMax }),
    });
    const p = r.picked[0];
    if (p === "heal") ask(Ask.Heal, local);
    if (p === "temper") ask(Ask.Temper, local);
    return;
  }
  const r = og.ui.rewardChoice(`Temper${seat}`, {
    options: tempers.map((i, k) => rewardOption(world, v.deck[i]! | UPGRADED, `t${k}`, say(world, "rest.upgrade"))),
    title: say(world, "rest.temper"),
    subtitle: say(world, "rest.temperPick"),
  });
  const p = r.picked[0];
  if (p) ask(Ask.TemperPick + Number(p.slice(1)), local);
}

// ── the settings, the summary, the documents ───────────────────────────

function settings(world: World): void {
  let done = false;
  const m = og.ui.modal("Settings", settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    prefs.master = slider("Master", "settings.master", prefs.master);
    prefs.music = slider("Music", "settings.music", prefs.music);
    prefs.effects = slider("Effects", "settings.effects", prefs.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), prefs.language, { horizontal: true });
    if (r.selected !== prefs.language) setLanguage(world, r.selected);
    prefs.reducedMotion = og.ui.toggle("Motion", prefs.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (settingsOpen && (done || m.dismissed)) {
    settingsOpen = false;
    must(og.platform.saveSettings(world, { masterVolume: prefs.master, musicVolume: prefs.music, sfxVolume: prefs.effects, language: LANGUAGES[prefs.language]!.code, reducedMotion: prefs.reducedMotion }));
    prefs.saves += 1;
  }
}

function summary(world: World, g: GameV): void {
  og.ui.modal("Summary", g.phase === Phase.Over, { title: say(world, g.won ? "over.won" : "over.lost"), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", g.won ? "crown" : "skull", { size: 48, tint: { token: g.won ? "primary" : "inkMuted" } }));
      if (g.won && g.overTicks === 20) og.ui.burst({ count: 14 });
    });
    og.ui.label("Line", say(world, g.won ? "over.wonLine" : "over.lostLine", { foe: g.killedBy >= 0 ? say(world, `foe.${FOES[g.killedBy]!.id}`) : "" }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Step", say(world, "over.step"), `${Math.min(g.step + 1, MAP.length)}/${MAP.length}`, true);
      og.ui.keyValue("Fights", say(world, "over.fights"), `${g.fights}`, true);
      og.ui.keyValue("Turns", say(world, "over.turns"), `${g.turns}`, true);
      og.ui.keyValue("Played", say(world, "over.played"), `${g.played}`, true);
      og.ui.keyValue("Dealt", say(world, "over.dealt"), `${g.dealt}`, true);
      og.ui.keyValue("Taken", say(world, "over.taken"), `${g.taken}`, true);
    });
    og.ui.label("Record", say(world, "over.record", { runs: g.runs, wins: g.wins }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "over.again"), { variant: "primary", width: grow }).clicked) ask(Ask.Again);
      if (og.ui.button("Menu", say(world, "over.menu"), { width: grow }).clicked) ask(Ask.Title);
    });
  });
}

function layer(world: World, name: string, shown: boolean): string[] {
  if (!shown) {
    if (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, {}).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

let wasPhase = -1;

/** The screens, the buses, the platform's answers; the time held while paused. */
export const Ui = defineSystem("Ui", { reads: [Hero, Foe, og.Lockstep, og.Transform, og.Camera], writes: [Game, og.Audio] }, (_q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  platform(world, g);
  applyPrefs(world);
  if (LOCAL) og.ui.setPlayers(2);
  // The record: a run begun is counted, a run won too.
  if (g.phase !== wasPhase) {
    if (g.phase === Phase.Map && g.step === 0 && wasPhase !== Phase.Map) saveRecord(world, g);
    if (g.phase === Phase.Over) {
      if (g.won) g.wins += 1;
      saveRecord(world, g);
      must(og.platform.telemetry(world, "run.over", { won: g.won, step: g.step, fights: g.fights, turns: g.turns, played: g.played }));
    }
    wasPhase = g.phase;
  }
  const hold = g.phase !== Phase.Title && g.phase !== Phase.Over && (g.paused || (settingsOpen && !inRoom(world)));
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  const names = ["title", "map", "battle", "reward", "rest", "over"];
  const shown = og.ui.screenTransition("Screens", names[g.phase === Phase.Reward ? Phase.Battle : g.phase]!, { kind: prefs.reducedMotion ? "fade" : "dissolve", durationMs: 500 }).shown;
  for (const e of layer(world, "main-menu", shown === "title")) {
    if (!settingsOpen && e === "play") ask(Ask.Begin);
    if (e === "settings") settingsOpen = true;
  }
  const pauseUp = g.paused && !settingsOpen && shown !== "title" && shown !== "over";
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") press("pause");
    if (e === "settings") settingsOpen = true;
    if (e === "quit") ask(Ask.Title);
  }
  if (shown === "title") og.ui.setNavigation(true);
  else if (shown === "map") mapScreen(world, g);
  else if (shown === "battle") {
    if (g.phase === Phase.Reward) rewardScreen(world, g);
    else battleHud(world, g);
  } else if (shown === "rest") restScreen(world, g);
  else if (shown === "over") {
    og.ui.setNavigation(true);
    summary(world, g);
  }
  settings(world);
});

export { CARDS, ONLINE };
