// The runner's systems, in tick order: Director (phases and pace), Jump (the
// runner's body), Scroll (the world moves left past a runner who stays put),
// Spawn (obstacle and coin patterns from the dice), Collide (coins and
// crashes, with their feel), Voices (the jump, coin, crash and start
// sounds the systems before it asked for), Dust (puffs on the run clip's footfalls),
// Animate (the courier's clip and afterimage) and Hud (text).
//
// Each system sees the actor rows through its typed views. The runner row
// (Actor.kind === Kind.Runner) carries the run's state and tuning for every
// system (src/components.ts says why). Motion uses Time.scaledDt, so a hit
// stop freezes the world while the camera still shakes.

import { og, defineSystem } from "@origingame/engine";
import type { ArchetypeView, Entity, Query, World } from "@origingame/engine";
import { COURIER } from "./art.ts";
import { Actor, Hud, Kind, Phase, Run, Sfx, Sound, Sounds, Tuning } from "./components.ts";

/** The ground's top: the runner's feet rest here. */
export const GROUND_Y = 0;
/** Where the runner stands; the world scrolls past this x. */
export const RUNNER_X = -5;
/** The runner sprite's size. Its position is its feet; its hit box is centred RUNNER_SIZE.y / 2 above them. */
export const RUNNER_SIZE = { x: 1.3, y: 1.625 } as const;
/** Ticks an early press waits for the landing. */
const JUMP_BUFFER = 6;
/** A release before this many ticks still rises this long: the shortest hop. */
const MIN_HOLD = 6;
/** Ticks the game-over card holds before a press restarts. */
const OVER_HOLD = 30;
/** Where a pooled row waits while parked. */
const PARKED_X = -1000;
/**
 * Patterns spawn at SPAWN_X and pooled rows retire past RETIRE_X: fixed
 * numbers beyond the widest view (21:9 at the 10 m height is 23.3 m wide),
 * so the play surface's shape never changes when an obstacle arrives.
 */
export const SPAWN_X = 13;
export const RETIRE_X = -13;
/** Clouds wrap over this span: wider than the widest view plus a piece. */
export const SCENERY_SPAN = 30;

/** The feel: tuned by eye, not by the tests (except that they never change the rules). */
const FEEL = {
  /** The take-off stretch and the landing squash (x, y factors at the impact). */
  takeOff: { x: 0.78, y: 1.28 },
  landing: { x: 1.3, y: 0.72 },
  /** A crash: ticks of hit stop, screen shake trauma, how far the courier is knocked back and up. */
  hitStop: 8,
  shake: 0.5,
  knockBack: 0.9,
  knockUp: 6,
  /** The red flash of a crash. */
  crashFlash: { r: 1, g: 0.12, b: 0.08 },
  /** The afterimage shows within this much of the top speed. */
  afterimageBelowMax: 0.75,
} as const;

interface Row {
  readonly a: ArchetypeView;
  readonly i: number;
}

/** The runner row among the system's archetypes, or null before `start` placed it. */
function runnerRow(q: Query): Row | null {
  for (const a of q.archetypes()) {
    const kind = a.column(Actor).kind;
    for (let i = 0; i < a.count; i++) if (kind[i] === Kind.Runner) return { a, i };
  }
  return null;
}

function pressed(i: { pressed(k: string): boolean; buttonPressed(b: "left"): boolean }): boolean {
  return i.pressed("Space") || i.pressed("ArrowUp") || i.pressed("KeyW") || i.buttonPressed("left");
}

function held(i: { down(k: string): boolean; buttonDown(b: "left"): boolean }): boolean {
  return i.down("Space") || i.down("ArrowUp") || i.down("KeyW") || i.buttonDown("left");
}

function pooled(kind: number | undefined): boolean {
  return kind === Kind.Crate || kind === Kind.Fence || kind === Kind.Coin || kind === Kind.Dust;
}

/** Retire every pooled row: inactive and parked. */
function retirePools(q: Query): void {
  for (const a of q.archetypes()) {
    const actor = a.column(Actor);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      if (!pooled(actor.kind[i])) continue;
      actor.active[i] = 0;
      p[i * 3] = PARKED_X;
    }
  }
}

