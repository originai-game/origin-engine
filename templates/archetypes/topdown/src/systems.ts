// Tomb Room's systems, in tick order (after src/seats.ts's Seats, which
// gives each seat's input to og.PlayerInput): Control (a hero per seat:
// walk, swing, dash, hurt; the party's room), Slimes (take hits, knockback and death; what a slime decides is
// assets/ai/slime.ogt, run by og.ai before these systems and moved by og.nav
// after them), Pickups (coins and the key), Door (the locked doorway and the
// rooms), Sfx (the voices) and Hud (screen text). `biteHero` is the
// behaviour's `bite` command.
//
// Control reads the hero's flipbook events: a walk clip's "step" counts a
// footstep, an attack clip's "hit" is the sword's strike frame, so the
// damage lands on the frame the blade is out. Every hit is felt five ways:
// the slime flashes white and squashes, it is knocked back, the world holds
// for a few ticks (og.time.hitStop), the camera shakes and kicks, and the
// damage floats up (the third hit is a crit).

import { og, defineSystem } from "@origingame/engine";
import type { Entity, Value, Vector3, View, World } from "@origingame/engine";
import { HeroClip, heroClip } from "./art.ts";
import { Facing, Game, Hero, Pickup, PickupKind, Report, Slime, Sfx, Sound, Tuning } from "./components.ts";
import { LOCAL, pressedNow, SEATS } from "./seats.ts";

/** The swing's length: the attack clips' two frames (0.07 s + 0.18 s). */
export const ATTACK_TICKS = 15;
/** Ticks after hp reaches 0 before the hero stands up at the spawn. */
const DOWN_TICKS = 90;
/** Damage per hit, in order; later hits repeat the last. */
const DAMAGE = [12, 12, 26] as const;
/** A struck slime's squash. */
const IMPACT = { x: 1.45, y: 0.6 } as const;
/** The chest: the sword and the pickups measure from this high above the feet. */
const CHEST = 0.45;

const WHITE = { r: 1, g: 1, b: 1 };
const GOLD = { r: 1, g: 0.72, b: 0.2 };
const RED = { r: 1, g: 0.25, b: 0.2 };
const MINT = { r: 0.55, g: 0.95, b: 0.7 };

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

function fontOf(g: { font: number }): never {
  return g.font as unknown as never;
}

/** A triangle wave of the tick, 0..1 over `period` ticks (tick arithmetic only, so every host agrees). */
function tri(tick: number, period: number): number {
  const p = (tick % period) / period;
  return p < 0.5 ? p * 2 : 2 - p * 2;
}

function float(world: World, font: never, text: string, at: Vector3, color: { r: number; g: number; b: number }, crit = false): void {
  must(og.render.floatingText(world, { text, position: at, font, size: 0.42, color, crit, rise: 1.1, lifetime: 0.75 }));
}

/** A seat's hero: `hero` for seat 0, `hero-<seat>` for the others. */
export function heroName(seat: number): string {
  return seat === 0 ? "hero" : `hero-${seat}`;
}

/** Seat 1's hero stands this far right of seat 0's at the spawn. */
export const SEAT_GAP = 1;

/** sqrt(x² + y²) the same on every JavaScript engine (Math.hypot is not: og check's lockstep lint). */
export const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);

/** Where a straggler, or a hero standing up while the party is in the east room, comes in: just inside the party's room by the doorway. */
function entry(g: GameValue, room: number, seat: number): { x: number; y: number } {
  return { x: g.doorX + (room === 2 ? 2.5 : -2.5), y: g.doorY - 0.5 - 0.6 * seat };
}

/**
 * The heroes, one per seat: each reads its seat's og.PlayerInput (the move,
 * src/creation.ts's attack and dash bits) on the tick it arrives; attack
 * swings, dash dashes; hurt knocks it back and flickers it. The party is in
 * one room: once a standing hero crosses into the other room, the others
 * are brought through the doorway beside it (the camera's zones then
 * scroll), and a hero standing up again does so in the party's room. The
 * camera's view entity follows the heroes' middle.
 */
