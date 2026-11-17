// The market alley as a blockout: the ground, the stall, the buildings on
// both sides, the bridge and the gate at the alley's end, the towers
// behind, the courier and the hover-bike, each a scaled primitive until its
// model replaces it. The alley runs down -Z from the camera at the origin's
// near side; +X is the camera's right.
import { og } from "@origingame/engine";
import type { Color, Entity, World } from "@origingame/engine";
import { hex, mat, must, pitch, place, rgb, scaled, shapes, v3, yaw } from "./util.ts";
import type { Shapes } from "./util.ts";

/** The neon palette of the concept: magenta, cyan, amber and lantern orange. */
export const NEON = {
  magenta: hex("#ff2fb4"),
  pink: hex("#ff5a8a"),
  cyan: hex("#22e6ff"),
  amber: hex("#ffb02e"),
  lantern: hex("#ff7a22"),
  red: hex("#ff3b30"),
  violet: hex("#8a4dff"),
} as const;

/** A glowing panel with the rect light it throws: a sign on a wall facing `facing` degrees about +Y (0 faces +Z, the camera). */
function sign(world: World, s: Shapes, name: string, at: { x: number; y: number; z: number }, w: number, h: number, color: Color, facing: number, power = 3.5): Entity {
  const panel = mat(world, { color: rgb(0.02, 0.02, 0.02), emissive: color, emissiveIntensity: power, roughness: 0.4, weathering: 0 });
  const e = place(world, s.quad, panel, { name, at: v3(at.x, at.y, at.z), size: v3(w, h, 1), turn: yaw(facing), shadow: false });
  // The rect light faces along its local -Z: turned half a turn from the panel, a hand in front of it.
  const light = must(world.spawn({ name: `${name}-light`, parent: e, position: v3(0, 0, 0.05), quaternion: yaw(180) }));
  must(world.insert(light, og.RectLight, { color, intensity: power * 1.2, width: w, height: h }));
  return e;
}

/** A paper lantern: a glowing ball and the warm point light inside it. */
function lantern(world: World, s: Shapes, name: string, x: number, y: number, z: number, color: Color = NEON.lantern, size = 0.55): Entity {
  const paper = mat(world, { color: scaled(color, 0.4), emissive: color, emissiveIntensity: 3, roughness: 0.9, weathering: 0 });
  const e = place(world, s.sphere, paper, { name, at: v3(x, y, z), size: v3(size, size * 1.15, size), shadow: false });
  must(world.insert(e, og.PointLight, { color, intensity: 4, distance: 8 }));
  return e;
}