/** Phases and pace: a press starts a round from the title or after a crash; the speed ramps while running. */
export const Director = defineSystem(
  "Director",
  { reads: [og.Input, Tuning], writes: [Run, Actor, og.Transform, og.Sprite, Sounds] },
  (q, time, world) => {
    const r = runnerRow(q);
    const input = world.resource(og.Input);
    if (r === null || !input.ok) return;
    const run = r.a.column(Run);
    const tune = r.a.column(Tuning);
    const i = r.i;
    run.phaseTicks[i] = run.phaseTicks[i]! + 1;
    const phase = run.phase[i];
    const go = pressed(input.value);
    if (phase === Phase.Running) {
      run.speed[i] = Math.min(tune.speedMax[i]!, run.speed[i]! + tune.speedRamp[i]! * time.scaledDt);
      run.distance[i] = run.distance[i]! + run.speed[i]! * time.scaledDt;
      run.score[i] = Math.floor(run.distance[i]!) + run.coins[i]! * tune.coinValue[i]!;
      return;
    }
    // The title scrolls the city slowly; the game-over card holds the world still.
    run.speed[i] = phase === Phase.Title ? 1.5 : 0;
    if (!go || (phase === Phase.Over && run.phaseTicks[i]! < OVER_HOLD)) return;
    run.phase[i] = Phase.Running;
    run.phaseTicks[i] = 0;
    run.speed[i] = tune.speed[i]!;
    run.distance[i] = 0;
    run.coins[i] = 0;
    run.score[i] = 0;
    run.rounds[i] = run.rounds[i]! + 1;
    ask(world, Sound.Start);
    run.spawnIn[i] = Math.max(1, tune.spawnInterval[i]! * 1.5);
    run.grounded[i] = 1;
    run.buffer[i] = 0;
    run.hold[i] = 0;
    run.rising[i] = 0;
    r.a.column(Actor).vy[i] = 0;
    // The crash's knock-back, squash and flash end before the card's hold; stop them anyway and stand the courier up.
    og.tween.cancelEntity(world, r.a.entity(i));
    const t = r.a.column(og.Transform);
    t.position[i * 3] = RUNNER_X;
    t.position[i * 3 + 1] = GROUND_Y;
    t.scale[i * 3] = 1;
    t.scale[i * 3 + 1] = 1;
    r.a.column(og.Sprite).flash[i] = 0;
    retirePools(q);
  },
);

/** The runner's body: a buffered press takes off, holding keeps the rise, releasing early cuts it; gravity brings it down. Take-off stretches, landing squashes. */
export const Jump = defineSystem(
  "Jump",
  { reads: [og.Input, Tuning], writes: [Run, Actor, og.Transform, Sounds], after: ["Director"] },
  (q, time, world) => {
    const r = runnerRow(q);
    const input = world.resource(og.Input);
    if (r === null || !input.ok) return;
    const { a, i } = r;
    const run = a.column(Run);
    const tune = a.column(Tuning);
    const actor = a.column(Actor);
    const p = a.column(og.Transform).position;
    if (run.phase[i] === Phase.Title) return;
    if (run.phase[i] === Phase.Running) {
      // The press that started the round does not also jump.
      if (pressed(input.value) && run.phaseTicks[i]! > 0) run.buffer[i] = JUMP_BUFFER;
      if (run.buffer[i]! > 0 && run.grounded[i]) {
        actor.vy[i] = tune.jumpSpeed[i]!;
        run.grounded[i] = 0;
        run.rising[i] = 1;
        run.hold[i] = 0;
        run.buffer[i] = 0;
        og.tween.squash(world, { entity: a.entity(i), impact: FEEL.takeOff, duration: 0.22 });
        ask(world, Sound.Jump);
      } else if (run.buffer[i]! > 0) {
        run.buffer[i] = run.buffer[i]! - 1;
      }
      if (run.rising[i]) {
        run.hold[i] = run.hold[i]! + 1;
        if (actor.vy[i]! <= 0) {
          run.rising[i] = 0;
        } else if (!held(input.value) && run.hold[i]! >= MIN_HOLD) {
          actor.vy[i] = actor.vy[i]! * tune.jumpCut[i]!;
          run.rising[i] = 0;
        }
      }
    }
    if (run.grounded[i]) return;
    const dt = time.scaledDt;
    if (dt === 0) return;
    // After a crash the runner still falls back to the roof.
    actor.vy[i] = actor.vy[i]! - tune.gravity[i]! * dt;
    const feet = p[i * 3 + 1]! + actor.vy[i]! * dt;
    if (feet <= GROUND_Y) {
      p[i * 3 + 1] = GROUND_Y;
      actor.vy[i] = 0;
      run.grounded[i] = 1;
      run.rising[i] = 0;
      og.tween.squash(world, { entity: a.entity(i), impact: FEEL.landing, duration: 0.2 });
    } else {
      p[i * 3 + 1] = feet;
    }
  },
);

