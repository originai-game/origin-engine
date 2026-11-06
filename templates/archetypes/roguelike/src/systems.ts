// Lamp Dive's systems, in tick order after the seats (src/seats.ts) and the
// screens (src/ui.ts): Hero (carving a floor, each delver's turn: a step, a
// strike, a wait or a draught), Foes (placing them, the strikes that land
// on them, their deaths and whether the lamp shows them), Loot (placing
// finds, picking them up) and Fog (what the lamps light). The foes' own
// turn is data: assets/ai/*.ogt run by og.ai, whose commands (src/foes.ts)
// move and strike.
//
// A turn: each delver up acts on the tick its seat's input (og.PlayerInput)
// asks; once every delver up has acted, the world's turn: Game.foesTurn
// goes up; on the next tick og.ai thinks with it up, so every foe queues
// one command, and the Foes system takes it down; the commands run before
// the tick after. A delver acts again once its step's tween is done. Alone
// there is one delver, so its every action is a turn.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, RngStream, Value, View, World } from "@origingame/engine";
import { clipOf, TILE } from "./art.ts";
import { floorStream, H, planFloor, W } from "./dive.ts";
import type { Plan } from "./dive.ts";
import { cellOf, Foe, FoeKind, FOE_SLOTS, FOES, Foes, Game, Hero, hpMaxOf, hpOf, Loot, LOOT, LootKind, Phase, setCell, setHp, Sfx, Sound, Tuning } from "./components.ts";
import { prefs } from "./device.ts";
import { LOCAL, ONLINE, pressedNow, SEATS } from "./seats.ts";
import { say, startDive } from "./ui.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;

const WHITE = { r: 1, g: 1, b: 1 };
const GOLD = { r: 1, g: 0.82, b: 0.35 };
const RED = { r: 1, g: 0.3, b: 0.28 };
const GREEN = { r: 0.45, g: 1, b: 0.55 };
const EMBER = { r: 1, g: 0.6, b: 0.3 };
/** Where parked rows wait. */
const PARK = { x: -60, y: -60, z: 0 } as const;
/** Ticks a card waits after the lamp goes out or the relic is taken. */
export const OVER_DELAY = 50;

/** The camera feel's scale: 0.3 under this device's reduced motion (og-feel), 1 otherwise. */
export function feel(): number {
  return prefs.reducedMotion ? 0.3 : 1;
}

export function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

export function must2<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Tuning, read on its first use in this call (a resource read encodes every field). */
export function lazyTuning(world: World): TuneV {
  let v: TuneV | null = null;
  return new Proxy({} as TuneV, { get: (_, k) => ((v ??= must2(world.resource(Tuning))) as unknown as Record<string | symbol, unknown>)[k] });
}

/** A cell's centre in world units: the map's top-left corner is at (0, H). */
export function cellCenter(x: number, y: number): { x: number; y: number } {
  return { x: x + 0.5, y: H - y - 0.5 };
}

export function mapOf(g: GameV): Entity {
  return g.map as unknown as Entity;
}

export function isWall(world: World, g: GameV, x: number, y: number): boolean {
  if (x < 0 || y < 0 || x >= W || y >= H) return true;
  return og.tilemap.getTile(world, mapOf(g), "walls", x, y) >= 0;
}

/** Whether the lamp lights a cell now (its fog is clear). */
export function lit(world: World, g: GameV, x: number, y: number): boolean {
  return og.tilemap.getTile(world, mapOf(g), "fog", x, y) < 0;
}

export function float(world: World, text: string, x: number, y: number, color: { r: number; g: number; b: number }, crit = false, size = 0.5): void {
  must(og.render.floatingText(world, { text, position: { x, y, z: 0 }, size, color, crit, rise: 0.9, lifetime: 0.8 }));
}

/** A room runs this world in lockstep (every peer's the same): what is this device's own (its settings, its record) stays out of it. */
export function inRoom(world: World): boolean {
  if (!ONLINE) return false;
  const lock = world.resource(og.Lockstep);
  return lock.ok && lock.value.running;
}

/** Every delver is down: the run is lost. */
export function partyDown(g: GameV): boolean {
  for (let s = 0; s < SEATS; s++) if (hpOf(g, s) > 0) return false;
  return true;
}

