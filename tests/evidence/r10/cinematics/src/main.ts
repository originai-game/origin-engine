// The R10 cinematics sample: a small room for timelines/cinematic.ogt.
// tests/evidence/r10/NOTES.md has the commands and what each frame shows.
//
// - a lamp on a table (an emissive sphere and an og.PointLight), dark at
//   the start, warmed up by a light track and a material track;
// - "fill", a cold directional light, and the sky's ambient light
//   (og.RenderSettings.environmentIntensity), both turned down by property
//   tracks while the lamp warms up;
// - "hero", a pillar in front of the camera, sharp until the camera's
//   focus moves to the back wall;
// - "crate", hidden by a visibility track at 3.5 s;
// - "door", a slab in the back wall that a nested timeline slides open.

import { og } from "@origingame/engine";
import type { Entity, Geometry, Material, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

function mesh(world: World, name: string, geometry: Geometry, material: Material, position: { x: number; y: number; z: number }, scale: { x: number; y: number; z: number }): Entity {
  const e = must(world.spawn({ name, position }));
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
  return e;
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    const eye = { x: -3.5, y: 2.2, z: 7 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 1.2, z: 0 });
    cam.fov = 45;
    cam.background = { r: 0.01, g: 0.012, b: 0.02 };

    const fill = must(world.spawn({ name: "fill", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.5, y: -0.7, z: -0.5 }) }));
    world.insert(fill, og.DirectionalLight, { intensity: 1.2, color: { r: 0.55, g: 0.7, b: 1 } });

    const box = must(og.render.boxGeometry(world, {}));
    const ball = must(og.render.sphereGeometry(world, { radius: 0.5 }));
    const mat = (r: number, g: number, b: number, roughness = 0.8) => must(og.render.standardMaterial(world, { color: { r, g, b }, roughness }));
    const wall = mat(0.32, 0.3, 0.28, 0.95);

    mesh(world, "floor", box, mat(0.22, 0.17, 0.13, 0.9), { x: 0, y: -0.5, z: 0 }, { x: 12, y: 1, z: 10 });
    mesh(world, "back-left", box, wall, { x: -3.6, y: 2, z: -4 }, { x: 4.8, y: 4, z: 0.2 });
    mesh(world, "back-right", box, wall, { x: 3.6, y: 2, z: -4 }, { x: 4.8, y: 4, z: 0.2 });
    mesh(world, "lintel", box, wall, { x: 0, y: 3.5, z: -4 }, { x: 2.4, y: 1, z: 0.2 });
    // Behind the doorway, a bright yard the door reveals.
    mesh(world, "yard", box, must(og.render.standardMaterial(world, { color: { r: 0.4, g: 0.6, b: 0.9 }, emissive: { r: 0.35, g: 0.55, b: 0.85 }, emissiveIntensity: 1.5 })), { x: 0, y: 2, z: -6 }, { x: 8, y: 4, z: 0.2 });
    mesh(world, "door", box, mat(0.45, 0.24, 0.1, 0.7), { x: 0, y: 1.5, z: -3.9 }, { x: 2.2, y: 3, z: 0.15 });

    mesh(world, "table", box, mat(0.35, 0.2, 0.1, 0.6), { x: 1.4, y: 0.45, z: -0.6 }, { x: 1.6, y: 0.9, z: 1 });
    const glow = must(og.render.standardMaterial(world, { color: { r: 1, g: 0.8, b: 0.5 }, emissive: { r: 1, g: 0.55, b: 0.2 }, emissiveIntensity: 0 }));
    const lamp = mesh(world, "lamp", ball, glow, { x: 1.4, y: 1.2, z: -0.6 }, { x: 0.45, y: 0.45, z: 0.45 });
    world.insert(lamp, og.PointLight, { intensity: 0, distance: 9, color: { r: 1, g: 0.55, b: 0.25 }, castShadow: true });

    mesh(world, "hero", box, mat(0.15, 0.45, 0.4, 0.5), { x: -0.6, y: 1, z: 1.4 }, { x: 0.5, y: 2, z: 0.5 });
    mesh(world, "crate", box, mat(0.9, 0.28, 0.04, 0.7), { x: -2.4, y: 0.4, z: -1.2 }, { x: 0.8, y: 0.8, z: 0.8 });

    // Loaded in setup: loading grants og.timeline the records its tracks write.
    must(og.load.timeline(world, "timelines/cinematic.ogt"));
  },
});
