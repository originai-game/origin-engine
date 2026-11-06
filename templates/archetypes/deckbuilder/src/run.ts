// Ashen Crown's rules: the Run system, after the screens (src/ui.ts) and
// before the stage (src/stage.ts). Every change to the run comes from a
// seat's og.PlayerInput (src/seats.ts), so alone, two on one device and
// two over a room in lockstep play the same rules on the same ticks:
//
//   the play bit   with aimX 0 (Space, E, Enter, /, a pad's A) plays the
//                  card the d-pad is on at the foe it is on; with aimX a
//                  card's deck index + 1 (a tap or a drag on the hand) that
//                  card at foe slot aimY - 1; with a menu's code (Ask.* of
//                  src/components.ts) that choice;
//   the end bit    (R, Backspace, a pad's Y, End turn) ends the seat's turn;
//   the move       left and right walk the hand, up and down the foes;
//   pause          pauses and resumes.
//
// A fight: each hero up plays cards with its own energy (3 a turn) from
// its own hand (5 drawn a turn from its own draw pile, the discard pile
// shuffled back in when it runs out) and ends its turn; once every hero up
// has, the foes act one by one on a beat, each the move its intent showed;
// then the heroes' next turn. Every shuffle and reward is drawn from the
// world's og.Rng, so a run replays from its inputs and every peer of a
// room draws the same cards.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, RngStream, Value, World } from "@origingame/engine";
import { CARDS, costOf, defOf, DECK_MAX, HAND_MAX, numbers, REWARD_POOL, STARTER_DECKS, UPGRADED, isUpgraded } from "./cards.ts";
import { Ask, Foe, foeName, Game, Hero, heroName, Phase, Side, Sound, Tuning } from "./components.ts";
import type { FoeV, GameV, HeroV } from "./components.ts";
import { prefs } from "./device.ts";
import { FOES, FOE_SLOTS, foeHp, Intent, MAP, Node } from "./foes.ts";
import type { Move } from "./foes.ts";
import { LOCAL, ONLINE, pressedNow, SEATS } from "./seats.ts";
import { fightFoes, foeSpot, heroSpot, lunge, TOP, unitSize } from "./stage.ts";

type TuneV = Value<typeof Tuning>;

const WHITE = { r: 1, g: 1, b: 1 };
const GOLD = { r: 1, g: 0.8, b: 0.3 };
const RED = { r: 1, g: 0.3, b: 0.26 };
const BLUE = { r: 0.45, g: 0.75, b: 1 };
const GREEN = { r: 0.5, g: 1, b: 0.4 };

/** Ticks a fight's end waits before the reward (the last foe's fall), and the run's end before the summary. */
export const END_DELAY = 60;
/** The pool of cards that fly to their target. */
export const FLY_SLOTS = 4;

export function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

export function must2<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The camera feel's scale: 0.3 under this device's reduced motion (og-feel). */
export function feel(): number {
  return prefs.reducedMotion ? 0.3 : 1;
}

/** A room runs this world in lockstep: what is this device's own (its settings, its record) stays out of it. */
export function inRoom(world: World): boolean {
  if (!ONLINE) return false;
  const lock = world.resource(og.Lockstep);
  return lock.ok && lock.value.running;
}

/** The textures of the cards' art, by card (loaded in start). */
export const cardArt: number[] = [];

interface Unit<V> {
  e: Entity;
  v: V;
}

/** One tick's working copy of the heroes and the foes, written back once at its end. */
export interface Ctx {
  world: World;
  g: GameV;
  t: TuneV;
  rng: RngStream;
  heroes: Unit<HeroV>[];
  foes: Unit<FoeV>[];
}

function copyHero(v: HeroV): HeroV {
  return { ...v, deck: [...v.deck], draw: [...v.draw], hand: [...v.hand], discard: [...v.discard], fresh: [...v.fresh] };
}

export function load(world: World): Ctx | null {
  const game = world.resource(Game);
  const rng = world.resource(og.Rng);
  if (!game.ok || !rng.ok) return null;
  const heroes: Unit<HeroV>[] = [];
  for (let s = 0; s < SEATS; s++) {
    const e = world.find(heroName(s));
    if (e === null) return null;
    heroes.push({ e, v: copyHero(must2(world.get(e, Hero))) });
  }
  const foes: Unit<FoeV>[] = [];
  for (let i = 0; i < FOE_SLOTS; i++) {
    const e = world.find(foeName(i));
    if (e === null) return null;
    foes.push({ e, v: { ...must2(world.get(e, Foe)) } });
  }
  return { world, g: game.value, t: must2(world.resource(Tuning)), rng: rng.value, heroes, foes };
}