export const Control = defineSystem(
  "Control",
  {
    reads: [og.PlayerInput, Tuning, og.FlipbookEvent],
    writes: [Game, Hero, og.Transform, og.CharacterController, og.Flipbook, og.Sprite, og.Afterimage, og.Perceivable],
  },
  (q, time, world) => {
    const game = world.resource(Game);
    const tune = world.resource(Tuning);
    if (!game.ok || !tune.ok) return;
    const g = game.value;
    const t = tune.value;
    // The party's room: the room a standing hero has crossed into (alone, the hero's room).
    let room = g.room;
    for (const a of q.archetypes()) {
      const hero = a.column(Hero);
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        const r = p[i * 3]! < g.roomSplit ? 1 : 2;
        if (SEATS === 1 || (hero.hp[i]! > 0 && r !== g.room)) room = r;
      }
    }
    g.room = room;
    const xs = [0, 0];
    const ys = [0, 0];
    const up = [0, 0];
    const strikeX: number[] = [];
    const strikeY: number[] = [];
    const strikerX: number[] = [];
    const strikerY: number[] = [];
    let sx = 0;
    let sy = 0;
    let n = 0;
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const hero = a.column(Hero);
      const p = a.column(og.Transform).position;
      const v = a.column(og.CharacterController).move;
      const fb = a.column(og.Flipbook);
      const sprite = a.column(og.Sprite);
      const ghost = a.column(og.Afterimage);
      const seen = a.column(og.Perceivable);
      for (let i = 0; i < a.count; i++) {
        const me = a.entity(i);
        const seat = input.player[i]!;
        if (SEATS > 1 && (p[i * 3]! < g.roomSplit ? 1 : 2) !== room) {
          // A straggler: brought through the doorway beside the others.
          const at = entry(g, room, seat);
          p[i * 3] = at.x;
          p[i * 3 + 1] = at.y;
          hero.knockX[i] = 0;
          hero.knockY[i] = 0;
        }
        const x = p[i * 3]!;
        const y = p[i * 3 + 1]!;
        xs[seat] = x;
        ys[seat] = y;
        up[seat] = hero.hp[i]! > 0 ? 1 : 0;
        sx += x;
        sy += y;
        n += 1;
        // The seat's presses, read every tick (a press during a hit stop is spent, as a key's was).
        const buttons = input.buttons[i]!;
        const attackPressed = pressedNow(buttons, hero.before[i]!, "attack");
        const dashPressed = pressedNow(buttons, hero.before[i]!, "dash");
        hero.before[i] = buttons;
        // The flipbook's frame events from the hero's clips.
        for (const e of world.events(og.FlipbookEvent)) {
          if (e.entity !== me) continue;
          if (e.name === "step") {
            g.steps += 1;
            g.sfx |= Sound.Step;
          }
          if (e.name === "hit") {
            const d = facingVector(hero.facing[i]!);
            g.swings += 1;
            strikeX.push(x + d.x * t.reach);
            strikeY.push(y + CHEST + d.y * t.reach);
            strikerX.push(x);
            strikerY.push(y);
          }
        }
        // A downed hero is hidden from the slimes (their own team's).
        seen.team[i] = hero.hp[i]! > 0 ? 1 : 2;
        // A hit stop holds the body (physics is frozen); the timers wait with it.
        if (time.hitStop > 0) continue;

        if (hero.iframes[i]! > 0) hero.iframes[i] = hero.iframes[i]! - 1;
        sprite.opacity[i] = hero.iframes[i]! > 0 && Math.floor(hero.iframes[i]! / 4) % 2 === 1 ? 0.35 : 1;
        let kx = hero.knockX[i]! * 0.8;
        let ky = hero.knockY[i]! * 0.8;
        if (Math.abs(kx) < 0.05) kx = 0;
        if (Math.abs(ky) < 0.05) ky = 0;
        hero.knockX[i] = kx;
        hero.knockY[i] = ky;

        // Down: the hero lies still, then stands up at the spawn (the party's room's way in) with full hearts.
        if (hero.hp[i] === 0) {
          hero.attack[i] = 0;
          hero.dash[i] = 0;
          ghost.active[i] = 0;
          v[i * 3] = 0;
          v[i * 3 + 1] = 0;
          hero.downTicks[i] = hero.downTicks[i]! + 1;
          if (hero.downTicks[i]! >= DOWN_TICKS) {
            hero.hp[i] = g.hpMax;
            hero.downTicks[i] = 0;
            hero.iframes[i] = t.iframes;
            const at = SEATS > 1 && room === 2 ? entry(g, room, seat) : { x: g.spawnX + seat * SEAT_GAP, y: g.spawnY };
            p[i * 3] = at.x;
            p[i * 3 + 1] = at.y;
          }
          continue;
        }

        // Which way the seat points: og.PlayerInput's move (WASD, the arrows,
        // a stick, a d-pad or the touch stick; src/seats.ts), a diagonal at
        // most 1 long.
        const len = hypot(input.x[i]!, input.y[i]!);
        const mx = len > 1 ? input.x[i]! / len : input.x[i]!;
        const my = len > 1 ? input.y[i]! / len : input.y[i]!;
        const moving = len > 0;
        hero.moving[i] = moving ? 1 : 0;

        if (hero.dashCooldown[i]! > 0) hero.dashCooldown[i] = hero.dashCooldown[i]! - 1;

        // Facing follows the stronger axis; a diagonal keeps the old facing when it is one of its two keys.
        if (moving && hero.attack[i] === 0) hero.facing[i] = faceOf(mx, my, hero.facing[i]!);
        const facing = hero.facing[i]!;

        if (attackPressed && hero.attack[i] === 0 && hero.dash[i] === 0) hero.attack[i] = ATTACK_TICKS;
        if (dashPressed && hero.attack[i] === 0 && hero.dash[i] === 0 && hero.dashCooldown[i] === 0) {
          const d = moving ? { x: mx, y: my } : facingVector(facing);
          hero.dash[i] = t.dashTicks;
          hero.dashCooldown[i] = t.dashCooldown;
          hero.dashX[i] = d.x;
          hero.dashY[i] = d.y;
          hero.dashes[i] = hero.dashes[i]! + 1;
        }

        let vx = 0;
        let vy = 0;
        if (hero.dash[i]! > 0) {
          hero.dash[i] = hero.dash[i]! - 1;
          vx = hero.dashX[i]! * t.dashSpeed;
          vy = hero.dashY[i]! * t.dashSpeed;
        } else if (hero.attack[i]! > 0) {
          // Rooted while swinging: the blade, not the feet, carries the hit.
          hero.attack[i] = hero.attack[i]! - 1;
        } else if (moving) {
          vx = mx * t.speed;
          vy = my * t.speed;
        }
        ghost.active[i] = hero.dash[i]! > 0 ? 1 : 0;
        v[i * 3] = vx + kx;
        v[i * 3 + 1] = vy + ky;
        v[i * 3 + 2] = 0;

        // The clip: a swing, a walk or a stand, in the facing's row. A clip change restarts it.
        const swinging = hero.attack[i]! > 0;
        const action = swinging ? HeroClip.Attack : moving || hero.dash[i]! > 0 ? HeroClip.Walk : HeroClip.Idle;
        const want = heroClip(action, facing);
        if (fb.clip[i] !== want) fb.clip[i] = want;
        sprite.flipX[i] = facing === Facing.Left ? 1 : 0;
      }
    }
    g.heroX = xs.slice(0, SEATS);
    g.heroY = ys.slice(0, SEATS);
    g.heroUp = up.slice(0, SEATS);
    g.strikeX = strikeX;
    g.strikeY = strikeY;
    g.strikerX = strikerX;
    g.strikerY = strikerY;
    // The camera follows the heroes' middle (og.Follow2D on the view; the rooms' og.CameraZone keep it in the party's room).
    const view = world.find("view");
    if (view !== null && n > 0) must(world.set(view, og.Transform, { position: { x: sx / n, y: sy / n, z: 0 } }));
  },
);

