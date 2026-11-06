// What the explorers do to the isle: gathering (every explorer holding
// Gather at a tree or a boulder adds its time; a tree gives wood, a
// boulder stone, and both grow back at the next dawn), building the bridge
// at the strait's landing (its wood and stone spent, its planks rising
// into place one after another, its deck solid), and the landmarks: one
// is discovered when an explorer comes within sight of it (its name on
// the compass and the map) and claimed when one reaches it (the standard's
// flag rises, the brazier catches, a burst of sparks, a punch of the
// camera). The fifth claimed ends the run, lit.
//
// The quests (assets/story/) count it all: og.story is world state, reset
// with the run.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { BRIDGE_COST, Ending, Game, Hero, Kind, Node, Phase, Site, SITE_COUNT, Sound, Target, Tuning, YIELD } from "./components.ts";
import { heroOf, hypot, lazyTuning, NODES } from "./hero.ts";
import { SITES } from "./layout.ts";
import { SEATS } from "./seats.ts";
import { bridgeParts, deckParts, FLAG, ground, must, nodeOf, standardOf } from "./world.ts";
import { prefs } from "./device.ts";

type GameV = Value<typeof Game>;

/** Seconds between strokes of the axe or the pick. */
const STROKE = 0.36;
/** A site's flag, claimed: its colours back, and greyed while it waits. */
const FLAG_TINT = { r: 1, g: 1, b: 1 };
const FLAG_WAITING = { r: 0.45, g: 0.45, b: 0.45 };

function part(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`${name} is missing`);
  return e;
}

/** The quests from nothing: started, tracked, and the lights pointed at the nearest landmark. */
export function startQuests(world: World, g: GameV): void {
  og.story.reset(world);
  for (const id of ["lights", "bridge"]) {
    must(og.story.startQuest(world, id));
    og.story.track(world, id, true);
  }
  markNext(world, g);
}

/** Points the lights quest (the tracker, the compass, the map) at the unclaimed landmark nearest seat 0's explorer. */
function markNext(world: World, g: GameV): void {
  const p = must(world.get(heroOf(g, 0), og.Transform)).position;
  let best = -1;
  let bestD = 1e9;
  SITES.forEach((s, i) => {
    if ((g.claimMask & (1 << i)) !== 0) return;
    const d = hypot(s.x - p.x, s.z - p.z);
    if (d < bestD) {
      bestD = d;
      best = i;
    }
  });
  if (best < 0) return;
  const s = SITES[best]!;
  og.story.mark(world, "lights", { position: { x: s.x, y: ground(world, g, s.x, s.z).height, z: s.z } });
}

/** Puts the isle back as a run starts: every tree and boulder standing, the sites waiting, the bridge unbuilt. */
function resetIsle(world: World, g: GameV): void {
  for (let i = 0; i < NODES.length; i++) regrow(world, i);
  for (let i = 0; i < SITE_COUNT; i++) {
    must(world.set(part(world, `site-${i}`), Site, { claimed: false, claimedAt: 0, seen: false }));
    const at = standardOf(i);
    const flag = part(world, `site-${i}-flag`);
    og.tween.cancelEntity(world, flag);
    must(world.set(flag, og.Transform, { position: { x: at.x, y: ground(world, g, at.x, at.z).height + FLAG.down, z: at.z } }));
    must(world.set(flag, og.Model, { tint: FLAG_WAITING }));
    for (const p of ["fire", "smoke", "burst"]) must(world.set(part(world, `site-${i}-${p}`), og.ParticleSystem, { visible: false }));
    const light = part(world, `site-${i}-light`);
    og.tween.cancelEntity(world, light);
    must(world.set(light, og.PointLight, { intensity: 0 }));
  }
  for (const b of bridgeParts(world, g)) {
    og.tween.cancelEntity(world, b.entity);
    const at = must(world.get(b.entity, og.Transform)).position;
    must(world.set(b.entity, og.Transform, { position: { x: at.x, y: b.y - 6, z: at.z } }));
    must(world.set(b.entity, og.Mesh, { visible: false }));
  }
  for (const d of deckParts(world, g)) {
    const at = must(world.get(d.entity, og.Transform)).position;
    must(world.set(d.entity, og.Transform, { position: { x: at.x, y: -40, z: at.z } }));
  }
  startQuests(world, g);
}

function regrow(world: World, i: number): void {
  const e = nodeOf(world, i);
  const n = must(world.get(e, Node));
  if (!n.felled && n.progress === 0) return;
  must(world.set(e, Node, { felled: false, progress: 0 }));
  must(world.set(n.model as unknown as Entity, og.Model, { visible: true }));
  must(world.set(n.stump as unknown as Entity, og.Model, { visible: false }));
}

