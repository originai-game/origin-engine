// The run: what Start hands the player. The trickster becomes seat 0's
// hero (og.CharacterController, its idle, walk and run clips) and, with two
// seats, a second walker in a cooler cloth stands beside it; the camera
// follows each in the view src/creation.ts names (src/view.ts). Five power
// cells pop up around the fenced yard: bring them all to the dropship, which
// lifts off and spins over the pad at the yard's middle, then stand on the
// lit pad to escape (a win). The guard hunts the nearest hero it sees; its
// grab costs a hero a third of its Health and puts it back by the pad, and a
// hero with none left loses the run. Then the summary, and again.
//
// Every hero is a seat: its og.PlayerInput (src/seats.ts) is all these
// systems read, so the yard plays alone, two on one device or two over a
// room in lockstep. The run's state is the dropship's Mission (the yard's
// objective); everything the run adds is spawned on Start, so the title and
// the intro are the scene they always were.

import { og, defineSystem, math, SHAPE_BOX, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Cell, Ending, Health, Hero, Mission, Phase, Rise, Run, Sound, Tuning, Voice } from "./components.ts";
import { creation } from "./creation.ts";
import { held, PAUSE, pressedNow, SEATS } from "./seats.ts";
import { bodyLayer, FIRST_PERSON, ISOMETRIC, SeatLook, setupView, walkDirection } from "./view.ts";

export const CELLS = 5;
/** Where the cells stand: spread round the yard, one behind the wall by the guard's post. */
const CELL_AT = [
  { x: -7.5, z: 6.5 },
  { x: 7, z: -6.5 },
  { x: -8.5, z: -6 },
  { x: 8, z: 6 },
  { x: 0.5, z: -9 },
] as const;
/** The guard's post: where a new run puts it back. */
const POST = { x: -6, z: -3 } as const;
/** Seat 1's start, out of the guard's sight on the title (tests/guard.og.ts). */
const SECOND_START = { x: 7.5, z: 0.5 } as const;
/** Seat 1's cloth beside the trickster's. */
const SECOND_TINT = { r: 0.45, g: 0.75, b: 1 };
/** The fence: a square of low rails 12 m out, inside the navmesh's bounds. */
const FENCE = 12;
/** The dropship hovers this high over the pad once it has lifted. */
const LIFT = 3.6;
/** The capsule (radius 0.3, a 1.1 m segment): its centre this far above the feet. */
const WAIST = 0.85;
/** Ticks the summary shows before Play again answers. */
export const SUMMARY_READY = 60;
const UP = { x: 0, y: 1, z: 0 };
const DEG = Math.PI / 180;
/** The isometric view's heading (the look's ArtCamera yaw): the stick walks relative to the screen. */
const ISO_YAW = ((creation.camera as { yaw?: number }).yaw ?? 45) * DEG;

type MissionV = Value<typeof Mission>;
type TuneV = Value<typeof Tuning>;
type HeroV = Value<typeof Hero>;

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** sqrt(x² + y²), the same on every JavaScript engine (Math.hypot is not). */
const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);

function wrap(a: number): number {
  while (a > Math.PI) a -= 2 * Math.PI;
  while (a < -Math.PI) a += 2 * Math.PI;
  return a;
}

function turn(from: number, to: number, step: number): number {
  const d = wrap(to - from);
  return wrap(from + Math.max(-step, Math.min(step, d)));
}

/** Seat `seat`'s hero: the trickster, then `trickster-1`. */
export function heroName(seat: number): string {
  return seat === 0 ? "trickster" : `trickster-${seat}`;
}

export function heroOf(world: World, seat: number): Entity {
  const e = world.find(heroName(seat));
  if (e === null) throw new Error(`${heroName(seat)} is missing`);
  return e;
}

/** The dropship, which carries the run. */
export function ship(world: World): Entity {
  const e = world.find("dropship");
  if (e === null) throw new Error("the dropship is missing");
  return e;
}

export function missionOf(world: World): MissionV {
  return must(world.get(ship(world), Mission));
}

/**
 * In start: every seat's hero and its og.PlayerInput (the trickster is seat
 * 0's), and the run's state on the dropship. Nothing here is drawn or moved
 * until Start (beginRun).
 */
