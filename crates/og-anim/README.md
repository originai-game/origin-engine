# og-anim

Animation for the engine (`docs/design.md` §2.4, §2.7): glTF skins and
clips (cooked by `og-cook` into the `<model>#anim` blob), a blend-tree graph
asset with layered state machines, root motion, two-bone and FABRIK IK,
look-at constraints, joint limits, foot locking, spring bones and blendshapes, evaluated by one
scheduled system, `og.anim`, over the ECS.
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
| `GraphAsset` (`parse(text, resolve)`), `Motion`, `Layer`, `State`, `Transition`, `Condition`, `MaskEntry` | the `animgraph` asset; `Layer::node_weights(skeleton)` resolves a mask |
| `Trs`, `nlerp`, `rotation_arc`, `twist_y` | deterministic transform math |
| `two_bone`, `fabrik`, `ik_chain`, `look_at`, `JointLimit::clamp` | the solvers and the limit on a pose (used by the system, public for tools); each solver takes the chain's limits aligned with it, or `&[]` |
| `HumanoidMap`, `RigAsset`, `retarget`, `RetargetOptions`, `HUMAN_BONES`, `SCHEMES` | humanoid bone maps and clips retargeted between rigs (§Retargeting) |
| `mirror`, `mirror_name`, `mirror_pairs` | a clip played on the other side of its skeleton, left and right paired by the schemes' side tokens (§Mirroring) |
| `FootPlants`, `PLANT_HEIGHT` | where a foot meets the ground over a clip: the engine's one notion of "planted" (retargeting's contact checks, `og.FootLock`) |
| `SYSTEM_NAME` | `og.anim`, the system's and the state block's name |

Assets are not world state (like physics meshes): a fresh engine registers
them again before or after a restore. Clips bind to a skeleton's nodes by
name (then by index), so clips of one rig play on another with the same node
names; a rig with other names or proportions plays them retargeted
(§Retargeting).

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
| `stepFps` | f32, 0…60, tunable | 0 | stepped playback: the pose sampled at the clip time floored to 1/stepFps (`pose::stepped`); 0 samples every tick |
| `finished` | bool | false | written: a one-shot clip reached its end |

A clip change starts the new clip at 0 (or at `time` if the game wrote it in
the same tick).

