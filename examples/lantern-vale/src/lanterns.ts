// The lanterns: which one is within reach is an og.physics.overlap query
// (a sphere of Tuning.reach round the hero, on the lantern layer only);
// holding E (the pad's X) there for Tuning.kindleSeconds lights it: the
// glass glows and its light swells in, a burst of sparks, a camera punch, a
// "+1" rising, a toast and a chime. All four lit wins the run.

import { og, defineSystem, SHAPE_SPHERE } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Ending, Game, Lantern, LANTERN_COUNT, Phase, Sound, Tuning } from "./components.ts";
import { FEET } from "./hero.ts";
import { say } from "./ui.ts";
import { douseLanterns, LANTERNS, LAYER, must, part, setGlow, styleModels } from "./world.ts";

type GameV = Value<typeof Game>;

/** Lantern `i` catches: the glow, the burst, the punch, the toast; the fourth wins the run. */
export function lightLantern(world: World, g: GameV, i: number): void {
  if ((g.litMask & (1 << i)) !== 0) return;
  const e = world.find(`lantern-${i}`);
  if (e === null) return;
  must(world.set(e, Lantern, { lit: true, litAt: g.time, glow: 0 }));
  must(world.set(part(world, i, "burst"), og.ParticleSystem, { visible: true, startTick: world.tick }));
  g.litMask |= 1 << i;
  g.lit += 1;
  g.kindle = 0;
  g.near = -1;
  g.sfx |= Sound.Light;
  must(og.camera.punch(world, 0.08));
  const at = LANTERNS[i]!;
  const y = must(world.get(e, og.Transform)).position.y;
  must(og.render.floatingText(world, { text: "+1", position: { x: at.x, y: y + 3, z: at.z }, size: 0.8, color: { r: 1, g: 0.8, b: 0.3 }, rise: 1.2, lifetime: 1 }));
  if (g.lit < LANTERN_COUNT) og.ui.toast(world, say(world, "hud.lit", { n: g.lit, total: LANTERN_COUNT }));
  else {
    g.ending = Ending.Lit;
    g.sfx |= Sound.Win;
  }
}

/** The unlit lantern nearest the hero within reach, by an overlap query on the lantern layer; -1 none. */
function nearLantern(world: World, g: GameV, reach: number): number {
  const hero = g.hero as unknown as Entity;
  const p = must(world.get(hero, og.Transform)).position;
  const hits = must(og.physics.overlap(world, { shape: { shape: SHAPE_SPHERE, radius: reach }, position: { x: p.x, y: p.y - FEET + 1, z: p.z }, filter: { mask: LAYER.lantern, exclude: hero } }));
  let best = -1;
  let nearest = Infinity;
  for (const h of hits) {
    const l = world.get(h, Lantern);
    if (!l.ok || l.value.lit) continue;
    const at = LANTERNS[l.value.index]!;
    const d = Math.hypot(at.x - p.x, at.z - p.z);
    if (d < nearest) {
      nearest = d;
      best = l.value.index;
    }
  }
  return best;
}

export const Lanterns = defineSystem(
  "Lanterns",
  { reads: [Tuning, og.Input, og.Gamepad, og.Transform], writes: [Game, Lantern, og.PointLight, og.ParticleSystem, og.Mesh, og.Camera], after: ["Hero"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    if (g.start) {
      douseLanterns(world);
      // Behind the title the four already glow.
      if (g.phase === Phase.Title) for (let i = 0; i < LANTERN_COUNT; i++) {
        must(world.set(must(entityOf(world, i)), Lantern, { lit: true, glow: 1 }));
        setGlow(world, i, 1);
      }
    }
    if (!g.styled && styleModels(world)) g.styled = true;
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    const dt = time.scaledDt;
    const near = playing ? nearLantern(world, g, t.reach) : -1;
    if (near !== g.near) {
      g.near = near;
      g.kindle = 0;
    }
    const inp = must(world.resource(og.Input));
    const gp = must(world.resource(og.Gamepad));
    const held = playing && near >= 0 && (inp.down("KeyE") || gp.down("x"));
    if (held && !g.kindling) g.sfx |= Sound.Kindle;
    if (held !== g.kindling) g.kindling = held;
    if (held) g.kindle += dt;
    else if (g.kindle > 0) g.kindle = Math.max(0, g.kindle - dt * 2);
    if (playing && near >= 0 && g.kindle >= t.kindleSeconds) lightLantern(world, g, near);
    // Each lit lantern's light swells in over half a second, then breathes.
    if (dt === 0) return;
    for (let i = 0; i < LANTERN_COUNT; i++) {
      const e = must(entityOf(world, i));
      const l = must(world.get(e, Lantern));
      if (!l.lit) continue;
      const glow = Math.min(1, l.glow + dt * 2);
      if (glow !== l.glow) must(world.set(e, Lantern, { glow }));
      const flicker = 0.9 + 0.06 * Math.sin(time.elapsed * 11 + i * 2.1) + 0.04 * Math.sin(time.elapsed * 23 + i * 4.7);
      setGlow(world, i, glow * flicker);
    }
  },
);

function entityOf(world: World, i: number): { ok: true; value: Entity } | { ok: false; reason: string } {
  const e = world.find(`lantern-${i}`);
  return e === null ? { ok: false, reason: `lantern-${i} is missing` } : { ok: true, value: e };
}
