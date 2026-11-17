// The market alley: the street, the stall, the tenements on both sides,
// the bridge and the gate at the alley's end and the towers behind (the
// cast is in cast.ts). The alley runs down -Z from the camera at the origin's
// near side; +X is the camera's right.
import { og } from "@origingame/engine";
import type { Color, Entity, World } from "@origingame/engine";
import { NEON } from "./palette.ts";
import { mat, must, pitch, place, rgb, shapes, tex, v3, yaw } from "./util.ts";
import type { Shapes } from "./util.ts";

/** A neon sign with the rect light it throws: `art` (a glyph on black, under textures/signs/) glowing on a panel on a wall facing `facing` degrees about +Y (0 faces +Z, the camera). */
function sign(world: World, s: Shapes, name: string, art: string, at: { x: number; y: number; z: number }, w: number, h: number, color: Color, facing: number, power = 3.5): Entity {
  const glyph = tex(world, `textures/signs/${art}.png`);
  const panel = mat(world, { color: rgb(0.02, 0.02, 0.02), emissive: rgb(1, 1, 1), emissiveMap: glyph, emissiveIntensity: power * 2.2, roughness: 0.4, weathering: 0 });
  const e = place(world, s.quad, panel, { name, at: v3(at.x, at.y, at.z), size: v3(w, h, 1), turn: yaw(facing), shadow: false });
  // The rect light faces along its local -Z: turned half a turn from the panel, a hand in front of it.
  const light = must(world.spawn({ name: `${name}-light`, parent: e, position: v3(0, 0, 0.05), quaternion: yaw(180) }));
  must(world.insert(light, og.RectLight, { color, intensity: power * 1.2, width: w, height: h }));
  return e;
}

/** A paper lantern hanging at its hook, lit from inside, and the warm point light it throws. The model is 1.8 units tall about its centre; `size` is its width in metres. */
function lantern(world: World, name: string, x: number, y: number, z: number, color: Color = NEON.lantern, size = 0.55): Entity {
  const k = size / 1.32;
  const e = must(world.instantiate(must(og.load.gltf(world, "models/lantern.glb", { castShadow: false })), { name, position: v3(x, y, z), scale: v3(k, k, k) }));
  const light = must(world.spawn({ name: `${name}-light`, parent: e, position: v3(0, 0, 0) }));
  must(world.insert(light, og.PointLight, { color, intensity: 7, distance: 9 }));
  return e;
}