export function spawnHeroes(world: World): void {
  const dropship = ship(world);
  must(world.insert(dropship, Mission));
  must(world.insert(dropship, Tuning));
  const walker = must(og.load.gltf(world, "models/walker.glb"));
  for (let seat = 0; seat < SEATS; seat++) {
    let hero = world.find(heroName(seat));
    const at = seat === 0 ? { x: 2, z: 0 } : SECOND_START;
    if (hero === null) {
      hero = must(world.instantiate(walker, { name: heroName(seat), position: { x: at.x, y: 0, z: at.z } }));
      must(world.set(hero, og.Transform, { quaternion: { x: 0, y: Math.SQRT1_2, z: 0, w: Math.SQRT1_2 } }));
      must(world.set(hero, og.Model, { tint: SECOND_TINT }));
      must(world.insert(hero, Health));
    }
    must(world.insert(hero, og.PlayerInput, { player: seat }));
    must(world.insert(hero, Hero, { seat, facing: Math.PI / 2, startX: at.x, startZ: at.z }));
  }
}

/** Every hero, in seat order. */
function heroes(world: World): Entity[] {
  const out: Entity[] = [];
  for (let seat = 0; seat < SEATS; seat++) out.push(heroOf(world, seat));
  return out;
}

/**
 * The tick Start reaches the world: the heroes take their capsules, clips
 * and cameras, the yard its fence, cells, pad and voices, and the dropship
 * lifts off.
 */
function beginRun(world: World, m: MissionV): void {
  const anim = must(og.load.animation(world, "models/walker.glb"));
  const idle = og.anim.clip(anim, "idle");
  const all = heroes(world);
  for (const hero of all) {
    must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 1.1, offset: { x: 0, y: WAIST, z: 0 } }));
    must(world.insert(hero, og.CharacterController, { maxSlope: 45, stepHeight: 0.3, snapDistance: 0.3, coyoteTime: 0.12, pushBodies: true }));
    must(world.insert(hero, og.Skeleton, { asset: anim.skeleton }));
    must(world.insert(hero, og.AnimationPlayer, { clip: idle, loop: true, fade: 0.2 }));
    // The guard sees the nearest hero (og.Perception picks it) and hunts it; the trickster already is one.
    if (!world.get(hero, og.Perceivable).ok) must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 1 }));
  }
  // First person: each body on its seat's layer, which its own eye does not draw.
  if (FIRST_PERSON) all.forEach((hero, seat) => must(world.set(hero, og.Model, { layers: bodyLayer(seat) })));
  // Third person sits 4.5 m behind and 1.6 m above the hero's feet (raised as the look tips down).
  setupView(world, all, { behind: 4.5, above: 1.6 });
  // Each hero turns its back to the camera: forward is where the view looks (-Z).
  for (const hero of all) must(world.set(hero, Hero, { facing: Math.PI }));

  // The fence rises round the yard.
  const rail = must(og.render.standardMaterial(world, { color: { r: 0.36, g: 0.33, b: 0.3 }, roughness: 0.85 }));
  const long = must(og.render.boxGeometry(world, { width: FENCE * 2 + 0.4, height: 0.7, depth: 0.4 }));
  const side = must(og.render.boxGeometry(world, { width: 0.4, height: 0.7, depth: FENCE * 2 + 0.4 }));
  const fences = [
    { x: 0, z: -FENCE, geo: long, size: { x: FENCE * 2 + 0.4, y: 0.7, z: 0.4 } },
    { x: 0, z: FENCE, geo: long, size: { x: FENCE * 2 + 0.4, y: 0.7, z: 0.4 } },
    { x: -FENCE, z: 0, geo: side, size: { x: 0.4, y: 0.7, z: FENCE * 2 + 0.4 } },
    { x: FENCE, z: 0, geo: side, size: { x: 0.4, y: 0.7, z: FENCE * 2 + 0.4 } },
  ];
  fences.forEach((f, i) => {
    const e = must(world.spawn({ name: `fence-${i}`, position: { x: f.x, y: -0.4, z: f.z } }));
    must(world.insert(e, og.Mesh, { geometry: f.geo, material: rail, castShadow: true, receiveShadow: true }));
    // Taller than it looks: a jump does not clear it.
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: f.size.x, y: 4, z: f.size.z }, offset: { x: 0, y: 1.6, z: 0 } }));
    must(world.insert(e, Rise, { from: -0.4, to: 0.35, seconds: 0.6 }));
  });

  // The pad under the dropship: dim until every cell is in.
  const ring = must(og.render.ringGeometry(world, { innerRadius: 1.5, outerRadius: 1.75, thetaSegments: 48 }));
  const padMat = must(og.render.standardMaterial(world, { color: { r: 0.12, g: 0.32, b: 0.36 }, emissive: { r: 0.1, g: 0.75, b: 0.85 }, emissiveIntensity: 0.6, roughness: 0.6 }));
  const pad = must(world.spawn({ name: "pad", position: { x: 0, y: 0.02, z: 0 }, quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2) }));
  must(world.insert(pad, og.Mesh, { geometry: ring, material: padMat, receiveShadow: true }));

  // The cells: glowing octahedra that pop up round the yard.
  const gem = must(og.render.octahedronGeometry(world, { radius: 0.32 }));
  const glow = must(og.render.standardMaterial(world, { color: { r: 1, g: 0.5, b: 0.05 }, emissive: { r: 1, g: 0.42, b: 0.02 }, emissiveIntensity: 0.9, roughness: 0.35 }));
  CELL_AT.forEach((c, i) => {
    const e = must(world.spawn({ name: `cell-${i}`, position: { x: c.x, y: 0.9, z: c.z }, scale: { x: 0, y: 0, z: 0 } }));
    must(world.insert(e, og.Mesh, { geometry: gem, material: glow, castShadow: true }));
    must(world.insert(e, Cell, { x: c.x, z: c.z }));
  });

  spawnVoices(world);
  // The dropship lifts off the pad, clear of the heroes' heads.
  const dropship = ship(world);
  must(world.insert(dropship, Rise, { from: must(world.get(dropship, og.Transform)).position.y, to: LIFT, seconds: 2 }));
  m.phase = Phase.Play;
  m.sfx |= Sound.Ready;
  m.runs += 1;
}

