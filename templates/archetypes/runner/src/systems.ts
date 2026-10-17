// The runner's systems, in tick order: Director (phases and pace), Jump (the
// runner's body), Scroll (the world moves left past a runner who stays put),
// Spawn (obstacle and coin patterns from the dice), Collide (coins and
// crashes), Animate (which sprite each row shows) and Hud (text).
//
// Each system sees the actor rows through its typed views. The runner row
// (Actor.kind === Kind.Runner) carries the run's state and tuning for every
// system (src/components.ts says why).

import { og, defineSystem } from "@origingame/engine";
import type { ArchetypeView, Query } from "@origingame/engine";
import { Actor, Hud, Kind, Phase, Run, Tuning } from "./components.ts";

/** The ground's top: the runner's feet rest here. */
export const GROUND_Y = 0;
/** Where the runner stands; the world scrolls past this x. */
export const RUNNER_X = -5;
/** The runner sprite's size; its feet are at the bottom edge. */
export const RUNNER_SIZE = { x: 1.3, y: 1.625 } as const;
/** Ticks an early press waits for the landing. */
const JUMP_BUFFER = 6;
/** A release before this many ticks still rises this long: the shortest hop. */
const MIN_HOLD = 6;
/** Ticks the game-over card holds before a press restarts. */
const OVER_HOLD = 30;
/** Where a pooled row waits while inactive. */
const PARKED_X = -1000;
/**
 * Patterns spawn at SPAWN_X and pooled rows retire past RETIRE_X: fixed
 * numbers beyond the widest view (21:9 at the 10 m height is 23.3 m wide),
 * so the play surface's shape never changes when an obstacle arrives.
 */
export const SPAWN_X = 13;
export const RETIRE_X = -13;
/** Scenery wraps over this span: wider than the widest view plus a piece. */
export const SCENERY_SPAN = 30;

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

/** Copy a Material handle (two u32 lanes) from one column to another. */
function copyHandle(from: Uint32Array, fi: number, to: Uint32Array, ti: number): void {
  to[ti * 2] = from[fi * 2]!;
  to[ti * 2 + 1] = from[fi * 2 + 1]!;
}

/** Retire every pooled row: inactive, hidden, parked. */
function retirePools(q: Query): void {
  for (const a of q.archetypes()) {
    const actor = a.column(Actor);
    const mesh = a.column(og.Mesh);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      const k = actor.kind[i];
      if (k === Kind.Crate || k === Kind.Fence || k === Kind.Coin) {
        actor.active[i] = 0;
        mesh.visible[i] = 0;
        p[i * 3] = PARKED_X;
      }
    }
  }
}

/** Phases and pace: a press starts a round from the title or after a crash; the speed ramps while running. */
export const Director = defineSystem(
  "Director",
  { reads: [og.Input, Tuning], writes: [Run, Actor, og.Transform, og.Mesh] },
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
      run.speed[i] = Math.min(tune.speedMax[i]!, run.speed[i]! + tune.speedRamp[i]! * time.dt);
      run.distance[i] = run.distance[i]! + run.speed[i]! * time.dt;
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
    run.spawnIn[i] = Math.max(1, tune.spawnInterval[i]! * 1.5);
    run.grounded[i] = 1;
    run.buffer[i] = 0;
    run.hold[i] = 0;
    run.rising[i] = 0;
    r.a.column(Actor).vy[i] = 0;
    r.a.column(og.Transform).position[i * 3 + 1] = GROUND_Y + RUNNER_SIZE.y / 2;
    retirePools(q);
  },
);

