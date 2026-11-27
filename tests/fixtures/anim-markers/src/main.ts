// Where a marker came from (README.md): the walker's walk cross-fading into
// its run over a second, and its wave fading into an even walk and run
// blend. Every og.AnimationEvent is checked against the weight it carries:
// in a cross-fade of two clips the heavier is dominant (a tie goes to the
// state faded to); in the three-way fade the wave until the blend outweighs
// it, then the run (the blend's later clip wins its tie with the walk).
//
//   og run tests/fixtures/anim-markers --headless --commands "2:quit"
import { Entity, og, defineResource, defineSystem, f64, u32 } from "@origingame/engine";
import type { GraphAsset, Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The tick both graphs start their one-second fade. */
const FADE_AT = 30;

const Seen = defineResource("Seen", {
  fade: Entity(),
  blend: Entity(),
  walk: f64(),
  run: f64(),
  wave: f64(),
  markers: u32(),
  dominant: u32(),
});

/** go rises on FADE_AT: both base layers leave their first state. */
const Drive = defineSystem("Drive", { writes: [og.AnimationGraph] }, (q, time) => {
  for (const a of q.archetypes()) {
    const g = a.column(og.AnimationGraph);
    for (let i = 0; i < a.count; i++) g.p0[i] = time.tick >= FADE_AT ? 1 : 0;
  }
});

const Check = defineSystem("Check", { reads: [og.AnimationEvent], writes: [Seen] }, (_q, _t, world) => {
  const s = must(world.resource(Seen));
  for (const e of world.events(og.AnimationEvent)) {
    s.markers += 1;
    if (e.dominant) s.dominant += 1;
    og.assert.always(e.layer === 0, "both graphs have one layer", { layer: e.layer });
    const fading = e.weight < 1 - 1e-6;
    if (e.entity === s.fade) {
      // walk (state 0, weight 1 - alpha) into run (state 1, weight alpha).
      const walk = e.clip === s.walk;
      og.assert.always(e.state === (walk ? 0 : 1), "a cross-fade's marker names its state", { state: e.state, walk });
      og.assert.always(e.dominant === (walk ? e.weight > 0.5 : e.weight >= 0.5), "the heavier clip of a cross-fade is dominant", { weight: e.weight, dominant: e.dominant, walk });
      og.assert.sometimes(fading && walk && e.dominant, "the clip faded from dominates the fade's first half");
      og.assert.sometimes(fading && !walk && e.dominant, "the clip faded to dominates its second half");
      og.assert.sometimes(fading && !e.dominant, "a marker of the lighter clip fires too");
    } else if (e.entity === s.blend) {
      // wave (state 0, weight 1 - alpha) into walk and run (state 1, alpha / 2 each).
      const wave = e.clip === s.wave;
      og.assert.always(e.state === (wave ? 0 : 1), "a blend's marker names its state", { state: e.state, wave });
      const expected = wave ? 3 * e.weight > 1 : e.clip === s.run && 3 * e.weight >= 1;
      og.assert.always(e.dominant === expected, "the heaviest clip of a three-way fade is dominant", { weight: e.weight, dominant: e.dominant, clip: e.clip });
      og.assert.sometimes(e.clip === s.walk && !e.dominant && 3 * e.weight >= 1, "a blend's tie goes to its later clip");
    }
  }
});

function start(world: World): void {
  const a = must(og.load.animation(world, "base/models/walker"));
  const clip = (name: string) => {
    const c = a.clips.find((c) => c.name === name);
    if (!c) throw new Error(`walker.glb has no ${name} clip`);
    return c.clip;
  };
  const graph = (first: string, second: string) =>
    must(og.anim.graph(world, {
      clips: a.clips,
      params: ["go", "mix"],
      motions: {
        walk: { kind: "clip", clip: "walk" }, run: { kind: "clip", clip: "run" },
        wave: { kind: "clip", clip: "wave" },
        stride: { kind: "blend1d", param: "mix", children: [{ motion: "walk", threshold: 0 }, { motion: "run", threshold: 1 }] },
      },
      layers: [{ name: "base", entry: "first", states: [{ name: "first", motion: first }, { name: "second", motion: second }],
        transitions: [{ from: "first", to: "second", conditions: [{ param: "go", op: "gt", value: 0.5 }], fade: 1 }],
      }],
    }));
  const spawn = (name: string, x: number, asset: GraphAsset, mix: number) => {
    const e = must(world.spawn({ name, position: { x, y: 0, z: 0 } }));
    must(world.insert(e, og.Skeleton, { asset: a.skeleton, rootMotion: "off" }));
    must(world.insert(e, og.AnimationGraph, { asset, p1: mix }));
    return e;
  };
  const s = must(world.resource(Seen));
  s.walk = clip("walk");
  s.run = clip("run");
  s.wave = clip("wave");
  s.fade = spawn("fade", -1, graph("walk", "run"), 0);
  s.blend = spawn("blend", 1, graph("wave", "stride"), 0.5);
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Seen],
  systems: [Drive, Check],
  start,
  facts: (world) => {
    const s = world.resource(Seen);
    return s.ok ? { markers: s.value.markers, dominant: s.value.dominant } : {};
  },
});
