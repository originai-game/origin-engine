// The game entry: its default export is og.game(...). `og run .` runs it
// headless with the probe; `og dev .` reloads it on every save; `og build .`
// bundles it for the play page.
//
// This is the hello of docs/design.md §3.2 (Health, Velocity, Move) grown
// into a small game. scenes/main.ogt (cooked from assets/scenes/main.ogt)
// spawns the ground, the dropship and the trickster before the first tick;
// `start` gives the ground its mesh and collider, holds a crate above it,
// puts the camera, the UI font and the string tables (assets/strings/) in
// place and gives every seat its hero (src/play.ts). The menu panel's
// Start button (or S, Enter, a pad's A) lets the crate fall, spins the
// dropship up as it lifts off, and hands the trickster to the player: walk
// it, run and jump it through the yard in the view src/creation.ts names
// (src/view.ts), gather the five power cells and escape from the pad under
// the dropship before the guard grabs you three times.
//
// A guard walks the yard: a state machine in assets/ai/guard.ogt (og.ai)
// patrols its post on the navmesh baked from assets/nav/yard.ogt (og.nav),
// sees the nearest hero only inside its view cone and not through the wall,
// raises the alarm and chases it, then searches and goes back.
//
// The intro cutscene (assets/timelines/intro.ogt, `timeline:play:intro`)
// plays over the title. The frame's look is data: GameDesc.look names a
// file under assets/looks/ that extends a preset of the style catalogue,
// applied before start runs.

import { og, defineSystem, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Alarm, Health, Run, Spin, Velocity } from "./components.ts";
import { Hud, LANGUAGES, prefs, setLanguage } from "./hud.ts";
import { CELLS, Flow, Hands, heroName, heroOf, HeroMove, Lift, missionOf, must, spawnHeroes, Voices, Yard } from "./play.ts";
import { Seats, SEATS, setBot, setupSeats } from "./seats.ts";
import { ViewRig } from "./view.ts";

export { Alarm, Health, Run, Spin, Velocity } from "./components.ts";

export const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const v = a.column(Velocity).v;
    for (let i = 0; i < a.count; i++) {
      p[i * 3] += v[i * 3] * time.dt;
      p[i * 3 + 1] += v[i * 3 + 1] * time.dt;
      p[i * 3 + 2] += v[i * 3 + 2] * time.dt;
    }
  }
});

/** Once started, every Spin turns at its target speed. */
export const StartSpin = defineSystem("StartSpin", { reads: [Run], writes: [Spin], after: ["Flow"] }, (q, _time, world) => {
  const run = world.resource(Run);
  if (!run.ok || !run.value.started) return;
  for (const a of q.archetypes()) {
    const s = a.column(Spin);
    for (let k = 0; k < a.count; k++) if (s.speed[k] === 0) s.speed[k] = s.target[k]!;
  }
});

/** Once started, held bodies (gravityScale 0) fall. */
export const Drop = defineSystem("Drop", { reads: [Run], writes: [og.RigidBody], after: ["Flow"] }, (q, _time, world) => {
  const run = world.resource(Run);
  if (!run.ok || !run.value.started) return;
  for (const a of q.archetypes()) {
    const g = a.column(og.RigidBody).gravityScale;
    for (let k = 0; k < a.count; k++) if (g[k] === 0) g[k] = 1;
  }
});

/** Once started, the crate's blip plays once (an AudioSource inserted held, released here). */
export const Chime = defineSystem("Chime", { writes: [Run, og.AudioSource], after: ["Flow"] }, (q, _time, world) => {
  const run = world.resource(Run);
  if (!run.ok || !run.value.started || run.value.chimed) return;
  run.value.chimed = true;
  for (const a of q.archetypes()) a.column(og.AudioSource).playing.fill(1);
});

