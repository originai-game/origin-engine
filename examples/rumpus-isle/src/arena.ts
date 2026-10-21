// The isle, built in start: the camera and the sun, the sea (og.Water with an
// og.WaterSurface, so a critter that falls in splashes it), the island's body
// (kinematic: the crumble round sinks it out of the way), the four critters
// (off stage until someone sits in their seat), the crumble round's tiles,
// the berry round's berries, the podium and the effects' emitters.

import { og, BODY_DYNAMIC, BODY_KINEMATIC, LOCK_ROTATION_X, LOCK_ROTATION_Z, SHAPE_BOX, SHAPE_CYLINDER, SHAPE_SPHERE } from "@origingame/engine";
import type { Entity, Material, World } from "@origingame/engine";
import { Berry, Bits, COLOURS, Critter, Effect, MAX_PLAYERS, Tile } from "./components.ts";
import { linear, must, named, yawQuat } from "./util.ts";

/** The island's top radius (its grass and rim), m. */
export const ISLAND_R = 6.9;
/** The tiles: GRID × GRID, PITCH apart, centred on the origin. */
export const GRID = 6;
export const PITCH = 1.55;
/** Berries in the ring (up to Tuning.berriesUp shown at once). */
export const BERRIES = 10;
/** Where things wait off stage. */
export const PARK_Y = -40;
export const ISLAND_PARK_Y = -30;
export const SEA_Y = -0.62;

/** Where seat k starts a round: on a circle about the centre, facing it. */
export function startOf(k: number, radius = 3.4): { x: number; z: number; yaw: number } {
  const a = Math.PI / 4 + (k * Math.PI) / 2;
  const x = Math.cos(a) * radius, z = Math.sin(a) * radius;
  return { x, z, yaw: Math.atan2(-x, -z) };
}

/** Where seat k waits while nobody sits there, or out of a round. */
export function parkOf(k: number): { x: number; y: number; z: number } {
  return { x: -6 + k * 4, y: PARK_Y, z: 30 };
}

export const CAMERA = {
  play: { eye: { x: 0, y: 15.2, z: 11.6 }, at: { x: 0, y: 0, z: 0.9 } },
  title: { eye: { x: 0, y: 7.5, z: 15.5 }, at: { x: 0, y: 0.6, z: 0 } },
  podium: { eye: { x: 0, y: 4.2, z: 8.6 }, at: { x: 0, y: 1.5, z: -1 } },
} as const;

/** The podium's steps: where first, second and third stand (on the island, its front to the camera). */
export const PODIUM = { x: 0, z: -1.2, steps: [{ x: 0, y: 1.2 }, { x: -1.7, y: 0.8 }, { x: 1.7, y: 0.5 }] } as const;

function sphere(world: World, radius: number) {
  return must(og.render.sphereGeometry(world, { radius, widthSegments: 24, heightSegments: 16 }));
}

/** A child entity drawing one mesh. */
function part(world: World, parent: Entity, name: string, geometry: ReturnType<typeof sphere>, material: Material, position: { x: number; y: number; z: number }, scale = { x: 1, y: 1, z: 1 }): Entity {
  const e = must(world.spawn({ name, parent, position }));
  must(world.set(e, og.Transform, { position, scale }));
  must(world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true }));
  return e;
}

