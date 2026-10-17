# og-anim

Animation for the engine (`docs/design.md` §2.4, §2.7): glTF skins and
clips (cooked by `og-cook` into the `<model>#anim` blob), a blend-tree graph
asset with layered state machines, root motion, two-bone and FABRIK IK and
blendshapes, evaluated by one scheduled system, `og.anim`, over the ECS.
Game code owns the components; the system writes the pose, the palette and
root motion back each tick.

```rust
let anim = og_anim::install(&mut world)?;                       // after the systems that set players and parameters
let cooked = bundle.load_animation("models/walker.glb")?;        // og-cook runtime: skeleton, clips, skin and morph streams
anim.add_skeleton(1, cooked.skeleton.clone())?;
for (i, c) in cooked.clips.iter().enumerate() { anim.add_clip(10 + i as u64, c.clone())?; }
let graph = GraphAsset::parse(&text, &|name| cooked.clips.iter().position(|c| c.name == name).map(|i| 10 + i as u64))?;
anim.add_graph(2, graph)?;
let e = world.spawn((Transform::default(), Skeleton { asset: 1, root_motion: ROOT_MOTION_PLANAR, ..Default::default() },
                     AnimationGraph::new(2)))?;
world.spawn((IkChain { skeleton: e.to_bits(), tip: 5, target, ..Default::default() },))?;
world.step()?;                                                   // pose, palette, root motion, events
anim.write_outputs(&mut render_sink);                            // PaletteSink: joint palettes, blendshape weights
```

Dependencies: og-core, og-ecs, glam. No serde, no C, builds for
`wasm32-unknown-unknown`.

## API map

| Item | What it is |
|---|---|
| `install(&mut World) -> AnimationSystem` | registers the types, schedules `og.anim` here in the schedule, adds the `og.anim` state block |
| `AnimationSystem` | `add_skeleton`/`add_clip`/`add_graph(handle, asset)` (handle ≠ 0; validated), `remove_*`, `skeleton`/`clip`/`graph(handle)`, `pose(entity)` (final local pose), `output(entity) -> AnimOutput`, `write_outputs(&mut dyn PaletteSink)` |
| `SkeletonAsset { nodes, skins }`, `SkeletonNode`, `Skin` | a rig: every glTF node in topological order with its rest TRS and mesh, skins with joints and inverse bind matrices; `rest_pose`, `model_space`, `model_matrices`, `palette`, `joint_node`, `node` |
| `ClipAsset`, `Channel`, `ChannelPath`, `ClipEvent` | a clip resampled at `rate` (with `source_rate` kept); `frame_pos`, `vec3`, `quat`, `weight`; `frames_for` |
| `GraphAsset` (`parse(text, resolve)`), `Motion`, `Layer`, `State`, `Transition`, `Condition` | the `animgraph` asset |
| `Trs`, `nlerp`, `rotation_arc`, `twist_y` | deterministic transform math |
| `two_bone`, `fabrik`, `ik_chain` | the solvers on a pose (used by the system, public for tools) |
| `SYSTEM_NAME` | `og.anim`, the system's and the state block's name |

Assets are not world state (like physics meshes): a fresh engine registers
them again before or after a restore. Clips bind to a skeleton's nodes by
name (then by index), so clips of one rig play on another with the same node
names.

## Records (for `api.ogt`)

Field order, names and storage types are the layout; the IDL declares them
exactly so. `u64` asset fields are handles registered with the system (0 is
none); joint fields index the skin's `joints`. Enum fields are `u32` with
the variant names given.