/** A new run after the summary: every hero home, every cell back, the guard to its post. */
function resetRun(world: World, m: MissionV): void {
  for (const hero of heroes(world)) {
    const me = must(world.get(hero, Hero));
    must(world.set(hero, og.Transform, { position: { x: me.startX, y: 0, z: me.startZ } }));
    must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
    const hp = must(world.get(hero, Health));
    must(world.set(hero, Health, { hp: hp.max }));
    must(world.set(hero, Hero, { ...me, invulnerable: 0, airTime: 0 }));
  }
  for (let i = 0; i < CELLS; i++) {
    const e = world.find(`cell-${i}`);
    if (e !== null) {
      must(world.set(e, Cell, { taken: false, pop: 0 }));
      must(world.set(e, og.Mesh, { visible: true }));
    }
  }
  const guard = world.find("guard");
  if (guard !== null) {
    must(world.set(guard, og.Transform, { position: { x: POST.x, y: 0, z: POST.z } }));
    must(og.ai.restart(world, guard));
  }
  m.phase = Phase.Play;
  m.ending = Ending.None;
  m.cells = 0;
  m.time = 0;
  m.catches = 0;
  m.jumps = 0;
  m.overTicks = 0;
  m.paused = false;
  m.runs += 1;
  m.sfx |= Sound.Ready;
}

/**
 * The run's flow, from the seats' buttons alone: Start (the menu's button,
 * S or Enter, a pad's A) begins the run on the title and again on the
 * summary; Pause (Escape, a pad's Start, the HUD's Pause) holds it.
 */
