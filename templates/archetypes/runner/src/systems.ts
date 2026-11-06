// The runner's systems, in tick order: Riders (each seat's og.PlayerInput
// onto its courier), Director (phases and pace), Jump (each courier's body:
// og.PlatformerController's jump, fed its seat's press), Scroll (the world
// moves left past couriers who stay put), Spawn (obstacle and coin patterns
// from the dice), Collide (coins and crashes, with their feel), Voices (the
// jump, coin, crash and start sounds the systems before it asked for), Dust
// (puffs on the run clip's footfalls), Animate (each courier's clip and
// afterimage) and Hud (text).
//
// Each system sees the actor rows through its typed views. The runner row
// (Actor.kind === Kind.Runner, Run.seat 0) carries the round's state and
// tuning for every system (src/components.ts says why). With two seats
// (src/seats.ts) a second courier, `runner-1`, races on the same roofs: its
// row's Run holds its own distance, coins and crash (Run.out), and the round
// goes on while one courier still runs. Gameplay reads og.PlayerInput alone
// (through Run.buttons). Motion uses Time.scaledDt, so a hit stop freezes
// the world while the camera still shakes.

import { og, defineSystem, CHARACTER_JUMPED, CHARACTER_LANDED } from "@origingame/engine";
import type { ArchetypeView, Entity, Query, World } from "@origingame/engine";
import { COURIER } from "./art.ts";
import { Actor, Hud, Kind, Phase, Race, Run, Sfx, Sound, Sounds, Tuning } from "./components.ts";
import { held, LOCAL, ONLINE, pressedNow, SEATS } from "./seats.ts";

/** The ground's top: the runner's feet rest here. */
export const GROUND_Y = 0;
/** Where the runner stands; the world scrolls past this x. */
export const RUNNER_X = -5;
/** How far behind seat 0 the second courier runs. */
export const SEAT_STEP = 1;
/** Where a seat's courier stands. */
export const homeX = (seat: number): number => RUNNER_X - seat * SEAT_STEP;
/** The runner sprite's size. Its position is its feet; its hit box is centred RUNNER_SIZE.y / 2 above them. */
export const RUNNER_SIZE = { x: 1.3, y: 1.625 } as const;
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

/** The runner row (seat 0's courier, which holds the round) among the system's archetypes, or null before `start` placed it. */
function runnerRow(q: Query): Row | null {
  for (const a of q.archetypes()) {
    const kind = a.column(Actor).kind;
    const seat = a.column(Run).seat;
    for (let i = 0; i < a.count; i++) if (kind[i] === Kind.Runner && seat[i] === 0) return { a, i };
  }
  return null;
}