/** The world scrolls left at the run speed times each row's parallax; scenery wraps, pooled rows retire behind the left edge. */
export const Scroll = defineSystem(
  "Scroll",
  { reads: [Run], writes: [Actor, og.Transform], after: ["Jump"] },
  (q, time) => {
    const r = runnerRow(q);
    if (r === null) return;
    const step = r.a.column(Run).speed[r.i]! * time.scaledDt;
    if (step === 0) return;
    for (const a of q.archetypes()) {
      const actor = a.column(Actor);
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        const k = actor.kind[i];
        if (k === Kind.Runner || actor.parallax[i] === 0) continue;
        const wrap = actor.wrap[i]!;
        if (wrap > 0) {
          let x = p[i * 3]! - step * actor.parallax[i]!;
          if (x < -wrap / 2) x += wrap;
          p[i * 3] = x;
        } else if (p[i * 3]! > PARKED_X / 2) {
          // In play, or a collected coin or a dust puff finishing its tween: carried along until it retires.
          const x = p[i * 3]! - step;
          if (x < RETIRE_X) {
            actor.active[i] = 0;
            p[i * 3] = PARKED_X;
          } else {
            p[i * 3] = x;
          }
        }
      }
    }
  },
);

/** Takes an inactive pooled row of a kind and puts it in play at (x, y), whole and opaque; false when the pool is spent. */
function place(q: Query, world: World, kind: number, x: number, y: number): boolean {
  for (const a of q.archetypes()) {
    const actor = a.column(Actor);
    for (let i = 0; i < a.count; i++) {
      if (actor.kind[i] !== kind || actor.active[i]) continue;
      og.tween.cancelEntity(world, a.entity(i));
      actor.active[i] = 1;
      const t = a.column(og.Transform);
      t.position[i * 3] = x;
      t.position[i * 3 + 1] = y;
      t.scale[i * 3] = 1;
      t.scale[i * 3 + 1] = 1;
      a.column(og.Sprite).opacity[i] = 1;
      return true;
    }
  }
  return false;
}

/** Where each kind rests: its centre height on the roof. */
export const REST_Y: Readonly<Record<number, number>> = { [Kind.Crate]: 0.4, [Kind.Fence]: 0.8, [Kind.Coin]: 0.6 };

/**
 * A pattern of obstacles and coins, placed from x0 rightwards; returns its length in metres.
 * 0 crate, 1 crate under a coin arc, 2 a line of coins, 3 a fence, 4 a fence under a high coin, 5 two crates.
 */
export function placePattern(q: Query, world: World, pattern: number, x0: number): number {
  const at = (kind: number, dx: number, y: number) => place(q, world, kind, x0 + dx, y);
  const coin = (dx: number, y: number) => at(Kind.Coin, dx, y);
  switch (pattern) {
    case 0:
      at(Kind.Crate, 0, REST_Y[Kind.Crate]!);
      return 1;
    case 1:
      at(Kind.Crate, 0, REST_Y[Kind.Crate]!);
      coin(-0.9, 1.7);
      coin(0, 2.2);
      coin(0.9, 1.7);
      return 2;
    case 2:
      for (let k = 0; k < 4; k++) coin(k * 0.9, REST_Y[Kind.Coin]!);
      return 3;
    case 3:
      at(Kind.Fence, 0, REST_Y[Kind.Fence]!);
      return 1;
    case 4:
      at(Kind.Fence, 0, REST_Y[Kind.Fence]!);
      coin(0, 2.9);
      return 1;
    default:
      at(Kind.Crate, 0, REST_Y[Kind.Crate]!);
      at(Kind.Crate, 4.2, REST_Y[Kind.Crate]!);
      return 5;
  }
}