export function buildArena(world: World): void {
  const cam = must(world.resource(og.Camera));
  cam.position = CAMERA.title.eye;
  cam.quaternion = og.transform.lookAt(CAMERA.title.eye, CAMERA.title.at);
  cam.fov = 50;
  cam.background = linear("#cfeefc");

  // The sun: from the south-east, high, its shadows short and soft.
  const sun = named(world, "sun");
  must(world.set(sun, og.Transform, { position: { x: 6, y: 14, z: 8 }, quaternion: og.transform.lookAt({ x: 6, y: 14, z: 8 }, { x: 0, y: 0, z: 0 }) }));

  // The sea: a calm lagoon whose surface takes waves and splashes.
  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: SEA_Y, z: 0 } }));
  must(world.insert(sea, og.Water, {
    size: { x: 70, y: 70 },
    shallow: linear("#5fd3d0"),
    deep: linear("#1f6fa8"),
    depthFade: 2.5,
    foam: linear("#ffffff"),
    foamWidth: 0.35,
    waveStrength: 0.25,
    reflectivity: 0.35,
  }));
  must(world.insert(sea, og.WaterSurface, { size: 34, resolution: 64, splash: 0.6, buoyancy: 0.6 }));

  // The island's body: a cylinder under its top (kinematic, so the crumble round can move it away).
  const island = named(world, "island");
  must(world.insert(island, og.RigidBody, { kind: BODY_KINEMATIC }));
  must(world.insert(island, og.Collider, { shape: SHAPE_CYLINDER, radius: ISLAND_R + 0.1, height: 2, offset: { x: 0, y: -1, z: 0 }, friction: 0.6, layers: Bits.Ground, mask: Bits.Critter }));

  // The critters: a body sphere in the seat's colour, eyes, cheeks and feet.
  const ball = sphere(world, 0.5);
  const eye = sphere(world, 0.13);
  const pupil = sphere(world, 0.07);
  const foot = sphere(world, 0.16);
  const white = must(og.render.standardMaterial(world, { color: { r: 0.95, g: 0.95, b: 0.95 }, roughness: 0.35 }));
  const ink = must(og.render.standardMaterial(world, { color: { r: 0.03, g: 0.025, b: 0.05 }, roughness: 0.3 }));
  const cheek = must(og.render.standardMaterial(world, { color: linear("#ff8fa3"), roughness: 0.6 }));
  for (let k = 0; k < MAX_PLAYERS; k++) {
    const colour = COLOURS[k]!;
    const body = must(og.render.standardMaterial(world, { color: linear(colour.rgb), roughness: 0.55, sheen: 0.4, sheenColor: { r: 1, g: 1, b: 1 } }));
    const footMat = must(og.render.standardMaterial(world, { color: linear(colour.rgb), roughness: 0.7 }));
    const p = parkOf(k);
    const c = must(world.spawn({ name: `p${k}`, position: p }));
    must(world.insert(c, og.RigidBody, { kind: BODY_DYNAMIC, mass: 1, lock: LOCK_ROTATION_X | LOCK_ROTATION_Z, linearDamping: 0.4, angularDamping: 2, gravityScale: 0, ccd: 1 }));
    must(world.insert(c, og.Collider, { shape: SHAPE_SPHERE, radius: 0.5, offset: { x: 0, y: 0.5, z: 0 }, friction: 0.25, restitution: 0.35, layers: Bits.Critter, mask: Bits.Ground | Bits.Critter }));
    must(world.insert(c, og.Velocity, {}));
    const mesh = part(world, c, `p${k}-body`, ball, body, { x: 0, y: 0.48, z: 0 }, { x: 1, y: 0.92, z: 1 });
    for (const s of [-1, 1]) {
      part(world, c, `p${k}-eye${s < 0 ? "L" : "R"}`, eye, white, { x: 0.17 * s, y: 0.66, z: 0.36 });
      part(world, c, `p${k}-pupil${s < 0 ? "L" : "R"}`, pupil, ink, { x: 0.17 * s, y: 0.67, z: 0.47 });
      part(world, c, `p${k}-cheek${s < 0 ? "L" : "R"}`, pupil, cheek, { x: 0.3 * s, y: 0.5, z: 0.38 }, { x: 1.2, y: 0.7, z: 0.5 });
      part(world, c, `p${k}-foot${s < 0 ? "L" : "R"}`, foot, footMat, { x: 0.2 * s, y: 0.08, z: 0.12 }, { x: 1, y: 0.55, z: 1.3 });
    }
    must(world.insert(c, Critter, { seat: k, body, mesh, yaw: startOf(k).yaw }));
    must(world.set(c, og.Transform, { position: p, quaternion: yawQuat(startOf(k).yaw) }));
  }

  // The crumble round's tiles, waiting under the sea; tinted in rows of pastel.
  const tile = must(og.load.gltf(world, "models/tile.glb"));
  const tints = ["#ffd6a5", "#fdffb6", "#caffbf", "#9bf6ff", "#bdb2ff", "#ffc6ff"];
  for (let i = 0; i < GRID; i++) {
    for (let j = 0; j < GRID; j++) {
      const x = (i - (GRID - 1) / 2) * PITCH, z = (j - (GRID - 1) / 2) * PITCH;
      const t = must(world.instantiate(tile, { name: `tile-${i}-${j}`, position: { x, y: PARK_Y, z } }));
      must(world.insert(t, og.RigidBody, { kind: BODY_KINEMATIC }));
      must(world.insert(t, og.Collider, { shape: SHAPE_BOX, size: { x: 1.44, y: 0.56, z: 1.44 }, offset: { x: 0, y: -0.28, z: 0 }, friction: 0.6, layers: Bits.Ground, mask: Bits.Critter }));
      must(world.insert(t, Tile, { homeX: x, homeZ: z, down: true }));
      must(world.set(t, og.Model, { tint: linear(tints[(i + j) % tints.length]!) }));
    }
  }

  // The berry ring.
  const berry = must(og.load.gltf(world, "models/berry.glb"));
  for (let b = 0; b < BERRIES; b++) {
    const e = must(world.instantiate(berry, { name: `berry-${b}`, position: { x: b, y: PARK_Y, z: 0 } }));
    must(world.insert(e, Berry, {}));
  }

  // The podium, under the island until the end.
  const podium = must(og.load.gltf(world, "models/podium.glb"));
  must(world.instantiate(podium, { name: "podium", position: { x: PODIUM.x, y: PARK_Y, z: PODIUM.z } }));

  // The effects: one entity per emitter, moved where it plays and restarted (og-game §10).
  const fx: [string, string, number][] = [["fx-splash", "fx/splash.ogt", 0], ["fx-dust", "fx/dust.ogt", 1], ["fx-sparkle", "fx/sparkle.ogt", 2], ["fx-crumbs", "fx/crumbs.ogt", 3]];
  for (const [name, path, kind] of fx) {
    const e = must(world.spawn({ name, position: { x: 0, y: PARK_Y, z: 0 } }));
    must(world.insert(e, og.ParticleSystem, { emitter: must(og.load.emitter(world, path)), seed: kind + 1, visible: false }));
    must(world.insert(e, Effect, { kind }));
  }
  const confetti = must(world.spawn({ name: "fx-confetti", position: { x: PODIUM.x, y: 6, z: PODIUM.z } }));
  must(world.insert(confetti, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/confetti.ogt")), seed: 9, visible: false }));
  // Confetti falls: the emitter points down.
  must(world.set(confetti, og.Transform, { quaternion: { x: 1, y: 0, z: 0, w: 0 } }));
}