export function save(c: Ctx): void {
  for (const h of c.heroes) must(c.world.set(h.e, Hero, h.v));
  for (const f of c.foes) must(c.world.set(f.e, Foe, f.v));
}

const up = (h: Unit<HeroV>) => h.v.active && h.v.hp > 0;
const alive = (f: Unit<FoeV>) => f.v.active && f.v.hp > 0;

export function partyDown(c: Ctx): boolean {
  return !c.heroes.some(up);
}

function float(c: Ctx, text: string, x: number, y: number, color: { r: number; g: number; b: number }, crit = false, size = 0.6): void {
  must(og.render.floatingText(c.world, { text, position: { x, y, z: 1 }, size, color, crit, rise: 1.1, lifetime: 0.9, pop: 1.3 }));
}

function spotOf(c: Ctx, e: Entity): { x: number; y: number } {
  const t = c.world.get(e, og.Transform);
  return t.ok ? { x: t.value.position.x, y: t.value.position.y } : { x: 0, y: 0 };
}

/** The top of a unit, where its numbers rise from. */
function headOf(c: Ctx, e: Entity, size: number): { x: number; y: number } {
  const p = spotOf(c, e);
  return { x: p.x, y: p.y + size * 0.85 };
}

// ── damage and block ────────────────────────────────────────────────────

/** An attack's damage: its base plus strength, a quarter less weakened, half again on the vulnerable. */
export function damageOf(base: number, strength: number, weak: number, vulnerable: number): number {
  let d = base + strength;
  if (weak > 0) d = Math.floor(d * 0.75);
  if (vulnerable > 0) d = Math.floor(d * 1.5);
  return Math.max(0, d);
}

/** Hits a foe: block first, then health; the pop, the flash, the hit stop. */
function hitFoe(c: Ctx, f: Unit<FoeV>, dmg: number, heavy: boolean): void {
  const v = f.v;
  const blocked = Math.min(v.block, dmg);
  v.block -= blocked;
  const lost = Math.min(v.hp, dmg - blocked);
  v.hp -= lost;
  c.g.dealt += lost;
  // The damage pops from the body (the intent stands over the head), a little aside each time.
  const at = spotOf(c, f.e);
  const head = { x: at.x + (c.rng.next() - 0.5) * 0.8, y: at.y + unitSize(v.kind) * 0.55 };
  float(c, lost > 0 ? `${lost}` : "0", head.x, head.y, lost >= 12 ? GOLD : WHITE, lost >= 12, lost >= 12 ? 1.25 : 0.95);
  if (blocked > 0) float(c, `-${blocked}`, head.x + 0.8, head.y - 0.5, BLUE, false, 0.45);
  must(og.tween.flash(c.world, { entity: f.e, color: WHITE, duration: 0.14 }));
  must(og.tween.squash(c.world, { entity: f.e, impact: TOP ? { x: 0.82, y: 0.82 } : { x: 0.8, y: 1.18 }, duration: 0.2 }));
  c.g.sfx |= heavy || lost >= 12 ? Sound.Heavy : blocked > 0 && lost === 0 ? Sound.Block : Sound.Hit;
  must(og.time.hitStop(c.world, heavy ? c.t.hitStop + 3 : c.t.hitStop));
  must(og.camera.shake(c.world, (heavy ? 0.3 : 0.14) * feel()));
  if (v.hp <= 0) fell(c, f);
}

function fell(c: Ctx, f: Unit<FoeV>): void {
  f.v.hp = 0;
  f.v.dying = 1;
  f.v.block = 0;
  c.g.sfx |= Sound.Die;
  const d = FOES[f.v.kind]!;
  must(og.time.hitStop(c.world, d.boss ? 14 : d.elite ? 10 : 6));
  must(og.camera.shake(c.world, (d.boss ? 0.6 : 0.3) * feel()));
  if (d.boss || d.elite) must(og.camera.punch(c.world, 0.08 * feel()));
  must(og.tween.squash(c.world, { entity: f.e, impact: TOP ? { x: 1.3, y: 1.3 } : { x: 0.4, y: 1.6 }, duration: 0.6 }));
}