**Stepped playback** (`stepFps`, the sculpted-clay look's stop motion "on
twos" at 12): only the sampled pose steps. `time` advances every tick as
before, so markers fire in the tick playback passes them and root motion
travels smoothly; the pose (of the clip and of the one fading out) is
sampled at `floor(time × stepFps) / stepFps`, so it holds for a whole step
whatever the tick rate, and a one-shot that reached its end shows its last
frame. A pure function of the clip time: nothing new in the state block,
snapshots and restores as before.

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

### `og.LookAt` (component, on its own entity; many per skeleton)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `skeleton` | u64 (Entity) | 0 | the entity with the `og.Skeleton` |
| `joint` | u32 | 0 | the joint that aims (the head, an eye) |
| `bones` | u32, 1…8 | 1 | joints sharing the turn: `joint` and its parents |
| `target` | vec3 m | 0 | the point looked at |
| `space` | enum `world` `model` | world | as `og.IkChain.space`, for `target` and `up` |
| `axis` | vec3 | 0 0 1 | the joint's forward in its own frame (three.js `lookAt` aims +Z) |
| `upAxis` | vec3 | 0 1 0 | the joint's up in its own frame: yaw is about it, pitch towards it |
| `up` | vec3 | 0 1 0 | the up hint `upAxis` rolls towards (three.js `Object3D.up`); zero keeps the animated roll |
| `maxYaw` | f32 rad, 0…π, tunable | π | the largest turn about `upAxis` from the animated forward |
| `maxPitch` | f32 rad, 0…π/2, tunable | π/2 | the largest turn towards or away from `upAxis` |
| `weight` | f32, 0…1, tunable | 1 | blend of aimed and animated rotations |
| `clamped` | bool | false | written: the target lay outside the angles |

Applied after the IK chains, in query order (`look_at`): the direction
from the joint to the target is taken into the joint's animated frame,
its yaw and pitch held within the angles (the held direction then aims
the joint wherever its parents carry it; an unheld one aims at the target
from where the joint ends up). The joint's model rotation is the one
turning `axis` onto that direction and, with an `up` hint, `upAxis`
onto the hint made perpendicular to it (else the shortest turn, keeping
the animated roll). With `bones` n, each of the n − 1 parents (top first)
turns by an even share of what remains (`nlerp(identity, remaining,
1/(n − i))`) and the joint takes the rest. Joint limits clamp each joint
as it turns; `weight` blends every turned joint's local rotation with its
animated one.

### `og.JointLimit` (component, on its own entity; one per joint)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `skeleton` | u64 (Entity) | 0 | the entity with the `og.Skeleton` |
| `joint` | u32 | 0 | the limited joint |
| `axis` | vec3 | 0 1 0 | the twist axis in the joint's rest frame: along the bone (ball joint) or the hinge |
| `swing` | f32 rad, 0…π, tunable | π | the cone half-angle `axis` may swing from rest; 0 is a hinge |
| `twistMin`, `twistMax` | f32 rad, −π…π, tunable | −π, π | the twist about `axis` from rest; a hinge's bend range |

`JointLimit::clamp(rest, local)`: the local rotation relative to the
node's rest (`rest⁻¹ · local`) is split into twist (its projection on
`axis`) and swing (the rest), each angle from `atan2`; the twist is
clamped into the range and the swing to the cone about its own axis, and
the result is `rest · swing · twist`. A rotation inside is returned bit
for bit, so a limit that never binds changes nothing. The defaults hold
nothing. Limits bind only what og.anim's solvers turn: two-bone IK clamps
each joint as it is turned, root first (a target the limits keep away is
missed, `reached` false); FABRIK with a limited joint turns and clamps
the chain after every pass and starts the next pass from it (without
limits it runs as before, bit for bit); `og.LookAt` clamps each joint it
turns. The clip's pose itself is never clamped. A second limit on the same
joint replaces the first (query order). A hinge from a two-bone solve
whose bend plane is not exactly the hinge's wants a little `swing` play
(0.1–0.2), or the tip misses by the off-plane part.

### `og.FootLock` (component, on its own entity; one per foot)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `skeleton` | u64 (Entity) | 0 | the entity with the `og.Skeleton` |
| `foot` | u32 | 0 | the foot joint; the two bones above it are the leg |
| `weight` | f32, 0…1, tunable | 1 | blend of held and animated leg |
| `release` | f32 s, 0…2, tunable | 0.15 | how long a lifted foot takes to rejoin its animation |
| `maxDistance` | f32 m, tunable | 0.5 | farther than this from the animated foot, the lock lets go |
| `planted` | bool | false | written: the clips plant the foot this tick |
| `locked` | bool | false | written: the foot is held this tick |

**Planted** is `FootPlants` (`src/retarget.rs`), the notion retargeting's
foot checks use: the foot node's model height over the clip's frames
(sampled over the rest pose), planted where it is within `PLANT_HEIGHT`
(5 mm) of its lowest; at a clip time the height is interpolated between
frames. Computed once per (skeleton, clip, foot) and cached like the
bindings. Each tick the system records the clips that posed the legs
(the base layer's leaves and the player's clip and fading clip, at the
times they were sampled, with their shares of the pose: graph weight,
layer weight, blend and fade weights, the player's weight over the
graph); the foot is planted when the planting clips hold at least half of
that.

After root motion has moved the Transform and before the IK chains, in
query order, each lock steps: free and planted → held at the animated
foot's position in the frame the skeleton entity's Transform is written
in; held and lifted (or farther than `maxDistance` from the animated
foot) → releasing; releasing → free after `release` seconds, the target
moving linearly from the lock to the animated foot. While held or
releasing, the leg is solved by two-bone IK (keeping its bend plane,
clamped by `og.JointLimit`s) onto the target by `weight`, and the foot
turned back to its animated model rotation. Because the lock lives in
the entity frame, root motion and clip blends do not move a held foot.
The `og.anim` state block keeps every lock that is held or releasing
(its entity, the skeleton it holds, the point, the release time); a
restored world holds the same feet in the same places. A world without
foot locks saves exactly the bytes it did before.