/** Every courier's row (one per seat). */
function couriers(q: Query): Row[] {
  const out: Row[] = [];
  for (const a of q.archetypes()) {
    const kind = a.column(Actor).kind;
    for (let i = 0; i < a.count; i++) if (kind[i] === Kind.Runner) out.push({ a, i });
  }
  return out;
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

/** Each seat's og.PlayerInput onto its courier's Run: this tick's buttons and last tick's (for presses). */
export const Riders = defineSystem("Riders", { reads: [og.PlayerInput], writes: [Run] }, (q) => {
  for (const a of q.archetypes()) {
    const input = a.column(og.PlayerInput);
    const run = a.column(Run);
    for (let i = 0; i < a.count; i++) {
      run.before[i] = run.buttons[i]!;
      run.buttons[i] = input.buttons[i]!;
    }
  }
});

/**
 * Phases and pace: any courier's press starts a round from the title or
 * after the card's hold; the speed ramps while running, and each courier
 * still running adds the scroll to its distance.
 */
export const Director = defineSystem(
  "Director",
  { reads: [Tuning], writes: [Run, Actor, og.Transform, og.Sprite, Sounds, Race] },
  (q, time, world) => {
    const r = runnerRow(q);
    if (r === null) return;
    const run = r.a.column(Run);
    const tune = r.a.column(Tuning);
    const i = r.i;
    const riders = couriers(q);
    run.phaseTicks[i] = run.phaseTicks[i]! + 1;
    const phase = run.phase[i];
    if (phase === Phase.Running) {
      run.speed[i] = Math.min(tune.speedMax[i]!, run.speed[i]! + tune.speedRamp[i]! * time.scaledDt);
      const step = run.speed[i]! * time.scaledDt;
      run.course[i] = run.course[i]! + step;
      const race = world.resource(Race);
      for (const c of riders) {
        const cr = c.a.column(Run);
        if (cr.out[c.i]) continue;
        cr.distance[c.i] = cr.distance[c.i]! + step;
        cr.score[c.i] = Math.floor(cr.distance[c.i]!) + cr.coins[c.i]! * tune.coinValue[i]!;
        if (cr.seat[c.i] === 1 && race.ok) race.value.distance = cr.distance[c.i]!;
      }
      return;
    }
    // The title scrolls the city slowly; the game-over card holds the world still.
    run.speed[i] = phase === Phase.Title ? 1.5 : 0;
    // A joined courier's press starts the round; another seat's first press joins it to the next.
    let go = false;
    const race = world.resource(Race);
    for (const c of riders) {
      const cr = c.a.column(Run);
      if (!pressedNow(cr.buttons[c.i]!, cr.before[c.i]!, "jump")) continue;
      if (cr.joined[c.i]) {
        go = true;
      } else {
        cr.joined[c.i] = 1;
        if (race.ok) race.value.joined = true;
        ask(world, Sound.Hold);
      }
    }
    if (!go) return;
    if (phase === Phase.Over && run.phaseTicks[i]! < OVER_HOLD) {
      // The card holds, but the press is heard: never ignore an input silently.
      ask(world, Sound.Hold);
      return;
    }
    run.phase[i] = Phase.Running;
    run.phaseTicks[i] = 0;
    run.speed[i] = tune.speed[i]!;
    run.course[i] = 0;
    run.rounds[i] = run.rounds[i]! + 1;
    ask(world, Sound.Start);
    run.spawnIn[i] = Math.max(1, tune.spawnInterval[i]! * 1.5);
    let racers = 0;
    for (const c of riders) racers += c.a.column(Run).joined[c.i]!;
    if (race.ok) {
      race.value.racing = racers > 1;
      race.value.distance = 0;
      race.value.out = false;
      race.value.winner = 0;
    }
    for (const c of riders) {
      const cr = c.a.column(Run);
      const k = c.i;
      // A courier whose seat has not joined sits the round out: the roofs carry it off (Scroll).
      cr.out[k] = cr.joined[k] ? 0 : 1;
      cr.distance[k] = 0;
      cr.coins[k] = 0;
      cr.score[k] = 0;
      cr.grounded[k] = 1;
      cr.startHeld[k] = 1;
      // Jump hands the body this velocity (0: standing) on the next tick's step.
      cr.kick[k] = 1;
      c.a.column(Actor).vy[k] = 0;
      // The crash's knock-back, squash and flash end before the card's hold; stop them anyway and stand the courier up.
      og.tween.cancelEntity(world, c.a.entity(k));
      const t = c.a.column(og.Transform);
      t.position[k * 3] = homeX(cr.seat[k]!);
      t.position[k * 3 + 1] = GROUND_Y;
      t.scale[k * 3] = 1;
      t.scale[k * 3 + 1] = 1;
      c.a.column(og.Sprite).flash[k] = 0;
    }
    retirePools(q);
  },
);

/**
 * Each courier's body is an og.PlatformerController that never runs
 * (moveX 0; the world scrolls past it): this system holds its jump while its
 * seat holds `jump` (the press that started the round excepted), copies the
 * runner row's tunables to it, and turns its events into feel. The
 * controller buffers an early press, cuts a released jump, hangs at the apex
 * and lands on the roof's collider; take-off stretches and landing
 * squashes, a tick after the step that did them (og.CharacterEvent).
 */
export const Jump = defineSystem(
  "Jump",
  { reads: [Tuning, og.CharacterEvent], writes: [Run, Actor, og.PlatformerController, Sounds], after: ["Director"] },
  (q, _time, world) => {
    const r = runnerRow(q);
    if (r === null) return;
    const running = r.a.column(Run).phase[r.i] === Phase.Running;
    const tune = r.a.column(Tuning);
    const ti = r.i;
    for (const { a, i } of couriers(q)) {
      const run = a.column(Run);
      const me = a.entity(i);
      const body = a.column(og.PlatformerController);
      const actor = a.column(Actor);
      // Director (a new round) and Collide (a crash's knock up) set the body's
      // vertical speed through Actor.vy and Run.kick; otherwise the body owns it.
      if (run.kick[i]) {
        run.kick[i] = 0;
        body.velocity[i * 3] = 0;
        body.velocity[i * 3 + 1] = actor.vy[i]!;
        run.grounded[i] = actor.vy[i]! > 0 ? 0 : 1;
      } else {
        actor.vy[i] = body.velocity[i * 3 + 1]!;
        run.grounded[i] = body.grounded[i]!;
      }
      const racing = running && !run.out[i];
      for (const e of world.events(og.CharacterEvent)) {
        if (e.entity !== me) continue;
        if (e.kind === CHARACTER_JUMPED) {
          og.tween.squash(world, { entity: me, impact: FEEL.takeOff, duration: 0.22 });
          ask(world, Sound.Jump);
        } else if (e.kind === CHARACTER_LANDED && racing) {
          og.tween.squash(world, { entity: me, impact: FEEL.landing, duration: 0.2 });
        }
      }
      const down = held(run.buttons[i]!, "jump");
      if (!down) run.startHeld[i] = 0;
      body.jump[i] = racing && down && !run.startHeld[i] ? 1 : 0;
      body.jumpHeight[i] = tune.jumpHeight[ti]!;
      body.timeToApex[i] = tune.timeToApex[ti]!;
      body.jumpCut[i] = tune.jumpCut[ti]!;
      body.apexGravity[i] = tune.apexGravity[ti]!;
      body.fallGravity[i] = tune.fallGravity[ti]!;
    }
  },
);

/**
 * The world scrolls left at the run speed times each row's parallax;
 * scenery wraps, pooled rows retire behind the left edge. A courier that
 * crashed while another runs has stopped: the roofs carry it off to the left.
 */
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
      const out = a.column(Run).out;
      const p = a.column(og.Transform).position;
      for (let i = 0; i < a.count; i++) {
        const k = actor.kind[i];
        if (k === Kind.Runner) {
          if (out[i] && p[i * 3]! > RETIRE_X) p[i * 3] = p[i * 3]! - step;
          continue;
        }
        if (actor.parallax[i] === 0) continue;
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
    const distance = run.course[i]!;
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

/** Whether this peer plays in a running lockstep room. */
function roomRunning(world: World): boolean {
  const lock = world.resource(og.Lockstep);
  return lock.ok && lock.value.running;
}

/** A crash: hit stop, a shake and a kick, the courier knocked back (in a race the roofs carry it instead) and up, flashing red. */
function crashFeel(world: World, runner: Entity, racing: boolean): void {
  og.time.hitStop(world, FEEL.hitStop);
  og.camera.shake(world, FEEL.shake);
  og.camera.kick(world, { x: -1, y: 0, z: 0 }, 0.3);
  if (!racing) og.tween.to(world, { entity: runner, field: "position.x", to: [-FEEL.knockBack], relative: true, duration: 0.3, ease: "quartOut" });
  // The red holds through the hit stop, then fades.
  og.tween.flash(world, { entity: runner, color: FEEL.crashFlash, amount: 0.85, duration: 0.4 });
}

/**
 * Overlap tests against each running courier's hit box: a coin is the
 * first courier's to touch it, a crate or a fence stops that courier (its
 * distance kept). The round is over once no courier runs; alone, the score
 * is the player's record.
 */
export const Collide = defineSystem(
  "Collide",
  { reads: [og.Transform, Tuning, og.Lockstep], writes: [Run, Actor, Sounds, Race], after: ["Spawn"] },
  (q, _time, world) => {
    const r = runnerRow(q);
    if (r === null) return;
    const run = r.a.column(Run);
    const ri = r.i;
    if (run.phase[ri] !== Phase.Running) return;
    const value = r.a.column(Tuning).coinValue[ri]!;
    const riders = couriers(q);
    const race = world.resource(Race);
    const racing = race.ok && race.value.racing;
    for (const c of riders) {
      const cr = c.a.column(Run);
      const ci = c.i;
      if (cr.out[ci]) continue;
      const rp = c.a.column(og.Transform).position;
      const ra = c.a.column(Actor);
      const rx = rp[ci * 3]!, ry = rp[ci * 3 + 1]! + RUNNER_SIZE.y / 2, rw = ra.w[ci]! / 2, rh = ra.h[ci]! / 2;
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
            cr.coins[ci] = cr.coins[ci]! + 1;
            coinPop(world, a.entity(i), p[i * 3]!, p[i * 3 + 1]!, value);
            ask(world, Sound.Coin);
          } else {
            crashed = true;
          }
        }
      }
      cr.score[ci] = Math.floor(cr.distance[ci]!) + cr.coins[ci]! * value;
      if (!crashed) continue;
      cr.out[ci] = 1;
      cr.grounded[ci] = 0;
      ra.vy[ci] = FEEL.knockUp;
      cr.kick[ci] = 1;
      crashFeel(world, c.a.entity(ci), racing);
      ask(world, Sound.Crash);
      if (cr.seat[ci] === 1 && race.ok) race.value.out = true;
    }
    if (riders.some((c) => !c.a.column(Run).out[c.i])) return;
    run.phase[ri] = Phase.Over;
    run.phaseTicks[ri] = 0;
    if (racing) {
      // A race: the card names the courier who ran farther.
      const d0 = run.distance[ri]!, d1 = race.value.distance;
      race.value.winner = d0 > d1 ? 0 : d1 > d0 ? 1 : 2;
      return;
    }
    const score = run.score[ri]!;
    // In a room every peer runs this round: the records are the player's own, kept on their own device.
    const own = !roomRunning(world);
    if (score > run.best[ri]!) {
      run.best[ri] = score;
      // The best survives a reload: the player's store (the Player's save, or this browser's).
      if (own) og.platform.storeSet(world, { key: "best", value: score });
    }
    if (!own) return;
    og.platform.leaderboardSubmit(world, { board: "main", score });
    og.platform.telemetry(world, "round.over", { score, coins: run.coins[ri]!, distance: Math.round(run.distance[ri]!) });
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
    if (r === null || r.a.column(Run).phase[r.i] !== Phase.Running) return;
    for (const c of couriers(q)) {
      const run = c.a.column(Run);
      if (!run.out[c.i] && run.grounded[c.i]) puffs(q, world, c.a.entity(c.i), c.a.column(og.Transform).position[c.i * 3]!);
    }
  },
);