/** The seat of the delver on its feet standing on a cell, or -1. */
export function delverAt(g: GameV, x: number, y: number): number {
  for (let s = 0; s < SEATS; s++) {
    const c = cellOf(g, s);
    if (hpOf(g, s) > 0 && c.x === x && c.y === y) return s;
  }
  return -1;
}

/** The seat of the delver on its feet nearest a cell (by steps; seat 0 on a tie), or -1. */
export function nearestDelver(g: GameV, x: number, y: number): number {
  let best = -1;
  let far = 1e9;
  for (let s = 0; s < SEATS; s++) {
    const c = cellOf(g, s);
    const d = Math.abs(c.x - x) + Math.abs(c.y - y);
    if (hpOf(g, s) > 0 && d < far) {
      best = s;
      far = d;
    }
  }
  return best;
}

/** The plan of the floor the dive is on. */
export function planOf(rng: { fork(label: string): RngStream }, g: GameV): Plan {
  return planFloor(floorStream(rng, g.seed, g.floor), g.floor);
}

/** Carves the floor: every cell of the walls, ground, decor and fog layers, and a fresh og.nav grid. */
function carve(world: World, g: GameV, plan: Plan): void {
  const map = mapOf(g);
  const costs = new Array<number>(W * H).fill(0);
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const wall = plan.walls[y * W + x] === 1;
      must(og.tilemap.setTile(world, { entity: map, layer: "walls", x, y, tile: wall ? 1 : 0 }));
      must(og.tilemap.setTile(world, { entity: map, layer: "ground", x, y, tile: wall ? -1 : TILE.floor + ((x * 7 + y * 3) % 5 === 0 ? 1 : (x + y * 5) % 11 === 0 ? 2 : 0) }));
      must(og.tilemap.setTile(world, { entity: map, layer: "decor", x, y, tile: -1 }));
      must(og.tilemap.setTile(world, { entity: map, layer: "fog", x, y, tile: TILE.unseen }));
      costs[(H - 1 - y) * W + x] = wall ? 0 : 1;
    }
  }
  if (plan.stairs) must(og.tilemap.setTile(world, { entity: map, layer: "decor", x: plan.stairs.x, y: plan.stairs.y, tile: TILE.stairs }));
  if (plan.relic) must(og.tilemap.setTile(world, { entity: map, layer: "decor", x: plan.relic.x, y: plan.relic.y, tile: TILE.pedestal }));
  g.grid = must2(og.nav.grid(world, { width: W, height: H, cellSize: 1, origin: { x: 0, y: 0 }, costs })) as unknown as number;
  g.stairsX = plan.stairs ? plan.stairs.x : -1;
  g.stairsY = plan.stairs ? plan.stairs.y : -1;
}

/** The second delver's start: the first open cell beside the first's (right, left, below, above) that no foe stands on. */
function beside(plan: Plan): { x: number; y: number } {
  const { x, y } = plan.start;
  for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
    const cx = x + dx;
    const cy = y + dy;
    if (cx < 0 || cy < 0 || cx >= W || cy >= H || plan.walls[cy * W + cx] === 1) continue;
    if (plan.foes.some((f) => f.x === cx && f.y === cy)) continue;
    return { x: cx, y: cy };
  }
  return { x, y };
}

/** A move (og.PlayerInput's x, y) as a direction: 0 none, 1 up, 2 right, 3 down, 4 left. */
function direction(x: number, y: number): number {
  if (y >= 0.5) return 1;
  if (x >= 0.5) return 2;
  if (y <= -0.5) return 3;
  if (x <= -0.5) return 4;
  return 0;
}

const STEP = [null, { x: 0, y: -1 }, { x: 1, y: 0 }, { x: 0, y: 1 }, { x: -1, y: 0 }] as const;

/**
 * The delvers' turns, each from its seat's og.PlayerInput. A direction (an
 * arrow, WASD, a d-pad, a tap on the next cell) steps there, strikes the
 * foe standing there, or bumps the stone (or the other delver); Wait (Space,
 * the pad's A) waits; Drink (Q, 1, the pad's X) drinks. A held direction
 * repeats after 14 ticks, then every 7. Every action but a bump or an empty
 * flask is the delver's turn; the world's turn comes once every delver up
 * has had its own. Stepping on the stairs takes the party down (a downed
 * delver rises on the next floor). The seats' pause, dive and quit bits
 * (the menus press them, src/ui.ts) pause, start a dive and leave one.
 */