export const Flow = defineSystem(
  "Flow",
  {
    reads: [og.PlayerInput, og.Lockstep],
    // Start's tick inserts what the run plays on (beginRun), and a new run resets it.
    writes: [Run, Mission, Hero, Health, Cell, Rise, Voice, SeatLook, og.Transform, og.CharacterController, og.Collider, og.Skeleton, og.AnimationPlayer, og.Perceivable, og.FollowCamera, og.FirstPersonCamera, og.CameraView, og.Camera, og.Model, og.Mesh, og.AudioSource, og.AiAgent],
    after: ["Seats"],
  },
  (_q, _time, world) => {
    const run = world.resource(Run);
    const m = missionOf(world);
    let start = false;
    let pause = false;
    for (const hero of heroes(world)) {
      const me = must(world.get(hero, Hero));
      const input = must(world.get(hero, og.PlayerInput));
      if (pressedNow(input.buttons, me.before, "start")) start = true;
      if ((input.buttons & PAUSE) !== 0) pause = true;
    }
    if (m.phase === Phase.Title && start) {
      if (run.ok) run.value.started = true;
      beginRun(world, m);
    } else if (m.phase === Phase.Over) {
      m.overTicks += 1;
      if (start && m.overTicks >= SUMMARY_READY) resetRun(world, m);
    } else if (m.phase === Phase.Play && pause) {
      m.paused = !m.paused;
    }
    const hold = m.phase === Phase.Play && m.paused;
    must(og.time.setScale(world, hold ? 0 : 1));
    must(world.set(ship(world), Mission, m));
  },
);

/**
 * Per tick, for each hero: reads its og.PlayerInput, walks the capsule
 * along the view (the screen, isometric), runs and jumps it, turns the body
 * and picks its clip.
 */
export const HeroMove = defineSystem(
  "HeroMove",
  { reads: [og.PlayerInput, Tuning], writes: [Mission, Hero, SeatLook, og.CharacterController, og.Transform, og.AnimationPlayer, og.Model], after: ["Flow"] },
  (_q, time, world) => {
    const m = missionOf(world);
    for (const hero of heroes(world)) {
      const me = must(world.get(hero, Hero));
      const input = must(world.get(hero, og.PlayerInput));
      if (m.phase !== Phase.Title) step(world, m, must(world.get(ship(world), Tuning)), hero, me, input, time.scaledDt);
      me.before = input.buttons;
      must(world.set(hero, Hero, me));
    }
    must(world.set(ship(world), Mission, m));
  },
);

let clips: { idle: number; walk: number; run: number } | null = null;

function step(world: World, m: MissionV, t: TuneV, hero: Entity, me: HeroV, input: Value<typeof og.PlayerInput>, dt: number): void {
  if (dt === 0) return;
  const look = must(world.get(hero, SeatLook));
  const cc = must(world.get(hero, og.CharacterController));
  const playing = m.phase === Phase.Play && m.ending === Ending.None;
  const mag = Math.min(1, hypot(input.x, input.y));
  const dir = walkDirection(ISOMETRIC ? ISO_YAW : look.yaw, input.x, input.y);
  const n = hypot(dir.x, dir.z);
  const running = held(input.buttons, "run");
  let speed = 0;
  if (playing && mag > 0.05 && n > 1e-4) speed = running ? t.runSpeed : t.walkSpeed * Math.min(1, mag / 0.85);
  const dx = n > 1e-4 ? dir.x / n : 0;
  const dz = n > 1e-4 ? dir.z / n : 0;
  const jump = playing && held(input.buttons, "jump");
  must(world.set(hero, og.CharacterController, { move: { x: dx * speed, y: 0, z: dz * speed }, jump, jumpSpeed: t.jumpSpeed }));

  // The body: the look's heading in first person; else where it walks, and
  // isometric, where the look turns it while it stands.
  if (FIRST_PERSON) me.facing = wrap(look.yaw + Math.PI);
  else if (speed > 0) me.facing = turn(me.facing, math.atan2(dx, dz), 12 * dt);
  else if (ISOMETRIC && (input.aimX !== 0 || input.aimY !== 0)) me.facing = wrap(look.yaw + Math.PI);
  if (ISOMETRIC && speed > 0) must(world.set(hero, SeatLook, { yaw: wrap(me.facing - Math.PI) }));
  must(world.set(hero, og.Transform, { quaternion: og.transform.fromAxisAngle(UP, me.facing) }));

  // The clip: idle, walk or run by the speed across the ground.
  const v = cc.velocity;
  const hs = hypot(v.x, v.z);
  me.speed = hs;
  const want = hs > 3.6 ? 2 : hs > 0.3 ? 1 : 0;
  if (want !== me.clip || clips === null) {
    if (clips === null) {
      const anim = must(og.load.animation(world, "models/walker.glb"));
      clips = { idle: og.anim.clip(anim, "idle"), walk: og.anim.clip(anim, "walk"), run: og.anim.clip(anim, "run") };
    }
    me.clip = want;
    must(world.set(hero, og.AnimationPlayer, { clip: (want === 2 ? clips.run : want === 1 ? clips.walk : clips.idle) as never, loop: true }));
  }
  must(world.set(hero, og.AnimationPlayer, { speed: want === 0 ? 1 : Math.max(0.6, hs / (want === 2 ? 5 : 2.4)) }));

  // Steps, jumps and landings, heard.
  if (cc.grounded && hs > 0.5) {
    me.stepClock += hs * dt;
    const stride = hs > 3.6 ? 1.5 : 1.05;
    if (me.stepClock >= stride) {
      me.stepClock -= stride;
      m.sfx |= Sound.Step;
    }
  }
  if (!cc.grounded && me.grounded && v.y > 1) {
    m.jumps += 1;
    m.sfx |= Sound.Jump;
  }
  if (cc.grounded && me.airTime > 0.3) m.sfx |= Sound.Land;
  me.airTime = cc.grounded ? 0 : me.airTime + dt;
  me.grounded = cc.grounded;
  if (me.invulnerable > 0) {
    me.invulnerable = Math.max(0, me.invulnerable - dt);
    // Blinks while it cannot be grabbed again.
    must(world.set(hero, og.Model, { visible: me.invulnerable === 0 || Math.floor(me.invulnerable * 10) % 2 === 0 }));
  }
}