/** Hits a hero: block first; the red pop, the flash, the kick. */
function hitHero(c: Ctx, h: Unit<HeroV>, dmg: number, from: Unit<FoeV>): void {
  const v = h.v;
  const blocked = Math.min(v.block, dmg);
  v.block -= blocked;
  const lost = Math.min(v.hp, dmg - blocked);
  v.hp -= lost;
  c.g.taken += lost;
  const head = headOf(c, h.e, 3);
  if (lost > 0) float(c, `-${lost}`, head.x, head.y, RED, lost >= 10, lost >= 10 ? 1.2 : 0.95);
  if (blocked > 0) float(c, lost > 0 ? `-${blocked}` : `${blocked}`, head.x - 0.8, head.y - 0.4, BLUE, false, 0.5);
  must(og.tween.flash(c.world, { entity: h.e, color: lost > 0 ? RED : BLUE, duration: 0.18 }));
  must(og.tween.squash(c.world, { entity: h.e, impact: TOP ? { x: 0.85, y: 0.85 } : { x: 1.2, y: 0.8 }, duration: 0.2 }));
  c.g.sfx |= lost > 0 ? Sound.Hurt : Sound.Block;
  const k = feel();
  must(og.time.hitStop(c.world, lost >= 10 ? 8 : 4));
  must(og.camera.shake(c.world, (lost >= 10 ? 0.4 : lost > 0 ? 0.22 : 0.08) * k));
  const a = spotOf(c, h.e);
  const b = spotOf(c, from.e);
  must(og.camera.kick(c.world, { x: a.x - b.x, y: a.y - b.y, z: 0 }, (lost > 0 ? 0.14 : 0.05) * k));
  if (v.hp <= 0) {
    v.hp = 0;
    v.block = 0;
    v.ended = true;
    if (partyDown(c)) c.g.killedBy = from.v.kind;
  }
}

function gainBlock(c: Ctx, h: Unit<HeroV>, n: number): void {
  h.v.block += n;
  const head = headOf(c, h.e, 3);
  float(c, `+${n}`, head.x, head.y, BLUE, false, 0.55);
  must(og.tween.squash(c.world, { entity: h.e, impact: { x: 1.1, y: 0.92 }, duration: 0.16 }));
  c.g.sfx |= Sound.Block;
}

// ── cards ───────────────────────────────────────────────────────────────

/** Draws n cards into a hand: the discard pile shuffled back in (og.Rng) when the draw pile runs out. */
export function drawCards(c: Ctx, h: Unit<HeroV>, n: number): void {
  const v = h.v;
  const fresh: number[] = v.freshAt === c.world.tick ? [...v.fresh] : [];
  const hand = [...v.hand];
  let draw = [...v.draw];
  for (let i = 0; i < n && hand.length < HAND_MAX; i++) {
    if (draw.length === 0) {
      if (v.discard.length === 0) break;
      draw = shuffled(c, v.discard);
      v.discard = [];
    }
    const top = draw.pop()!;
    hand.push(top);
    fresh.push(top);
  }
  v.hand = hand;
  v.draw = draw;
  v.fresh = fresh.slice(-HAND_MAX);
  v.freshAt = c.world.tick;
  if (n > 0) c.g.sfx |= Sound.Draw;
}

function shuffled(c: Ctx, a: readonly number[]): number[] {
  return c.rng.permutation(a.length).map((i) => a[i]!);
}

function refuse(c: Ctx, h: Unit<HeroV>, deckIndex: number): void {
  h.v.refusedAt = c.world.tick;
  h.v.refused = deckIndex;
}

/** The other hero up (the ally a Ward or a Rally reaches), or none. */
function ally(c: Ctx, h: Unit<HeroV>): Unit<HeroV> | null {
  return c.heroes.find((o) => o !== h && up(o)) ?? null;
}

