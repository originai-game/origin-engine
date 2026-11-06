// The sounds. Each of this device's cars has an engine loop pitched by its
// revs (rising through five gears, dropping at each shift) and loudened by
// the throttle, and a tyre loop faded in with the slide; each rival carries
// its engine loop on its own car, heard where it is (spatial). Every other
// sound is a one-shot voice played once when a system asks for its bit:
// the race's (Game.sfx: the countdown, contacts, the touch controls' taps)
// on every device, a seat's own (Car.sfx: its checkpoints, laps, best lap
// and finish) on the device that drives it. In a room the sources are
// world state every peer must hold alike, so each seat's engine, tyres and
// cues sound on its own car for every peer, heard through each device's
// camera (its own car the near one). "Redline" loops on the music bus,
// ducked under the effects; a limiter on the master.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { Car, Game, Phase, Sfx, Sound, Tuning } from "./components.ts";
import { CARS, tuning } from "./car.ts";
import { LOCAL, localSeats, ONLINE, SEATS } from "./seats.ts";
import { must } from "./world.ts";

const ONE_SHOTS: readonly [string, number, number][] = [
  ["audio/beep.wav", Sound.Beep, 0.6],
  ["audio/go.wav", Sound.Go, 0.7],
  ["audio/bump.wav", Sound.Bump, 0.55],
  ["audio/crash.wav", Sound.Crash, 0.75],
  ["audio/sector.wav", Sound.Sector, 0.45],
  ["audio/lap.wav", Sound.Lap, 0.6],
  ["audio/best.wav", Sound.Best, 0.7],
  ["audio/boost.wav", Sound.Boost, 0.55],
  ["audio/finish.wav", Sound.Finish, 0.75],
];

export function spawnVoices(world: World): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  for (const [path, bit, volume] of ONE_SHOTS) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority: 200 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const tap = must(world.spawn({ name: `sfx-${Sound.Tap}` }));
  must(world.insert(tap, og.AudioSource, { sound: sound("styles/redline/sfx-tap.ogt"), playing: false, bus: "ui", volume: 0.5, priority: 150 }));
  must(world.insert(tap, Sfx, { bit: Sound.Tap }));
  const engine = sound("audio/engine.wav");
  const tyres = sound("audio/skid.wav");
  CARS.forEach((car, i) => {
    const seat = i < SEATS;
    // Alone and on one device the seats' cars are heard as their drivers hear them, the rivals where they are.
    // In a room every peer's world holds the same sources (they are hashed): each seat's car is heard where it
    // is too, through this device's camera (the listener, each peer's own), so its own car is the loud one.
    const near = seat && !ONLINE;
    const parent = seat && ONLINE ? { parent: car } : {};
    const e = seat ? must(world.spawn({ name: `engine-${i}`, ...parent })) : car;
    must(world.insert(e, og.AudioSource, { sound: engine, bus: "sfx", loop: true, volume: seat ? 0.22 : 0.5, pitch: 0.7, priority: seat ? 250 : 120, fade: 0.05, spatial: !near, refDistance: seat ? 9 : 6, maxDistance: 90 }));
    if (seat) {
      const t = must(world.spawn({ name: `tyres-${i}`, ...parent }));
      must(world.insert(t, og.AudioSource, { sound: tyres, bus: "sfx", loop: true, volume: 0, priority: 240, fade: 0.08, spatial: !near, refDistance: 9, maxDistance: 60 }));
    }
    // A seat's own cues in a room: a voice of each on its car.
    if (seat && ONLINE) {
      for (const [path, bit, volume] of ONE_SHOTS) {
        if ((bit & SEAT_CUES) === 0) continue;
        const v = must(world.spawn({ name: `sfx-${bit}-${i}`, parent: car }));
        must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority: 200, spatial: true, refDistance: 12, maxDistance: 80 }));
        must(world.insert(v, Sfx, { bit }));
      }
    }
  });
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/redline.wav"), bus: "music", loop: true, volume: 0.45, priority: 255 }));
  must(world.insert(must(world.spawn({ name: "music-bus" })), og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 4, duckRelease: 0.5 }));
  must(world.insert(must(world.spawn({ name: "master-bus" })), og.AudioBusFx, { bus: "master", threshold: -1, ratio: 20, attack: 0.001 }));
}