/** A stroke at node i: chips fly from it, it shudders, the axe or the pick is heard. */
function stroke(world: World, g: GameV, i: number): void {
  const n = NODES[i]!;
  const row = must(world.get(nodeOf(world, i), Node));
  const chips = part(world, row.kind === Kind.Tree ? "chips-wood" : "chips-stone");
  const h = ground(world, g, n.x, n.z).height;
  must(world.set(chips, og.Transform, { position: { x: n.x, y: h + (row.kind === Kind.Tree ? 1.1 : 0.8), z: n.z } }));
  must(world.set(chips, og.ParticleSystem, { visible: true, startTick: world.tick }));
  const model = row.model as unknown as Entity;
  const s = must(world.get(model, og.Transform)).scale.y;
  og.tween.cancelEntity(world, model);
  must(og.tween.to(world, { entity: model, field: "scale", from: [s * 1.05, s * 0.95, s * 1.05], to: [s, s, s], duration: 0.2, ease: "backOut" }));
  g.sfx |= row.kind === Kind.Tree ? Sound.Chop : Sound.Stone;
}

/** Node i is gathered: hidden until dawn, its stump or rubble left, what it gives in the bag. */
function gathered(world: World, g: GameV, i: number): void {
  const e = nodeOf(world, i);
  const n = must(world.get(e, Node));
  must(world.set(e, Node, { felled: true, progress: 0 }));
  must(world.set(n.model as unknown as Entity, og.Model, { visible: false }));
  must(world.set(n.stump as unknown as Entity, og.Model, { visible: true }));
  const at = NODES[i]!;
  const h = ground(world, g, at.x, at.z).height;
  const wood = n.kind === Kind.Tree;
  const amount = wood ? YIELD.wood : YIELD.stone;
  if (wood) {
    g.wood += amount;
    g.woodTotal += amount;
  } else {
    g.stone += amount;
    g.stoneTotal += amount;
  }
  g.felled += 1;
  og.story.progress(world, "bridge", wood ? "wood" : "stone", amount);
  g.sfx |= Sound.Pickup;
  // Numbers alone: the text is world state, the same on every peer whatever its language.
  must(og.render.floatingText(world, { text: `+${amount}`, position: { x: at.x, y: h + 2.2, z: at.z }, size: 0.5, color: { r: 1, g: 0.95, b: 0.75 }, outline: 0.25, rise: 1.2, lifetime: 1.2, facing: "camera" }));
}

/** The bridge is finished: the wood and stone spent, its planks rising one after another, its deck solid. */
function buildBridge(world: World, g: GameV): void {
  g.bridge = true;
  g.building = 0;
  g.builtAt = g.time;
  g.wood -= BRIDGE_COST.wood;
  g.stone -= BRIDGE_COST.stone;
  for (const b of bridgeParts(world, g)) {
    must(world.set(b.entity, og.Mesh, { visible: true }));
    const at = must(world.get(b.entity, og.Transform)).position;
    must(og.tween.to(world, { entity: b.entity, field: "position", from: [at.x, b.y - 1.6, at.z], to: [at.x, b.y, at.z], duration: 0.35, delay: b.order * 0.05, ease: "backOut" }));
  }
  for (const d of deckParts(world, g)) {
    const at = must(world.get(d.entity, og.Transform)).position;
    must(world.set(d.entity, og.Transform, { position: { x: at.x, y: d.y, z: at.z } }));
  }
  // What the bridge took counts too, however it reached the bag.
  og.story.progress(world, "bridge", "wood", BRIDGE_COST.wood);
  og.story.progress(world, "bridge", "stone", BRIDGE_COST.stone);
  og.story.progress(world, "bridge", "span", 1);
  g.sfx |= Sound.Build;
  if (!prefs.reducedMotion) must(og.camera.shake(world, 0.25));
}

