// The beacons: the hero kindles the nearest one by holding F (X, Kindle)
// within reach; Tuning.kindleSeconds later it catches (src/world.ts
// lightBeacon: fire, smoke, a burst of embers, the scorch and the rune),
// the camera punches and a toast counts it. Every burning beacon's light
// swells in and flickers. All four lit, dawn will find the vale lit: the
// run is won.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Beacon, BEACONS, Ending, Game, Phase, Sound, Tuning } from "./components.ts";
import { entity, STAND } from "./hero.ts";
import { say } from "./ui.ts";
import { BEACON_AT, douseBeacons, lightBeacon, must, part } from "./world.ts";

/** Beacon `i` catches: its fire, the punch, the toast; the last one wins the run (no toast). */
export function catchBeacon(world: World, g: Value<typeof Game>, i: number): void {
  if ((g.litMask & (1 << i)) !== 0) return;
  lightBeacon(world, i, g.time);
  g.litMask |= 1 << i;
  g.lit += 1;
  g.kindle = 0;
  g.near = -1;
  g.sfx |= Sound.Ignite;
  must(og.camera.punch(world, 0.35));
  // The last one's news is the dawn stinger and the summary, not a toast under it.
  if (g.lit < BEACONS) og.ui.toast(world, say(world, "hud.lit", { n: g.lit, total: BEACONS }));
  if (g.lit >= BEACONS) {
    g.ending = Ending.Dawn;
    g.sfx |= Sound.Dawn;
  }
}

/** A burning beacon's light at full flame (og.PointLight.intensity). */
const FLAME = 30;

export const Beacons = defineSystem(
  "Beacons",
  { reads: [Tuning, og.Transform], writes: [Game, Beacon, og.PointLight, og.ParticleSystem, og.Decal, og.Mesh], after: ["Hero"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    if (g.start) {
      // Behind the title the four burn; a run starts with them dark.
      douseBeacons(world);
      if (g.phase === Phase.Title) for (let i = 0; i < BEACONS; i++) lightBeacon(world, i, -1);
    }
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    const p = must(world.get(g.hero as unknown as Entity, og.Transform)).position;
    let near = -1;
    let nearest = Infinity;
    if (playing) {
      BEACON_AT.forEach((b, i) => {
        if ((g.litMask & (1 << i)) !== 0) return;
        const d = Math.hypot(p.x - b.x, p.z - b.z);
        if (d < t.reach + 0.8 && d < nearest) {
          nearest = d;
          near = i;
        }
      });
    }
    if (near !== g.near) {
      g.near = near;
      g.kindle = 0;
    }
    // Kindling counts once the hero stands at the plinth with its hand in the bowl.
    if (playing && g.kindling && near >= 0 && nearest <= STAND + 0.3) g.kindle += time.scaledDt;
    else if (g.kindle > 0) g.kindle = Math.max(0, g.kindle - time.scaledDt * 2);
    if (playing && near >= 0 && g.kindle >= t.kindleSeconds) catchBeacon(world, g, near);
    // The lights: each swells in over half a second and flickers.
    for (let i = 0; i < BEACONS; i++) {
      const b = must(world.get(entity(world, `beacon-${i}`), Beacon));
      if (!b.lit) continue;
      const since = g.phase === Phase.Title ? 10 : g.time - b.litAt;
      const swell = Math.min(1, since / 0.5);
      const e = time.elapsed;
      const flicker = 0.86 + 0.08 * Math.sin(e * 13 + i * 2.1) + 0.06 * Math.sin(e * 29 + i * 5.3);
      must(world.set(part(world, i, "light"), og.PointLight, { intensity: FLAME * swell * flicker }));
    }
  },
);