### `og.SpringBone` (component, on its own entity; one per chain)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `skeleton` | u64 (Entity) | 0 | the entity with the `og.Skeleton` |
| `joint` | u32 | 0 | the chain's last joint |
| `bones` | u32, 1…16 | 1 | joints that swing: `joint` and its parents |
| `frequency` | f32 Hz, 0…30, tunable | 3 | the spring's natural frequency (0: hangs and swings only) |
| `damping` | f32, 0…2, tunable | 0.3 | damping ratio (1 is critical) |
| `gravity` | vec3 m/s² | 0 | acceleration on every tail, in `space` |
| `space` | enum `world` `model` | world | world: the skeleton entity's Transform frame (the chain lags behind the entity); model: only the animation moves it |
| `weight` | f32, 0…1, tunable | 1 | blend of swung and animated rotations |
| `lag` | f32 m | 0 | written: the farthest a tail is from its animated place |

Three-vrm's VRMSpringBone, as a damped spring rather than a Verlet
stiffness. Applied last (after foot locks, IK chains and look-ats), in
query order, top joint first: each joint's tail (the next chain joint;
for the last its first child, else its own rest offset continued) is a
point with a velocity. Per tick, with the joints above already swung,
the target is where the pose puts the tail; the velocity steps by
implicit Euler on `x'' = ω²(target − x) − 2ζω x' + g` (ω = 2π·frequency;
stable at any frequency and tick), the point is put back at the bone's
length from the joint, the velocity becomes the distance it actually
moved over dt, and the joint turns by the shortest arc onto it
(`weight` blends the local rotation). A first tick, or a spring moved to
another skeleton, space or length, starts at the animated pose at rest.
The `og.anim` state block keeps every spring's tails (section 3).

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
clip walkOther "walk" {
  mirror true               # played on the other side of the skeleton (default false)
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
  state idle idle           # state <name> <motion> [{ speed s  sync <group> }]
  state move move {
    sync gait               # a sync group: phase-locked across layers
  }
  entry idle
  transition idle move {    # transition <from|any> <to>
    when speed gt 0.2       # all `when` must hold: gt ge lt le eq ne
    exit 0.9                # optional: the source state's phase (cycles since entry) reached
    fade 0.25               # cross-fade seconds (default 0.2)
    interrupt next          # what may cut its fade: any (default) | none | source | next
  }
}
layer upper {
  mode additive             # override (default) | additive (difference from each clip's first frame)
  weight wave               # a number 0..1 or a parameter
  mask "spine" 0.5          # mask "<node>" [weight 0..1, default 1]: per-joint weights
  mask "arm.L"
  state wave waving {
    sync gait
  }
}
```

Semantics: each state advances a phase by `dt · graph.speed · state.speed /
duration`, where `duration` is the blend-weighted clip durations, so blended
clips stay in step; each clip samples at its phase (`frac` when looping,
clamped otherwise). Per tick and layer, transitions are checked first (file
order, the first that holds fires; `any` never re-enters the current state),
then phases advance, then poses blend. A transition during a fade replaces
it (the fading-out state is dropped) when the fading transition's
`interrupt` allows it: `any` (the default, the old behaviour) lets every
transition that could fire from the new state, `any` ones included; `none`
lets nothing until the fade completes; `source` only transitions out of
the state being left (to a third state: the fade is re-targeted from the
state being left, the abandoned target dropped, its `exit` counted on the
left state's phase); `next` only the new state's own, not `any` ones. The
guard of a fade in progress is saved in the state block (tagged section 2,
written only when some layer has one other than `any`). Parse errors name
the line.

**Mirrored clips.** `mirror true` on a clip motion plays it on the
other side of whatever skeleton the graph runs on: the system makes the
clip with `mirror` (§Mirroring) the first time a graph plays it on a
skeleton and keeps it, with its binding and foot plants, beside the
bindings (forgotten when the skeleton or the clip is replaced), so the
result is bit for bit the graph playing a clip `og.anim.mirror` made and
registered. Its markers fire under the original clip's handle; foot
locks read the mirrored clip's plants. Nothing is saved: the cache is
derived from the assets. The cook does the same once, offline:
`animation { mirror "<clip>" ["<name>"] }` (og-cook's README) writes the
mirrored clip into the model's `#anim`, so it is loaded like any other.