/** Plays a card of a hero's hand on a foe slot (-1: the focused one, or the first up). True when it was played. */
export function playCard(c: Ctx, h: Unit<HeroV>, deckIndex: number, slot: number): boolean {
  const v = h.v;
  const at = v.hand.indexOf(deckIndex);
  if (at < 0) return false;
  const code = v.deck[deckIndex]!;
  const def = defOf(code);
  const cost = costOf(code);
  let target: Unit<FoeV> | null = null;
  if (def.target) {
    target = (slot >= 0 && c.foes[slot] && alive(c.foes[slot]!) ? c.foes[slot]! : null) ?? (alive(c.foes[v.focusFoe] ?? c.foes[0]!) ? c.foes[v.focusFoe]! : null) ?? c.foes.find(alive) ?? null;
  }
  if (cost > v.energy || (def.target && target === null)) {
    refuse(c, h, deckIndex);
    return false;
  }
  v.energy -= cost;
  v.hand = v.hand.filter((_, k) => k !== at);
  if (!def.exhaust && def.kind !== "power") v.discard = [...v.discard, deckIndex];
  v.focusCard = Math.min(v.focusCard, Math.max(0, v.hand.length - 1));
  c.g.played += 1;
  c.g.sfx |= Sound.Card;
  const { a, b } = numbers(code);
  const me = spotOf(c, h.e);
  const atk = (f: Unit<FoeV>, base: number, heavy = false) => hitFoe(c, f, damageOf(base, v.strength, v.weak, f.v.vulnerable), heavy);
  const swing = () => {
    const to = lunge(me, 1);
    must(og.tween.sequence(c.world, [
      { entity: h.e, field: "position", to: [to.x, to.y, 0], duration: 0.06, ease: "quadOut" },
      { entity: h.e, field: "position", to: [me.x, me.y, 0], duration: 0.12, ease: "quadIn" },
    ]));
  };
  fly(c, code, me, target ? spotOf(c, target.e) : TOP ? { x: me.x, y: me.y + 4 } : { x: me.x + 1.5, y: me.y + 3.5 });
  switch (def.id) {
    case "strike":
    case "heavy-blow":
      swing();
      atk(target!, a, def.id === "heavy-blow");
      break;
    case "bash":
      swing();
      atk(target!, a, true);
      if (target!.v.hp > 0) status(c, target!, "vulnerable", b);
      break;
    case "sunder":
      swing();
      atk(target!, a);
      if (target!.v.hp > 0) status(c, target!, "weak", b);
      break;
    case "venom-dart":
      swing();
      atk(target!, a);
      if (target!.v.hp > 0) status(c, target!, "poison", b);
      break;
    case "flurry":
      swing();
      for (let i = 0; i < b && target!.v.hp > 0; i++) atk(target!, a);
      break;
    case "cleave":
      swing();
      for (const f of c.foes) if (alive(f)) atk(f, a);
      break;
    case "toxic-cloud":
      for (const f of c.foes) if (alive(f)) status(c, f, "poison", a);
      break;
    case "defend":
      gainBlock(c, h, a);
      break;
    case "ward": {
      gainBlock(c, h, a);
      const o = ally(c, h);
      if (o) gainBlock(c, o, b);
      break;
    }
    case "insight":
      drawCards(c, h, a);
      break;
    case "adrenaline":
      v.energy += a;
      drawCards(c, h, b);
      buff(c, h, `+${a}`);
      break;
    case "rally": {
      drawCards(c, h, a);
      const o = ally(c, h);
      if (o) drawCards(c, o, b);
      break;
    }
    case "iron-hymn":
      v.strength += a;
      buff(c, h, `+${a}`);
      break;
    case "kindle":
      v.kindle += a;
      buff(c, h, `+${a}`);
      break;
    default:
      throw new Error(`card ${def.id} has no rule`);
  }
  return true;
}

function buff(c: Ctx, h: Unit<HeroV>, text: string): void {
  const head = headOf(c, h.e, 3);
  float(c, text, head.x, head.y, GOLD, false, 0.55);
  must(og.tween.flash(c.world, { entity: h.e, color: GOLD, duration: 0.25 }));
  c.g.sfx |= Sound.Buff;
}

function status(c: Ctx, f: Unit<FoeV>, what: "vulnerable" | "weak" | "poison", n: number): void {
  f.v[what] += n;
  const head = headOf(c, f.e, unitSize(f.v.kind));
  float(c, `+${n}`, head.x - 0.9, head.y + 0.4, what === "poison" ? GREEN : GOLD, false, 0.45);
  if (what === "poison") c.g.sfx |= Sound.Poison;
}

/** The played card arcs from the hero to its target (one of the pooled flying cards). */
function fly(c: Ctx, code: number, from: { x: number; y: number }, to: { x: number; y: number }): void {
  const e = c.world.find(`fly-${c.g.flyNext % FLY_SLOTS}`);
  c.g.flyNext = (c.g.flyNext + 1) % FLY_SLOTS;
  if (e === null) return;
  const s = { x: from.x, y: from.y + 2 };
  const t = { x: to.x, y: to.y + 1.5 };
  const peak = { x: (s.x + t.x) / 2, y: Math.max(s.y, t.y) + 2.2 };
  must(c.world.set(e, og.Sprite, { texture: (cardArt[code & 255] ?? 0) as never, opacity: 1, visible: true }));
  must(c.world.set(e, og.Transform, { position: { x: s.x, y: s.y, z: 2 } }));
  must(og.tween.sequence(c.world, [
    { entity: e, field: "position", to: [peak.x, peak.y, 2], duration: 0.11, ease: "quadOut" },
    { entity: e, field: "position", to: [t.x, t.y, 2], duration: 0.11, ease: "quadIn" },
    { entity: e, component: og.Sprite, field: "opacity", to: [0], duration: 0.12, ease: "quadOut" },
  ]));
}