export function buildSet(world: World): void {
  const s = shapes(world);
  const asphalt = mat(world, { color: rgb(0.035, 0.035, 0.04), roughness: 0.75, metalness: 0 });
  const concrete = mat(world, { color: rgb(0.09, 0.085, 0.09), roughness: 0.85 });
  const darkWall = mat(world, { color: rgb(0.05, 0.045, 0.055), roughness: 0.8 });
  const wood = mat(world, { color: rgb(0.11, 0.06, 0.035), roughness: 0.7 });
  const tarp = mat(world, { color: rgb(0.16, 0.05, 0.04), roughness: 0.6 });
  const steel = mat(world, { color: rgb(0.35, 0.35, 0.36), roughness: 0.35, metalness: 1 });
  const tower = mat(world, { color: rgb(0.02, 0.02, 0.03), roughness: 0.5, metalness: 0.4 });

  // The street: 28 m wide, running 90 m down the alley.
  place(world, s.quad, asphalt, { name: "street", at: v3(0, 0, -30), size: v3(28, 90, 1), turn: pitch(-90), shadow: false });

  // Left: the stall under its awning, the buildings stepping back behind it.
  place(world, s.box, darkWall, { name: "left-block-a", at: v3(-10.5, 7, -2), size: v3(8, 14, 12) });
  place(world, s.box, concrete, { name: "left-block-b", at: v3(-9.5, 9, -14), size: v3(8, 18, 10) });
  place(world, s.box, darkWall, { name: "left-block-c", at: v3(-8.5, 6, -25), size: v3(8, 12, 10) });
  place(world, s.box, wood, { name: "stall-counter", at: v3(-4.6, 0.55, -1.5), size: v3(3.4, 1.1, 5.5) });
  place(world, s.box, wood, { name: "stall-back", at: v3(-6.3, 1.6, -1.5), size: v3(0.2, 3.2, 6) });
  place(world, s.box, tarp, { name: "stall-awning", at: v3(-4.4, 3.25, -1.5), size: v3(4.6, 0.08, 6.6), turn: og.transform.fromAxisAngle(v3(0, 0, 1), -0.2) });
  for (const z of [1.6, -4.6]) place(world, s.cylinder, wood, { name: `stall-post-${z}`, at: v3(-2.6, 1.5, z), size: v3(0.12, 3, 0.12) });
  for (const [x, z] of [[-4.9, -0.2], [-4.3, -2.6]] as const) place(world, s.cylinder, steel, { name: `stall-pot-${z}`, at: v3(x, 1.4, z), size: v3(0.9, 0.6, 0.9) });
  place(world, s.cylinder, mat(world, { color: rgb(0.03, 0.03, 0.03), roughness: 0.4, metalness: 0.6 }), { name: "bin", at: v3(-2.2, 0.5, 2.6), size: v3(0.8, 1, 0.8) });
  // The stall's warm interior: a strip lamp under the awning.
  const strip = must(world.spawn({ name: "stall-lamp", position: v3(-4.6, 2.9, -1.5), quaternion: pitch(-90) }));
  must(world.insert(strip, og.RectLight, { color: NEON.amber, intensity: 2.5, width: 2.4, height: 4.5 }));
  // The stall's glow spilling over the street: the courier's warm key.
  const spill = must(world.spawn({ name: "stall-spill", position: v3(-2.4, 2.4, 1.5), quaternion: og.transform.lookAt(v3(-2.4, 2.4, 1.5), v3(2.2, 0.8, 3.2)) }));
  must(world.insert(spill, og.SpotLight, { color: NEON.amber, intensity: 26, distance: 14, angle: 0.75, penumbra: 0.7, castShadow: true }));
  lantern(world, s, "lantern-stall-a", -2.7, 2.7, 1.2);
  lantern(world, s, "lantern-stall-b", -2.5, 2.5, -3.2, NEON.lantern, 0.45);
  lantern(world, s, "lantern-stall-c", -2.7, 2.3, -6.4, NEON.lantern, 0.4);

  // Right: the buildings, their signs, the lanterns under the blossom.
  place(world, s.box, darkWall, { name: "right-block-a", at: v3(11, 8, 0), size: v3(8, 16, 12) });
  place(world, s.box, concrete, { name: "right-block-b", at: v3(10, 10, -13), size: v3(8, 20, 10) });
  place(world, s.box, darkWall, { name: "right-block-c", at: v3(9, 6.5, -25), size: v3(8, 13, 10) });
  lantern(world, s, "lantern-right-a", 5.4, 3.3, -1.5, NEON.red, 0.6);
  lantern(world, s, "lantern-right-b", 5.0, 6.6, -9, NEON.red, 0.5);
  sign(world, s, "sign-cyan-tall", { x: 5.95, y: 7.6, z: -9.5 }, 1.5, 3.6, NEON.cyan, -90);
  sign(world, s, "sign-magenta-shop", { x: 6.95, y: 2.6, z: -3 }, 1.4, 1.4, NEON.magenta, -90);
  sign(world, s, "sign-magenta-tall", { x: -5.45, y: 8.2, z: -12 }, 1.9, 4.2, NEON.magenta, 90);
  sign(world, s, "sign-cyan-small", { x: -5.45, y: 6.6, z: -17 }, 0.8, 2, NEON.cyan, 90);
  sign(world, s, "sign-amber", { x: -4.45, y: 4.3, z: -20.2 }, 1.6, 1.6, NEON.amber, 60);

  // The alley's end: a footbridge, the gate under it, the lit street beyond.
  place(world, s.box, concrete, { name: "bridge", at: v3(0, 5, -19), size: v3(14, 0.7, 2.2) });
  const gate = mat(world, { color: rgb(0.25, 0.03, 0.03), emissive: NEON.red, emissiveIntensity: 1.2, roughness: 0.5 });
  for (const x of [-2.4, 2.4]) place(world, s.box, gate, { name: `gate-post-${x}`, at: v3(x, 2, -24), size: v3(0.35, 4, 0.35) });
  place(world, s.box, gate, { name: "gate-beam", at: v3(0, 4.1, -24), size: v3(6.2, 0.35, 0.45) });
  sign(world, s, "sign-gate", { x: 0, y: 3.2, z: -24.2 }, 2.6, 0.7, NEON.magenta, 0);

  // The towers behind, fading into the fog: dark slabs with lit edges.
  const towers: readonly (readonly [number, number, number, number, number, Color])[] = [
    [-2, -80, 16, 70, 16, NEON.magenta], [-22, -70, 14, 52, 14, NEON.cyan], [18, -75, 15, 60, 15, NEON.violet],
    [6, -120, 20, 110, 20, NEON.cyan], [-34, -110, 18, 85, 18, NEON.magenta], [36, -105, 18, 78, 18, NEON.pink],
    [-12, -150, 22, 130, 22, NEON.violet], [24, -160, 24, 140, 24, NEON.magenta],
  ];
  for (const [i, [x, z, w, h, d, c]] of towers.entries()) {
    place(world, s.box, tower, { name: `tower-${i}`, at: v3(x, h / 2, z), size: v3(w, h, d), shadow: false });
    const edge = mat(world, { color: rgb(0, 0, 0), emissive: c, emissiveIntensity: 3, weathering: 0 });
    place(world, s.box, edge, { name: `tower-${i}-strip`, at: v3(x - w * 0.3, h * 0.55, z + d / 2 + 0.1), size: v3(0.5, h * 0.5, 0.1), shadow: false });
    place(world, s.box, edge, { name: `tower-${i}-band`, at: v3(x, h * 0.82, z + d / 2 + 0.1), size: v3(w * 0.7, 0.6, 0.1), shadow: false });
  }

  // The courier, the pack on his back, and the hover-bike beside him.
  const jacket = mat(world, { color: hex("#c9731c"), roughness: 0.55 });
  const courier = must(world.spawn({ name: "courier", position: v3(1.55, 0, 3.3) }));
  place(world, s.capsule, jacket, { name: "courier-body", parent: courier, at: v3(0, 0.95, 0), size: v3(0.62, 0.95, 0.42) });
  place(world, s.sphere, jacket, { name: "courier-hood", parent: courier, at: v3(0, 1.72, 0), size: v3(0.36, 0.4, 0.36) });
  const packShell = mat(world, { color: hex("#3a3024"), roughness: 0.5, metalness: 0.3 });
  place(world, s.box, packShell, { name: "pack", parent: courier, at: v3(0, 1.25, 0.34), size: v3(0.62, 0.72, 0.34) });
  const glow = mat(world, { color: rgb(0, 0, 0), emissive: NEON.cyan, emissiveIntensity: 5, weathering: 0 });
  for (const x of [-0.27, 0.27]) place(world, s.box, glow, { name: `pack-strip-${x}`, parent: courier, at: v3(x, 1.25, 0.515), size: v3(0.035, 0.5, 0.02), shadow: false });
  const packLight = must(world.spawn({ name: "pack-light", parent: courier, position: v3(0, 1.25, 0.8) }));
  must(world.insert(packLight, og.PointLight, { color: NEON.cyan, intensity: 0.8, distance: 4 }));

  const bike = must(world.spawn({ name: "bike", position: v3(4.3, 0, 3.4), quaternion: yaw(-62) }));
  const hull = mat(world, { color: hex("#8a3414"), roughness: 0.4, metalness: 0.5 });
  place(world, s.box, hull, { name: "bike-hull", parent: bike, at: v3(0, 0.85, 0), size: v3(1.15, 0.6, 3.1) });
  place(world, s.box, packShell, { name: "bike-nose", parent: bike, at: v3(0, 0.7, 1.75), size: v3(0.95, 0.4, 0.9) });
  for (const z of [-1.0, 0.2, 1.4]) place(world, s.cylinder, steel, { name: `bike-pad-${z}`, parent: bike, at: v3(0, 0.34, z), size: v3(1.1, 0.12, 1.1) });
  for (const x of [-0.59, 0.59]) place(world, s.box, glow, { name: `bike-strip-${x}`, parent: bike, at: v3(x, 0.8, 0.3), size: v3(0.02, 0.05, 1.5), shadow: false });
  const under = must(world.spawn({ name: "bike-underglow", parent: bike, position: v3(0, 0.22, 0.2) }));
  must(world.insert(under, og.PointLight, { color: NEON.cyan, intensity: 4, distance: 4 }));

  // Pedestrians down the alley, each under an umbrella.
  const coat = mat(world, { color: rgb(0.04, 0.04, 0.05), roughness: 0.6 });
  const canopy = mat(world, { color: rgb(0.03, 0.03, 0.05), roughness: 0.35 });
  for (const [i, [x, z]] of ([[-1.2, -5], [0.4, -8.5], [-0.6, -10], [1.6, -12.5], [-1.9, -15]] as const).entries()) {
    const p = must(world.spawn({ name: `pedestrian-${i}`, position: v3(x, 0, z) }));
    place(world, s.capsule, coat, { name: `pedestrian-${i}-body`, parent: p, at: v3(0, 0.9, 0), size: v3(0.5, 0.9, 0.36) });
    place(world, s.cone, canopy, { name: `pedestrian-${i}-umbrella`, parent: p, at: v3(0, 2.05, 0), size: v3(1.25, 0.3, 1.25) });
  }

  // Craft over the alley: a hull and its running light.
  for (const [i, [x, y, z]] of ([[2.5, 19, -40], [-1.5, 14, -52], [5, 11, -34]] as const).entries()) {
    const craft = place(world, s.sphere, tower, { name: `craft-${i}`, at: v3(x, y, z), size: v3(3.4, 0.7, 2.2), shadow: false });
    const lamp = mat(world, { color: rgb(0, 0, 0), emissive: i === 1 ? NEON.amber : NEON.cyan, emissiveIntensity: 10, weathering: 0 });
    place(world, s.sphere, lamp, { name: `craft-${i}-lamp`, parent: craft, at: v3(0, -0.5, 0.3), size: v3(0.12, 0.3, 0.16), shadow: false });
  }
}