/** The runner's body: a buffered press takes off, holding keeps the rise, releasing early cuts it; gravity brings it down. */
export const Jump = defineSystem(
  "Jump",
  { reads: [og.Input, Tuning], writes: [Run, Actor, og.Transform], after: ["Director"] },
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
    // After a crash the runner still falls back to the roof.
    actor.vy[i] = actor.vy[i]! - tune.gravity[i]! * time.dt;
    const feet = p[i * 3 + 1]! - RUNNER_SIZE.y / 2 + actor.vy[i]! * time.dt;
    if (feet <= GROUND_Y) {
      p[i * 3 + 1] = GROUND_Y + RUNNER_SIZE.y / 2;
      actor.vy[i] = 0;
      run.grounded[i] = 1;
      run.rising[i] = 0;
    } else {
      p[i * 3 + 1] = feet + RUNNER_SIZE.y / 2;
    }
  },
);

/** The world scrolls left at the run speed times each row's parallax; scenery wraps, pooled rows retire behind the left edge. */
export const Scroll = defineSystem(
  "Scroll",
  { reads: [Run], writes: [Actor, og.Transform, og.Mesh], after: ["Jump"] },
  (q, time) => {
    const r = runnerRow(q);
    if (r === null) return;
    const speed = r.a.column(Run).speed[r.i]!;
    if (speed === 0) return;
    for (const a of q.archetypes()) {
      const actor = a.column(Actor);
      const mesh = a.column(og.Transform).position;
      const visible = a.column(og.Mesh).visible;
      for (let i = 0; i < a.count; i++) {
        const k = actor.kind[i];
        if (k === Kind.Runner || actor.parallax[i] === 0) continue;
        const wrap = actor.wrap[i]!;
        if (wrap > 0) {
          let x = mesh[i * 3]! - speed * actor.parallax[i]! * time.dt;
          if (x < -wrap / 2) x += wrap;
          mesh[i * 3] = x;
        } else if (actor.active[i]) {
          const x = mesh[i * 3]! - speed * time.dt;
          if (x < RETIRE_X) {
            actor.active[i] = 0;
            visible[i] = 0;
            mesh[i * 3] = PARKED_X;
          } else {
            mesh[i * 3] = x;
          }
        }
      }
    }
  },
);