**Layer masks.** A layer's `mask` lines give per-joint weights by node
name, resolved against the skeleton the graph plays on: a node takes the
weight of the nearest of itself and its ancestors a line names, and nodes
under none take 0 (Unreal's branch filter; a name the skeleton lacks
masks nothing). An override layer moves node `i` towards its pose by
`layer weight × mask[i]`, an additive layer adds its difference by the
same; blendshape weights follow the layer weight. A layer without `mask`
lines takes the old path, bit for bit.

**Sync groups.** `sync <group>` in a state's block puts it in a group
(groups are per graph, named in first-use order, `GraphAsset::sync_groups`).
Per tick, the first layer (file order) whose current state is in a group
leads it: its phase advances as usual. Every later layer whose current
state is in the group plays at the leader's phase (its phase is set to
the leader's, its markers fire over the leader's span), so a wave on the
upper body stays in step with the gait whatever the clips' lengths. A
fading-out state keeps its own phase. A transition between two states of
one group (in one layer) starts the new state at the old state's phase
fraction (cycles since entry restart at 0, so `exit` counts from the
transition): a walk-to-run blend keeps the feet in step. Nothing new is
saved: the phases already are.

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
the shorter arc, yaw comes from a swing-twist split, no host trigonometry
(joint limits and look-at angles use og-core's deterministic `atan2` and
`sin_cos`), no clocks. Skeletons are processed in query order (archetype creation, then
row); after all skeletons, foot locks, then IK chains, then look-at
constraints, then spring bones, each in its own query order; everything else by entity id.
Joint limits are read in query order before the solvers run. The
`og.anim` state block saves, per skeleton entity: the
skeleton handle, skin, mesh, the final pose and blendshape weights, the
player (clip, time, the time last written, speed, loop, the fading-out clip
with its time, speed and loop, fade progress) and each graph layer (state,
phase as whole cycles plus fraction, seconds in state, previous state and
phase, fade), then, after the entities and only when non-empty, tagged
sections: 1 the foot locks (per `og.FootLock` entity holding or
releasing a foot: the skeleton, held or releasing, the point, the release
time), 2 the interrupt guards of fades in progress (entity, layer, rule),
3 the spring bones (per `og.SpringBone` entity: the skeleton, the space,
each tail's position and velocity). `snapshot`, `restore` and `world_hash` therefore cover the
animation; a restore recomputes the palettes from the restored poses. All
writes happen after every computation, so a failing tick changes nothing.

Composition in model space (IK, root motion) assumes uniform scale along a
chain; palettes use exact `Mat4` products.

## Tests

`cargo test -p og-anim` (the walk takes ~14 s in debug, 2 s in release):

| Test | Asserts |
|---|---|
| `walk.rs` | 20 walkers (state machine + additive layer, cross-fading players, planar root motion, blendshapes, 60 IK chains over hinged knees and a limited shoulder, a two-joint look-at circling each head, both feet locked while planted, a hand on a spring under gravity) for 3600 ticks give the same hash twice; a snapshot at 1800 restored into a fresh world continues to the same hashes and palettes; both graph states visited, footsteps fired, every walker travelled, chains reached, heads both held at their angles and free, feet held; writes `anim-walk-determinism.json` |
| `playback.rs` | markers fire in the tick playback passes them (forwards, backwards, at double speed, at 0 and at the loop seam); a clip change cross-fades and fires both clips' markers by weight, ending bit-identical to the new clip; a seek fires nothing; a one-shot holds its end; poses at sampled times are the clip's samples; a 12-step player at 60 Hz changes pose 12 times a second (smooth: 60), each held pose the clip's sample at its twelfth, its time continuous |
| `ik.rs` | two-bone reaches within 1e-3 and bends to the pole, keeps bone lengths, clamps an unreachable target straight at full length with `reached` false; world targets; FABRIK reaches and clamps; half weight; a bad tip is refused; a joint limit clamps swing and twist separately and leaves a rotation inside bit for bit; a hinged knee never bends backwards for a pole behind (and reaches with the pole in front); FABRIK keeps a limited shoulder in its cone; a head looks at a world target with no roll, half weight turns part way; a look past maxYaw/maxPitch is held at them exactly and shared with the spine, whose own limit holds its share |
| `output.rs` | the bind-pose palette is the identity and moves with a clip; component weights write through to the output and the sink; clip weights drive the component; root motion moves the Transform and pins the hips; unregistered assets refused with the entity |
| `graph.rs` | transitions fire in the tick their condition holds and cross-fade; blend1d and the additive layer; bad text refused with its line |
| `footlock.rs` | the walk's `FootPlants` are a few frames of the cycle; a stomp (half a cycle on a standing leg while root motion carries the walker on) holds the locked foot still to 1e-6 m over every plant (it slides 0.25 m without the lock), across a cross-fade to a jog started mid-plant, and a lifted foot is back on its animation 10 ticks later; a snapshot mid-plant restores to the same hashes and foot positions |
| `graph.rs` (mirroring) | a graph clip with `mirror true` gives, tick after tick, the pose, root motion, foot-lock plants and markers (names, times; reported against the walk) of a graph playing the clip `mirror` made, and differs from the walk; `mirror maybe` refused with its line |
| `graph.rs` (interruption) | per `interrupt` rule, which of the new state's, the source's and an `any` transition cut a 0.5 s fade at once and which wait for it to complete; a bad rule refused with its line; a guarded fade restored from a snapshot continues to the same hashes and states |
| `layers.rs` | a masked override layer leaves the nodes outside its branches bit for bit, takes a named node's whole branch and blends a half-weight branch halfway; an additive layer's mask; a bad mask weight refused with its line; a synced upper layer samples its clip at the base's phase (not its own); a transition inside a group continues from the old state's phase |
| `spring.rs` | a three-joint tail trails a body moving at 2 m/s by over 0.1 m, swings past rest when it stops and settles onto the animated pose, bones keeping their length (without a spring the tip rides rigidly); a model-space spring ignores the entity's motion; gravity droops a 1 Hz tail over 3× a 4 Hz one; a single-joint spring turns only that joint; a bad joint refused with the field; a restore mid-swing continues to the same hashes and pose |
| `retarget.rs` (hands) | finger names of every scheme on their bones; a curled index (0.5 rad per joint) on a T-posed source bends the A-posed Mixamo target's by the same angles (1e-3), its bones keeping their lengths, with the arm not animated; an Unreal arm's forearm twist bone, halfway, takes half the hand's 1.2 rad roll and the upper arm's twist bone half the upper arm's 0.6 against the spine; a rig lifted 0.2 m with `footHeight 0.1` plants its feet on that ground (2 mm), without it they stay over 0.25 m up; per-side heights and a bad line |
| `retarget.rs` | the schemes detected from the walker's and a Mixamo rig's names, rig files (forced scheme, a bone named and one removed, mistakes by line); the walk retargeted onto longer legs keeps planted feet on the ground and still, without foot IK it misses by over 1 cm; the hips' travel scaled; an A-posed arm waves parallel to the T-posed one; the clip plays on the tall rig; side tokens swap in every scheme's names; a mirrored walk moves each foot as the other did, reflected, within 1e-5 (on the walker and on the tall Mixamo rig), the hips' sway flips, and mirroring twice gives the walk back |

`crates/og-cook/tests/anim_compress.rs` checks compressed clips against
f32 ones on every committed animated model (each lane within the
tolerance, each node within 0.5 mm per metre, a posed frame within the
gfx tolerance). `crates/og-cook/tests/animation.rs` covers the cook side: every cooked
sample of `walker.glb` (at its source rates and at 60 Hz) and
`trickster.glb` equals an independent glTF sampler within 1e-5 (largest
error 1.2e-7); joints, weights and morph deltas follow the cooked vertices;
markers from `extras."og.events"` and import settings; a cooked walker
plays here with markers at the right ticks.

The walker rig the tests use is `tests/support/` (procedural) and
`templates/game/assets/models/walker.glb` (generated by
`tools/make_rig.py`, CC0).

## Retargeting (phase 2)

`HumanoidMap` says which node of a skeleton is each of the 52 standard
humanoid bones (`HUMAN_BONES`, VRM's and Unity's names: `hips`, `spine`,
`chest`, `upperChest`, `neck`, `head`, then per side `Shoulder`,
`UpperArm`, `LowerArm`, `Hand`, `UpperLeg`, `LowerLeg`, `Foot`, `Toes`,
then per hand VRM 1.0's fingers: `ThumbMetacarpal`, `ThumbProximal`,
`ThumbDistal` and `Proximal`, `Intermediate`, `Distal` of `Index`,
`Middle`, `Ring` and `Little`; the 22 body bones keep their indices).
Finger names are read in every scheme: Mixamo's `LeftHandIndex1`..`3`
(the `4` is the end, not a bone), VRoid's `J_Bip_L_Thumb1`, Rigify's
`DEF-thumb.01.L` and `DEF-f_pinky.02.R`, Unreal's `index_01_l`, plain
`index.01.L` or `finger_index1_L` (`pinky` and `little` alike; thumb 1
is the metacarpal).
`HumanoidMap::detect` reads it from the node names in each scheme of
`SCHEMES` and keeps the one that recognises most bones:

| Scheme | Names it reads |
|---|---|
| `mixamo` | `mixamorig:Hips`, `Spine`, `Spine1` (chest), `Spine2` (upper chest), `LeftArm`, `LeftForeArm`, `LeftUpLeg`, `LeftLeg`, `LeftToeBase` (any `mixamorig<n>:`/`_` prefix, or none) |
| `vrm` | VRoid's `J_Bip_C_Hips`, `J_Bip_L_UpperArm`, `J_Bip_R_ToeBase` |
| `unity` | the standard names themselves, any case (`Hips`, `LeftUpperLeg`, `leftUpperLeg`) |
| `rigify` | Rigify's deform bones: `DEF-spine` (hips), `DEF-spine.001`..`.004`, `DEF-spine.006` (head), `DEF-upper_arm.L`, `DEF-thigh.R`, `DEF-toe.L` |
| `ue` | Unreal's mannequin: `pelvis`, `spine_01`..`03`, `neck_01`, `clavicle_l`, `upperarm_l`, `lowerarm_l`, `thigh_r`, `calf_r`, `ball_r` |
| `blender` | plain names with a side suffix (`.L`, `_R`, `.left`): `hips`/`pelvis`, `spine`, `chest`, `upper_arm`/`arm`, `forearm`, `thigh`, `shin`/`calf`, `foot`, `toe` |

A `type rig` .ogt (`RigAsset`; cooked as `og.Rig`, `og.load.rig`) forces a
scheme and names bones outright:

```text
type rig
version 1
---
scheme auto                 # auto (default) | mixamo | vrm | unity | rigify | ue | blender
hips "Root_Pelvis"          # a standard bone and the node it is
leftToes none               # this rig has no such bone
footHeight 0.08             # the ankles above the ground standing: one height, or the left's and the right's
```

`retarget(source, source_map, target, target_map, clip, options)` rewrites a
clip of the source rig for the target, frame by frame:

- each bone mapped on both rigs and animated by the clip gets the source
  bone's model-space rotation change from its rest pose, applied after a
  rest correction that turns the target's rest bone direction onto the
  source's (towards the next mapped bone; bones with none inherit their
  parent's), so a T-posed source drives an A-posed target and differently
  oriented bone frames (Mixamo's +Y along the bone, Unreal's +X) agree;
- the target keeps its own bone lengths; nodes no mapped bone covers keep
  their rest pose, and bones the clip does not animate get no channel (a
  wave stays a wave);
- the hips travel by the source's travel scaled by the leg ratio (hips
  above the feet at rest, target over source), so root motion is kept in
  the target's proportions and the hip height is scaled;