/** The engine's pitch at a speed: through five gears, each a step higher, dropping at the shift. */
export function enginePitch(speed: number, top: number): number {
  const r = Math.max(0, Math.min(1.15, Math.abs(speed) / top)) * 5;
  const gear = Math.min(4, Math.floor(r));
  const rpm = r - gear;
  return 0.62 + rpm * 0.55 + gear * 0.09;
}

/** The cues that are a seat's own (Car.sfx and its boost): this device's alone, or in a room on each seat's car. */
const SEAT_CUES = Sound.Lap | Sound.Best | Sound.Sector | Sound.Finish | Sound.Boost;

/** Whether each car was boosting last tick (its boost's whoosh as the boost lights). */
const boosted: boolean[] = [];

/** Starts a voice (stopped this tick if it still sounds, to start on the next): whether it started. */
function fire(world: World, v: Entity, pitch?: number): boolean {
  const src = must(world.get(v, og.AudioSource));
  if (src.playing) {
    must(world.set(v, og.AudioSource, { playing: false }));
    return false;
  }
  must(world.set(v, og.AudioSource, { playing: true, ...(pitch === undefined ? {} : { pitch }) }));
  must(world.set(v, Sfx, { plays: must(world.get(v, Sfx)).plays + 1 }));
  return true;
}

/** Plays each requested sound once; a contact's pitch varies a little. Pitches and fades the engines and tyres. */
export const Voices = defineSystem("Voices", { reads: [Tuning, og.Lockstep], writes: [Game, Car, Sfx, og.AudioSource, og.Rng], after: ["Scene"] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  // The last system: every system has seen this tick's start.
  g.start = false;
  const t = tuning(world);
  const held = g.held || time.scaledDt === 0;
  const mine = LOCAL ? [0, 1] : localSeats(world);
  let want = g.sfx;
  CARS.forEach((e, i) => {
    const c = must(world.get(e, Car));
    // In a room every seat is heard alike on every peer (its sources are on its car); otherwise this device's alone.
    const own = c.seat >= 0 && (ONLINE || mine.includes(c.seat));
    const cues = (own ? c.sfx : 0) | (c.boosting && !boosted[i] && own ? Sound.Boost : 0);
    boosted[i] = c.boosting;
    if (c.sfx !== 0) must(world.set(e, Car, { sfx: 0 }));
    if (ONLINE) {
      for (const [, bit] of ONE_SHOTS) {
        const v = (cues & bit) !== 0 ? world.find(`sfx-${bit}-${i}`) : null;
        if (v !== null) fire(world, v);
      }
    } else want |= cues;
    const live = g.phase !== Phase.Title || i >= SEATS;
    const pitch = enginePitch(c.speed, t.topSpeed) + (c.boosting ? 0.08 : 0);
    const engine = i < SEATS ? world.find(`engine-${i}`) : e;
    if (engine !== null) {
      const quiet = held || (i < SEATS && !own);
      must(world.set(engine, og.AudioSource, { pitch, volume: quiet ? 0 : i < SEATS ? (live ? 0.14 + c.throttle * 0.16 : 0.06) : 0.45 }));
    }
    const tyres = i < SEATS ? world.find(`tyres-${i}`) : null;
    if (tyres !== null) {
      const slide = !held && own && (c.drifting || Math.abs(c.slip) > 3.5) ? Math.min(0.5, 0.15 + Math.abs(c.slip) * 0.04) : 0;
      must(world.set(tyres, og.AudioSource, { volume: slide, pitch: 0.9 + Math.min(0.25, Math.abs(c.slip) * 0.02) }));
    }
  });
  if (want === 0) return;
  const rng = must(world.resource(og.Rng));
  let left = want;
  for (const [, bit] of [...ONE_SHOTS, ["", Sound.Tap, 0] as const]) {
    if ((want & bit) === 0) continue;
    const v = world.find(`sfx-${bit}`);
    if (v === null) continue;
    // Still sounding: stopped this tick, started again on the next.
    if (fire(world, v, bit === Sound.Bump || bit === Sound.Crash ? rng.range(0.88, 1.12) : undefined)) left &= ~bit;
  }
  // The race's requests still waiting (a voice that was still sounding) play next tick; a seat's own are this device's and go.
  g.sfx &= left;
});