// ── turns ───────────────────────────────────────────────────────────────

/** A foe's next move. */
export function moveOf(v: FoeV): Move {
  const d = FOES[v.kind]!;
  return d.moves[v.move % d.moves.length]!;
}

/** The heroes' turn: block gone, energy back, five cards, each foe's intent and target chosen. */
function heroTurn(c: Ctx): void {
  const g = c.g;
  g.turn += 1;
  g.turns += 1;
  g.side = Side.Heroes;
  g.acting = -1;
  g.banner = 1;
  g.bannerAt = c.world.tick;
  g.sfx |= Sound.Turn;
  for (const h of c.heroes) {
    if (!up(h)) continue;
    h.v.block = 0;
    h.v.energy = c.t.energy + h.v.kindle;
    h.v.ended = false;
    drawCards(c, h, c.t.draw);
    h.v.focusCard = 0;
  }
  const ups = c.heroes.filter(up);
  for (const f of c.foes) {
    if (!alive(f)) continue;
    const m = moveOf(f.v);
    f.v.target = m.kind === Intent.AttackAll ? -1 : ups.length > 0 ? ups[c.rng.integer(0, ups.length - 1)]!.v.seat : 0;
  }
  for (const h of c.heroes) if (up(h) && !alive(c.foes[h.v.focusFoe]!)) h.v.focusFoe = c.foes.findIndex(alive);
}

/** Every hero up has ended its turn: their hands are discarded and the foes act. */
function foesTurn(c: Ctx): void {
  for (const h of c.heroes) {
    if (!h.v.active) continue;
    h.v.discard = [...h.v.discard, ...h.v.hand];
    h.v.hand = [];
    if (h.v.weak > 0) h.v.weak -= 1;
    if (h.v.vulnerable > 0) h.v.vulnerable -= 1;
  }
  c.g.side = Side.Foes;
  c.g.acting = -1;
  c.g.timer = 24;
  c.g.banner = 2;
  c.g.bannerAt = c.world.tick;
  c.g.sfx |= Sound.FoeTurn;
}

/** One foe's beat: its block falls, its poison bites, then it does what its intent said. */
function foeAct(c: Ctx, f: Unit<FoeV>): void {
  const v = f.v;
  v.block = 0;
  if (v.poison > 0) {
    const n = Math.min(v.hp, v.poison);
    v.hp -= n;
    c.g.dealt += n;
    v.poison -= 1;
    const head = headOf(c, f.e, unitSize(v.kind));
    float(c, `${n}`, head.x, head.y, GREEN, false, 0.6);
    must(og.tween.flash(c.world, { entity: f.e, color: GREEN, duration: 0.2 }));
    c.g.sfx |= Sound.Poison;
    if (v.hp <= 0) {
      fell(c, f);
      return;
    }
  }
  const m = moveOf(v);
  const ups = c.heroes.filter(up);
  const pick = () => c.heroes.find((h) => h.v.seat === v.target && up(h)) ?? ups[0] ?? null;
  const me = spotOf(c, f.e);
  const strikeAt = (h: Unit<HeroV>) => {
    for (let i = 0; i < (m.hits ?? 1) && h.v.hp > 0; i++) hitHero(c, h, damageOf(m.dmg ?? 0, v.strength, v.weak, h.v.vulnerable), f);
    if (h.v.hp > 0 && m.vulnerable) h.v.vulnerable += m.vulnerable;
    if (h.v.hp > 0 && m.weak) h.v.weak += m.weak;
  };
  if (m.kind === Intent.Attack || m.kind === Intent.AttackAll || m.kind === Intent.AttackBlock) {
    const to = lunge(me, -1, 0.9);
    must(og.tween.sequence(c.world, [
      { entity: f.e, field: "position", to: [to.x, to.y, 0], duration: 0.07, ease: "quadOut" },
      { entity: f.e, field: "position", to: [me.x, me.y, 0], duration: 0.14, ease: "quadIn" },
    ]));
    if (m.kind === Intent.AttackAll) for (const h of ups) strikeAt(h);
    else {
      const h = pick();
      if (h) strikeAt(h);
    }
  }
  if (m.kind === Intent.Debuff) {
    const h = pick();
    if (h) {
      h.v.weak += m.weak ?? 0;
      h.v.vulnerable += m.vulnerable ?? 0;
      const head = headOf(c, h.e, 3);
      float(c, "↓", head.x, head.y, GREEN, false, 0.6);
      must(og.tween.flash(c.world, { entity: h.e, color: GREEN, duration: 0.25 }));
      c.g.sfx |= Sound.Poison;
    }
  }
  if (m.block) {
    v.block += m.block;
    const head = headOf(c, f.e, unitSize(v.kind));
    float(c, `+${m.block}`, head.x, head.y, BLUE, false, 0.55);
    c.g.sfx |= Sound.Block;
  }
  if (m.strength) {
    v.strength += m.strength;
    const head = headOf(c, f.e, unitSize(v.kind));
    float(c, `+${m.strength}`, head.x + 0.6, head.y + 0.5, GOLD, false, 0.5);
    must(og.tween.flash(c.world, { entity: f.e, color: GOLD, duration: 0.25 }));
    c.g.sfx |= Sound.Buff;
  }
  if (v.weak > 0) v.weak -= 1;
  if (v.vulnerable > 0) v.vulnerable -= 1;
  v.move += 1;
}