export const HeroTurn = defineSystem(
  "Hero",
  { reads: [og.PlayerInput, og.SeatDropped, og.SeatRejoined, og.Lockstep, Foes, Tuning], writes: [Game, Hero, og.Transform, og.Sprite, og.Perceivable, og.Rng], after: ["Ui"] },
  (q, time, world) => {
    const game = world.resource(Game);
    const rngR = world.resource(og.Rng);
    if (!game.ok || !rngR.ok) return;
    const g = game.value;
    const rng = rngR.value;
    const t = lazyTuning(world);
    if (ONLINE) {
      // A seat a room dropped holds nobody's turn up until it is back.
      for (const e of world.events(og.SeatDropped)) g.away |= 1 << e.seat;
      for (const e of world.events(og.SeatRejoined)) g.away &= ~(1 << e.seat);
    }
    // The menus' bits first: two seats pressing pause on one tick toggle it once.
    let pause = false;
    let dive = false;
    let quit = false;
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const hero = a.column(Hero);
      for (let i = 0; i < a.count; i++) {
        const b = input.buttons[i]!;
        const before = hero.before[i]!;
        pause ||= pressedNow(b, before, "pause");
        dive ||= pressedNow(b, before, "dive");
        quit ||= pressedNow(b, before, "quit");
      }
    }
    if (pause && g.phase === Phase.Play && (g.paused || (!partyDown(g) && !g.won))) g.paused = !g.paused;
    if (quit && (g.phase === Phase.Over || (g.phase === Phase.Play && g.paused))) {
      g.paused = false;
      g.phase = Phase.Title;
    }
    if (dive && g.phase !== Phase.Play) startDive(world, g);
    // Online, a dive started before the room ran has nobody at seat 1 (its delver waits, dim): the room's start brings it in.
    if (SEATS > 1 && g.hpMax2 === 0 && g.phase === Phase.Play && inRoom(world)) {
      g.hpMax2 = t.heroHp;
      g.hp2 = t.heroHp;
    }

    // The map is adopted on its first tick, so the first carve waits a tick.
    const plan = g.build && world.tick > 0 ? planOf(rng, g) : null;
    if (plan !== null) {
      const down = partyDown(g);
      carve(world, g, plan);
      setCell(g, 0, plan.start.x, plan.start.y);
      if (SEATS > 1) {
        const b = beside(plan);
        setCell(g, 1, b.x, b.y);
      }
      // A delver downed on the floor above rises on this one with half its health.
      for (let s = 0; s < SEATS && !down; s++) if (hpOf(g, s) <= 0) setHp(g, s, Math.ceil(hpMaxOf(g, s) / 2));
      g.build = false;
      g.place = true;
      g.fogDirty = true;
      g.deepest = Math.max(g.deepest, g.floor);
    }

    let descended = false;
    let sx = 0;
    let sy = 0;
    let n = 0;
    for (const a of q.archetypes()) {
      const hero = a.column(Hero);
      const input = a.column(og.PlayerInput);
      const p = a.column(og.Transform).position;
      const sprite = a.column(og.Sprite);
      const eyes = a.column(og.Perceivable);
      for (let i = 0; i < a.count; i++) {
        const me = a.entity(i);
        const seat = hero.seat[i]!;
        const buttons = input.buttons[i]!;
        const before = hero.before[i]!;
        if (before !== buttons) hero.before[i] = buttons;
        if (plan !== null) {
          const c = cellOf(g, seat);
          const at = cellCenter(c.x, c.y);
          p[i * 3] = at.x;
          p[i * 3 + 1] = at.y;
          hero.lock[i] = 0;
          hero.queued[i] = 0;
          hero.acted[i] = 0;
        }
        if (SEATS > 1) {
          // A downed delver lies dim, and the foes look past it.
          const upNow = hpOf(g, seat) > 0;
          const opacity = upNow ? 1 : 0.35;
          if (Math.abs(sprite.opacity[i]! - opacity) > 0.01) sprite.opacity[i] = opacity;
          const team = upNow ? 1 : 2;
          if (eyes.team[i] !== team) eyes.team[i] = team;
        }
        sx += p[i * 3]!;
        sy += p[i * 3 + 1]!;
        n += 1;
        // The lamp's warm light around the delver (under the fog), behind the title too.
        must(og.render.sprite(world, { texture: g.glow as unknown as never, position: { x: p[i * 3]!, y: p[i * 3 + 1]!, z: 0 }, width: g.lamp * 2.4, height: g.lamp * 2.4, color: { r: 1, g: 0.72, b: 0.35 }, opacity: 0.28, blend: "additive", layer: 1 }));
        const dir = direction(input.x[i]!, input.y[i]!);
        const last = hero.lastDir[i]!;
        // On a menu (the title, pause, settings, the summary) the arrows and the d-pad move the focus: a tick.
        if (g.phase !== Phase.Play || g.paused) {
          if (dir !== 0 && dir !== last) g.sfx |= Sound.Move;
          if (dir !== last) hero.lastDir[i] = dir;
          continue;
        }
        if (hpOf(g, seat) <= 0 || g.won) continue;
        if (hero.lock[i]! > 0 && time.hitStop === 0) hero.lock[i] = hero.lock[i]! - 1;
        // Read the input every tick (so a press during the foes' turn is held, not lost), act when the turn is ours.
        if (dir !== 0 && dir === last) hero.repeat[i] = hero.repeat[i]! + 1;
        else if (hero.repeat[i] !== 0) hero.repeat[i] = 0;
        if (dir !== last) hero.lastDir[i] = dir;
        const r = hero.repeat[i]!;
        let want = dir !== 0 && (r === 0 || (r >= 14 && (r - 14) % 7 === 0)) ? dir : 0;
        const from = cellOf(g, seat);
        if (pressedNow(buttons, before, "step")) {
          // A tap on the next cell (up, down, left or right) acts toward it.
          const dx = Math.floor(input.aimX[i]!) - from.x;
          const dy = H - 1 - Math.floor(input.aimY[i]!) - from.y;
          if (Math.abs(dx) + Math.abs(dy) === 1) want = dx === 1 ? 2 : dx === -1 ? 4 : dy === 1 ? 3 : 1;
        }
        const wait = pressedNow(buttons, before, "wait");
        const drink = pressedNow(buttons, before, "drink");
        // An action asked for while the last one plays out waits for its turn (one, the latest).
        if (want !== 0 || wait || drink) hero.queued[i] = drink ? 6 : wait ? 5 : want;
        const action = hero.queued[i]!;
        if (g.foesTurn || hero.lock[i]! > 0 || action === 0 || hero.acted[i]) continue;
        // One strike lands a tick: the other delver's waits for the next.
        if (action <= 4 && g.strikeSlot >= 0 && foeAt(world, from.x + STEP[action]!.x, from.y + STEP[action]!.y) >= 0) continue;
        hero.queued[i] = 0;
        const acted = () => {
          hero.acted[i] = 1;
          hero.lock[i] = Math.max(1, Math.round(t.stepTime * 60));
        };
        const c = cellCenter(from.x, from.y);
        if (action === 6) {
          const hp = hpOf(g, seat);
          const max = hpMaxOf(g, seat);
          if (g.potions === 0 || hp >= max) {
            g.refusedTick = world.tick;
            g.refusedSeat = seat;
            float(world, say(world, g.potions === 0 ? "float.empty" : "float.whole"), c.x, c.y + 0.8, WHITE, false, 0.35);
            continue;
          }
          const healed = Math.min(t.potionHeal, max - hp);
          g.potions -= 1;
          setHp(g, seat, hp + healed);
          g.sfx |= Sound.Drink;
          float(world, say(world, "float.heal", { n: healed }), c.x, c.y + 0.7, GREEN);
          must(og.tween.flash(world, { entity: me, color: GREEN, duration: 0.2 }));
          must(og.tween.squash(world, { entity: me, impact: { x: 0.85, y: 1.2 }, duration: 0.2 }));
          acted();
          continue;
        }
        if (action === 5) {
          must(og.tween.squash(world, { entity: me, impact: { x: 1.08, y: 0.92 }, duration: 0.15 }));
          g.sfx |= Sound.Move;
          acted();
          continue;
        }
        const step = STEP[action]!;
        const tx = from.x + step.x;
        const ty = from.y + step.y;
        const foe = foeAt(world, tx, ty);
        if (foe >= 0) {
          // A strike: the lunge, the die's +0 or +1; the Foes system lands it this tick.
          g.strikeSlot = foe;
          g.strikeSeat = seat;
          g.strikeDamage = g.attack + (rng.chance(0.5) ? 1 : 0);
          must(og.tween.sequence(world, [
            { entity: me, field: "position", to: [c.x + step.x * 0.35, c.y - step.y * 0.35, 0], duration: 0.05, ease: "quadOut" },
            { entity: me, field: "position", to: [c.x, c.y, 0], duration: 0.09, ease: "quadIn" },
          ]));
          acted();
          continue;
        }
        if (isWall(world, g, tx, ty) || delverAt(g, tx, ty) >= 0) {
          hero.bumps[i] = hero.bumps[i]! + 1;
          g.sfx |= Sound.Bump;
          must(og.tween.squash(world, { entity: me, impact: step.x !== 0 ? { x: 0.85, y: 1.1 } : { x: 1.1, y: 0.85 }, duration: 0.12 }));
          continue;
        }
        setCell(g, seat, tx, ty);
        hero.steps[i] = hero.steps[i]! + 1;
        g.sfx |= Sound.Step;
        g.fogDirty = true;
        const to = cellCenter(tx, ty);
        must(og.tween.to(world, { entity: me, field: "position", to: [to.x, to.y, 0], duration: t.stepTime, ease: "quadOut" }));
        acted();
        if (tx === g.stairsX && ty === g.stairsY) {
          // Down the stairs, the whole party: the next floor is carved on the next tick.
          g.floor += 1;
          g.build = true;
          g.sfx |= Sound.Stairs;
          float(world, say(world, "float.floor", { n: g.floor }), to.x, to.y + 0.8, GOLD, true);
          must(og.camera.punch(world, 0.1 * feel()));
          descended = true;
        }
      }
    }
    // The world's turn, once every delver up has acted; the foes of a floor left behind take none.
    const away = g.away;
    let waiting = 0;
    let upCount = 0;
    for (const a of q.archetypes()) {
      const hero = a.column(Hero);
      for (let i = 0; i < a.count; i++) {
        const seat = hero.seat[i]!;
        if (hpOf(g, seat) <= 0 || (away & (1 << seat)) !== 0) continue;
        upCount += 1;
        if (!hero.acted[i]) waiting += 1;
      }
    }
    if (descended || (upCount > 0 && waiting === 0)) {
      g.turns += 1;
      g.foesTurn = !descended;
      if (!descended) g.foesAt = world.tick;
      for (const a of q.archetypes()) {
        const hero = a.column(Hero);
        for (let i = 0; i < a.count; i++) if (hero.acted[i]) hero.acted[i] = 0;
      }
    }
    // Two on one device: the camera frames the delvers' middle (src/main.ts's `view`).
    if (LOCAL && n > 0) {
      const view = world.find("view");
      if (view !== null) must(world.set(view, og.Transform, { position: { x: sx / n, y: sy / n, z: 0 } }));
    }
  },
);