export function facingVector(f: number): { x: number; y: number } {
  return f === Facing.Up ? { x: 0, y: 1 } : f === Facing.Down ? { x: 0, y: -1 } : f === Facing.Left ? { x: -1, y: 0 } : { x: 1, y: 0 };
}

function faceOf(x: number, y: number, old: number): number {
  const keeps = (old === Facing.Left && x < 0) || (old === Facing.Right && x > 0) || (old === Facing.Up && y > 0) || (old === Facing.Down && y < 0);
  if (Math.abs(x) > 0.01 && Math.abs(y) > 0.01 && keeps) return old;
  if (Math.abs(x) >= Math.abs(y)) return x < 0 ? Facing.Left : Facing.Right;
  return y < 0 ? Facing.Down : Facing.Up;
}

/**
 * Slimes take the sword's hits, are knocked back and stunned, and die. The
 * walking is og.nav's (the agent's velocity) except while stunned, held by a
 * hit stop or dead, when `NavAgent.drive` is off and this system owns the
 * velocity; dead slimes stop thinking. Perception's sight follows the tunable.
 */
export const Slimes = defineSystem(
  "Slimes",
  { reads: [Tuning], writes: [Game, Slime, og.Transform, og.Velocity, og.Flipbook, og.Sprite, og.NavAgent, og.AiAgent, og.Perception], after: ["Control"] },
  (q, time, world) => {
    const game = world.resource(Game);
    const tune = world.resource(Tuning);
    if (!game.ok || !tune.ok) return;
    const g = game.value;
    const t = tune.value;
    const font = fontOf(g);
    for (const a of q.archetypes()) {
      const s = a.column(Slime);
      const p = a.column(og.Transform).position;
      const v = a.column(og.Velocity).linear;
      const fb = a.column(og.Flipbook);
      const sprite = a.column(og.Sprite);
      const nav = a.column(og.NavAgent);
      const ai = a.column(og.AiAgent);
      const eyes = a.column(og.Perception);
      for (let i = 0; i < a.count; i++) {
        const me = a.entity(i);
        const x = p[i * 3]!;
        const y = p[i * 3 + 1]!;
        // It sees the nearest standing hero (og.Perception picks it; a downed one is hidden), and none while every hero is down.
        eyes.sight[i] = g.heroUp.includes(1) ? t.slimeSight : 0;
        if (s.dead[i]) {
          nav.drive[i] = 0;
          ai.active[i] = 0;
          v[i * 3] = 0;
          v[i * 3 + 1] = 0;
          if (time.hitStop > 0) continue;
          s.deadTicks[i] = s.deadTicks[i]! + 1;
          if (s.deadTicks[i] === 30) sprite.visible[i] = 0;
          continue;
        }

        // Each sword strike of this tick: a circle of radius reach around its swing point.
        for (let k = 0; k < g.strikeX.length && !s.dead[i]; k++) {
          if (hypot(g.strikeX[k]! - x, g.strikeY[k]! - (y + 0.3)) >= t.reach + 0.35) continue;
          hitSlime(world, me, g, t, s, i, x, y, font, k);
          fb.clip[i] = s.hurt[i]!;
        }
        if (s.dead[i] || time.hitStop > 0) {
          nav.drive[i] = 0;
          v[i * 3] = 0;
          v[i * 3 + 1] = 0;
          continue;
        }
        if (s.stun[i]! > 0) {
          nav.drive[i] = 0;
          s.stun[i] = s.stun[i]! - 1;
          v[i * 3] = s.knockX[i]!;
          v[i * 3 + 1] = s.knockY[i]!;
          v[i * 3 + 2] = 0;
          s.knockX[i] = s.knockX[i]! * 0.82;
          s.knockY[i] = s.knockY[i]! * 0.82;
          if (s.stun[i] === 0) fb.clip[i] = s.bob[i]!;
          continue;
        }
        nav.drive[i] = 1;
      }
    }
  },
);