// ── the run ─────────────────────────────────────────────────────────────

/** A new run: the heroes whole with their starting decks, the map's first step. */
export function startRun(c: Ctx): void {
  const g = c.g;
  g.seed = c.rng.integer(1, 1_000_000_000);
  g.party = SEATS > 1 && (!ONLINE || inRoom(c.world)) ? SEATS : 1;
  g.phase = Phase.Map;
  g.paused = false;
  g.step = 0;
  g.path = 0;
  g.won = false;
  g.overTicks = 0;
  g.fights = 0;
  g.played = 0;
  g.dealt = 0;
  g.taken = 0;
  g.turns = 0;
  g.killedBy = -1;
  g.decided = 0;
  g.cleared = false;
  g.runs += 1;
  for (const h of c.heroes) {
    const v = h.v;
    v.active = v.seat < g.party;
    v.hpMax = c.t.heroHp;
    v.hp = v.active ? c.t.heroHp : 0;
    v.deck = [...STARTER_DECKS[v.seat]!];
    v.draw = [];
    v.hand = [];
    v.discard = [];
    v.block = 0;
    v.energy = 0;
    v.strength = 0;
    v.weak = 0;
    v.vulnerable = 0;
    v.kindle = 0;
    v.ended = false;
    const s = heroSpot(v.seat, g.party);
    must(c.world.set(h.e, og.Transform, { position: { x: s.x, y: s.y, z: 0 } }));
  }
  for (const f of c.foes) f.v.active = false;
}

/** The step's node, taken: a fight (its foes in their slots, the decks shuffled) or a fire. */
export function enter(c: Ctx, path: number): void {
  const g = c.g;
  const step = MAP[g.step];
  if (!step) return;
  g.path = Math.max(0, Math.min(step.options.length - 1, path));
  const node = step.options[g.path]!;
  if (node.kind === Node.Rest) {
    g.phase = Phase.Rest;
    g.decided = inactiveBits(c);
    g.tempers0 = [];
    g.tempers1 = [];
    return;
  }
  g.phase = Phase.Battle;
  g.turn = 0;
  g.cleared = false;
  g.timer = 0;
  const kinds = fightFoes(g.step, g.path);
  c.foes.forEach((f, i) => {
    const kind = kinds[i];
    const v = f.v;
    v.active = kind !== undefined;
    v.dying = 0;
    if (kind === undefined) return;
    v.kind = kind;
    v.hpMax = foeHp(FOES[kind]!, g.party);
    v.hp = v.hpMax;
    v.block = 0;
    v.strength = 0;
    v.weak = 0;
    v.vulnerable = 0;
    v.poison = 0;
    // Two of a kind start on different moves, so they do not move as one.
    v.move = kinds.slice(0, i).filter((k) => k === kind).length;
    const s = foeSpot(i, kinds.length);
    must(c.world.set(f.e, og.Transform, { position: { x: s.x, y: s.y, z: 0 } }));
  });
  for (const h of c.heroes) {
    const v = h.v;
    if (!v.active) continue;
    v.draw = shuffled(c, v.deck.map((_, i) => i));
    v.hand = [];
    v.discard = [];
    v.block = 0;
    v.strength = 0;
    v.weak = 0;
    v.vulnerable = 0;
    v.kindle = 0;
    v.focusFoe = 0;
  }
  heroTurn(c);
}

function inactiveBits(c: Ctx): number {
  let bits = 0;
  for (const h of c.heroes) if (!h.v.active) bits |= 1 << h.v.seat;
  return bits;
}

const allDecided = (c: Ctx) => c.heroes.every((h) => (c.g.decided & (1 << h.v.seat)) !== 0);