/** The foe slot standing on a cell, from the Foes resource, or -1. */
function foeAt(world: World, x: number, y: number): number {
  const foes = world.resource(Foes);
  if (!foes.ok) return -1;
  const f = foes.value as unknown as Record<string, number>;
  for (let i = 0; i < FOE_SLOTS; i++) if (f[`x${i}`] === x && f[`y${i}`] === y) return i;
  return -1;
}

/** A foe's numbers by FoeKind.*. */
export function stats(t: TuneV, kind: number): { hp: number; attack: number } {
  if (kind === FoeKind.Guardian) return { hp: t.guardianHp, attack: t.guardianAttack };
  if (kind === FoeKind.Skeleton) return { hp: t.skeletonHp, attack: t.skeletonAttack };
  return { hp: t.ratHp, attack: t.ratAttack };
}

/**
 * The foes: placed when a floor is carved; the lamp-bearer's strike landing
 * (a flash, a squash, the number, a hit stop and a little shake; the
 * guardian's fall far heavier); their deaths; whether the lamp shows them;
 * the telegraph of a charged strike; the foes' turn taken down once og.ai
 * has thought with it up.
 */
export const FoeTurn = defineSystem(
  "Foes",
  { reads: [Tuning, og.Rng], writes: [Game, Foes, Foe, og.Transform, og.Sprite, og.Flipbook, og.AiAgent, og.Perception], after: ["Hero"] },
  (q, time, world) => {
    const game = world.resource(Game);
    const foes = world.resource(Foes);
    const rngR = world.resource(og.Rng);
    if (!game.ok || !foes.ok || !rngR.ok) return;
    const g = game.value;
    const board = foes.value as unknown as Record<string, number>;
    const t = lazyTuning(world);
    if (g.foesTurn && world.tick > g.foesAt) g.foesTurn = false;
    const plan = g.place ? planOf(rngR.value, g) : null;
    for (const a of q.archetypes()) {
      const f = a.column(Foe);
      const p = a.column(og.Transform).position;
      const sprite = a.column(og.Sprite);
      const fb = a.column(og.Flipbook);
      const ai = a.column(og.AiAgent);
      const eyes = a.column(og.Perception);
      for (let i = 0; i < a.count; i++) {
        const me = a.entity(i);
        const slot = f.slot[i]!;
        if (plan !== null) {
          const spot = plan.foes[slot];
          if (spot) {
            const s = stats(t, spot.kind);
            f.active[i] = 1;
            f.kind[i] = spot.kind;
            f.hp[i] = s.hp;
            f.hpMax[i] = s.hp;
            f.attack[i] = s.attack;
            f.cx[i] = spot.x;
            f.cy[i] = spot.y;
            f.turns[i] = 0;
            f.charging[i] = 0;
            f.dying[i] = 0;
            const c = cellCenter(spot.x, spot.y);
            p[i * 3] = c.x;
            p[i * 3 + 1] = c.y;
            fb.clip[i] = clipOf(FOES[spot.kind]!);
            // A handle column is two u32 per row: the handle's low and high halves.
            const mind = spot.kind === FoeKind.Guardian ? g.guardianMind : g.foeMind;
            ai.behaviour[i * 2] = mind % 0x1_0000_0000;
            ai.behaviour[i * 2 + 1] = Math.floor(mind / 0x1_0000_0000);
            ai.active[i] = 1;
            eyes.sight[i] = t.sight;
            sprite.opacity[i] = 1;
          } else {
            f.active[i] = 0;
            f.dying[i] = 0;
            ai.active[i] = 0;
            p[i * 3] = PARK.x;
            p[i * 3 + 1] = PARK.y;
          }
        }
        if (g.strikeSlot === slot && f.active[i]) strike(world, g, f, i, me, p);
        if (f.dying[i]! > 0) {
          if (time.hitStop === 0) f.dying[i] = f.dying[i]! - 1;
          if (f.dying[i] === 0) {
            p[i * 3] = PARK.x;
            p[i * 3 + 1] = PARK.y;
            sprite.visible[i] = 0;
          }
        } else {
          sprite.visible[i] = f.active[i] && g.phase !== Phase.Title && lit(world, g, f.cx[i]!, f.cy[i]!) ? 1 : 0;
        }
        // A charged strike glows red the turn before it lands.
        const red = f.charging[i] ? 0.45 : 1;
        sprite.color[i * 3 + 1] = red;
        sprite.color[i * 3 + 2] = red;
        const x = f.active[i] ? f.cx[i]! : -1;
        const y = f.active[i] ? f.cy[i]! : -1;
        if (board[`x${slot}`] !== x) board[`x${slot}`] = x;
        if (board[`y${slot}`] !== y) board[`y${slot}`] = y;
      }
    }
    if (g.strikeSlot >= 0) g.strikeSlot = -1;
  },
);