/**
 * The yard's rules each tick of a run: the clock, the cells a hero reaches,
 * the guard's grab, the lit pad's escape and the run's end.
 */
export const Yard = defineSystem(
  "Yard",
  { reads: [Tuning, og.Transform], writes: [Mission, Cell, Hero, Health, og.Mesh, og.Transform, og.Model, og.CameraView], after: ["HeroMove"] },
  (_q, time, world) => {
    const m = missionOf(world);
    const dt = time.scaledDt;
    // The cells pop up and bob (also behind the summary).
    for (let i = 0; i < CELLS; i++) {
      const e = world.find(`cell-${i}`);
      if (e === null) continue;
      const c = must(world.get(e, Cell));
      if (c.taken) continue;
      c.pop = Math.min(1, c.pop + dt * 2.5);
      const s = c.pop < 1 ? 1 - (1 - c.pop) * (1 - c.pop) * (1 - 2.4 * c.pop) : 1;
      const y = 0.9 + 0.12 * math.sin(time.elapsed * 2.4 + i);
      must(world.set(e, og.Transform, { position: { x: c.x, y, z: c.z }, quaternion: og.transform.fromAxisAngle(UP, time.elapsed * 1.6 + i), scale: { x: s, y: s, z: s } }));
      must(world.set(e, Cell, c));
    }
    if (m.phase !== Phase.Play || m.ending !== Ending.None || dt === 0) return;
    const t = must(world.get(ship(world), Tuning));
    m.time += dt;
    const guard = world.find("guard");
    const g = guard === null ? null : must(world.get(guard, og.Transform)).position;
    const alert = guard !== null && (og.ai.describe(world, guard) as { state: string } | null)?.state === "alert";
    for (const hero of heroes(world)) {
      const me = must(world.get(hero, Hero));
      const p = must(world.get(hero, og.Transform)).position;
      for (let i = 0; i < CELLS; i++) {
        const e = world.find(`cell-${i}`);
        if (e === null) continue;
        const c = must(world.get(e, Cell));
        if (c.taken || hypot(p.x - c.x, p.z - c.z) > 1 || p.y > 1.8) continue;
        must(world.set(e, Cell, { taken: true }));
        must(world.set(e, og.Mesh, { visible: false }));
        m.cells += 1;
        m.sfx |= Sound.Cell;
        og.camera.punch(world, 0.25);
      }
      if (g !== null && alert && me.invulnerable === 0 && hypot(p.x - g.x, p.z - g.z) < t.catchReach) {
        // The grab: a third of its Health, and back by the pad.
        const hp = must(world.get(hero, Health));
        const left = Math.max(0, hp.hp - t.grabDamage);
        must(world.set(hero, Health, { hp: left }));
        must(world.set(hero, og.Transform, { position: { x: me.startX, y: 0, z: me.startZ } }));
        must(world.set(hero, og.Model, { flash: 1 }));
        must(world.set(hero, Hero, { invulnerable: 2 }));
        m.catches += 1;
        m.sfx |= Sound.Hurt;
        og.camera.shake(world, 0.55);
        must(og.time.hitStop(world, 6));
        if (guard !== null) must(og.ai.restart(world, guard));
        if (left <= 0) end(m, Ending.Caught);
      }
      if (m.cells >= CELLS && m.ending === Ending.None && hypot(p.x, p.z) < t.padRadius) end(m, Ending.Escaped);
    }
    // The pad lights once every cell is in.
    const pad = world.find("pad");
    if (pad !== null) {
      const ready = m.cells >= CELLS;
      const pulse = ready ? 1 + 0.25 * math.sin(time.elapsed * 6) : 1;
      must(world.set(pad, og.Transform, { scale: { x: pulse, y: pulse, z: 1 } }));
      must(world.set(pad, og.Mesh, { flash: ready ? 0.65 : 0 }));
    }
    must(world.set(ship(world), Mission, m));
  },
);

