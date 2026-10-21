// What the courier uses with F (the pad's Y): the three sigils (gold,
// turning in the air), the hillwort, the waystones (rest: strength back,
// the chapter reached, the world saved, src/save.ts), the summit gate (it
// rises with three sigils: its collider goes, its navmesh obstacle is
// lifted) and the bell (the climb's end). Whichever is nearest within
// reach is Game.nearKind / nearIndex; the HUD shows its prompt.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Ending, Game, MAX_HP, Near, OneByOne, Phase, Pickup, Scene, Sound, Tuning } from "./components.ts";
import { BELL, CHAPTERS, GATE, WAYSTONES } from "./layout.ts";
import { saveSlot } from "./save.ts";
import { say } from "./text.ts";
import { entity, gateObstacle, must, setWaystoneGlow } from "./world.ts";

type GameV = Value<typeof Game>;

const REACH = 1.9;
const SIGIL_BITS = 0b111;

/** Takes sigil `i`: its burst, its sound, the HUD's count and a toast. */
export function takeSigil(world: World, g: GameV, i: number): void {
  if ((g.sigils & (1 << i)) !== 0) return;
  const e = entity(world, `sigil-${i}`);
  must(world.set(e, Pickup, { taken: true }));
  must(world.set(e, og.Mesh, { visible: false }));
  must(world.set(e, og.PointLight, { intensity: 0 }));
  g.sigils |= 1 << i;
  g.sfx |= Sound.Sigil;
  const n = [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length;
  const burst = entity(world, "sigil-burst");
  must(world.set(burst, og.Transform, { position: { x: g.heroX, y: g.heroY + 1, z: g.heroZ } }));
  must(world.set(burst, og.ParticleSystem, { visible: true, startTick: world.tick }));
  must(og.camera.punch(world, 0.08));
  must(og.render.floatingText(world, { text: `${n}/3`, position: { x: g.heroX, y: g.heroY + 2.2, z: g.heroZ }, size: 0.5, color: { r: 1, g: 0.82, b: 0.4 }, rise: 1.2, lifetime: 1 }));
  og.ui.toast(world, say(world, "toast.sigil", { n }), { tone: "success" });
}

/** Rests at waystone `i`: strength back, the chapter reached, the world saved. */
export function rest(world: World, g: GameV, i: number): void {
  g.hp = MAX_HP;
  g.rested |= 1 << i;
  g.reached |= 1 << i;
  g.chaptersReached |= 1 << i;
  g.sfx |= Sound.Waystone;
  setWaystoneGlow(world, i, 1);
  saveSlot(world, g, i);
  og.ui.toast(world, say(world, "toast.saved", { place: say(world, `place.${WAYSTONES[i]!.key}`) }), { tone: "success" });
}

/** Eats a hillwort from the bag (the bag's Use, or H). */
export function eatHerb(world: World, g: GameV): void {
  const t = must(world.resource(Tuning));
  if (g.herbs === 0) {
    og.ui.toast(world, say(world, "toast.noHerb"));
    g.sfx |= Sound.Error;
    return;
  }
  if (g.hp >= MAX_HP) {
    og.ui.toast(world, say(world, "toast.full"));
    g.sfx |= Sound.Error;
    return;
  }
  g.herbs -= 1;
  g.herbsUsed += 1;
  g.hp = Math.min(MAX_HP, g.hp + t.herbHeal);
  g.sfx |= Sound.Heal;
  must(og.render.floatingText(world, { text: `+${Math.round(t.herbHeal)}`, position: { x: g.heroX, y: g.heroY + 2.1, z: g.heroZ }, size: 0.45, color: { r: 0.45, g: 1, b: 0.45 }, rise: 1, lifetime: 0.9 }));
  og.ui.toast(world, say(world, "toast.healed"), { tone: "success" });
}

/** Raises the summit gate (three sigils carried). */
export function raiseGate(world: World, g: GameV): void {
  if (g.gateOpen) return;
  g.gateOpen = true;
  g.sfx |= Sound.Gate;
  must(world.remove(entity(world, "gate"), og.Collider));
  gateObstacle(world, g, false);
  must(og.camera.shake(world, 0.2));
  og.ui.toast(world, say(world, "toast.gate"), { tone: "success" });
}

/** Rings the bell: the letter delivered, the ending cutscene. */
export function ringBell(world: World, g: GameV): void {
  if (g.ending !== Ending.None) return;
  g.ending = Ending.Rung;
  g.phase = Phase.Ending;
  g.scene = Scene.End;
  g.line = 0;
  g.sfx |= Sound.Win;
  const bell = entity(world, "bell");
  must(world.set(bell, og.AudioSource, { playing: false }));
  must(world.set(bell, og.AudioSource, { playing: true }));
  must(og.tween.to(world, { entity: bell, component: og.Transform, field: "position.x", to: [0.12], duration: 0.4, ease: "quadInOut", yoyo: true, loops: 5 }));
}

/** Puts the sigils, herbs, gate and waystones back for a run starting at `chapter`. */
function reset(world: World, g: GameV): void {
  const given = CHAPTERS[g.chapter]?.sigils ?? 0;
  for (let i = 0; i < 3; i++) {
    const e = entity(world, `sigil-${i}`);
    const p = must(world.get(e, Pickup));
    const carried = (given & (1 << i)) !== 0;
    // The warden's sigil shows only once it is dropped.
    const shown = !carried && i !== 2;
    must(world.set(e, Pickup, { taken: carried }));
    must(world.set(e, og.Mesh, { visible: shown }));
    must(world.set(e, og.PointLight, { intensity: shown ? 3 : 0 }));
    if (i === 2) must(world.set(e, og.Transform, { position: { x: p.x, y: -20, z: p.z } }));
  }
  g.sigils = given;
  for (let i = 0; ; i++) {
    const e = world.find(`herb-${i}`);
    if (e === null) break;
    must(world.set(e, Pickup, { taken: false }));
    const p = must(world.get(e, Pickup));
    must(world.set(e, og.Transform, { position: { x: p.x, y: p.y, z: p.z }, scale: { x: 1, y: 1, z: 1 } }));
  }
  g.gathered = 0;
  for (let i = 0; i < WAYSTONES.length; i++) setWaystoneGlow(world, i, 0);
  const gate = entity(world, "gate");
  if (!world.has(gate, og.Collider)) {
    const [x0, y0, z0] = GATE.min;
    const [x1, y1, z1] = GATE.max;
    must(world.insert(gate, og.Collider, { shape: 0, size: { x: x1 - x0, y: y1 - y0, z: z1 - z0 }, layers: 8 }));
  }
  must(world.set(gate, og.Transform, { position: { x: (GATE.min[0] + GATE.max[0]) / 2, y: (GATE.min[1] + GATE.max[1]) / 2, z: (GATE.min[2] + GATE.max[2]) / 2 } }));
  must(world.set(gate, og.Mesh, { visible: true }));
  if (g.gateOpen) gateObstacle(world, g, true);
  g.gateOpen = false;
  g.gateLift = 0;
}

export const Pickups = defineSystem(
  "Pickups",
  { reads: [og.Input, og.Gamepad, Tuning, OneByOne], writes: [Game, Pickup, og.Transform, og.Mesh, og.PointLight, og.ParticleSystem, og.Collider, og.AudioSource, og.Camera], after: ["Foes"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    if (g.start) reset(world, g);
    const playing = g.phase === Phase.Play && g.ending === Ending.None && !g.held;
    // The sigils turn and bob; the gate rises over a second and a half once opened.
    for (let i = 0; i < 3; i++) {
      const e = entity(world, `sigil-${i}`);
      const p = must(world.get(e, Pickup));
      if (p.taken) continue;
      const at = must(world.get(e, og.Transform)).position;
      must(world.set(e, og.Transform, { position: { x: at.x, y: at.y, z: at.z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, time.elapsed * 1.6) }));
    }
    if (g.gateOpen && g.gateLift < 4.2) {
      // It rises into the wall's top over a second and three quarters, then is gone from view.
      g.gateLift = Math.min(4.2, g.gateLift + time.scaledDt * 2.4);
      const gate = entity(world, "gate");
      must(world.set(gate, og.Transform, { position: { x: 0, y: (GATE.min[1] + GATE.max[1]) / 2 + g.gateLift, z: (GATE.min[2] + GATE.max[2]) / 2 } }));
      if (g.gateLift >= 4.2) must(world.set(gate, og.Mesh, { visible: false }));
    }
    if (!playing) {
      g.nearKind = Near.None;
      g.nearIndex = -1;
      return;
    }
    // What is within reach: the nearest.
    let kind: number = Near.None;
    let index = -1;
    let best = REACH;
    const consider = (k: number, i: number, x: number, y: number, z: number, reach = REACH) => {
      const d = Math.hypot(x - g.heroX, z - g.heroZ);
      if (d < Math.min(best, reach) && Math.abs(y - g.heroY) < 2.2) {
        best = d;
        kind = k;
        index = i;
      }
    };
    for (let i = 0; i < 3; i++) {
      const p = must(world.get(entity(world, `sigil-${i}`), Pickup));
      const at = must(world.get(entity(world, `sigil-${i}`), og.Transform)).position;
      if (!p.taken && at.y > -10) consider(Near.Sigil, i, at.x, at.y - 0.8, at.z);
    }
    for (let i = 0; ; i++) {
      const e = world.find(`herb-${i}`);
      if (e === null) break;
      const p = must(world.get(e, Pickup));
      if (!p.taken) consider(Near.Herb, i, p.x, p.y, p.z);
    }
    WAYSTONES.forEach((w, i) => consider(Near.Waystone, i, w.x, w.y, w.z));
    if (!g.gateOpen) consider(Near.Gate, 0, 0, GATE.min[1], (GATE.min[2] + GATE.max[2]) / 2 + 1, 3.4);
    consider(Near.Bell, 0, BELL.x, BELL.y, BELL.z, 3);
    g.nearKind = kind;
    g.nearIndex = index;
    const inp = must(world.resource(og.Input));
    const gp = must(world.resource(og.Gamepad));
    if (!(inp.pressed("KeyF") || gp.pressed("y"))) return;
    if (kind === Near.Sigil) takeSigil(world, g, index);
    else if (kind === Near.Herb) {
      const e = entity(world, `herb-${index}`);
      must(world.set(e, Pickup, { taken: true }));
      must(og.tween.to(world, { entity: e, component: og.Transform, field: "scale", to: [0, 0, 0], duration: 0.18, ease: "backIn" }));
      g.herbs += 1;
      g.gathered |= 1 << index;
      g.sfx |= Sound.Herb;
      og.ui.toast(world, say(world, "toast.herb"));
    } else if (kind === Near.Waystone) rest(world, g, index);
    else if (kind === Near.Gate) {
      if ((g.sigils & SIGIL_BITS) === SIGIL_BITS) raiseGate(world, g);
      else {
        // A refusal shakes the prompt (src/ui.ts), never the camera (og-feel §4).
        g.sfx |= Sound.Error;
        g.refusedAt = world.tick;
      }
    } else if (kind === Near.Bell) {
      if (g.gateOpen) ringBell(world, g);
    }
  },
);