### `og.Skeleton` (component)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `asset` | u64 | 0 | the skeleton asset |
| `skin` | u32 | 0 | which skin the palette is for |
| `rootMotion` | enum `off` `planar` `full` | off | planar: the root joint's xz travel and yaw move `og.Transform`; full: all of its translation and rotation |
| `rootJoint` | u32 | 0 | the joint root motion is extracted from (0, the skin's first joint, is usually the hips) |
| `jointCount` | u32 | 0 | written: palette length |

### `og.AnimationPlayer` (component)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `clip` | u64 | 0 | the clip; changing it cross-fades from the previous one |
| `time` | f32 s | 0 | written every tick; a different value written by the game is a seek (no markers fire across it) |
| `speed` | f32, −10…10, tunable | 1 | negative plays backwards |
| `loop` | bool | true | wrap; otherwise hold the end and set `finished` |
| `weight` | f32, 0…1, tunable | 1 | how much of the pose it decides (over the graph when there is one, else over the rest pose) |
| `fade` | f32 s, 0…10, tunable | 0.2 | cross-fade duration on a clip change |
| `finished` | bool | false | written: a one-shot clip reached its end |

A clip change starts the new clip at 0 (or at `time` if the game wrote it in
the same tick).

### `og.AnimationGraph` (component)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `asset` | u64 | 0 | the graph asset |
| `weight` | f32, 0…1, tunable | 1 | over the rest pose |
| `speed` | f32, 0…10, tunable | 1 | rate multiplier for every layer |
| `p0` … `p7` | f32, tunable | 0 | the graph's parameters, in `param` order |
| `state` | u32 | 0 | written: the base layer's state (index in the file) |
| `stateTime` | f32 s | 0 | written: seconds in that state |

### `og.Blendshapes` (component)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `source` | enum `component` `clips` | component | component: game code writes `w0…w7`; clips: the clips' weight channels write them |
| `mesh` | u32 | 0 | the glTF mesh index the weights drive |
| `w0` … `w7` | f32, −1…2, tunable | 0 | weights in the mesh's target order |

### `og.IkChain` (component, on its own entity; many per skeleton)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `skeleton` | u64 (Entity) | 0 | the entity with the `og.Skeleton` |
| `tip` | u32 | 0 | end joint |
| `bones` | u32, 1…32 | 2 | bones above the tip that turn (two-bone always 2) |
| `solver` | enum `twoBone` `fabrik` | twoBone | |
| `target` | vec3 m | 0 | where the tip should be |
| `pole` | vec3 m | 0 | two-bone bend hint; zero keeps the animated bend |
| `space` | enum `world` `model` | world | world: relative to the skeleton entity's `og.Transform` |
| `weight` | f32, 0…1, tunable | 1 | blend of solved and animated rotations |
| `iterations` | u32, 1…64 | 10 | FABRIK passes at most |
| `tolerance` | f32 m | 0.001 | reach threshold |
| `reached` | bool | false | written: within `tolerance` |
| `error` | f32 m | 0 | written: tip distance from the target |

### `og.AnimationEvent` (event)

| Field | Type | Meaning |
|---|---|---|
| `entity` | u64 | the skeleton entity |
| `clip` | u64 | the clip asset |
| `name` | str64 | the marker |
| `time` | f32 s | the marker's clip time |
| `weight` | f32 | the clip's blend weight when it fired |

`og.Transform` (`position: vec3`, `quaternion: quat`, `scale: vec3`) is
adopted, not declared again.

## The graph asset (`.ogt`, `type animgraph`, `version 1`)

```text
type animgraph
version 1
---
param speed                 # p0
param wave                  # p1
clip idle "idle"            # clip <motion> "<clip>"  (the loader's resolve turns the string into a handle)
clip walk "walk" {
  speed 1                   # duration multiplier (> 0)
  loop true
}
clip run "run"
clip waving "wave"
blend1d move speed {        # blend1d <motion> <param>: linear between the two thresholds around the value
  child walk 1.5
  child run 4
}
blend2d strafe x y {        # blend2d <motion> <px> <py>: gradient-band weights
  child fwd 0 1
  child left -1 0
}
layer base {                # layer 0: the base, root motion and AnimationGraph.state come from it
  state idle idle           # state <name> <motion> [{ speed s }]
  state move move
  entry idle
  transition idle move {    # transition <from|any> <to>
    when speed gt 0.2       # all `when` must hold: gt ge lt le eq ne
    exit 0.9                # optional: the source state's phase (cycles since entry) reached
    fade 0.25               # cross-fade seconds (default 0.2)
  }
}
layer upper {
  mode additive             # override (default) | additive (difference from each clip's first frame)
  weight wave               # a number 0..1 or a parameter
  state wave waving
}
```

Semantics: each state advances a phase by `dt · graph.speed · state.speed /
duration`, where `duration` is the blend-weighted clip durations, so blended
clips stay in step; each clip samples at its phase (`frac` when looping,
clamped otherwise). Per tick and layer, transitions are checked first (file
order, the first that holds fires; `any` never re-enters the current state),
then phases advance, then poses blend. A transition during a fade replaces
it (the fading-out state is dropped). Parse errors name the line.

## Palette contract with og-render

`AnimationSystem::write_outputs(&mut dyn PaletteSink)` hands, per animated
entity in entity order:

- `joint_palette(entity, skin, &[Mat4])`: one matrix per joint of the skin
  (glTF `skin.joints` order, the order the vertices' `JOINTS_0` index),
  `model(joint) · inverseBind(joint)` in the skeleton's model space. At the
  bind pose every matrix is the identity. A skinned vertex is drawn at
  `MatrixWorld(entity) · Σᵢ wᵢ · palette[jᵢ] · v`; the skinned mesh node's
  own transform is not applied (glTF). `JointPalette` is `Vec<Mat4>` until
  og-render's own type lands; og-render implements `PaletteSink` (or the
  extraction converts).
- `morph_weights(entity, mesh, &[f32; 8])` for entities with
  `og.Blendshapes`: the weights for the glTF mesh `mesh`'s targets.

Vertex data comes from the cook, per model primitive, indexed as
`CookedModel::primitives` and per cooked vertex in the model blob's order:
`CookedAnimation::skin_streams` (`joints: [u16; 4]`, `weights: [f32; 4]`
summing to 1) and `morphs` (per target position deltas, normal deltas when
every target has them, names, default weights). The renderer needs a skinned
vertex input (joints + weights, storage-buffer palette with a vertex-texture
fallback on GL) and per-primitive morph deltas; bounds for culling should be
widened by the palette (owed on the render side).

## Determinism and the state block

f32 with `+ - * /` and `sqrt` only: rotations blend by normalised lerp on
the shorter arc, yaw comes from a swing-twist split, no host trigonometry,
no clocks. Skeletons are processed in query order (archetype creation, then
row), IK chains after all skeletons in their query order, everything else by
entity id. The `og.anim` state block saves, per skeleton entity: the
skeleton handle, skin, mesh, the final pose and blendshape weights, the
player (clip, time, the time last written, speed, loop, the fading-out clip
with its time, speed and loop, fade progress) and each graph layer (state,
phase as whole cycles plus fraction, seconds in state, previous state and
phase, fade). `snapshot`, `restore` and `world_hash` therefore cover the
animation; a restore recomputes the palettes from the restored poses. All
writes happen after every computation, so a failing tick changes nothing.

Composition in model space (IK, root motion) assumes uniform scale along a
chain; palettes use exact `Mat4` products.

## Tests

`cargo test -p og-anim` (the walk takes ~14 s in debug, 2 s in release):

| Test | Asserts |
|---|---|
| `walk.rs` | 20 walkers (state machine + additive layer, cross-fading players, planar root motion, blendshapes, 60 IK chains) for 3600 ticks give the same hash twice; a snapshot at 1800 restored into a fresh world continues to the same hashes and palettes; both graph states visited, footsteps fired, every walker travelled, chains reached; writes `anim-walk-determinism.json` |
| `playback.rs` | markers fire in the tick playback passes them (forwards, backwards, at double speed, at 0 and at the loop seam); a clip change cross-fades and fires both clips' markers by weight, ending bit-identical to the new clip; a seek fires nothing; a one-shot holds its end; poses at sampled times are the clip's samples |
| `ik.rs` | two-bone reaches within 1e-3 and bends to the pole, keeps bone lengths, clamps an unreachable target straight at full length with `reached` false; world targets; FABRIK reaches and clamps; half weight; a bad tip is refused |
| `output.rs` | the bind-pose palette is the identity and moves with a clip; component weights write through to the output and the sink; clip weights drive the component; root motion moves the Transform and pins the hips; unregistered assets refused with the entity |
| `graph.rs` | transitions fire in the tick their condition holds and cross-fade; blend1d and the additive layer; bad text refused with its line |

`crates/og-cook/tests/animation.rs` covers the cook side: every cooked
sample of `walker.glb` (at its source rates and at 60 Hz) and
`trickster.glb` equals an independent glTF sampler within 1e-5 (largest
error 1.2e-7); joints, weights and morph deltas follow the cooked vertices;
markers from `extras."og.events"` and import settings; a cooked walker
plays here with markers at the right ticks.

The walker rig the tests use is `tests/support/` (procedural) and
`templates/game/assets/models/walker.glb` (generated by
`tools/make_rig.py`, CC0).

## Owed

- **Retargeting**: clips bind by node name only; no rest-pose compensation
  between rigs with different proportions or orientations, no humanoid map.
- **Ragdoll**: no physics-driven joints or blending between animation and
  og-physics bodies.
- **Procedural animation**: look-at/aim constraints, foot locking, joint
  limits and twist constraints in IK, springs/jiggle bones.
- Layer masks (per-joint weights), sync groups across layers, mirroring,
  transition interruption rules, clip compression (samples are f32).
- The IDL transcription and the runtime's `install` in og-api, og-render's
  skinned and morph vertex path (`PaletteSink`), widened culling bounds.