- with `foot_ik` (the default), each leg is solved (two bones, the knee
  kept in its bend plane) so the foot reaches the source's foot travel
  scaled the same way, and turned back to its transferred rotation; when a
  leg cannot reach (a straight source leg on other proportions) the hips
  are lowered until it does. A planted source foot stays planted on the
  target's ground and does not slide;
- the ground is y = 0 of each rig's model space, each foot standing
  `footHeight` above it (the rig file's, else the rest pose's foot
  height): a foot's goal is the target's height plus the source foot's
  height above its own, scaled, so a rig whose rest pose floats or
  crouches still plants its feet on the ground (the leg ratio is the hips'
  rest height above those foot heights). Without `footHeight` on either
  rig the result is bit for bit what it was;
- fingers turn from their hand: a finger's model rotation is the target
  hand's current one times the transferred change from hand to finger,
  so a hand pose holds whatever the clip does to the arm (it equals the
  plain transfer whenever the hand itself is animated); a hand points at
  its middle finger (else index, ring, little) for its rest correction;
- the target's twist bones (unmapped nodes whose name says `twist`, as
  Unreal's `lowerarm_twist_01_l`, or Rigify's deform segments
  `DEF-….001`) on an upper or lower arm or leg get rotation channels: a
  twist bone at fraction `t` along the segment from bone A to the next
  bone C (its rest position projected on A→C) twists about A's axis by
  `t·τ(C rel A) − (1 − t)·τ(A rel its parent)` (swing-twist angles from
  rest), so the twist runs linearly from none relative to A's parent at
  A's head to C's at its end (a forearm does not candy-wrap; an upper
  arm's twist bone counter-rolls). Every other node keeps its model
  rotation. `Retargeted::twists` counts them. A rig without twist bones
  takes the old path, bit for bit.

The cook applies it to a model's `animation { library … }` (og-cook's
README) and the runtime to `og.anim.retarget`. `tests/retarget.rs` retargets
the walker onto a taller Mixamo-named rig (armature scale 0.01, A-posed
arms, thigh 0.70 and shin 0.44 m against 0.45 and 0.45): planted feet within
2 mm of the ground and not sliding (0 measured; 11.5 mm off with forward
kinematics alone), the hips' travel scaled by the leg ratio, the A-posed
arm waving parallel to the walker's, the clip played by og.anim.