/** A fight won: the boss is the run; else each hero's reward of three cards, and the fallen rise. */
function fightWon(c: Ctx): void {
  const g = c.g;
  g.fights += 1;
  for (const f of c.foes) f.v.active = false;
  if (MAP[g.step]!.options[g.path]!.kind === Node.Boss) {
    g.won = true;
    g.phase = Phase.Over;
    g.overTicks = 0;
    g.sfx |= Sound.Won;
    return;
  }
  for (const h of c.heroes) {
    const v = h.v;
    if (!v.active) continue;
    v.hand = [];
    v.draw = [];
    v.discard = [];
    v.block = 0;
    if (v.hp <= 0) v.hp = Math.ceil(v.hpMax / 4);
    const pool = c.rng.permutation(REWARD_POOL.length).slice(0, 3).map((i) => REWARD_POOL[i]!);
    if (v.seat === 0) g.rewards0 = pool;
    else g.rewards1 = pool;
  }
  g.phase = Phase.Reward;
  g.decided = inactiveBits(c);
  g.sfx |= Sound.Heal;
}

/** On to the map's next step. */
function onward(c: Ctx): void {
  c.g.step += 1;
  c.g.path = 0;
  c.g.phase = Phase.Map;
}

function rewardsOf(g: GameV, seat: number): readonly number[] {
  return seat === 0 ? g.rewards0 : g.rewards1;
}

export function tempersOf(g: GameV, seat: number): readonly number[] {
  return seat === 0 ? g.tempers0 : g.tempers1;
}

/** A seat's choice on a menu (its play bit with a menu's code). */
function choose(c: Ctx, h: Unit<HeroV>, ask: number): void {
  const g = c.g;
  const seat = h.v.seat;
  const bit = 1 << seat;
  if (g.phase === Phase.Title || g.phase === Phase.Over) {
    if (ask === Ask.Begin || ask === Ask.Again) startRun(c);
    else if (ask === Ask.Title && g.phase === Phase.Over) g.phase = Phase.Title;
    return;
  }
  if (g.paused) return;
  if (g.phase === Phase.Map && ask >= Ask.Path && ask < Ask.Path + 2) {
    enter(c, ask - Ask.Path);
    return;
  }
  if (g.phase === Phase.Reward && h.v.active && (g.decided & bit) === 0) {
    const offer = rewardsOf(g, seat);
    if (ask >= Ask.Reward && ask < Ask.Reward + offer.length && h.v.deck.length < DECK_MAX) {
      h.v.deck = [...h.v.deck, offer[ask - Ask.Reward]!];
      g.decided |= bit;
      g.sfx |= Sound.Card;
    } else if (ask === Ask.Skip) g.decided |= bit;
    if (allDecided(c)) onward(c);
    return;
  }
  if (g.phase === Phase.Rest && h.v.active && (g.decided & bit) === 0) {
    if (ask === Ask.Heal) {
      const n = Math.min(h.v.hpMax - h.v.hp, Math.round(h.v.hpMax * c.t.restHeal));
      h.v.hp = Math.max(1, h.v.hp + n);
      g.decided |= bit;
      g.sfx |= Sound.Heal;
      const head = headOf(c, h.e, 3);
      float(c, `+${n}`, head.x, head.y, GREEN, false, 0.7);
      must(og.tween.flash(c.world, { entity: h.e, color: GREEN, duration: 0.3 }));
    } else if (ask === Ask.Temper) {
      const can = h.v.deck.map((code, i) => (isUpgraded(code) ? -1 : i)).filter((i) => i >= 0);
      const offer = c.rng.permutation(can.length).slice(0, 3).map((i) => can[i]!);
      if (seat === 0) g.tempers0 = offer;
      else g.tempers1 = offer;
      if (offer.length === 0) g.decided |= bit;
    } else if (ask >= Ask.TemperPick && ask < Ask.TemperPick + 3) {
      const offer = tempersOf(g, seat);
      const at = offer[ask - Ask.TemperPick];
      if (at !== undefined) {
        const deck = [...h.v.deck];
        deck[at] = deck[at]! | UPGRADED;
        h.v.deck = deck;
        g.decided |= bit;
        g.sfx |= Sound.Buff;
        buff(c, h, "+");
      }
    }
    if (allDecided(c)) onward(c);
  }
}

/** The d-pad's direction: 0 none, 1 up, 2 right, 3 down, 4 left. */
function direction(x: number, y: number): number {
  if (y >= 0.5) return 1;
  if (x >= 0.5) return 2;
  if (y <= -0.5) return 3;
  if (x <= -0.5) return 4;
  return 0;
}