function strike(world: World, g: GameV, f: View<typeof Foe>, i: number, me: Entity, p: Float32Array): void {
  const dmg = g.strikeDamage;
  const hp = Math.max(0, f.hp[i]! - dmg);
  f.hp[i] = hp;
  g.hits += 1;
  g.sfx |= Sound.Hit;
  const x = p[i * 3]!;
  const y = p[i * 3 + 1]!;
  const kind = f.kind[i]!;
  const crit = dmg > g.attack;
  float(world, say(world, "float.hit", { n: dmg }), x, y + 0.6, crit ? GOLD : WHITE, crit);
  must(og.tween.flash(world, { entity: me, duration: 0.12 }));
  must(og.tween.squash(world, { entity: me, impact: { x: 1.3, y: 0.7 }, duration: 0.2 }));
  const t = lazyTuning(world);
  must(og.time.hitStop(world, t.hitStop + (crit ? 2 : 0)));
  const k = feel();
  must(og.camera.shake(world, (kind === FoeKind.Guardian ? 0.3 : 0.15) * k));
  const by = cellCenter(cellOf(g, g.strikeSeat).x, cellOf(g, g.strikeSeat).y);
  must(og.camera.kick(world, { x: x - by.x, y: y - by.y, z: 0 }, 0.1 * k));
  if (hp === 0) {
    f.active[i] = 0;
    f.dying[i] = 24;
    g.kills += 1;
    g.sfx |= kind === FoeKind.Guardian ? Sound.Roar : Sound.Kill;
    if (kind === FoeKind.Guardian) {
      must(og.time.hitStop(world, 12));
      must(og.camera.shake(world, 0.6 * feel()));
    }
    must(og.tween.to(world, { entity: me, field: "scale", to: [1.6, 0, 1], duration: 0.3, delay: 0.05, ease: "backIn" }));
    must(og.tween.to(world, { entity: me, component: og.Sprite, field: "opacity", to: [0], duration: 0.3, delay: 0.05, ease: "quadIn" }));
  }
}