## Mirroring

`mirror(skeleton, clip)` (the runtime's `og.anim.mirror`) rewrites a clip
for the other side. `mirror_name` swaps the side tokens the naming
schemes write (§Retargeting): a `Left`/`Right` prefix after any namespace
(Mixamo, Unity, plain), VRM's `_L_`/`_R_`, a `.L`/`_R`/`.left`/`_l`
suffix (Blender, Rigify, Unreal), keeping the case; `mirror_pairs` pairs
each node with the node so named when that one names it back (a node
without a side is its own counterpart; a sided node whose counterpart is
missing has none, `NONE`). Per frame, in node order: a node's model
rotation is its counterpart's model-space change from rest reflected in
x = 0 (`(x, −y, −z, w)`), applied to its own rest model rotation; a
translated node sits at its rest position plus its counterpart's travel
from rest reflected (`(−x, y, z)`, so root motion's sideways travel and
yaw flip); scales are the counterpart's, blendshape weights copied; local
values are derived from the new parent transforms. Working in model space
means left and right bone frames need not mirror each other (Mixamo's,
the tall rig in the tests). A node gets a channel when it or its
counterpart is animated; one without a counterpart gets none. Assumes a
symmetric rest pose; mirroring twice gives the clip back (within f32).

## Owed

- Resident clips are not compressed in memory: cooked clips can be
  quantised with keyframe reduction (og-cook's `animation { compress true
  tolerance }`, `src/anim_codec.rs`), but the loader rebuilds every frame,
  so a registered `ClipAsset` holds f32 samples whichever way it was
  cooked.
- Spring bones have no colliders (three-vrm's sphere and capsule
  colliders keeping hair out of a head) and no per-joint stiffness along a
  chain; every joint of an `og.SpringBone` shares its settings.
- Twist bones are driven by retargeting only (the channels it writes); a
  clip authored for the rig itself that leaves its twist bones still is
  played as it is, with no per-tick twist distribution.
- Unity's legacy thumb names (`Thumb Proximal` as the metacarpal) are read
  as VRM 1.0's; a Unity-named rig whose thumbs are off by a bone names
  them in a `type rig` file.

## Flipbooks (sprite animation, E4)

`install_flipbook(&mut World) -> FlipbookSystem` registers `og.Flipbook`
and `og.FlipbookEvent` and schedules `og.flipbook` (with its state block)
where it is called; the runtime puts it after the game's systems, beside
`og.anim`. `FlipbookSystem::add_sheet(handle, FlipbookSheet)` registers a
sheet (assets are not world state, like clips); `sheet(handle)` reads it
back for drawing.

### The sheet (`.ogt` of `type flipbook`)

```text
type flipbook
version 1
---
texture "hero.png"        # relative to the sheet when that file exists, else an asset id
size 64 32                # the texture in pixels (the cook fills it in)
filter nearest            # nearest (default, pixel art) | linear
array false               # true: the frames become the layers of a texture array (one size each)
pivot 0.5 0               # the frame point at the entity's position, from the bottom left
margin 1                  # grid only
spacing 0
grid 16 16                # frames 0.. row-major, named by their index (columns, rows optional with size)
frame slash 0 32 32 16 0.25 0.5   # a named frame: x y w h [pivot x y], after the grid's
atlas "hero.json"         # Aseprite (frames with durations, frameTags as clips) or TexturePacker JSON
clip walk {
  frames 0 1 2 3          # frame names, or indices
  fps 10                  # or `durations 0.1 0.1 0.2 0.1` seconds per frame
  mode loop               # loop | pingPong | once
  event 2 "step"          # entering clip frame 2 sends og.FlipbookEvent `step`
}
```

The cook resolves the atlas into `frame` lines (Aseprite hashes in the
natural order of their names; TexturePacker pivots from the top left
turned to the bottom left) and tags into clips (`reverse` reversed,
`pingpong` as `pingPong`, the frames' own durations), lets a clip the
sheet declares win over a tag of the same name, and stores the canonical
text (`FlipbookSheet::to_text`, which parses back equal).

`array true` asks the cook to cut every frame out of the texture into
the layers of a texture array (`<sheet id>#array`, a `type texture-array`
output with its layers `<sheet id>#array#layer<n>`, sampled nearest under
`filter nearest`); every frame must then have one size. The sheet keeps
its frames, pivots and clips; the runtime draws frame `n` as layer `n`
(the whole layer), so every sprite of the sheet is one draw whichever
frame each shows.