/** Obstacle and coin patterns from the dice, one every Tuning.spawnInterval seconds (×0.85–1.25); fences after 60 m, double crates after 150 m. */
export const Spawn = defineSystem(
  "Spawn",
  { reads: [Tuning], writes: [og.Rng, Run, Actor, og.Transform, og.Sprite], after: ["Scroll"] },
  (q, time, world) => {
    const r = runnerRow(q);
    const rng = world.resource(og.Rng);
    if (r === null || !rng.ok) return;
    const run = r.a.column(Run);
    const i = r.i;
    if (run.phase[i] !== Phase.Running) return;
    run.spawnIn[i] = run.spawnIn[i]! - time.scaledDt;
    if (run.spawnIn[i]! > 0) return;
    const distance = run.distance[i]!;
    const top = distance < 60 ? 2 : distance < 150 ? 4 : 5;
    const pattern = rng.value.integer(0, top);
    const length = placePattern(q, world, pattern, SPAWN_X);
    // Read the tuning after placing: the views stay valid within the call.
    const interval = r.a.column(Tuning).spawnInterval[i]!;
    run.spawnIn[i] = interval * rng.value.range(0.85, 1.25) + length / Math.max(1, run.speed[i]!);
  },
);

const GOLD = { r: 1, g: 0.66, b: 0.05 };

/** A collected coin: "+10" rises from it while it pops (grows, lifts and fades). */
function coinPop(world: World, coin: Entity, x: number, y: number, value: number): void {
  og.render.floatingText(world, { text: `+${value}`, position: { x, y: y + 0.35, z: 5 }, size: 0.5, color: GOLD, rise: 1.1, spread: 0.15, lifetime: 0.7, pop: 0.6 });
  og.tween.parallel(world, [
    { entity: coin, field: "scale", to: [1.8, 1.8, 1], duration: 0.22, ease: "quadOut" },
    { entity: coin, field: "position.y", to: [0.5], relative: true, duration: 0.22, ease: "quadOut" },
    { entity: coin, component: og.Sprite, field: "opacity", to: [0], duration: 0.22, ease: "quadIn" },
  ]);
}

/** A crash: hit stop, a shake and a kick, the courier knocked back and up, flashing red. */
function crashFeel(world: World, runner: Entity): void {
  og.time.hitStop(world, FEEL.hitStop);
  og.camera.shake(world, FEEL.shake);
  og.camera.kick(world, { x: -1, y: 0, z: 0 }, 0.3);
  og.tween.to(world, { entity: runner, field: "position.x", to: [-FEEL.knockBack], relative: true, duration: 0.3, ease: "quartOut" });
  // The red holds through the hit stop, then fades.
  og.tween.flash(world, { entity: runner, color: FEEL.crashFlash, amount: 0.85, duration: 0.4 });
}

/** Overlap tests against the runner's hit box: a coin is collected, a crate or a fence ends the round. */
export const Collide = defineSystem(
  "Collide",
  { reads: [og.Transform, Tuning], writes: [Run, Actor, Sounds], after: ["Spawn"] },
  (q, _time, world) => {
    const r = runnerRow(q);
    if (r === null) return;
    const run = r.a.column(Run);
    const ri = r.i;
    if (run.phase[ri] !== Phase.Running) return;
    const rp = r.a.column(og.Transform).position;
    const ra = r.a.column(Actor);
    const value = r.a.column(Tuning).coinValue[ri]!;
    const rx = rp[ri * 3]!, ry = rp[ri * 3 + 1]! + RUNNER_SIZE.y / 2, rw = ra.w[ri]! / 2, rh = ra.h[ri]! / 2;
    let crashed = false;
    for (const a of q.archetypes()) {
      const actor = a.column(Actor);
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        if (!actor.active[i]) continue;
        const k = actor.kind[i];
        if (k !== Kind.Coin && k !== Kind.Crate && k !== Kind.Fence) continue;
        const overlap =
          Math.abs(p[i * 3]! - rx) < rw + actor.w[i]! / 2 && Math.abs(p[i * 3 + 1]! - ry) < rh + actor.h[i]! / 2;
        if (!overlap) continue;
        if (k === Kind.Coin) {
          actor.active[i] = 0;
          run.coins[ri] = run.coins[ri]! + 1;
          coinPop(world, a.entity(i), p[i * 3]!, p[i * 3 + 1]!, value);
          ask(world, Sound.Coin);
        } else {
          crashed = true;
        }
      }
    }
    run.score[ri] = Math.floor(run.distance[ri]!) + run.coins[ri]! * value;
    if (crashed) {
      run.phase[ri] = Phase.Over;
      run.phaseTicks[ri] = 0;
      const score = run.score[ri]!;
      if (score > run.best[ri]!) {
        run.best[ri] = score;
        // The best survives a reload: the player's store (the Player's save, or this browser's).
        og.platform.storeSet(world, { key: "best", value: score });
      }
      og.platform.leaderboardSubmit(world, { board: "main", score });
      og.platform.telemetry(world, "round.over", { score, coins: run.coins[ri]!, distance: Math.round(run.distance[ri]!) });
      run.grounded[ri] = 0;
      run.rising[ri] = 0;
      ra.vy[ri] = FEEL.knockUp;
      crashFeel(world, r.a.entity(ri));
      ask(world, Sound.Crash);
    }
  },
);