/** A dust puff from a courier's feet on each footfall it made this tick. */
function puffs(q: Query, world: World, runner: Entity, rx: number): void {
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
}

/** Each courier's clip from the round and its body (idle, run, jump, fall, crash), the run clip's pace from the speed, and the afterimage near top speed. */
export const Animate = defineSystem(
  "Animate",
  { reads: [Run, Actor, Tuning], writes: [og.Flipbook, og.Afterimage], after: ["Dust"] },
  (q) => {
    const r = runnerRow(q);
    if (r === null) return;
    const phase = r.a.column(Run).phase[r.i];
    const speed = r.a.column(Run).speed[r.i]!;
    const speedMax = r.a.column(Tuning).speedMax[r.i]!;
    for (const { a, i } of couriers(q)) {
      const run = a.column(Run);
      const actor = a.column(Actor);
      const flip = a.column(og.Flipbook);
      const ghost = a.column(og.Afterimage);
      const racing = phase === Phase.Running && !run.out[i];
      let clip: number = COURIER.idle;
      if (!run.joined[i]) clip = COURIER.idle;
      else if (phase === Phase.Over || run.out[i]) clip = COURIER.crash;
      else if (racing) clip = run.grounded[i] ? COURIER.run : actor.vy[i]! > 0 ? COURIER.jump : COURIER.fall;
      flip.clip[i] = clip;
      flip.speed[i] = clip === COURIER.run ? Math.min(1.6, Math.max(0.8, speed / 7)) : 1;
      ghost.active[i] = racing && speed >= speedMax - FEEL.afterimageBelowMax ? 1 : 0;
    }
  },
);

