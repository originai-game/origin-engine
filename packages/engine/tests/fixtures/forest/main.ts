// R6's forest: rows of palms from 6 m to 300 m down -z, the far ones drawn
// as the palm's octahedral impostor (assets/models/palm.glb.ogt), and
// `bias 0` drawing every one as its full mesh to compare.
// (--no-check: the fixture has no src/ for og check to read)
//   og run packages/engine/tests/fixtures/forest --no-check --size 1280x720 --out og-out \
//     --commands "0.2:screenshot:forest.png;0.3:bias:0;0.5:screenshot:forest-meshes.png;0.6:bias:1;0.7:dump_entity:far;0.8:quit"
import { og } from "@origingame/engine";

export default og.game({
  start(world) {
    const palm = og.load.gltf(world, "models/palm.glb", { castShadow: true });
    if (!palm.ok) throw new Error(palm.reason);
    // Rows across the view, each farther than the last by a growing step.
    let z = -6;
    for (let row = 0; row < 14; row++) {
      for (let k = -3; k <= 3; k++) {
        const x = k * (3 + -z * 0.25) + (row % 2) * 1.5;
        const name = row === 13 && k === 0 ? "far" : `palm-${row}-${k}`;
        const e = world.instantiate(palm.value, { name, position: { x, y: 0, z } });
        if (!e.ok) throw new Error(e.reason);
      }
      z *= 1.35;
    }
    // The sand: an xy plane laid flat.
    const flat = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2);
    const ground = world.spawn({ name: "ground", quaternion: flat });
    const plane = og.render.planeGeometry(world, { width: 800, height: 800 });
    const sand = og.render.standardMaterial(world, { color: { r: 0.62, g: 0.55, b: 0.4 }, roughness: 0.95 });
    if (ground.ok && plane.ok && sand.ok) {
      world.insert(ground.value, og.Mesh, { geometry: plane.value, material: sand.value, receiveShadow: true });
    }
    const cam = world.resource(og.Camera);
    if (cam.ok) {
      const eye = { x: 0, y: 3, z: 4 };
      cam.value.position = eye;
      cam.value.quaternion = og.transform.lookAt(eye, { x: 0, y: 2, z: -40 });
      cam.value.fov = 50;
      cam.value.far = 1000;
    }
    const sun = world.spawn({ name: "sun" });
    if (sun.ok) {
      world.insert(sun.value, og.DirectionalLight, { intensity: 3 });
      world.set(sun.value, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -1.5, z: -0.6 }) });
    }
  },
  commands: {
    /** `bias:<v>`: RenderSettings.lodBias (0 draws every palm as its full mesh). */
    bias: (world, args) => {
      const s = world.resource(og.RenderSettings);
      if (s.ok) s.value.lodBias = Number(args[0] ?? 1);
    },
  },
  commandHelp: {
    bias: { usage: "<v>", doc: "Set RenderSettings.lodBias (0: every palm its full mesh, no impostor)." },
  },
});