/** Ask for a sound this tick (Voices plays it). */
function ask(world: World, bit: number): void {
  const sounds = world.resource(Sounds);
  if (sounds.ok) sounds.value.sfx |= bit;
}

/** Plays each requested sound once: a voice already playing is stopped this tick and restarted the next. */
export const Voices = defineSystem("Voices", { reads: [Sfx], writes: [Sounds, og.AudioSource], after: ["Collide"] }, (q, _time, world) => {
  const sounds = world.resource(Sounds);
  if (!sounds.ok || sounds.value.sfx === 0) return;
  const s = sounds.value;
  let left = s.sfx;
  for (const a of q.archetypes()) {
    const bit = a.column(Sfx).bit;
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = bit[i]!;
      if ((s.sfx & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        left &= ~b;
      }
    }
  }
  s.sfx = left;
});

/** On each footfall of the run clip (its "step" events) a dust puff leaves the courier's feet. */
export const Dust = defineSystem(
  "Dust",
  { reads: [og.FlipbookEvent, Run], writes: [Actor, og.Transform], after: ["Collide"] },
  (q, _time, world) => {
    const r = runnerRow(q);
    if (r === null) return;
    const run = r.a.column(Run);
    if (run.phase[r.i] !== Phase.Running || !run.grounded[r.i]) return;
    const runner = r.a.entity(r.i);
    const rx = r.a.column(og.Transform).position[r.i * 3]!;
    for (const e of world.events(og.FlipbookEvent)) {
      if (e.entity !== runner || e.name !== "step") continue;
      // The oldest puff (the leftmost; parked ones first) is reused.
      let best: Row | null = null;
      let bestX = Infinity;
      for (const a of q.archetypes()) {
        const actor = a.column(Actor);
        const p = a.column(og.Transform).position;
        for (let i = 0; i < a.count; i++) {
          if (actor.kind[i] === Kind.Dust && p[i * 3]! < bestX) {
            best = { a, i };
            bestX = p[i * 3]!;
          }
        }
      }
      if (best === null) return;
      const puff = best.a.entity(best.i);
      og.tween.cancelEntity(world, puff);
      const t = best.a.column(og.Transform);
      t.position[best.i * 3] = rx - 0.25;
      t.position[best.i * 3 + 1] = GROUND_Y + 0.08;
      og.tween.parallel(world, [
        { entity: puff, field: "scale", from: [1, 1, 1], to: [0.2, 0.2, 1], duration: 0.3, ease: "quadIn" },
        { entity: puff, field: "position.y", to: [0.3], relative: true, duration: 0.3, ease: "quadOut" },
        { entity: puff, component: og.Sprite, field: "opacity", from: [0.7], to: [0], duration: 0.3, ease: "linear" },
      ]);
    }
  },
);

/** The courier's clip from the phase and the body (idle, run, jump, fall, crash), the run clip's pace from the speed, and the afterimage near top speed. */
export const Animate = defineSystem(
  "Animate",
  { reads: [Run, Actor, Tuning], writes: [og.Flipbook, og.Afterimage], after: ["Dust"] },
  (q) => {
    for (const a of q.archetypes()) {
      const run = a.column(Run);
      const actor = a.column(Actor);
      const flip = a.column(og.Flipbook);
      const ghost = a.column(og.Afterimage);
      const speedMax = a.column(Tuning).speedMax;
      for (let i = 0; i < a.count; i++) {
        if (actor.kind[i] !== Kind.Runner) continue;
        const phase = run.phase[i];
        let clip: number = COURIER.idle;
        if (phase === Phase.Over) clip = COURIER.crash;
        else if (phase === Phase.Running) clip = run.grounded[i] ? COURIER.run : actor.vy[i]! > 0 ? COURIER.jump : COURIER.fall;
        flip.clip[i] = clip;
        flip.speed[i] = clip === COURIER.run ? Math.min(1.6, Math.max(0.8, run.speed[i]! / 7)) : 1;
        ghost.active[i] = phase === Phase.Running && run.speed[i]! >= speedMax[i]! - FEEL.afterimageBelowMax ? 1 : 0;
      }
    }
  },
);