/**
 * `bite:<slime>`: the behaviour's attack (og.ai queues it with the slime's
 * handle when its target is in reach; it runs before the next tick). The
 * nearest standing hero in reach that is not still blinking (the lower seat
 * on a tie) is hurt and knocked away, and the slime bounces back off the
 * bite, so it does not sit on the hero.
 */
export function biteHero(world: World, args: readonly string[]): void {
  const me = Number(args[0]) as Entity;
  const game = world.resource(Game);
  const tune = world.resource(Tuning);
  const slime = world.get(me, Slime);
  const at = world.get(me, og.Transform);
  if (!game.ok || !tune.ok || !slime.ok || !at.ok) throw new Error(`bite: ${args[0]} is not a slime`);
  const g = game.value;
  const s = slime.value;
  if (s.dead || s.stun > 0) return;
  let best: { e: Entity; h: Value<typeof Hero>; seat: number; d: number; dx: number; dy: number } | null = null;
  for (let seat = 0; seat < g.heroX.length; seat++) {
    const e = world.find(heroName(seat));
    const h = e === null ? null : world.get(e, Hero);
    if (e === null || h === null || !h.ok || h.value.hp === 0 || h.value.iframes > 0) continue;
    const dx = g.heroX[seat]! - at.value.position.x;
    const dy = g.heroY[seat]! - at.value.position.y;
    const d = hypot(dx, dy);
    if (d <= 0.85 && (best === null || d < best.d)) best = { e, h: h.value, seat, d, dx, dy };
  }
  if (best === null) return;
  const ux = best.d > 0.01 ? best.dx / best.d : 0;
  const uy = best.d > 0.01 ? best.dy / best.d : 1;
  hurtHero(world, g, tune.value, best.e, best.h, best.seat, ux, uy, fontOf(g));
  must(world.set(me, Slime, { knockX: -ux * tune.value.knockback * 0.5, knockY: -uy * tune.value.knockback * 0.5, stun: 20, bites: s.bites + 1 }));
}