/** The foe slot up next to `from` in a direction (+1, -1), wrapping. */
function nextFoe(c: Ctx, from: number, step: number): number {
  for (let k = 1; k <= FOE_SLOTS; k++) {
    const i = (((from + step * k) % FOE_SLOTS) + FOE_SLOTS) % FOE_SLOTS;
    if (alive(c.foes[i]!)) return i;
  }
  return from;
}

/** Every rule, from the seats' input. */
export const Run = defineSystem(
  "Run",
  { reads: [og.PlayerInput, og.Lockstep, Tuning], writes: [Game, Hero, Foe, og.Transform, og.Sprite, og.Rng], after: ["Ui"] },
  (_q, time, world) => {
    const c = load(world);
    if (c === null) return;
    const g = c.g;
    // The pause bit first: two seats pressing it on one tick toggle it once.
    let pause = false;
    const inputs = c.heroes.map((h) => {
      const r = world.get(h.e, og.PlayerInput);
      const i = r.ok ? r.value : { x: 0, y: 0, aimX: 0, aimY: 0, buttons: 0 };
      const before = h.v.before;
      h.v.before = i.buttons;
      pause ||= pressedNow(i.buttons, before, "pause");
      return { i, before };
    });
    if (pause && g.phase !== Phase.Title && g.phase !== Phase.Over) g.paused = !g.paused;

    c.heroes.forEach((h, s) => {
      const { i, before } = inputs[s]!;
      const ask = Math.round(i.aimX);
      const play = pressedNow(i.buttons, before, "play");
      const dir = direction(i.x, i.y);
      const turned = dir !== 0 && dir !== h.v.lastDir;
      h.v.lastDir = dir;
      if (play && ask === Ask.Title && g.paused) {
        // Abandon the run, from the pause menu.
        g.paused = false;
        g.phase = Phase.Title;
        for (const f of c.foes) f.v.active = false;
        return;
      }
      if (g.phase !== Phase.Battle) {
        if (play && ask >= Ask.Reward) choose(c, h, ask);
        if (turned) g.sfx |= Sound.Move;
        return;
      }
      if (g.paused || g.side !== Side.Heroes || !up(h) || h.v.ended || g.cleared) return;
      if (turned && (dir === 2 || dir === 4) && h.v.hand.length > 0) {
        h.v.focusCard = (h.v.focusCard + (dir === 2 ? 1 : h.v.hand.length - 1)) % h.v.hand.length;
        g.sfx |= Sound.Move;
      }
      if (turned && (dir === 1 || dir === 3)) {
        h.v.focusFoe = nextFoe(c, h.v.focusFoe, dir === 1 ? 1 : -1);
        g.sfx |= Sound.Move;
      }
      if (play) {
        if (ask >= Ask.Card && ask < Ask.Reward) playCard(c, h, ask - Ask.Card, Math.round(i.aimY) - 1);
        else if (ask === Ask.Focus && h.v.hand.length > 0) playCard(c, h, h.v.hand[Math.min(h.v.focusCard, h.v.hand.length - 1)]!, -1);
      }
      if (pressedNow(i.buttons, before, "end")) h.v.ended = true;
    });

    if (g.phase === Phase.Battle && !g.paused) {
      const foesUp = c.foes.some(alive);
      if (!foesUp && !g.cleared) {
        g.cleared = true;
        g.timer = END_DELAY;
      }
      if (time.hitStop === 0) for (const f of c.foes) if (f.v.dying > 0 && f.v.dying < 60) f.v.dying += 1;
      if (g.cleared) {
        if (time.hitStop === 0 && g.timer > 0) g.timer -= 1;
        if (g.timer === 0) fightWon(c);
      } else if (partyDown(c)) {
        g.overTicks += 1;
        if (g.overTicks === END_DELAY) {
          g.phase = Phase.Over;
          g.won = false;
          g.overTicks = 0;
          g.sfx |= Sound.Lost;
        }
      } else if (g.side === Side.Heroes) {
        if (c.heroes.every((h) => !up(h) || h.v.ended)) foesTurn(c);
      } else if (time.hitStop === 0) {
        if (g.timer > 0) g.timer -= 1;
        if (g.timer === 0) {
          let next = -1;
          for (let k = g.acting + 1; k < FOE_SLOTS; k++) if (alive(c.foes[k]!)) {
            next = k;
            break;
          }
          if (next < 0) heroTurn(c);
          else {
            g.acting = next;
            foeAct(c, c.foes[next]!);
            g.timer = c.t.foeBeat;
          }
        }
      }
    }
    if (g.phase === Phase.Over) g.overTicks += 1;
    save(c);
  },
);

/** Every card's id, for the commands. */
export const CARD_IDS = CARDS.map((d) => d.id);
export { LOCAL };