export function buildSet(world: World): void {
  const s = shapes(world);
  const concreteArm = tex(world, "textures/brushed_concrete_arm_1k.jpg");
  const span = { x: 3, y: 1 };
  const concrete = mat(world, {
    map: tex(world, "textures/brushed_concrete_diff_1k.jpg"), color: rgb(0.45, 0.44, 0.47),
    normalMap: tex(world, "textures/brushed_concrete_nor_gl_1k.jpg"), aoMap: concreteArm, roughnessMap: concreteArm, roughness: 1,
    mapRepeat: span, normalMapRepeat: span, metalnessMapRepeat: span, aoMapRepeat: span,
  });
  const tower = mat(world, { color: rgb(0.02, 0.02, 0.03), roughness: 0.5, metalness: 0.4 });

  // The street: 28 m wide, running 90 m down the alley, in setts three
  // metres a tile. Their occlusion map is the relief the rain reads: the
  // joints sheet with water and the crowns drain.
  const tile = { x: 28 / 3, y: 90 / 3 };
  const arm = tex(world, "textures/cobblestone_01_arm_1k.jpg");
  const setts = mat(world, {
    map: tex(world, "textures/cobblestone_01_diff_1k.jpg"), color: rgb(0.55, 0.55, 0.6),
    normalMap: tex(world, "textures/cobblestone_01_nor_gl_1k.jpg"),
    aoMap: arm, roughnessMap: arm, roughness: 1, metalness: 0,
    mapRepeat: tile, normalMapRepeat: tile, metalnessMapRepeat: tile, aoMapRepeat: tile,
  });
  place(world, s.quad, setts, { name: "street", at: v3(0, 0, -30), size: v3(28, 90, 1), turn: pitch(-90), shadow: false });

  // The buildings and the stall are models, each about its centre with its
  // front on +Z: `front` is the way that face looks, in degrees about +Y
  // (90 looks along +X, across the alley from the left side).
  const model = (name: string, path: string, scale: number, half: number, x: number, z: number, front: number): Entity =>
    must(world.instantiate(must(og.load.gltf(world, path)), { name, position: v3(x, half * scale, z), quaternion: yaw(front), scale: v3(scale, scale, scale) }));
  const blockA = (name: string, x: number, z: number, front: number, scale = 11.6): Entity => model(name, "models/block-a.glb", scale, 0.95, x, z, front);
  const blockB = (name: string, x: number, z: number, front: number, scale = 8.4): Entity => model(name, "models/block-b.glb", scale, 0.95, x, z, front);

  // Left: the stall at the alley's mouth, the tenements stepping back behind it.
  model("stall", "models/stall.glb", 2.3, 0.615, -3.7, 3.1, 78);
  blockA("left-block-a", -11.4, -1.5, 90);
  blockB("left-block-b", -8.6, -14, 90);
  blockA("left-block-c", -9.6, -27, 90, 10.2);
  // Steam off its two pots.
  const steam = must(og.load.emitter(world, "fx/steam.ogt"));
  for (const [i, [x, z]] of ([[-3.47, 4.21], [-3.35, 3.65]] as const).entries()) {
    const pot = must(world.spawn({ name: `stall-steam-${i}`, position: v3(x, 1.55, z) }));
    must(world.insert(pot, og.ParticleSystem, { emitter: steam, seed: 21 + i }));
  }
  // The stall's warm interior: a strip lamp under the awning.
  const strip = must(world.spawn({ name: "stall-lamp", position: v3(-3.6, 2.35, 3.1), quaternion: pitch(-90) }));
  must(world.insert(strip, og.RectLight, { color: NEON.amber, intensity: 4, width: 1.6, height: 3.6 }));
  // The stall's glow spilling over the street: the courier's warm key.
  const spill = must(world.spawn({ name: "stall-spill", position: v3(-2.0, 2.3, 3.6), quaternion: og.transform.lookAt(v3(-2.0, 2.3, 3.6), v3(0.9, 0.9, 4.7)) }));
  must(world.insert(spill, og.SpotLight, { color: NEON.amber, intensity: 40, distance: 14, angle: 0.75, penumbra: 0.7, castShadow: true }));
  lantern(world, "lantern-stall-a", -2.1, 2.7, 4.3, NEON.lantern, 0.42);
  lantern(world, "lantern-stall-b", -2.3, 2.45, 1.9, NEON.lantern, 0.45);
  lantern(world, "lantern-stall-c", -2.9, 2.6, -1.2, NEON.lantern, 0.4);

  // Right: the shophouse, the tenements behind it, their signs and lanterns.
  blockB("right-block-a", 9.4, -2.5, -90);
  blockA("right-block-b", 9.6, -15, -90);
  blockB("right-block-c", 8.6, -27.5, -90, 9.2);
  lantern(world, "lantern-right-a", 4.2, 3.4, -0.5, NEON.red, 0.6);
  lantern(world, "lantern-right-b", 3.6, 6.6, -9, NEON.red, 0.5);
  sign(world, s, "sign-cyan-tall", "ring", { x: 3.75, y: 8.2, z: -11.5 }, 1.8, 3.6, NEON.cyan, -90);
  sign(world, s, "sign-magenta-shop", "bowl", { x: 4.55, y: 5.4, z: -4.5 }, 1.6, 1.6, NEON.magenta, -90);
  sign(world, s, "sign-magenta-tall", "chevrons", { x: -5.55, y: 8.6, z: -4 }, 2.1, 4.2, NEON.magenta, 90);
  sign(world, s, "sign-cyan-small", "ring", { x: -3.75, y: 6.6, z: -12.5 }, 1, 2, NEON.cyan, 90);
  sign(world, s, "sign-amber", "triangle", { x: -3.7, y: 4.3, z: -17.2 }, 1.6, 1.6, NEON.amber, 60);

  // The alley's end: a footbridge, the gate under it, the lit street beyond.
  place(world, s.box, concrete, { name: "bridge", at: v3(0, 5.6, -20.5), size: v3(9, 0.7, 2.2) });
  // The gate model is 1.9 units wide about its centre: 6.4 m wide here, its feet on the street.
  const gate = must(og.load.gltf(world, "models/gate.glb"));
  must(world.instantiate(gate, { name: "gate", position: v3(0, 2.16, -24), scale: v3(3.37, 3.37, 3.37) }));
  

  // The towers behind, fading into the haze: dark slabs, their windows lit
  // in a grid (a window every two and a half metres), a neon edge on each.
  const windows = tex(world, "textures/tower-windows.jpg");
  const towers: readonly (readonly [number, number, number, number, number, Color])[] = [
    [-3, -95, 11, 88, 11, NEON.magenta], [-21, -78, 12, 56, 12, NEON.cyan], [17, -84, 11, 66, 11, NEON.violet],
    [7, -135, 14, 132, 14, NEON.cyan], [-36, -120, 16, 92, 16, NEON.magenta], [34, -112, 15, 84, 15, NEON.pink],
    [-13, -170, 16, 150, 16, NEON.violet], [24, -180, 18, 165, 18, NEON.magenta], [52, -150, 16, 110, 16, NEON.cyan], [-55, -160, 18, 120, 18, NEON.pink],
  ];
  for (const [i, [x, z, w, h, d, c]] of towers.entries()) {
    const lit = { x: w / 28, y: h / 30 };
    const slab = mat(world, {
      color: rgb(0.02, 0.02, 0.03), roughness: 0.5, metalness: 0.4, weathering: 0,
      emissive: rgb(1, 1, 1), emissiveMap: windows, emissiveIntensity: 0.9, emissiveMapRepeat: lit, emissiveMapOffset: { x: i * 0.37, y: i * 0.21 },
    });
    place(world, s.box, slab, { name: `tower-${i}`, at: v3(x, h / 2, z), size: v3(w, h, d), shadow: false });
    const edge = mat(world, { color: rgb(0, 0, 0), emissive: c, emissiveIntensity: 3, weathering: 0 });
    place(world, s.box, edge, { name: `tower-${i}-strip`, at: v3(x - w * 0.3, h * 0.55, z + d / 2 + 0.1), size: v3(0.5, h * 0.5, 0.1), shadow: false });
    place(world, s.box, edge, { name: `tower-${i}-band`, at: v3(x, h * 0.82, z + d / 2 + 0.1), size: v3(w * 0.7, 0.6, 0.1), shadow: false });
  }

  // Craft over the alley: a hull and its running light.
  for (const [i, [x, y, z]] of ([[2.5, 19, -40], [-1.5, 14, -52], [5, 11, -34]] as const).entries()) {
    const craft = place(world, s.sphere, tower, { name: `craft-${i}`, at: v3(x, y, z), size: v3(3.4, 0.7, 2.2), shadow: false });
    const lamp = mat(world, { color: rgb(0, 0, 0), emissive: i === 1 ? NEON.amber : NEON.cyan, emissiveIntensity: 10, weathering: 0 });
    place(world, s.sphere, lamp, { name: `craft-${i}-lamp`, parent: craft, at: v3(0, -0.5, 0.3), size: v3(0.12, 0.3, 0.16), shadow: false });
  }
}