type GameValue = Value<typeof Game>;
type TuneValue = Value<typeof Tuning>;
type SlimeView = View<typeof Slime>;

/** Strike `k` of this tick (Game.strikeX) lands on slime row i. */
function hitSlime(world: World, me: Entity, g: GameValue, t: TuneValue, s: SlimeView, i: number, x: number, y: number, font: never, k: number): void {
  g.enemyHits += 1;
  const dmg = DAMAGE[Math.min(g.enemyHits, DAMAGE.length) - 1]!;
  const crit = g.enemyHits >= DAMAGE.length;
  const hp = Math.max(0, s.hp[i]! - dmg);
  s.hp[i] = hp;
  g.enemyHp = hp;
  // Knocked away from the hero that struck it (on top of it: the way the hero faces, its strike's direction).
  const hx = g.strikerX[k]!;
  const hy = g.strikerY[k]!;
  let kx = x - hx;
  let ky = y - hy;
  const kd = hypot(kx, ky);
  if (kd > 0.01) {
    kx /= kd;
    ky /= kd;
  } else {
    kx = g.strikeX[k]! - hx;
    ky = g.strikeY[k]! - CHEST - hy;
    const fd = hypot(kx, ky);
    kx /= fd;
    ky /= fd;
  }
  s.knockX[i] = kx * t.knockback;
  s.knockY[i] = ky * t.knockback;
  s.stun[i] = 16;
  must(og.tween.flash(world, { entity: me, duration: 0.14 }));
  must(og.tween.squash(world, { entity: me, impact: IMPACT, duration: 0.28 }));
  must(og.time.hitStop(world, crit ? t.hitStop + 3 : t.hitStop));
  must(og.camera.shake(world, Math.min(1, crit ? t.shake * 1.8 : t.shake)));
  must(og.camera.kick(world, { x: kx, y: ky, z: 0 }, crit ? 0.25 : 0.12));
  float(world, font, crit ? `${dmg}!` : `${dmg}`, { x, y: y + 1.1, z: 0 }, crit ? GOLD : WHITE, crit);
  g.sfx |= Sound.Hit;
  if (hp === 0) {
    s.dead[i] = 1;
    s.deadTicks[i] = 0;
    // The death: a last stretch upwards and a fade to nothing.
    must(og.tween.to(world, { entity: me, field: "scale", to: [1.7, 0, 1], duration: 0.4, delay: 0.1, ease: "backIn" }));
    must(og.tween.to(world, { entity: me, component: og.Sprite, field: "opacity", to: [0], duration: 0.4, delay: 0.1, ease: "quadIn" }));
  }
}