/** Finds: placed when a floor is carved, taken when a delver on its feet steps on them, shown where a lamp lights. */
export const LootTurn = defineSystem(
  "Loot",
  { reads: [og.Rng], writes: [Game, Loot, og.Transform, og.Sprite, og.Flipbook], after: ["Foes"] },
  (q, time, world) => {
    const game = world.resource(Game);
    const rngR = world.resource(og.Rng);
    if (!game.ok || !rngR.ok) return;
    const g = game.value;
    const plan = g.place ? planOf(rngR.value, g) : null;
    for (const a of q.archetypes()) {
      const l = a.column(Loot);
      const p = a.column(og.Transform).position;
      const sprite = a.column(og.Sprite);
      const fb = a.column(og.Flipbook);
      for (let i = 0; i < a.count; i++) {
        const me = a.entity(i);
        if (plan !== null) {
          const spot = plan.loot[l.slot[i]!];
          l.taken[i] = 0;
          if (spot) {
            l.active[i] = 1;
            l.kind[i] = spot.kind;
            l.amount[i] = spot.amount;
            l.cx[i] = spot.x;
            l.cy[i] = spot.y;
            const c = cellCenter(spot.x, spot.y);
            p[i * 3] = c.x;
            p[i * 3 + 1] = c.y;
            fb.clip[i] = clipOf(LOOT[spot.kind]!);
            sprite.opacity[i] = 1;
          } else {
            l.active[i] = 0;
            p[i * 3] = PARK.x;
            p[i * 3 + 1] = PARK.y;
          }
        }
        if (l.taken[i]! > 0) {
          if (time.hitStop === 0) l.taken[i] = l.taken[i]! + 1;
          if (l.taken[i]! > 30) {
            l.taken[i] = 0;
            sprite.visible[i] = 0;
            p[i * 3] = PARK.x;
            p[i * 3 + 1] = PARK.y;
          }
          continue;
        }
        if (!l.active[i]) {
          sprite.visible[i] = 0;
          continue;
        }
        sprite.visible[i] = g.phase !== Phase.Title && lit(world, g, l.cx[i]!, l.cy[i]!) ? 1 : 0;
        if (g.phase !== Phase.Play || delverAt(g, l.cx[i]!, l.cy[i]!) < 0) continue;
        take(world, g, l.kind[i]!, l.amount[i]!, p[i * 3]!, p[i * 3 + 1]!);
        l.active[i] = 0;
        l.taken[i] = 1;
        must(og.tween.to(world, { entity: me, field: "position.y", to: [p[i * 3 + 1]! + 0.8], duration: 0.35, ease: "quadOut" }));
        must(og.tween.to(world, { entity: me, component: og.Sprite, field: "opacity", to: [0], duration: 0.3, delay: 0.1, ease: "quadIn" }));
        must(og.tween.squash(world, { entity: me, impact: { x: 0.6, y: 1.5 }, duration: 0.3 }));
      }
    }
    if (plan !== null) g.place = false;
  },
);

