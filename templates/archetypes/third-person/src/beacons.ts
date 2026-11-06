// The beacons, shared by every strider: one kindles the nearest dark one by
// holding Kindle (F, X, Kindle) within reach, two at one plinth kindle it
// twice as fast; Tuning.kindleSeconds of kindling later it catches (src/world.ts
// lightBeacon: fire, smoke, a burst of embers, the scorch and the rune),
// the camera punches and a toast counts it. Every burning beacon's light
// swells in and flickers. All four lit, dawn will find the vale lit: the
// run is won.

import { og, defineSystem, math } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Beacon, BEACONS, Ending, Game, Hero, Phase, Sound, Tuning } from "./components.ts";
import { SEATS } from "./seats.ts";
import { entity, heroOf, hypot, STAND } from "./hero.ts";
import { say } from "./ui.ts";
import { BEACON_AT, douseBeacons, lightBeacon, must, part } from "./world.ts";

/** Beacon `i` catches: its fire, the punch, the toast; the last one wins the run (no toast). */
export function catchBeacon(world: World, g: Value<typeof Game>, i: number): void {
  if ((g.litMask & (1 << i)) !== 0) return;
  lightBeacon(world, i, g.time);
  must(world.set(entity(world, `beacon-${i}`), Beacon, { kindle: 0 }));
  g.litMask |= 1 << i;
  g.lit += 1;
  g.kindle = 0;
  g.near = -1;
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    if (must(world.get(hero, Hero)).near === i) must(world.set(hero, Hero, { near: -1 }));
  }
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
  { reads: [Tuning, og.Transform], writes: [Game, Hero, Beacon, og.PointLight, og.ParticleSystem, og.Decal, og.Mesh], after: ["Hero"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    if (g.start) {
      // Behind the title the four burn; a run starts with them dark.
      douseBeacons(world);
      if (g.phase === Phase.Title) for (let i = 0; i < BEACONS; i++) lightBeacon(world, i, -1);
    }
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    // Each strider's nearest dark beacon within reach, and who kindles it at its plinth.
    const near: number[] = [];
    const kindlers = new Array<number>(BEACONS).fill(0);
    for (let seat = 0; seat < SEATS; seat++) {
      const hero = heroOf(g, seat);
      const me = must(world.get(hero, Hero));
      const p = must(world.get(hero, og.Transform)).position;
      let at = -1;
      let nearest = Infinity;
      if (playing) {
        BEACON_AT.forEach((b, i) => {
          if ((g.litMask & (1 << i)) !== 0) return;
          const d = hypot(p.x - b.x, p.z - b.z);
          if (d < t.reach + 0.8 && d < nearest) {
            nearest = d;
            at = i;
          }
        });
      }
      if (at !== me.near) must(world.set(hero, Hero, { near: at }));
      // Kindling counts once the strider stands at the plinth with its hand in the bowl.
      if (playing && me.kindling && at >= 0 && nearest <= STAND + 0.3) kindlers[at]! += 1;
      near.push(at);
    }
    for (let i = 0; i < BEACONS; i++) {
      const e = entity(world, `beacon-${i}`);
      const b = must(world.get(e, Beacon));
      let kindle = b.kindle;
      // No strider within reach: its kindling is lost; none at it: it cools.
      if (!near.includes(i)) kindle = 0;
      else if (kindlers[i]! > 0) kindle += time.scaledDt * kindlers[i]!;
      else if (kindle > 0) kindle = Math.max(0, kindle - time.scaledDt * 2);
      if (kindle !== b.kindle) must(world.set(e, Beacon, { kindle }));
      if (playing && near.includes(i) && kindle >= t.kindleSeconds) catchBeacon(world, g, i);
    }
    // The facts and the HUD's progress: seat 0's beacon (none once it caught this tick).
    const first = near[0]!;
    const at = first >= 0 && (g.litMask & (1 << first)) === 0 ? first : -1;
    if (at !== g.near) g.near = at;
    const kindle = at >= 0 ? must(world.get(entity(world, `beacon-${at}`), Beacon)).kindle : 0;
    if (kindle !== g.kindle) g.kindle = kindle;
    // The lights: each swells in over half a second and flickers.
    for (let i = 0; i < BEACONS; i++) {
      const b = must(world.get(entity(world, `beacon-${i}`), Beacon));
      if (!b.lit) continue;
      const since = g.phase === Phase.Title ? 10 : g.time - b.litAt;
      const swell = Math.min(1, since / 0.5);
      const e = time.elapsed;
      const flicker = 0.86 + 0.08 * math.sin(e * 13 + i * 2.1) + 0.06 * math.sin(e * 29 + i * 5.3);
      must(world.set(part(world, i, "light"), og.PointLight, { intensity: FLAME * swell * flicker }));
    }
  },
);
