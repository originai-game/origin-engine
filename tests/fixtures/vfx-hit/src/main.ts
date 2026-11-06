// Two composed effects (og.vfx, README.md) on a dark ground at dusk: a heal
// on a target post and a heavy hit beside it. `heal` and `hit` spawn them.
//
//   og run tests/fixtures/vfx-hit --commands "0.5:heal;1.2:hit;3:quit"
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export const HIT = { x: -1.6, y: 0, z: 0.4 };
export const HEAL = { x: 1.8, y: 0, z: -0.3 };

/** The og.VfxEvent names seen, counted. */
const Seen = defineResource("Seen", { impacts: u32(), healed: u32() });

function build(world: World): void {
  const box = must(og.render.boxGeometry(world, {}));
  const solid = (at: { x: number; y: number; z: number }, size: { x: number; y: number; z: number }, color: { r: number; g: number; b: number }) => {
    const e = must(world.spawn({ position: at }));
    must(world.set(e, og.Transform, { scale: size }));
    must(world.insert(e, og.Mesh, { geometry: box, material: must(og.render.standardMaterial(world, { color, roughness: 0.85 })), castShadow: true }));
  };
  solid({ x: 0, y: -0.25, z: 0 }, { x: 30, y: 0.5, z: 30 }, { r: 0.12, g: 0.12, b: 0.13 });
  // The heal's target: a post.
  solid({ x: HEAL.x, y: 0.7, z: HEAL.z }, { x: 0.35, y: 1.4, z: 0.35 }, { r: 0.35, g: 0.3, b: 0.26 });
  solid({ x: -4.5, y: 0.6, z: -2.5 }, { x: 1.2, y: 1.2, z: 1.2 }, { r: 0.3, g: 0.28, b: 0.3 });

  const moon = must(world.spawn({ quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.3, y: -0.6, z: -0.6 }) }));
  must(world.insert(moon, og.DirectionalLight, { color: { r: 0.55, g: 0.6, b: 0.9 }, intensity: 0.25, castShadow: true }));
  const rs = must(world.resource(og.RenderSettings));
  rs.decalAtlas = must(og.load.texture(world, "textures/atlas.png"));
  rs.bloom = true;
  rs.environmentIntensity = 0.15;
  const cam = must(world.resource(og.Camera));
  cam.position = { x: 0, y: 2.6, z: 6.4 };
  cam.quaternion = og.transform.lookAt(cam.position, { x: 0, y: 0.7, z: 0 });
  cam.fov = 50;
  cam.background = { r: 0.02, g: 0.025, b: 0.05 };
}

const Watch = defineSystem("Watch", { reads: [og.VfxEvent], writes: [Seen] }, (_q, _t, world) => {
  const seen = must(world.resource(Seen));
  for (const e of world.events(og.VfxEvent)) {
    if (e.name === "impact") { seen.impacts += 1; og.assert.reachable("the hit lands"); }
    if (e.name === "healed") { seen.healed += 1; og.assert.reachable("the heal resolves"); }
  }
});

const play = (world: World, path: string, at: { x: number; y: number; z: number }) => {
  must(og.vfx.spawn(world, { asset: must(og.load.vfx(world, path)), position: at }));
};

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Seen],
  systems: [Watch],
  start: build,
  commands: {
    hit: (world) => play(world, "fx/hit.ogt", HIT),
    heal: (world) => play(world, "fx/heal.ogt", HEAL),
  },
  commandHelp: {
    hit: { usage: "", doc: "A heavy hit on the ground." },
    heal: { usage: "", doc: "A heal on the target post." },
  },
  facts: (world) => {
    const seen = world.resource(Seen);
    return { impacts: seen.ok ? seen.value.impacts : 0, healed: seen.ok ? seen.value.healed : 0 };
  },
});