function end(m: MissionV, ending: number): void {
  m.ending = ending;
  m.phase = Phase.Over;
  m.overTicks = 0;
  m.sfx |= ending === Ending.Escaped ? Sound.Win : Sound.Lose;
  if (ending === Ending.Escaped) {
    m.wins += 1;
    if (m.best === 0 || m.time < m.best) m.best = m.time;
  }
}

/** Eases every Rise's height (the fence, the dropship). */
export const Lift = defineSystem("Lift", { reads: [Rise], writes: [og.Transform, Rise] }, (q, time) => {
  for (const a of q.archetypes()) {
    const r = a.column(Rise);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      if (r.t[i]! >= 1) continue;
      r.t[i] = Math.min(1, r.t[i]! + time.dt / Math.max(0.01, r.seconds[i]!));
      const k = r.t[i]!;
      const e = k < 0.5 ? 4 * k * k * k : 1 - ((-2 * k + 2) ** 3) / 2;
      p[i * 3 + 1] = r.from[i]! + (r.to[i]! - r.from[i]!) * e;
    }
  }
});

function spawnVoices(world: World): void {
  const voices: [string, number, number][] = [
    ["audio/cell.ogt", Sound.Cell, 0.7],
    ["audio/jump.ogt", Sound.Jump, 0.45],
    ["audio/land.ogt", Sound.Land, 0.4],
    ["audio/step.ogt", Sound.Step, 0.25],
    ["audio/hurt.ogt", Sound.Hurt, 0.7],
    ["audio/sting.ogt", Sound.Win, 0.7],
    ["audio/lose.ogt", Sound.Lose, 0.7],
    ["audio/blip.wav", Sound.Ready, 0.5],
  ];
  for (const [path, bit, volume] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: must(og.load.sound(world, path)), playing: false, bus: "sfx", volume }));
    must(world.insert(v, Voice, { bit }));
  }
}

/** Plays each sound Mission.sfx asks for once (a voice still playing is stopped this tick and started the next). */
export const Voices = defineSystem("Voices", { writes: [Mission, Voice, og.AudioSource, og.Rng], after: ["Yard"] }, (q, _time, world) => {
  const m = missionOf(world);
  if (m.sfx === 0) return;
  const rng = must(world.resource(og.Rng));
  let left = m.sfx;
  for (const a of q.archetypes()) {
    const voice = a.column(Voice);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = voice.bit[i]!;
      if ((m.sfx & b) === 0) continue;
      if (src.playing[i]) {
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        voice.plays[i] = voice.plays[i]! + 1;
        if (b === Sound.Step) src.pitch[i] = rng.range(0.9, 1.1);
        left &= ~b;
      }
    }
  }
  must(world.set(ship(world), Mission, { sfx: left }));
});