function hurtHero(world: World, g: GameValue, t: TuneValue, hero: Entity, h: Value<typeof Hero>, seat: number, dx: number, dy: number, font: never): void {
  const hp = Math.max(0, h.hp - 1);
  g.hurts += 1;
  must(world.set(hero, Hero, { hp, iframes: t.iframes, knockX: dx * t.knockback, knockY: dy * t.knockback, ...(hp === 0 ? { downTicks: 0 } : {}) }));
  must(og.tween.flash(world, { entity: hero, color: RED, duration: 0.2 }));
  must(og.tween.squash(world, { entity: hero, impact: { x: 1.3, y: 0.75 }, duration: 0.22 }));
  // A hit taken holds the world a light hit's length (og-feel §3), shorter
  // than the hero's own blows, with the flash, the shake and the sound.
  must(og.time.hitStop(world, Math.max(2, t.hitStop - 1)));
  must(og.camera.shake(world, 0.4));
  float(world, font, "-1", { x: g.heroX[seat]!, y: g.heroY[seat]! + 1.2, z: 0 }, RED);
  g.sfx |= Sound.Hurt;
}

/** Coins and the key hover until a standing hero's chest touches them (the lower seat first), then pop up and fade; the party shares them. */
export const Pickups = defineSystem(
  "Pickups",
  { writes: [Game, Pickup, og.Transform, og.Sprite], after: ["Slimes"] },
  (q, time, world) => {
    const game = world.resource(Game);
    if (!game.ok) return;
    const g = game.value;
    if (time.hitStop > 0) return;
    const font = fontOf(g);
    for (const a of q.archetypes()) {
      const pk = a.column(Pickup);
      const p = a.column(og.Transform).position;
      const sprite = a.column(og.Sprite);
      for (let i = 0; i < a.count; i++) {
        if (pk.taken[i]) {
          pk.takenTicks[i] = pk.takenTicks[i]! + 1;
          if (pk.takenTicks[i] === 24) sprite.visible[i] = 0;
          continue;
        }
        const me = a.entity(i);
        const x = p[i * 3]!;
        const base = pk.baseY[i]!;
        p[i * 3 + 1] = base + 0.08 * tri(world.tick + i * 13, 70);
        let taker = -1;
        for (let seat = 0; seat < g.heroX.length && taker < 0; seat++) {
          if (g.heroUp[seat] === 1 && hypot(g.heroX[seat]! - x, g.heroY[seat]! + CHEST - (base + 0.4)) <= 0.7) taker = seat;
        }
        if (taker < 0) continue;
        pk.taken[i] = 1;
        pk.takenTicks[i] = 0;
        const at = { x, y: base + 1.1, z: 0 };
        if (pk.kind[i] === PickupKind.Key) {
          g.hasKey = true;
          g.sfx |= Sound.Key;
          float(world, font, "key!", at, GOLD, true);
        } else {
          g.coins += 1;
          g.sfx |= Sound.Coin;
          float(world, font, "+1", at, GOLD);
        }
        must(og.tween.to(world, { entity: me, field: "position.y", to: [base + 0.9], duration: 0.3, ease: "quadOut" }));
        must(og.tween.to(world, { entity: me, component: og.Sprite, field: "opacity", to: [0], duration: 0.3, delay: 0.08, ease: "quadIn" }));
        must(og.tween.squash(world, { entity: me, impact: { x: 0.6, y: 1.6 }, duration: 0.3 }));
        const hero = world.find(heroName(taker));
        if (hero !== null) must(og.tween.squash(world, { entity: hero, impact: { x: 0.85, y: 1.2 }, duration: 0.2 }));
      }
    }
  },
);