/** A rough advance for centring: Latin at 0.56 em, CJK at 1 em (the API has no text measure yet). */
function width(text: string, size: number): number {
  let w = 0;
  for (const c of text) w += c.charCodeAt(0) > 0x2e80 ? 1 : c === " " ? 0.3 : 0.56;
  return w * size;
}

/** The HUD: score, coins and best while running (in a race, each courier's metres); the title card and the game-over card. */
export const HudText = defineSystem("Hud", { reads: [Run, Hud, Tuning, Race, og.Lockstep], after: ["Animate"] }, (q, _time, world) => {
  const race = world.resource(Race);
  const lock = world.resource(og.Lockstep);
  // In a room, the courier this peer plays.
  const you = ONLINE && lock.ok && lock.value.running ? lock.value.localPlayer : -1;
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
      // Two seats: how the second joins the race, and that it has.
      const join = (y: number) =>
        centred(race.ok && race.value.joined ? "P2 is in: the farther courier wins" : LOCAL ? "P2: press Enter or Up to join the race" : "P2: press Jump to join the race", y, 0.42, RIVAL);
      const phase = run.phase[i];
      if (phase === Phase.Title) {
        centred("ROOFTOP RUN", top - 2, 1.3, GOLD);
        centred("屋顶快跑", top - 3, 0.6, WHITE);
        if (Math.floor(run.phaseTicks[i]! / 30) % 2 === 0) centred("Tap or press Space to run", top - 3.8, 0.45, MINT);
        centred("点击或按空格开始", top - 4.4, 0.4, MINT);
        if (SEATS > 1) join(top - 5.1);
        continue;
      }
      if (race.ok && race.value.racing) {
        // A race: each courier's metres (OUT once it crashed), and the winner on the card.
        const d = [run.distance[i]!, race.value.distance];
        const out = [run.out[i] === 1, race.value.out];
        // The tag in its courier's ink, then the metres in white past a clear gap: "P1  5 m", never "P15 m".
        const line = (seat: number, y: number) => {
          const tag = `P${seat + 1}`;
          text(tag, left + 0.4, y, 0.55, seat === 0 ? MINT : RIVAL);
          text(`${Math.floor(d[seat]!)} m${out[seat] ? "  OUT" : ""}${seat === you ? "  (YOU)" : ""}`, left + 0.4 + width(tag, 0.55) + 0.45, y, 0.55, WHITE);
        };
        line(0, top - 0.8);
        line(1, top - 1.5);
        if (phase === Phase.Running && run.phaseTicks[i]! < 240) {
          centred("Tap: hop   Hold: high jump", top - 2.4, 0.45, MINT);
          centred("点按小跳 · 按住高跳", top - 3, 0.4, MINT);
        }
        if (phase === Phase.Over) {
          const w = race.value.winner;
          centred(w === 2 ? "DRAW" : `P${w + 1} WINS`, top - 2, 1.1, w === 1 ? RIVAL : GOLD);
          centred(`P1 ${Math.floor(d[0]!)} m   P2 ${Math.floor(d[1]!)} m`, top - 2.9, 0.55, WHITE);
          if (run.phaseTicks[i]! >= 30) centred("Tap or press Space to run again · 再来一次", top - 3.7, 0.42, MINT);
        }
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
        if (SEATS > 1) join(top - 4.4);
      }
    }
  }
});

const WHITE = { r: 1, g: 1, b: 1 };
const MINT = { r: 0.3, g: 0.85, b: 0.75 };
const CORAL = { r: 1, g: 0.25, b: 0.18 };
/** The second courier's ink (its palette's blue, sprites/courier-p2), for its HUD lines. */
export const RIVAL = { r: 0.55, g: 0.8, b: 1 };

/** The best score from the player's store: asked for on the first tick, taken when the answer arrives (outside a running room). */
export const Records = defineSystem("Records", { reads: [Actor, og.Lockstep], writes: [Run] }, (q, time, world) => {
  const r = runnerRow(q);
  // In a running room the world is the room's (a peer's own store would split it).
  if (r === null || roomRunning(world)) return;
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