function take(world: World, g: GameV, kind: number, amount: number, x: number, y: number): void {
  g.finds += 1;
  const say1 = (key: string, args?: Record<string, number>) => float(world, say(world, key, args), x, y + 0.8, GOLD, kind === LootKind.Relic, 0.34);
  switch (kind) {
    case LootKind.Potion:
      g.potions += 1;
      g.sfx |= Sound.Pickup;
      say1("item.potion");
      break;
    case LootKind.Blade:
      g.attack += 1;
      g.sfx |= Sound.Item;
      say1("item.blade");
      break;
    case LootKind.Ward:
      g.armour += 1;
      g.sfx |= Sound.Item;
      say1("item.ward");
      break;
    case LootKind.Oil:
      g.lamp += 1;
      g.fogDirty = true;
      g.sfx |= Sound.Item;
      say1("item.oil");
      break;
    case LootKind.Coin:
      g.gold += amount;
      g.sfx |= Sound.Pickup;
      say1("item.coin", { n: amount });
      break;
    case LootKind.Relic:
      g.won = true;
      g.overTicks = 0;
      g.sfx |= Sound.Won;
      say1("item.relic");
      must(og.camera.punch(world, 0.15 * feel()));
      break;
  }
}

/**
 * What the lamps light: when a delver moves, every cell within the lamp's
 * reach that a straight line from a delver reaches without crossing stone
 * is cleared of fog; cells lit before and not now are remembered (dimmed);
 * cells never lit stay dark.
 */
