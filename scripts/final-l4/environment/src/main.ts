// The LDR environment scene the final L4 pass shoots (scripts/final-l4-pass.sh environment): the
// pinned AI sky lighting and behind a few shapes whose top faces catch the extracted sun.
// `sky:<name>` sets one of
//   full    assets/env/coast.ogt, the whole panorama as a sphere
//   upper   assets/env/upper.ogt, its upper half as a dome over a filled ground
//   blur    the full sphere with backgroundBlurriness 0.5
//   blend   the full sphere halfway to the upper dome (blendTo, blendWeight 0.5)
// and aims a DirectionalLight from og.load.environmentSun. `face:sun` faces the sun's azimuth (the
// default) and `face:away` its back. The sun's answer goes to stdout as `sun <json>`.
import { og, type Entity, type EnvironmentMap, type Vector3, type World } from "@origingame/engine";

let sunLight: Entity | undefined;
let facing: "sun" | "away" = "sun";
let sunDir: Vector3 = { x: 1, y: 0.3, z: 0 };

function load(world: World, path: string): EnvironmentMap {
  const map = og.load.environment(world, path);
  if (!map.ok) throw new Error(`${path}: ${map.reason}`);
  return map.value;
}

function aim(world: World): void {
  const cam = world.resource(og.Camera);
  if (!cam.ok) throw new Error(cam.reason);
  const h = Math.hypot(sunDir.x, sunDir.z) || 1;
  const s = facing === "sun" ? 1 : -1;
  const eye = { x: 0, y: 1.6, z: 0 };
  // A slight downward pitch: the horizon in the upper third, the shapes' tops in view.
  cam.value.position = eye;
  cam.value.quaternion = og.transform.lookAt(eye, { x: (s * 10 * sunDir.x) / h, y: 0.4, z: (s * 10 * sunDir.z) / h });
  cam.value.fov = 60;
  cam.value.far = 500;
}

function view(world: World, name: string): void {
  const env = world.resource(og.Environment);
  if (!env.ok) throw new Error(env.reason);
  const coast = load(world, "env/coast.ogt");
  const upper = load(world, "env/upper.ogt");
  const map = name === "upper" ? upper : coast;
  Object.assign(env.value, {
    map,
    intensity: 1,
    rotation: 0,
    background: true,
    backgroundIntensity: 1,
    backgroundBlurriness: name === "blur" ? 0.5 : 0,
    blendTo: name === "blend" ? upper : 0,
    blendWeight: name === "blend" ? 0.5 : 0,
  });
  if (!["full", "upper", "blur", "blend"].includes(name)) throw new Error(`view ${name}: one of full, upper, blur, blend`);
  const sun = og.load.environmentSun(world, map, 0, 1);
  if (!sun.ok) throw new Error(sun.reason);
  console.log(`sun ${name} ${JSON.stringify(sun.value)}`);
  sunDir = sun.value.direction;
  if (sunLight !== undefined) {
    world.set(sunLight, og.DirectionalLight, { color: sun.value.color, intensity: sun.value.intensity, castShadow: true });
    const d = sun.value.direction;
    world.set(sunLight, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -d.x, y: -d.y, z: -d.z }) });
  }
  aim(world);
}

// Shapes 7 m out along the camera's line, spread across it: a white and a grey box (flat tops), a
// rough and a polished sphere, a metal sphere, on a small sand pad so the sky shows past it.
function shapes(world: World): void {
  const box = og.render.boxGeometry(world, { width: 1, height: 1, depth: 1 });
  const ball = og.render.sphereGeometry(world, { radius: 0.5, widthSegments: 48, heightSegments: 24 });
  const pad = og.render.planeGeometry(world, { width: 14, height: 14 });
  if (!box.ok || !ball.ok || !pad.ok) throw new Error("geometry");
  const mat = (color: [number, number, number], roughness: number, metalness = 0) => {
    const m = og.render.standardMaterial(world, { color: { r: color[0], g: color[1], b: color[2] }, roughness, metalness });
    if (!m.ok) throw new Error(m.reason);
    return m.value;
  };
  const h = Math.hypot(sunDir.x, sunDir.z) || 1;
  const f = { x: sunDir.x / h, z: sunDir.z / h };
  const side = { x: -f.z, z: f.x };
  const at = (k: number, y: number, dist = 7) => ({ x: f.x * dist + side.x * k, y, z: f.z * dist + side.z * k });
  const put = (name: string, geometry: typeof box.value, material: ReturnType<typeof mat>, position: Vector3) => {
    const e = world.spawn({ name, position });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
  };
  put("white-box", box.value, mat([0.85, 0.85, 0.85], 0.6), at(-3, 0.5));
  put("grey-box", box.value, mat([0.35, 0.35, 0.35], 0.3), at(-1.5, 0.5));
  put("rough-ball", ball.value, mat([0.8, 0.8, 0.8], 0.9), at(0, 0.5));
  put("gloss-ball", ball.value, mat([0.7, 0.1, 0.08], 0.15), at(1.5, 0.5));
  put("metal-ball", ball.value, mat([0.95, 0.95, 0.95], 0.1, 1), at(3, 0.5));
  const flat = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2);
  const ground = world.spawn({ name: "pad", quaternion: flat, position: at(0, 0) });
  if (!ground.ok) throw new Error(ground.reason);
  world.insert(ground.value, og.Mesh, { geometry: pad.value, material: mat([0.55, 0.47, 0.36], 0.95), receiveShadow: true });
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const s = og.load.environmentSun(world, load(world, "env/coast.ogt"), 0, 1);
    if (s.ok && s.value.intensity > 0) sunDir = s.value.direction;
    const light = world.spawn({ name: "sun" });
    if (!light.ok) throw new Error(light.reason);
    world.insert(light.value, og.DirectionalLight, { intensity: 0 });
    sunLight = light.value;
    shapes(world);
    view(world, "full");
  },
  commands: {
    sky: (world, args) => view(world, args[0] ?? "full"),
    face: (world, args) => {
      facing = args[0] === "away" ? "away" : "sun";
      aim(world);
    },
  },
});