### `og.Flipbook` (component)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `sheet` | u64 | 0 | the sheet handle |
| `clip` | u32 | 0 | an index into the sheet's clips; a change restarts at the first frame (or at `time` if the game wrote it in the same tick) |
| `speed` | f32, 0…10, tunable | 1 | playback rate |
| `playing` | bool | true | whether time advances |
| `frame` | u32 | 0 | written: the clip frame shown |
| `time` | f32 s | 0 | written, within one cycle; a different value written by the game is a seek (no events fire across it) |
| `finished` | bool | false | written: a `once` clip holds its last frame |
| `loops` | u32 | 0 | written: cycles completed |

`og.FlipbookEvent { entity, clip, frame, name: str64 }`: one per frame
entered with an `event` (at most 64 per entity per tick), the first frame
of a clip started at 0 included. Time is the scaled tick (`og.TimeScale`):
slow motion slows it, a hit stop holds the frame. The `og.flipbook` state
block keeps, per entity, the clip and time it last wrote, the step (frames
entered since the clip started) and the whole cycles taken off `time`, so a
restored world sends the same events on the same ticks.

Drawing is the runtime's: an entity with `og.Flipbook` and `og.Sprite` is
drawn with the sheet's texture, the current frame's region and pivot (an
`array true` sheet: its array's layer of that frame, whole, and the
pivot); the sprite keeps its size, colour, flips and layer.

Tests: `tests/flipbook.rs` (the sheet's grid and named frames, mistakes by
line, the canonical round trip; a loop at 10 fps changing frame every six
ticks and firing once per pass; ping-pong's turn, once's hold and finish;
a clip change restarting, a silent seek, speed and pause; slow motion and
hit stop; a snapshot mid-clip giving the same transcript and hash). The
event transcript of a run is in `tests/evidence/E4/flipbook-events.json`.