/** Site i is claimed: the flag rises in its colours, the brazier catches, sparks, a punch. */
export function claim(world: World, g: GameV, i: number): void {
  must(world.set(part(world, `site-${i}`), Site, { claimed: true, claimedAt: g.time, seen: true }));
  g.claimed += 1;
  g.claimMask |= 1 << i;
  g.seenMask |= 1 << i;
  const at = standardOf(i);
  const h = ground(world, g, at.x, at.z).height;
  const flag = part(world, `site-${i}-flag`);
  must(world.set(flag, og.Model, { tint: FLAG_TINT }));
  must(og.tween.to(world, { entity: flag, field: "position.y", from: [h + FLAG.down], to: [h + FLAG.up - 1.4], duration: 1.4, ease: "cubicOut" }));
  for (const p of ["fire", "smoke", "burst"]) must(world.set(part(world, `site-${i}-${p}`), og.ParticleSystem, { visible: true, startTick: world.tick }));
  must(og.tween.to(world, { entity: part(world, `site-${i}-light`), component: og.PointLight, field: "intensity", from: [0], to: [9], duration: 0.6, ease: "cubicOut" }));
  og.story.progress(world, "lights", "sites", 1);
  must(og.render.floatingText(world, { text: `${g.claimed}/${SITE_COUNT}`, position: { x: at.x, y: h + 5, z: at.z }, size: 0.9, color: { r: 1, g: 0.88, b: 0.55 }, outline: 0.3, rise: 1.6, lifetime: 2, facing: "camera" }));
  g.sfx |= Sound.Claim;
  if (!prefs.reducedMotion) must(og.camera.punch(world, 0.35));
  if (g.claimed >= SITE_COUNT) {
    g.ending = Ending.Lit;
    g.sfx |= Sound.Dawn;
  } else markNext(world, g);
}

/** The chips' two emitters (wood and stone), spawned once in `start`. */
export function spawnChips(world: World): void {
  for (const [name, seed] of [["chips-wood", 7], ["chips-stone", 9]] as const) {
    const e = must(world.spawn({ name, position: { x: 0, y: -50, z: 0 } }));
    must(world.insert(e, og.ParticleSystem, { emitter: must(og.load.emitter(world, `fx/${name}.ogt`)), seed, visible: false }));
  }
}

/**
 * Per tick: the run's start puts the isle back; a new dawn grows the woods
 * back; each explorer's Gather works its target; each explorer near a
 * landmark discovers or claims it.
 */
export const Work = defineSystem(
  "Work",
  { reads: [Tuning, Hero], writes: [Game, Hero, Node, Site, og.Model, og.Transform, og.ParticleSystem, og.PointLight, og.Mesh], after: ["Hero"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    if (g.start) resetIsle(world, g);
    if (g.dawn) {
      for (let i = 0; i < NODES.length; i++) regrow(world, i);
      g.dawn = false;
    }
    const dt = time.scaledDt;
    if (dt === 0 || g.phase !== Phase.Play || g.ending !== Ending.None) return;
    const t = lazyTuning(world);
    const work = new Map<number, number>();
    let building = 0;
    for (let seat = 0; seat < SEATS; seat++) {
      const hero = heroOf(g, seat);
      const me = must(world.get(hero, Hero));
      const p = must(world.get(hero, og.Transform)).position;
      // The landmarks: seen within sight, claimed within reach.
      for (let i = 0; i < SITE_COUNT; i++) {
        const s = SITES[i]!;
        const d = hypot(s.x - p.x, s.z - p.z);
        if (d < t.sight && (g.seenMask & (1 << i)) === 0) {
          g.seenMask |= 1 << i;
          must(world.set(part(world, `site-${i}`), Site, { seen: true }));
          g.sfx |= Sound.Discover;
        }
        if (d < t.claimRadius && (g.claimMask & (1 << i)) === 0 && g.ending === Ending.None) claim(world, g, i);
      }
      if (!me.gathering) {
        if (me.workClock !== 0) must(world.set(hero, Hero, { workClock: 0 }));
        continue;
      }
      const canBuild = me.target === Target.Bridge && g.wood >= BRIDGE_COST.wood && g.stone >= BRIDGE_COST.stone;
      if (me.target === Target.Node) work.set(me.node, (work.get(me.node) ?? 0) + 1);
      else if (canBuild) building += 1;
      if (me.target === Target.Node || canBuild) {
        // A stroke at the first tick of the work and every STROKE seconds after.
        const clock = me.workClock === 0 ? STROKE : me.workClock + dt;
        if (clock >= STROKE) {
          if (me.target === Target.Node) stroke(world, g, me.node);
          else g.sfx |= Sound.Chop;
        }
        must(world.set(hero, Hero, { workClock: clock >= STROKE ? clock - STROKE + 1e-4 : clock }));
      }
    }
    // Every node worked this tick gains each worker's time; one left alone
    // a tick loses its work (its progress counts from 0 when work starts again).
    for (const [i, n] of work) {
      const e = nodeOf(world, i);
      const row = must(world.get(e, Node));
      const progress = (row.workedTick + 1 === world.tick ? row.progress : 0) + dt * n;
      if (progress >= (row.kind === Kind.Tree ? t.chopSeconds : t.breakSeconds)) gathered(world, g, i);
      else must(world.set(e, Node, { progress, workedTick: world.tick }));
    }
    if (!g.bridge) {
      g.building = building > 0 ? g.building + dt * building : 0;
      if (g.building >= t.buildSeconds) buildBridge(world, g);
    }
  },
);