export const Fog = defineSystem("Fog", { writes: [Game], after: ["Loot"] }, (_q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (!g.fogDirty || g.build) return;
  g.fogDirty = false;
  const map = mapOf(g);
  const r = g.lamp;
  const reach = Math.ceil(r);
  const seen = new Set<number>();
  for (let s = 0; s < SEATS; s++) {
    const from = cellOf(g, s);
    for (let dy = -reach; dy <= reach; dy++) {
      for (let dx = -reach; dx <= reach; dx++) {
        if (dx * dx + dy * dy > r * r) continue;
        const x = from.x + dx;
        const y = from.y + dy;
        if (x < 0 || y < 0 || x >= W || y >= H || seen.has(y * W + x)) continue;
        if (lineClear(world, g, from.x, from.y, x, y)) seen.add(y * W + x);
      }
    }
  }
  let known = 0;
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const now = og.tilemap.getTile(world, map, "fog", x, y);
      const want = seen.has(y * W + x) ? -1 : now === TILE.unseen ? TILE.unseen : TILE.remembered;
      if (want !== now) must(og.tilemap.setTile(world, { entity: map, layer: "fog", x, y, tile: want }));
      if (want !== TILE.unseen) known += 1;
    }
  }
  if (g.lit !== seen.size) g.lit = seen.size;
  if (g.seen !== known) g.seen = known;
});

/** Whether a straight line from one cell to another crosses no stone before it (the end cell itself may be stone: walls are seen). */
function lineClear(world: World, g: GameV, x0: number, y0: number, x1: number, y1: number): boolean {
  let x = x0;
  let y = y0;
  const dx = Math.abs(x1 - x0);
  const dy = -Math.abs(y1 - y0);
  const sx = x0 < x1 ? 1 : -1;
  const sy = y0 < y1 ? 1 : -1;
  let err = dx + dy;
  for (;;) {
    if (x === x1 && y === y1) return true;
    if ((x !== x0 || y !== y0) && isWall(world, g, x, y)) return false;
    const e2 = 2 * err;
    if (e2 >= dy) {
      err += dy;
      x += sx;
    }
    if (e2 <= dx) {
      err += dx;
      y += sy;
    }
  }
}

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Fog"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.sfx === 0) return;
  const rng = world.resource(og.Rng);
  let left = g.sfx;
  for (const a of q.archetypes()) {
    const sfx = a.column(Sfx);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = sfx.bit[i]!;
      if ((g.sfx & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        sfx.plays[i] = sfx.plays[i]! + 1;
        if ((b === Sound.Step || b === Sound.Hit || b === Sound.Pickup) && rng.ok) src.pitch[i] = rng.value.range(0.92, 1.08);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});

export { EMBER, RED, GOLD, WHITE, LootKind };