/** Rotates by Spin.speed about +Y: q' = q * (axis-angle about Y). */
export const Rotate = defineSystem("Rotate", { reads: [Spin], writes: [og.Transform], after: ["StartSpin"] }, (q, time) => {
  for (const a of q.archetypes()) {
    const r = a.column(og.Transform).quaternion;
    const s = a.column(Spin).speed;
    for (let i = 0; i < a.count; i++) {
      const half = (s[i] * time.dt) / 2;
      // Small-angle rotation, renormalised: deterministic arithmetic only.
      const sy = half - (half * half * half) / 6;
      const cw = 1 - (half * half) / 2;
      const x = r[i * 4], y = r[i * 4 + 1], z = r[i * 4 + 2], w = r[i * 4 + 3];
      const nx = x * cw - z * sy, ny = y * cw + w * sy, nz = z * cw + x * sy, nw = w * cw - y * sy;
      const n = Math.sqrt(nx * nx + ny * ny + nz * nz + nw * nw);
      r[i * 4] = nx / n;
      r[i * 4 + 1] = ny / n;
      r[i * 4 + 2] = nz / n;
      r[i * 4 + 3] = nw / n;
    }
  }
});

function spawnWave(world: World, count: number): void {
  const ship = og.load.gltf(world, "models/dropship.glb");
  if (!ship.ok) throw new Error(ship.reason);
  for (let i = 0; i < count; i++) {
    const e = world.instantiate(ship.value, { name: `wave-${world.tick}-${i}`, position: { x: -4 + i * 1.5, y: 4, z: -3 } });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, Velocity, { v: { x: 0, y: -0.5, z: 0 } });
  }
}