/** The doorway: locked until the key; with it (whoever took it), the wall cells clear as a standing hero walks up (re-tiling their neighbours and dropping their colliders). */
export const Door = defineSystem("Door", { writes: [Game], after: ["Pickups"] }, (_q, time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (time.hitStop > 0) return;
  if (g.lockedFor > 0) g.lockedFor -= 1;
  if (g.doorOpen) return;
  const near = g.heroX.some((x, seat) => g.heroUp[seat] === 1 && Math.abs(x - g.doorX) < 1.6 && Math.abs(g.heroY[seat]! - g.doorY) < 1.3);
  if (!near) return;
  const font = fontOf(g);
  if (!g.hasKey) {
    if (g.lockedFor === 0) {
      float(world, font, "locked", { x: g.doorX, y: g.doorY + 1.4, z: 0 }, WHITE);
      g.lockedFor = 90;
    }
    return;
  }
  openDoor(world, g);
  float(world, font, "open!", { x: g.doorX, y: g.doorY + 1.4, z: 0 }, MINT, true);
  must(og.camera.shake(world, 0.3));
});

/** Clears the doorway's wall and door cells. Also the `give:door` command's work. */
export function openDoor(world: World, g: GameValue): void {
  const map = world.find("tomb");
  if (map === null) throw new Error("no entity named tomb");
  for (let y = g.doorCellY; y < g.doorCellY + g.doorCellsH; y++) {
    for (let x = g.doorCellX; x < g.doorCellX + g.doorCellsW; x++) {
      must(og.tilemap.setTile(world, { entity: map, layer: "walls", x, y, tile: 0 }));
      must(og.tilemap.setTile(world, { entity: map, layer: "door", x, y, tile: -1 }));
    }
  }
  g.doorOpen = true;
  g.hasKey = false;
  g.sfx |= Sound.Door;
}

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next. */
export const Voices = defineSystem("Voices", { reads: [Sfx], writes: [Game, og.Rng, og.AudioSource], after: ["Door"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.sfx === 0) return;
  const rng = world.resource(og.Rng);
  let left = g.sfx;
  for (const a of q.archetypes()) {
    const bit = a.column(Sfx).bit;
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = bit[i]!;
      if ((g.sfx & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        // Footsteps vary a little, so a walk does not machine-gun (og-audio §1).
        if (b === Sound.Step && rng.ok) src.pitch[i] = rng.value.range(0.92, 1.08);
        left &= ~b;
      }
    }
  }
  g.sfx = left;
});

