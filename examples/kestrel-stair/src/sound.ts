// The sounds. One voice per effect (og sfx presets cooked from
// assets/audio/*.ogt, and the bell), played once when a system sets its bit
// in Game.sfx; the positional ones (the braziers' crackle, the imps'
// chitter, the wardens' slams, the crypt's drip, the bell) live on their
// entities (src/world.ts, src/foes.ts) and are heard from the listener on
// the camera (src/view.ts). "The Kestrel Stair" (2:40, cooked in 4-second
// chunks and streamed) loops on the music bus, ducked under the effects.
// The crypt is a reverb zone (og.ReverbZone, FINDINGS #21 fixed): with the
// listener inside, the engine eases the shared reverb's room and damping
// and the effects bus's send towards the crypt's, fading over a metre and a
// half past its walls; og.Audio and the bus keep the open air's.

import { og, defineSystem } from "@origingame/engine";
import type { Sound as SoundHandle, Value, World } from "@origingame/engine";
import { Game, Sfx, Sound } from "./components.ts";
import { CRYPT_ROOM } from "./layout.ts";
import { must } from "./world.ts";

/** The UI's sounds, for og.ui.configure (src/ui.ts). */
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
  return { hover: s("audio/ui-hover.ogt"), press: s("audio/ui-click.ogt"), open: s("audio/ui-open.ogt"), close: s("audio/ui-close.ogt"), success: s("audio/ui-open.ogt"), error: s("audio/ui-error.ogt") };
}

export function spawnVoices(world: World, g: Value<typeof Game>): void {
  const sound = (path: string) => must(og.load.sound(world, path));
  const voices: [string, number, number, number][] = [
    ["audio/step.ogt", Sound.Step, 0.35, 120],
    ["audio/swing.ogt", Sound.Swing, 0.55, 200],
    ["audio/hit.ogt", Sound.Hit, 0.8, 215],
    ["audio/hurt.ogt", Sound.Hurt, 0.8, 220],
    ["audio/death.ogt", Sound.Death, 0.7, 205],
    ["audio/jump.ogt", Sound.Jump, 0.45, 150],
    ["audio/land.ogt", Sound.Land, 0.45, 140],
    ["audio/sigil.ogt", Sound.Sigil, 0.8, 230],
    ["audio/herb.ogt", Sound.Herb, 0.6, 180],
    ["audio/heal.ogt", Sound.Heal, 0.6, 190],
    ["audio/gate.ogt", Sound.Gate, 0.9, 230],
    ["audio/waystone.ogt", Sound.Waystone, 0.7, 200],
    ["audio/alert.ogt", Sound.Alert, 0.5, 190],
    ["audio/bite.ogt", Sound.Bite, 0.6, 180],
    ["audio/win.ogt", Sound.Win, 0.8, 240],
    ["audio/lose.ogt", Sound.Lose, 0.8, 240],
    ["audio/ui-error.ogt", Sound.Error, 0.6, 200],
  ];
  for (const [path, bit, volume, priority] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus: "sfx", volume, priority }));
    must(world.insert(v, Sfx, { bit }));
  }
  const track = sound("music/stair.wav");
  g.music = track as unknown as number;
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: track, bus: "music", loop: true, volume: 0.55, priority: 250 }));
  must(world.insert(must(world.spawn({ name: "music-bus" })), og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 8, duckRelease: 0.5 }));
  must(world.insert(must(world.spawn({ name: "sfx-bus" })), og.AudioBusFx, { bus: "sfx", reverb: 0.12 }));
  const audio = must(world.resource(og.Audio));
  audio.reverbRoom = 0.5;
  audio.reverbDamping = 0.5;
  const b = CRYPT_ROOM;
  const crypt = must(world.spawn({ name: "crypt-reverb", position: { x: (b.min[0] + b.max[0]) / 2, y: (b.min[1] + b.max[1]) / 2, z: (b.min[2] + b.max[2]) / 2 } }));
  must(world.insert(crypt, og.ReverbZone, { size: { x: b.max[0] - b.min[0], y: b.max[1] - b.min[1], z: b.max[2] - b.min[2] }, fade: 1.5, bus: "sfx", send: 0.55, room: 0.92, damping: 0.25 }));
  must(world.insert(must(world.spawn({ name: "master-bus" })), og.AudioBusFx, { bus: "master", threshold: -1, ratio: 20, attack: 0.001 }));
}

/** Plays each requested sound once; steps, hits and swings vary their pitch a little. */
export const Voices = defineSystem("Voices", { writes: [Game, Sfx, og.AudioSource, og.Rng], after: ["Fx"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  // The last system: every system has seen this tick's start. (Written only when it changes;
  // FINDINGS #19, fixed: a field write is a typed store now, flushed at the system's end.)
  if (g.start) g.start = false;
  if (g.sfx === 0) return;
  const rng = must(world.resource(og.Rng));
  let left = g.sfx;
  for (const a of q.archetypes()) {
    const sfx = a.column(Sfx);
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) {
      const b = sfx.bit[i]!;
      if ((g.sfx & b) === 0) continue;
      if (src.playing[i]) {
        // Still sounding: stopped this tick, started again on the next.
        src.playing[i] = 0;
      } else {
        src.playing[i] = 1;
        sfx.plays[i] = sfx.plays[i]! + 1;
        if (b === Sound.Step || b === Sound.Hit || b === Sound.Swing) src.pitch[i] = rng.range(0.9, 1.1);
        left &= ~b;
      }
    }
  }
  if (left !== g.sfx) g.sfx = left;
});