/** A rough advance for centring: Latin at 0.56 em, CJK at 1 em (the API has no text measure yet). */
function width(text: string, size: number): number {
  let w = 0;
  for (const c of text) w += c.charCodeAt(0) > 0x2e80 ? 1 : c === " " ? 0.3 : 0.56;
  return w * size;
}

/** The HUD: score, coins and best while running; the title card and the game-over card. */
export const HudText = defineSystem("Hud", { reads: [Run, Hud, Tuning], after: ["Animate"] }, (q, _time, world) => {
  for (const a of q.archetypes()) {
    const run = a.column(Run);
    const hud = a.column(Hud);
    for (let i = 0; i < a.count; i++) {
      const font = hud.font[i] as never;
      const left = -hud.halfWidth[i]!;
      const top = hud.top[i]!;
      const text = (t: string, x: number, y: number, size: number, color = WHITE) =>
        og.render.text(world, { font, text: t, position: { x, y, z: 10 }, size, color, layer: 10 });
      const centred = (t: string, y: number, size: number, color = WHITE) => text(t, -width(t, size) / 2, y, size, color);
      const phase = run.phase[i];
      if (phase === Phase.Title) {
        centred("ROOFTOP RUN", top - 2, 1.3, GOLD);
        centred("屋顶快跑", top - 3, 0.6, WHITE);
        if (Math.floor(run.phaseTicks[i]! / 30) % 2 === 0) centred("Tap or press Space to run", top - 3.8, 0.45, MINT);
        centred("点击或按空格开始", top - 4.4, 0.4, MINT);
        continue;
      }
      text(`SCORE ${run.score[i]}`, left + 0.4, top - 0.8, 0.55);
      text(`COINS ${run.coins[i]}`, left + 0.4, top - 1.5, 0.45, GOLD);
      const best = `BEST ${run.best[i]}`;
      text(best, hud.halfWidth[i]! - 0.4 - width(best, 0.45), top - 0.8, 0.45, MINT);
      if (phase === Phase.Running && run.phaseTicks[i]! < 240) {
        centred("Tap: hop   Hold: high jump", top - 2.4, 0.45, MINT);
        centred("点按小跳 · 按住高跳", top - 3, 0.4, MINT);
      }
      if (phase === Phase.Over) {
        centred("GAME OVER", top - 2, 1.1, CORAL);
        centred(`Score ${run.score[i]}   Best ${run.best[i]}`, top - 2.9, 0.55, WHITE);
        if (run.phaseTicks[i]! >= 30) centred("Tap or press Space to run again · 再来一次", top - 3.7, 0.42, MINT);
      }
    }
  }
});

const WHITE = { r: 1, g: 1, b: 1 };
const MINT = { r: 0.3, g: 0.85, b: 0.75 };
const CORAL = { r: 1, g: 0.25, b: 0.18 };

/** The best score from the player's store: asked for on the first tick, taken when the answer arrives. */
export const Records = defineSystem("Records", { reads: [Actor], writes: [Run] }, (q, time, world) => {
  const r = runnerRow(q);
  if (r === null) return;
  const run = r.a.column(Run);
  if (time.tick === 0) {
    const asked = og.platform.storeGet(world, "best");
    run.bestRequest[r.i] = asked.ok ? asked.value : 0;
    return;
  }
  const request = run.bestRequest[r.i]!;
  if (request === 0) return;
  const answer = og.platform.result(world, request) as { ok: boolean; data: { value: unknown } } | null;
  if (answer === null) return;
  run.bestRequest[r.i] = 0;
  const saved = answer.ok && typeof answer.data.value === "number" ? answer.data.value : 0;
  run.best[r.i] = Math.max(run.best[r.i]!, saved);
});