export default og.game({
  tick: 60,
  seed: 1,
  scenes: { main: "scenes/main.ogt" },
  // The art direction as data (og-taste §12): applied after the scene
  // spawns and before start, which may still change any of it.
  look: "looks/origin-game.ogt",
  resources: [Run],
  // Seats first (every seat's input for the tick), then the run's flow and
  // its heroes, the view rig before the engine's camera controllers, the
  // yard's rules, the screens, and the hello's systems.
  systems: [Seats, Flow, HeroMove, ViewRig, ...og.systems.cameras, Hands, Yard, Voices, Hud, Move, StartSpin, Drop, Chime, Rotate, Lift],
  // MSAA keeps world-space sprites and text sharp; TAA would blur them across
  // frames. The UI draws over the finished frame either way.
  render: { antialias: "msaa4" },
  start(world) {
    // Lockstep for an online game and every local seat's actions (src/seats.ts).
    setupSeats(world);
    // The camera looks down -Z at the scene from in front.
    const camera = must(world.resource(og.Camera));
    camera.position = { x: 0.4, y: 1.8, z: 8 };
    camera.quaternion = { x: 0, y: 0, z: 0, w: 1 };
    // The title is the same scene in every view: a perspective camera at 50°,
    // whatever projection the look's ArtCamera names; Start gives the camera
    // its view's projection (setupView, src/view.ts).
    camera.orthographic = false;
    camera.fov = 50;
    must(world.resource(og.RenderSettings)).sky = true;
    // A body that has rested for half a second sleeps until something touches
    // it, and costs nothing meanwhile (og.Physics.sleepTicks, 0 to keep every body awake).
    must(world.resource(og.Physics)).sleepTicks = 30;

    // The ground: a 30 m slab whose top is y = 0, drawn and collided with.
    const ground = world.find("ground");
    if (ground !== null) {
      const slab = must(og.render.boxGeometry(world, { width: 30, height: 0.2, depth: 30 }));
      const stone = must(og.render.standardMaterial(world, { color: { r: 0.18, g: 0.2, b: 0.24 }, roughness: 0.95 }));
      world.set(ground, og.Transform, { position: { x: 0, y: -0.1, z: 0 } });
      world.insert(ground, og.Mesh, { geometry: slab, material: stone, receiveShadow: true });
      world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 30, y: 0.2, z: 30 } });
    }

    // The crate: a dynamic body held in the air (gravityScale 0) until Start.
    const cube = must(og.render.boxGeometry(world, {}));
    const amber = must(og.render.standardMaterial(world, { color: { r: 0.9, g: 0.55, b: 0.08 }, roughness: 0.6 }));
    const crate = must(world.spawn({ name: "crate", position: { x: -2.4, y: 4, z: 0.5 } }));
    world.insert(crate, og.Mesh, { geometry: cube, material: amber, castShadow: true, receiveShadow: true });
    world.insert(crate, og.RigidBody, { gravityScale: 0 });
    world.insert(crate, og.Collider, { shape: SHAPE_BOX });
    const blip = og.load.sound(world, "audio/blip.wav");
    if (blip.ok) world.insert(crate, og.AudioSource, { sound: blip.value, playing: false, spatial: true });

    const trickster = world.find("trickster");
    if (trickster !== null) {
      world.insert(trickster, Health);
      // What the guard looks for: team 1, looked at 1 m above its feet.
      world.insert(trickster, og.Perceivable, { team: 1, eyeHeight: 1 });
    }
    // The dropship spins up when the run starts (StartSpin), as it lifts off the pad.
    const dropship = world.find("dropship");
    if (dropship !== null) world.insert(dropship, Spin);
    // Seat 0's hero is the trickster; with two seats a second walker stands
    // apart from it. Their capsules, clips and cameras wait for Start.
    spawnHeroes(world);

    // The wall the navmesh was baked around (assets/nav/yard.ogt): drawn, and solid to the guard's sight.
    const wall = must(world.spawn({ name: "wall", position: { x: -3, y: 1, z: -2 } }));
    const slabGeo = must(og.render.boxGeometry(world, { width: 0.6, height: 2, depth: 6 }));
    const plaster = must(og.render.standardMaterial(world, { color: { r: 0.55, g: 0.52, b: 0.48 }, roughness: 0.9 }));
    world.insert(wall, og.Mesh, { geometry: slabGeo, material: plaster, castShadow: true, receiveShadow: true });
    world.insert(wall, og.Collider, { shape: SHAPE_BOX, size: { x: 0.6, y: 2, z: 6 } });

    // The guard: the walker model, moved by og.nav on the baked navmesh and run by og.ai's machine.
    const yard = must(og.nav.load(world, "nav/yard.ogt"));
    const mind = must(og.ai.load(world, "ai/guard.ogt"));
    const walker = must(og.load.gltf(world, "models/walker.glb"));
    const guard = must(world.instantiate(walker, { name: "guard", position: { x: -6, y: 0, z: -3 } }));
    // Facing east (+X), towards the wall: its view cone is 120° around the way it faces until it moves.
    world.set(guard, og.Transform, { quaternion: { x: 0, y: Math.SQRT1_2, z: 0, w: Math.SQRT1_2 } });
    world.insert(guard, og.NavAgent, { nav: yard.mesh, speed: 1.5, acceleration: 12, radius: 0.4, faceMovement: true, turnSpeed: 8 });
    // og.Crowd: ORCA avoidance, so the guards you add pass each other and keep off the navmesh's edges (og-ai §2).
    world.insert(guard, og.Crowd, {});
    world.insert(guard, og.AiAgent, { behaviour: mind.behaviour });
    world.insert(guard, og.Perception, { sight: 9, fov: 120, eyeHeight: 1.6, team: 2 });
    world.insert(guard, Alarm);
    // Every string the screens show (og-i18n §1): each locale's table, the base (English) first.
    for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
    setLanguage(world, 0);
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    // A press answers with a sound as well as its look (og-taste §6): the `click` preset, synthesised by og cook.
    const click = og.load.sound(world, "audio/click.ogt");
    og.ui.configure(world, { ...(font.ok ? { font: font.value } : {}), ...(click.ok ? { sounds: { press: click.value } } : {}) });
  },
  commands: {
    spawn_wave: (world, args) => spawnWave(world, Number(args[0] ?? 3)),
    // The guard's machine raises it on entering `alert`; og.ai passes the guard's handle first.
    alarm: (world, args) => {
      const guard = Number(args[0]) as never;
      const a = must(world.get(guard, Alarm));
      must(world.set(guard, Alarm, { raised: a.raised + 1 }) as never);
    },
    /** `lure:<x>:<z>`: moves the trickster on the ground. */
    lure: (world, args) => {
      const t = world.find("trickster");
      if (t === null) throw new Error("no trickster");
      must(world.set(t, og.Transform, { position: { x: Number(args[0]), y: 0, z: Number(args[1]) } }) as never);
    },
    /** `warp:<seat>:<x>:<z>`: puts a seat's hero on the ground. */
    warp: (world, args) => {
      const seat = num(args[0], "seat");
      if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seats are 0 to ${SEATS - 1}`);
      must(world.set(heroOf(world, seat), og.Transform, { position: { x: num(args[1], "x"), y: 0, z: num(args[2], "z") } }));
    },
    /** `bot:<n>`: a scripted player drives this device's seats from the tick (0 hands the input back). */
    bot: (_world, args) => setBot(num(args[0], "n")),
    /** `health:<seat>:<hp>`: sets a hero's Health (0 at the next grab ends the run). */
    health: (world, args) => {
      must(world.set(heroOf(world, num(args[0], "seat")), Health, { hp: num(args[1], "hp") }));
    },
    /** `viewport:<desktop|phone>`: lays the UI out for a phone in portrait (and shows the touch controls). */
    viewport: (world, args) => {
      const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
      og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
    },
    /** `language:<en|zh-CN>`: as the settings do. */
    language: (world, args) => {
      const at = LANGUAGES.findIndex((l) => l.code === args[0]);
      if (at < 0) throw new Error("write language:<en|zh-CN>");
      setLanguage(world, at);
    },
  },
  commandHelp: {
    spawn_wave: { usage: "[count]", doc: "Drop that many dropships (3 by default)." },
    alarm: { usage: "<guard>", doc: "The guard's state machine raises it on entering alert; og.ai passes the guard's handle." },
    lure: { usage: "<x>:<z>", doc: "Move the trickster on the ground." },
    warp: { usage: "<seat>:<x>:<z>", doc: "Put a seat's hero on the ground at x, z (seat 0 is the trickster)." },
    bot: { usage: "<n>", doc: "A scripted player drives this device's seats from the tick (0 hands the input back): two peers with bots play the same run." },
    health: { usage: "<seat>:<hp>", doc: "Set a hero's Health (out of 100; a grab costs 34)." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×, the touch controls shown)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
  },
  facts: (world) => {
    const run = world.resource(Run);
    const dropship = world.find("dropship");
    const spin = dropship === null ? null : world.get(dropship, Spin);
    const crate = world.find("crate");
    const at = crate === null ? null : world.get(crate, og.Transform);
    const guard = world.find("guard");
    const mind = guard === null ? null : (og.ai.describe(world, guard) as { state: string } | null);
    const m = missionOf(world);
    const r = (n: number) => Math.round(n * 100) / 100;
    const seats: Record<string, number> = {};
    for (let seat = 0; seat < SEATS; seat++) {
      const hero = world.find(heroName(seat));
      const p = hero === null ? null : world.get(hero, og.Transform);
      const hp = hero === null ? null : world.get(hero, Health);
      if (p?.ok) Object.assign(seats, { [`x${seat}`]: r(p.value.position.x), [`y${seat}`]: r(p.value.position.y), [`z${seat}`]: r(p.value.position.z) });
      if (hp?.ok) seats[`hp${seat}`] = hp.value.hp;
    }
    return {
      started: run.ok && run.value.started,
      guard: mind?.state ?? null,
      spin: spin?.ok ? spin.value.speed : null,
      crateY: at?.ok ? at.value.position.y : null,
      phase: ["title", "play", "over"][m.phase] ?? null,
      ending: ["none", "escaped", "caught"][m.ending] ?? null,
      paused: m.paused,
      cells: m.cells,
      total: CELLS,
      time: Math.round(m.time * 10) / 10,
      catches: m.catches,
      jumps: m.jumps,
      runs: m.runs,
      wins: m.wins,
      seats: SEATS,
      language: LANGUAGES[prefs.language]!.code,
      ...seats,
    };
  },
});

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}
