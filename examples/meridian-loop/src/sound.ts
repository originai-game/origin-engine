// The sounds. The engine is one looping voice whose pitch follows the
// revs (rising through each gear, dropping at the shift) and whose volume
// follows the throttle; the tyres are a second loop faded in with the
// slide. Every other sound is a one-shot voice played when a system sets
// its bit in Race.sfx. "Meridian" (57.6 s, streamed in 4-second chunks)
// loops on the music bus, ducked under the effects; a limiter on the
// master.

import { og, defineSystem } from "@origingame/engine";
import type { Sound as SoundHandle, World } from "@origingame/engine";
import { Phase, Race, Sfx, Sound } from "./components.ts";
import { entity, must } from "./world.ts";

export interface UiSounds {
  hover: SoundHandle;
  press: SoundHandle;
  open: SoundHandle;
  close: SoundHandle;
  success: SoundHandle;
  error: SoundHandle;
}

export function uiSounds(world: World): UiSounds {
  const s = (p: string) => must(og.load.sound(world, p));
  return { hover: s("audio/ui-hover.wav"), press: s("audio/ui-click.wav"), open: s("audio/ui-open.wav"), close: s("audio/ui-close.wav"), success: s("audio/ui-open.wav"), error: s("audio/ui-error.wav") };
}

export function spawnVoices(world: World): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number, number][] = [
    ["audio/beep.wav", Sound.Beep, 0.6, 230],
    ["audio/go.wav", Sound.Go, 0.7, 235],
    ["audio/sector.wav", Sound.Sector, 0.5, 220],
    ["audio/lap.wav", Sound.Lap, 0.6, 225],
    ["audio/best.wav", Sound.Best, 0.7, 240],
    ["audio/slower.wav", Sound.Slower, 0.5, 220],
    ["audio/bump.wav", Sound.Bump, 0.75, 210],
    ["audio/land.wav", Sound.Land, 0.8, 205],
    ["audio/ui-error.wav", Sound.Error, 0.6, 200],
  ];
  for (const [path, bit, volume, priority] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority }));
    must(world.insert(v, Sfx, { bit }));
  }
  const engine = must(world.spawn({ name: "engine" }));
  must(world.insert(engine, og.AudioSource, { sound: sound("audio/engine.wav"), bus: "sfx", loop: true, volume: 0.25, pitch: 0.8, priority: 250, fade: 0.05 }));
  const tyres = must(world.spawn({ name: "tyres" }));
  must(world.insert(tyres, og.AudioSource, { sound: sound("audio/skid.wav"), bus: "sfx", loop: true, volume: 0, priority: 240, fade: 0.08 }));
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("music/meridian.wav"), bus: "music", loop: true, volume: 0.5, priority: 255 }));
  must(world.insert(must(world.spawn({ name: "music-bus" })), og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 6, duckRelease: 0.5 }));
  must(world.insert(must(world.spawn({ name: "master-bus" })), og.AudioBusFx, { bus: "master", threshold: -1, ratio: 20, attack: 0.001 }));
}

/** The engine's pitch for the revs in a gear: up through the gear, a step higher each gear. */
export function enginePitch(gear: number, rpm: number): number {
  return 0.72 + rpm * 1.25 + Math.max(0, gear - 1) * 0.06;
}

export const Voices = defineSystem("Voices", { writes: [Race, Sfx, og.AudioSource, og.Rng], after: ["View"] }, (q, time, world) => {
  const r = must(world.resource(Race));
  const live = r.phase === Phase.Racing || r.phase === Phase.Countdown || r.phase === Phase.Finished;
  const held = r.held || time.scaledDt === 0;
  // The engine: idle at the title, the revs while driving, quiet under a menu.
  const engine = entity(world, "engine");
  const pitch = r.phase === Phase.Countdown ? 0.72 + Math.max(0, r.throttle) * 0.9 : enginePitch(r.gear, r.rpm);
  const vol = !live || held ? 0.08 : 0.16 + Math.abs(r.throttle) * 0.18 + (r.airborne ? 0.04 : 0);
  must(world.set(engine, og.AudioSource, { pitch: Math.max(0.3, Math.min(3, pitch)), volume: vol }));
  const tyres = entity(world, "tyres");
  const slide = !held && r.skidding ? Math.min(0.55, (Math.abs(r.slip) - 2.5) * 0.08) : 0;
  must(world.set(tyres, og.AudioSource, { volume: Math.max(0, slide), pitch: 0.9 + Math.min(0.3, Math.abs(r.slip) * 0.02) }));
  if (r.sfx === 0) return;
  const rng = must(world.resource(og.Rng));
  let left = r.sfx;
  for (const a of q.archetypes()) {
    const sfx = a.column(Sfx);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = sfx.bit[i]!;
      if ((r.sfx & b) === 0) continue;
      if (src.playing[i]) {
        // Still sounding: stopped this tick, started again on the next.
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        sfx.plays[i] = sfx.plays[i]! + 1;
        if (b === Sound.Bump || b === Sound.Land) src.pitch[i] = rng.range(0.9, 1.1);
        left &= ~b;
      }
    }
  }
  r.sfx = left;
});