/** Takes an inactive pooled row of a kind and puts it in play at (x, y); false when the pool is spent. */
function place(q: Query, kind: number, x: number, y: number): boolean {
  for (const a of q.archetypes()) {
    const actor = a.column(Actor);
    for (let i = 0; i < a.count; i++) {
      if (actor.kind[i] !== kind || actor.active[i]) continue;
      actor.active[i] = 1;
      a.column(og.Mesh).visible[i] = 1;
      const p = a.column(og.Transform).position;
      p[i * 3] = x;
      p[i * 3 + 1] = y;
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
export function placePattern(q: Query, pattern: number, x0: number): number {
  const coin = (dx: number, y: number) => place(q, Kind.Coin, x0 + dx, y);
  switch (pattern) {
    case 0:
      place(q, Kind.Crate, x0, REST_Y[Kind.Crate]!);
      return 1;
    case 1:
      place(q, Kind.Crate, x0, REST_Y[Kind.Crate]!);
      coin(-0.9, 1.7);
      coin(0, 2.2);
      coin(0.9, 1.7);
      return 2;
    case 2:
      for (let k = 0; k < 4; k++) coin(k * 0.9, REST_Y[Kind.Coin]!);
      return 3;
    case 3:
      place(q, Kind.Fence, x0, REST_Y[Kind.Fence]!);
      return 1;
    case 4:
      place(q, Kind.Fence, x0, REST_Y[Kind.Fence]!);
      coin(0, 2.9);
      return 1;
    default:
      place(q, Kind.Crate, x0, REST_Y[Kind.Crate]!);
      place(q, Kind.Crate, x0 + 4.2, REST_Y[Kind.Crate]!);
      return 5;
  }
}

/** Obstacle and coin patterns from the dice, one every Tuning.spawnInterval seconds (×0.85–1.25); fences after 60 m, double crates after 150 m. */
export const Spawn = defineSystem(
  "Spawn",
  { reads: [Tuning], writes: [og.Rng, Run, Actor, og.Transform, og.Mesh], after: ["Scroll"] },
  (q, time, world) => {
    const r = runnerRow(q);
    const rng = world.resource(og.Rng);
    if (r === null || !rng.ok) return;
    const run = r.a.column(Run);
    const i = r.i;
    if (run.phase[i] !== Phase.Running) return;
    run.spawnIn[i] = run.spawnIn[i]! - time.dt;
    if (run.spawnIn[i]! > 0) return;
    const distance = run.distance[i]!;
    const top = distance < 60 ? 2 : distance < 150 ? 4 : 5;
    const pattern = rng.value.integer(0, top);
    const length = placePattern(q, pattern, SPAWN_X);
    // Read the tuning after placing: the views stay valid within the call.
    const interval = r.a.column(Tuning).spawnInterval[i]!;
    run.spawnIn[i] = interval * rng.value.range(0.85, 1.25) + length / Math.max(1, run.speed[i]!);
  },
);

/** Overlap tests against the runner's hit box: a coin is collected, a crate or a fence ends the round. */
export const Collide = defineSystem(
  "Collide",
  { reads: [og.Transform, Tuning], writes: [Run, Actor, og.Mesh], after: ["Spawn"] },
  (q) => {
    const r = runnerRow(q);
    if (r === null) return;
    const run = r.a.column(Run);
    const ri = r.i;
    if (run.phase[ri] !== Phase.Running) return;
    const rp = r.a.column(og.Transform).position;
    const ra = r.a.column(Actor);
    const rx = rp[ri * 3]!, ry = rp[ri * 3 + 1]!, rw = ra.w[ri]! / 2, rh = ra.h[ri]! / 2;
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
          a.column(og.Mesh).visible[i] = 0;
          run.coins[ri] = run.coins[ri]! + 1;
        } else {
          crashed = true;
        }
      }
    }
    run.score[ri] = Math.floor(run.distance[ri]!) + run.coins[ri]! * r.a.column(Tuning).coinValue[ri]!;
    if (crashed) {
      run.phase[ri] = Phase.Over;
      run.phaseTicks[ri] = 0;
      run.best[ri] = Math.max(run.best[ri]!, run.score[ri]!);
    }
  },
);

/** Which look each row shows: the runner's pose from the phase and its stride; coins turn. */
export const Animate = defineSystem(
  "Animate",
  { reads: [Run, Actor], writes: [og.Mesh, og.Transform], after: ["Collide"] },
  (q, time) => {
    const r = runnerRow(q);
    if (r === null) return;
    const run = r.a.column(Run);
    const ri = r.i;
    const actor = r.a.column(Actor);
    let look = 0;
    if (run.phase[ri] === Phase.Over) look = 4;
    else if (run.phase[ri] === Phase.Running) look = run.grounded[ri] ? 1 + (Math.floor(run.distance[ri]! / 0.9) % 2) : 3;
    const looks = [actor.look0, actor.look1, actor.look2, actor.look3, actor.look4][look]!;
    copyHandle(looks, ri, r.a.column(og.Mesh).material, ri);
    // Coins turn: a triangle wave on their width (tick arithmetic only, so every host agrees).
    const phase = (time.tick % 40) / 40;
    const turn = 0.25 + 1.5 * Math.abs(phase - 0.5);
    for (const a of q.archetypes()) {
      const kind = a.column(Actor).kind;
      const s = a.column(og.Transform).scale;
      for (let i = 0; i < a.count; i++) if (kind[i] === Kind.Coin) s[i * 3] = 0.7 * turn;
    }
  },
);

/** A rough advance for centring: Latin at 0.55 em, CJK at 1 em (the API has no text measure yet). */
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
const GOLD = { r: 1, g: 0.66, b: 0.05 };
const MINT = { r: 0.3, g: 0.85, b: 0.75 };
const CORAL = { r: 1, g: 0.25, b: 0.18 };
