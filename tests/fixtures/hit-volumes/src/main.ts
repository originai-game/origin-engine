// og.play (README.md): a walker spun about its feet half a radian a tick,
// with an og.HitVolume on its left hand (og.BoneAttachment on hand.L), so
// the hand moves farther in a tick than the volume is thick. Two thin
// dummies (og.HurtVolume) and a thin wall collider stand on its hand's
// circle: only a sweep between ticks meets them. Each turn is a swing, so
// each is struck once a turn. Two levers stand around it, one behind the
// wall, for og.play.candidates.
//
//   og run tests/fixtures/hit-volumes --commands "3:quit"
import { og, defineComponent, defineResource, defineSystem, f32, u32, vec3 } from "@origingame/engine";
import type { World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Radians the walker turns each tick. */
export const SPEED = 0.5;
const Y = { x: 0, y: 1, z: 0 };

/** A game's interactable: what og.play.candidates looks for. */
const Lever = defineComponent("Lever", { pulls: u32() });

/** What the fixture counted: hits per target, the turns, what the levers' candidates said. */
const Tally = defineResource("Tally", {
  dummyA: u32(),
  dummyB: u32(),
  wall: u32(),
  turns: u32(),
  angle: f32(),
  /** Each target's last swing struck, plus one (0: never). */
  lastA: u32(),
  lastB: u32(),
  lastWall: u32(),
  /** Hits of a target already struck in that swing: og.play reports each once, so 0. */
  dups: u32(),
  leversSeen: u32(),
  leversHidden: u32(),
  nearestLever: u32(),
});

/** A thin radial slab on the hand's circle at `angle`, 1.6 m out from the axis and 2 m tall. */
function slab(angle: number): { position: { x: number; y: number; z: number }; quaternion: { x: number; y: number; z: number; w: number } } {
  const q = og.transform.fromAxisAngle(Y, -angle);
  const out = vec3.applyQuaternion({ x: 0.95, y: 1, z: 0 }, q);
  return { position: out, quaternion: q };
}

function build(world: World): void {
  const walker = must(og.load.animation(world, "models/walker.glb"));
  const player = must(world.spawn({ name: "player" }));
  must(world.insert(player, og.Skeleton, { asset: walker.skeleton, rootMotion: "off" }));
  must(world.insert(player, og.AnimationPlayer, { clip: walker.clips[0]!.clip, speed: 1 }));
  const blade = must(world.spawn({ name: "blade", parent: player }));
  must(world.insert(blade, og.BoneAttachment, { node: walker.nodes.indexOf("hand.L") }));
  must(world.insert(blade, og.HitVolume, { shape: "capsule", radius: 0.04, height: 0.25, active: true, team: 1, owner: player, colliders: 2 }));

  for (const [name, angle] of [["dummyA", Math.PI / 2], ["dummyB", (5 * Math.PI) / 4]] as const) {
    const at = slab(angle);
    const e = must(world.spawn({ name, position: at.position, quaternion: at.quaternion }));
    must(world.insert(e, og.HurtVolume, { shape: "box", size: { x: 1.6, y: 2, z: 0.02 }, team: 2 }));
  }
  // The wall: a static collider on layer 2, between the player and one lever.
  const at = slab(Math.PI);
  const wall = must(world.spawn({ name: "wall", position: at.position, quaternion: at.quaternion }));
  must(world.insert(wall, og.Collider, { shape: "box", size: { x: 1.6, y: 2, z: 0.02 }, layers: 2, mask: 0xffffffff }));

  const lever = (name: string, position: { x: number; y: number; z: number }) => {
    const e = must(world.spawn({ name, position }));
    must(world.insert(e, Lever, {}));
  };
  lever("leverAhead", { x: 0, y: 1, z: -3 });
  lever("leverBehindWall", { x: -3, y: 1, z: 0 });
}

/** The turn: the walker's rotation, and a new swing at each whole turn. */
const Spin = defineSystem("Spin", { writes: [og.Transform, og.HitVolume, Tally] }, (_q, time, world) => {
  const tally = must(world.resource(Tally));
  const angle = time.tick * SPEED;
  tally.angle = angle;
  tally.turns = Math.floor(angle / (2 * Math.PI));
  must(world.set(world.find("player")!, og.Transform, { quaternion: og.transform.fromAxisAngle(Y, angle) }));
  must(world.set(world.find("blade")!, og.HitVolume, { swing: tally.turns }));
});

/** The hits og.play sent last tick, by target. */
const Count = defineSystem("Count", { reads: [og.HitEvent, Lever], writes: [Tally] }, (_q, _t, world) => {
  const tally = must(world.resource(Tally));
  for (const hit of world.events(og.HitEvent)) {
    const once = (last: number) => {
      if (last === hit.swing + 1) tally.dups += 1;
      return hit.swing + 1;
    };
    if (hit.target === world.find("dummyA")) { tally.dummyA += 1; tally.lastA = once(tally.lastA); }
    if (hit.target === world.find("dummyB")) { tally.dummyB += 1; tally.lastB = once(tally.lastB); }
    if (hit.target === world.find("wall")) { tally.wall += 1; tally.lastWall = once(tally.lastWall); }
    og.assert.reachable("a dummy is struck");
  }
  const found = must(og.play.candidates(world, { from: { x: 0, y: 1, z: 0 }, forward: { x: 0, y: 0, z: -1 }, range: 5, with: "Lever", sight: true }));
  tally.leversSeen = found.filter((c) => c.visible).length;
  tally.leversHidden = found.filter((c) => !c.visible).length;
  tally.nearestLever = found[0]?.entity === world.find("leverAhead") ? 1 : 0;
});

/** What the fixture counted, as `og test`'s `state:` reads it and the cross-host test compares it. */
export function facts(world: World) {
  const t = must(world.resource(Tally));
  return {
    dummyA: t.dummyA,
    dummyB: t.dummyB,
    wall: t.wall,
    turns: t.turns,
    // At most once a turn, counting the turn under way.
    cap: t.turns + 1,
    dups: t.dups,
    leversSeen: t.leversSeen,
    leversHidden: t.leversHidden,
    nearestLever: t.nearestLever,
  };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Tally],
  systems: [Spin, Count],
  start: build,
  facts,
});