/** The HUD: each hero's hearts, the coins, the key and the room, in screen space; the controls for the first seconds. Also copies the facts into each hero's Report. */
export const Hud = defineSystem("Hud", { reads: [Game, Hero, og.PlayerInput], writes: [Report], after: ["Voices"] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  const map = world.find("tomb");
  const hearts = [0, 0];
  for (const a of q.archetypes()) {
    const r = a.column(Report);
    const hero = a.column(Hero);
    const seat = a.column(og.PlayerInput).player;
    for (let i = 0; i < a.count; i++) {
      hearts[seat[i]!] = hero.hp[i]!;
      r.room[i] = g.room;
      r.hp[i] = hero.hp[i]!;
      r.coins[i] = g.coins;
      r.hasKey[i] = g.hasKey ? 1 : 0;
      r.doorOpen[i] = g.doorOpen ? 1 : 0;
      r.enemyHp[i] = g.enemyHp;
      r.hits[i] = g.enemyHits;
      r.swings[i] = g.swings;
      r.steps[i] = g.steps;
      r.hurts[i] = g.hurts;
      r.facing[i] = hero.facing[i]!;
      r.doorWall[i] = map === null ? -1 : og.tilemap.getTile(world, map, "walls", g.doorCellX, g.doorCellY);
    }
  }
  const font = fontOf(g);
  // Screen text is placed and sized in CSS pixels: the UI's viewport, not
  // og.render.capabilities' render target, which is device pixels (twice as
  // wide at DPR 2: the room and the prompt fell off the screen, the text
  // doubled). The corners keep to the HUD band, centred on an ultrawide.
  const vp = og.ui.viewport(world);
  const w = vp.width > 0 ? vp.width : 1280;
  const h = vp.height > 0 ? vp.height : 720;
  const band = vp.hud && vp.hud.width > 0 ? vp.hud : { x: 0, y: 0, width: w, height: h };
  const size = Math.max(16, Math.round(h * 0.034));
  const pad = Math.round(size * 0.9);
  const left = band.x + pad;
  const right = band.x + band.width - pad;
  const line = (text: string, x: number, y: number, color: { r: number; g: number; b: number }, align: "start" | "center" | "end" = "start", s = size) =>
    must(og.render.text(world, { font, text, position: { x, y, z: 0 }, size: s, color, align, screen: true, outline: 0.5, outlineColor: { r: 0.008, g: 0.008, b: 0.011 } }));
  const hp = (seat: number) => `HP ${hearts[seat]}/${g.hpMax}`;
  line(SEATS > 1 ? `P1 ${hp(0)}` : hp(0), left, band.y + pad + size, hearts[0]! > 1 ? WHITE : RED);
  line(`COINS ${g.coins}/${g.coinsTotal}`, left, band.y + pad + size * 2.3, GOLD);
  if (g.hasKey) line("KEY", left, band.y + pad + size * 3.6, GOLD);
  line(`ROOM ${g.room}`, right, band.y + pad + size, WHITE, "end");
  if (SEATS > 1) line(`P2 ${hp(1)}`, right, band.y + pad + size * 2.3, hearts[1]! > 1 ? WHITE : RED, "end");
  if (world.tick < 300 && g.room === 1) {
    // The prompt names each action's binding on the device used last, so a
    // pad or a rebinding changes it: "W A S D move  ·  Space attack  ·  Shift dash".
    // Each local seat's own actions (src/seats.ts names them `<control>@<seat>`); a room shows this peer's.
    const prompt = (seat: number) => {
      // The font holds ← and → but not ↑ ↓: the arrow keys' composite is named.
      const named = (action: string, verb: string) => {
        const glyph = og.input.glyph(world, `${action}@${seat}`)?.text ?? "?";
        return `${glyph.includes("↑") ? "Arrows" : glyph} ${verb}`;
      };
      return [named("move", "move"), named("attack", "attack"), named("dash", "dash")].join("  ·  ");
    };
    const small = Math.round(size * 0.75);
    if (LOCAL) {
      line(`P1  ${prompt(0)}`, band.x + band.width / 2, band.y + band.height - pad - small * 1.4, MINT, "center", small);
      line(`P2  ${prompt(1)}`, band.x + band.width / 2, band.y + band.height - pad, MINT, "center", small);
    } else line(prompt(0), band.x + band.width / 2, band.y + band.height - pad, MINT, "center", small);
  }
  const fallen = hearts.slice(0, SEATS).findIndex((n) => n === 0);
  if (fallen >= 0) line(SEATS > 1 ? `P${fallen + 1} fell. Back to the start...` : "You fell. Back to the start...", w / 2, h / 2, RED, "center", Math.round(size * 1.2));
});
