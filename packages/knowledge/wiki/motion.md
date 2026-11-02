<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `motion` module

Generated motion for humanoids: the engine's motion model (a 17.6M-parameter student distilled from NVIDIA MotionBricks, og-motion) moves a character between the goals the game gives it, on the GPU. Give an entity with an og.Skeleton (a humanoid rig og.anim.humanoid recognises, or one an og.MotionAgent.rig names) an og.MotionAgent, then queue goals: og.motion.pose (a pose of the model's library where the character stands), og.motion.place (a pose at a place, facing a heading), og.motion.face (turn to a heading) and og.motion.walkTo (walk or run to a point). og.motion.poses lists the library: idle, walk, run, jump, crouch, wave, cheer.

**Presentation only.** The generated pose is drawn, never simulated (docs/design.md §2.4): it replaces og.anim's joint palette when the frame is drawn, and nothing a tick reads (og.anim's pose, its joint queries, root motion, physics, the world hash) depends on it, so a run is deterministic on every GPU. The model never moves the character: og.Transform is its root, always. With og.MotionAgent.drive the og.motion system moves the Transform to the goals itself with fixed math inside the tick (walkTo at its speed along a straight line, place over its duration, face at turnSpeed), and the generated pose is re-rooted onto the Transform, its planted feet absorbing the difference; without drive the game moves the Transform (its own physics, path or controller) and the character's motion follows it (walking, running or standing by how fast it moves). No model root motion ever reaches the simulation.

**Goals are the game's.** A goal is current from the tick it starts until it ends by fixed rules (walkTo when the mover arrives, pose, place and face after their duration), then the next queued one starts; og.motion.status answers the queue and the current goal, never the model's progress, so game code may branch on it. What the model is doing (the frames it generated ahead, its batches, its timings) is the probe's `dump_motion` only.

**The model.** Its weights (35 MB in f16) ship inside @origingame/engine; `og cook` writes them as an optional file, cooked/motion/model.ogb, only for a project that uses og.motion (the manifest names it), and the engine reads and uploads it in the background as soon as the bundle is open, before any agent needs it (natively on a thread of its own; on the web a fetch after the boot bundle, never in it). The model runs on the GPU beside the renderer (in the engine Worker on the web): one batch for every agent at once, submitted in slices of about 3 ms of GPU time a frame, so a batch never costs a frame its vsync, and decoded a few characters a frame. Until the weights are resident, or when they fail to load, og.motion.capabilities answers backend none and each agent is drawn as og.anim poses it: its og.AnimationPlayer or og.AnimationGraph is the fallback, so give it an idle or walk clip.

- **TypeScript**: functions are the namespace `og.motion`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-anim`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module motion {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 28776</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`MotionGoalKind`](#type-MotionGoalKind) | enum | What a goal asks for. |
| [`MotionGoalKind()`](#column-MotionGoalKind) | column kind | A MotionGoalKind field: the variant's index. |
| [`MotionBackend`](#type-MotionBackend) | enum | Where the motion model runs. |
| [`og.MotionAgent`](#record-MotionAgent) | component | A humanoid the motion model animates: the entity needs an og.Skeleton whose asset is a humanoid rig (og.anim.humanoid recognises its bone names, or `rig` names them). Goals queue with og.motion.pose, place, face and walkTo. The generated pose is drawn in place of og.anim's (presentation only); og.anim still poses the skeleton every tick for everything the simulation reads, and is what is drawn until the model is resident. |
| [`PoseGoalDesc`](#type-PoseGoalDesc) | desc struct | A pose of the model's library where the character stands (og.motion.pose). |
| [`PlaceGoalDesc`](#type-PlaceGoalDesc) | desc struct | A pose of the library at a place (og.motion.place): the character's hips over `position`, facing `heading`. |
| [`FaceGoalDesc`](#type-FaceGoalDesc) | desc struct | Turn to a heading where the character stands (og.motion.face). |
| [`WalkGoalDesc`](#type-WalkGoalDesc) | desc struct | Walk or run to a point (og.motion.walkTo); needs og.MotionAgent.drive. |
| [`MotionStatus`](#type-MotionStatus) | struct | An agent's goals, as the game set them: the current goal and how many wait. Deterministic: the same on every GPU, the model's progress never shows here. |
| [`MotionCapabilities`](#type-MotionCapabilities) | struct | Where the motion model runs and what it knows. Presentation: a frame's answer, not the tick's; never branch the simulation on it. |
| [`og.motion.pose`](#fn-pose) | function | Queue a library pose where the character stands, after its other goals. |
| [`og.motion.place`](#fn-place) | function | Queue a library pose at a place and heading; needs og.MotionAgent.drive (the mover takes the Transform there). |
| [`og.motion.face`](#fn-face) | function | Queue a turn to a heading where the character stands; needs og.MotionAgent.drive. |
| [`og.motion.walkTo`](#fn-walkTo) | function | Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive: the mover takes the Transform there at `speed` in a straight line, turning at turnSpeed, and the goal ends when it arrives. |
| [`og.motion.clear`](#fn-clear) | function | Drop the agent's goals, the current one included: the character holds where it is. |
| [`og.motion.status`](#fn-status) | function | The agent's current goal and queue. |
| [`og.motion.capabilities`](#fn-capabilities) | function | Where the model runs: gpu once its weights are resident, none before (loading) or when they failed to load. Asking starts the load when no agent has yet. |
| [`og.motion.poses`](#fn-poses) | function | The model's pose library, by name: idle, walk, run, jump, crouch, wave, cheer. Fixed by the engine's version, known before the weights are resident, so a goal's pose is checked the same everywhere. |

## Functions

<a id="fn-pose"></a>

### `og.motion.pose`

Queue a library pose where the character stands, after its other goals.

```ts
og.motion.pose(world: World, entity: Entity, desc?: PoseGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `PoseGoalDesc` | yes | The pose and how long it is current. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or a pose the library lacks (og.motion.poses).

Binding symbol `og_motion_pose` · publish builds: kept

```ogt
fn pose {
  doc "Queue a library pose where the character stands, after its other goals."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc PoseGoalDesc "The pose and how long it is current."
  returns Result.Void "Refused for an entity without og.MotionAgent or a pose the library lacks (og.motion.poses)."
}
```

<sub>`api.ogt` line 28907</sub>

<a id="fn-place"></a>

### `og.motion.place`

Queue a library pose at a place and heading; needs og.MotionAgent.drive (the mover takes the Transform there).

```ts
og.motion.place(world: World, entity: Entity, desc?: PlaceGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `PlaceGoalDesc` | yes | The pose, place, heading and duration. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or without drive, a pose the library lacks, or a position that is not finite.

Binding symbol `og_motion_place` · publish builds: kept

```ogt
fn place {
  doc "Queue a library pose at a place and heading; needs og.MotionAgent.drive (the mover takes the Transform there)."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc PlaceGoalDesc "The pose, place, heading and duration."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive, a pose the library lacks, or a position that is not finite."
}
```

<sub>`api.ogt` line 28915</sub>

<a id="fn-face"></a>

### `og.motion.face`

Queue a turn to a heading where the character stands; needs og.MotionAgent.drive.

```ts
og.motion.face(world: World, entity: Entity, desc?: FaceGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `FaceGoalDesc` | yes | The heading and the least time the goal is current. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or without drive.

Binding symbol `og_motion_face` · publish builds: kept

```ogt
fn face {
  doc "Queue a turn to a heading where the character stands; needs og.MotionAgent.drive."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc FaceGoalDesc "The heading and the least time the goal is current."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive."
}
```

<sub>`api.ogt` line 28923</sub>

<a id="fn-walkTo"></a>

### `og.motion.walkTo`

Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive: the mover takes the Transform there at `speed` in a straight line, turning at turnSpeed, and the goal ends when it arrives.

```ts
og.motion.walkTo(world: World, entity: Entity, desc?: WalkGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `WalkGoalDesc` | yes | Where, how fast, and the pose on arrival. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or without drive, or a position that is not finite.

Binding symbol `og_motion_walk_to` · publish builds: kept

```ogt
fn walkTo {
  doc "Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive: the mover takes the Transform there at `speed` in a straight line, turning at turnSpeed, and the goal ends when it arrives."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc WalkGoalDesc "Where, how fast, and the pose on arrival."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive, or a position that is not finite."
}
```

<sub>`api.ogt` line 28931</sub>

<a id="fn-clear"></a>

### `og.motion.clear`

Drop the agent's goals, the current one included: the character holds where it is.

```ts
og.motion.clear(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent.

Binding symbol `og_motion_clear` · publish builds: kept

```ogt
fn clear {
  doc "Drop the agent's goals, the current one included: the character holds where it is."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  returns Result.Void "Refused for an entity without og.MotionAgent."
}
```

<sub>`api.ogt` line 28939</sub>

<a id="fn-status"></a>

### `og.motion.status`

The agent's current goal and queue.

```ts
og.motion.status(world: World, entity: Entity): Result<MotionStatus>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |

**Returns** `Result<MotionStatus>`: Refused for an entity without og.MotionAgent.

Binding symbol `og_motion_status` · publish builds: kept

```ogt
fn status {
  doc "The agent's current goal and queue."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  returns Result.MotionStatus "Refused for an entity without og.MotionAgent."
}
```

<sub>`api.ogt` line 28946</sub>

<a id="fn-capabilities"></a>

### `og.motion.capabilities`

Where the model runs: gpu once its weights are resident, none before (loading) or when they failed to load. Asking starts the load when no agent has yet.

```ts
og.motion.capabilities(world: World): MotionCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `MotionCapabilities`: Never refused.

Binding symbol `og_motion_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "Where the model runs: gpu once its weights are resident, none before (loading) or when they failed to load. Asking starts the load when no agent has yet."
  param world World "The world."
  returns MotionCapabilities "Never refused."
}
```

<sub>`api.ogt` line 28953</sub>

<a id="fn-poses"></a>

### `og.motion.poses`

The model's pose library, by name: idle, walk, run, jump, crouch, wave, cheer. Fixed by the engine's version, known before the weights are resident, so a goal's pose is checked the same everywhere.

```ts
og.motion.poses(): readonly string[]
```

**Returns** `readonly string[]`: The names.

Binding symbol `og_motion_poses` · publish builds: kept

```ogt
fn poses {
  doc "The model's pose library, by name: idle, walk, run, jump, crouch, wave, cheer. Fixed by the engine's version, known before the weights are resident, so a goal's pose is checked the same everywhere."
  returns List.String "The names."
}
```

<sub>`api.ogt` line 28959</sub>

## Components, resources and events

<a id="record-MotionAgent"></a>

### `og.MotionAgent` (component)

A humanoid the motion model animates: the entity needs an og.Skeleton whose asset is a humanoid rig (og.anim.humanoid recognises its bone names, or `rig` names them). Goals queue with og.motion.pose, place, face and walkTo. The generated pose is drawn in place of og.anim's (presentation only); og.anim still poses the skeleton every tick for everything the simulation reads, and is what is drawn until the model is resident.

Token `og.MotionAgent`, registered under the schema name `og.MotionAgent`; its value type is `MotionAgent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rig` | `Rig` | `0` | — | — | — | The humanoid map of the skeleton (og.load.rig); 0 detects the bones from their names, as og.anim.humanoid does. |
| `drive` | `boolean` | `false` | — | — | — | og.motion moves og.Transform to the goals in the tick (walkTo, place, face) with fixed math, and the pose is re-rooted onto it; off, the game moves the Transform and the motion follows it, and only pose goals are taken. |
| `turnSpeed` | `number` | `6` | 0.1 … 50 | `radps` | tunable | How fast a driven character turns to its heading. |
| `lookahead` | `number` | `0.6` | 0.1 … 4 | `s` | tunable | How much motion the model keeps generated ahead of what is drawn: more absorbs a slow GPU, less follows a changed goal sooner. Presentation only. |

```ogt
component MotionAgent {
  doc "A humanoid the motion model animates: the entity needs an og.Skeleton whose asset is a humanoid rig (og.anim.humanoid recognises its bone names, or `rig` names them). Goals queue with og.motion.pose, place, face and walkTo. The generated pose is drawn in place of og.anim's (presentation only); og.anim still poses the skeleton every tick for everything the simulation reads, and is what is drawn until the model is resident."
  field rig Rig "The humanoid map of the skeleton (og.load.rig); 0 detects the bones from their names, as og.anim.humanoid does."
  field drive bool "og.motion moves og.Transform to the goals in the tick (walkTo, place, face) with fixed math, and the pose is re-rooted onto it; off, the game moves the Transform and the motion follows it, and only pose goals are taken."
  field turnSpeed f32 "How fast a driven character turns to its heading." {
    default 6
    min 0.1
    max 50
    unit radps
    tunable true
  }
  field lookahead f32 "How much motion the model keeps generated ahead of what is drawn: more absorbs a slow GPU, less follows a changed goal sooner. Presentation only." {
    default 0.6
    min 0.1
    max 4
    unit s
    tunable true
  }
}
```

<sub>`api.ogt` line 28805</sub>

## Types

<a id="type-MotionGoalKind"></a>

### `MotionGoalKind` (enum)

What a goal asks for.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No goal: the character holds where its last goal left it. |
| `"pose"` | A library pose where the character stands, keeping its facing. |
| `"place"` | A library pose at a place, facing a heading. |
| `"face"` | Turn to a heading where the character stands. |
| `"walkTo"` | Walk or run to a point, then take a pose. |

```ogt
enum MotionGoalKind {
  doc "What a goal asks for."
  variant none "No goal: the character holds where its last goal left it."
  variant pose "A library pose where the character stands, keeping its facing."
  variant place "A library pose at a place, facing a heading."
  variant face "Turn to a heading where the character stands."
  variant walkTo "Walk or run to a point, then take a pose."
}
```

<sub>`api.ogt` line 28783</sub>

<a id="type-MotionBackend"></a>

### `MotionBackend` (enum)

Where the motion model runs.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Nowhere yet: the weights are not resident (still loading, or failed: MotionCapabilities.reason says why); agents draw og.anim's pose. |
| `"gpu"` | On the GPU beside the renderer. |

```ogt
enum MotionBackend {
  doc "Where the motion model runs."
  variant none "Nowhere yet: the weights are not resident (still loading, or failed: MotionCapabilities.reason says why); agents draw og.anim's pose."
  variant gpu "On the GPU beside the renderer."
}
```

<sub>`api.ogt` line 28799</sub>

<a id="type-PoseGoalDesc"></a>

### `PoseGoalDesc` (desc struct)

A pose of the model's library where the character stands (og.motion.pose).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pose` | `string` | `""` | — | — | — | The library pose (og.motion.poses); empty is `idle`. |
| `duration` | `number` | `1.5` | ≥ 0 | `s` | — | How long the goal is current. |

```ogt
struct PoseGoalDesc {
  doc "A pose of the model's library where the character stands (og.motion.pose)."
  desc true
  field pose String "The library pose (og.motion.poses); empty is `idle`."
  field duration f32 "How long the goal is current." {
    default 1.5
    min 0
    unit s
  }
}
```

<sub>`api.ogt` line 28825</sub>

<a id="type-PlaceGoalDesc"></a>

### `PlaceGoalDesc` (desc struct)

A pose of the library at a place (og.motion.place): the character's hips over `position`, facing `heading`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pose` | `string` | `""` | — | — | — | The library pose; empty is `idle`. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the character stands (y is ignored: the ground is the Transform's). |
| `heading` | `number` | `0` | — | `rad` | — | Its facing: 0 looks down +Z, π/2 down +X. |
| `duration` | `number` | `1.5` | ≥ 0 | `s` | — | How long the goal is current; a driven character moves there in a straight line over it. |

```ogt
struct PlaceGoalDesc {
  doc "A pose of the library at a place (og.motion.place): the character's hips over `position`, facing `heading`."
  desc true
  field pose String "The library pose; empty is `idle`."
  field position Vector3 "Where the character stands (y is ignored: the ground is the Transform's)." {
    unit m
  }
  field heading f32 "Its facing: 0 looks down +Z, π/2 down +X." {
    unit rad
  }
  field duration f32 "How long the goal is current; a driven character moves there in a straight line over it." {
    default 1.5
    min 0
    unit s
  }
}
```

<sub>`api.ogt` line 28836</sub>

<a id="type-FaceGoalDesc"></a>

### `FaceGoalDesc` (desc struct)

Turn to a heading where the character stands (og.motion.face).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `heading` | `number` | `0` | — | `rad` | — | The facing: 0 looks down +Z, π/2 down +X. |
| `duration` | `number` | `0.8` | ≥ 0 | `s` | — | How long the goal is current at least; a driven character's turn at turnSpeed may take longer. |

```ogt
struct FaceGoalDesc {
  doc "Turn to a heading where the character stands (og.motion.face)."
  desc true
  field heading f32 "The facing: 0 looks down +Z, π/2 down +X." {
    unit rad
  }
  field duration f32 "How long the goal is current at least; a driven character's turn at turnSpeed may take longer." {
    default 0.8
    min 0
    unit s
  }
}
```

<sub>`api.ogt` line 28853</sub>

<a id="type-WalkGoalDesc"></a>

### `WalkGoalDesc` (desc struct)

Walk or run to a point (og.motion.walkTo); needs og.MotionAgent.drive.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where to go (y is ignored). |
| `speed` | `number` | `1.4` | 0.1 … 10 | `mps` | — | How fast: the motion walks below 2.5 m/s and runs above. |
| `pose` | `string` | `""` | — | — | — | The pose taken on arrival; empty is `idle`. |

```ogt
struct WalkGoalDesc {
  doc "Walk or run to a point (og.motion.walkTo); needs og.MotionAgent.drive."
  desc true
  field position Vector3 "Where to go (y is ignored)." {
    unit m
  }
  field speed f32 "How fast: the motion walks below 2.5 m/s and runs above." {
    default 1.4
    min 0.1
    max 10
    unit mps
  }
  field pose String "The pose taken on arrival; empty is `idle`."
}
```

<sub>`api.ogt` line 28866</sub>

<a id="type-MotionStatus"></a>

### `MotionStatus` (struct)

An agent's goals, as the game set them: the current goal and how many wait. Deterministic: the same on every GPU, the model's progress never shows here.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `goal` | `MotionGoalKind` | `"none"` | — | — | — | The current goal (none: the queue is empty and the character holds). |
| `pose` | `string` | `""` | — | — | — | The current goal's pose (walkTo's arrival pose), empty for none and face. |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the current goal goes (pose and face: where the goal started). |
| `heading` | `number` | `0` | — | `rad` | — | The heading the current goal faces (walkTo: towards its target). |
| `elapsed` | `number` | `0` | — | `s` | — | How long the current goal has been current. |
| `queued` | `number` | `0` | — | — | — | Goals waiting after the current one. |
| `completed` | `number` | `0` | — | — | — | Goals finished since the agent's first. |

```ogt
struct MotionStatus {
  doc "An agent's goals, as the game set them: the current goal and how many wait. Deterministic: the same on every GPU, the model's progress never shows here."
  field goal MotionGoalKind "The current goal (none: the queue is empty and the character holds)."
  field pose String "The current goal's pose (walkTo's arrival pose), empty for none and face."
  field target Vector3 "Where the current goal goes (pose and face: where the goal started)." {
    unit m
  }
  field heading f32 "The heading the current goal faces (walkTo: towards its target)." {
    unit rad
  }
  field elapsed f32 "How long the current goal has been current." {
    unit s
  }
  field queued u32 "Goals waiting after the current one."
  field completed u32 "Goals finished since the agent's first."
}
```

<sub>`api.ogt` line 28881</sub>

<a id="type-MotionCapabilities"></a>

### `MotionCapabilities` (struct)

Where the motion model runs and what it knows. Presentation: a frame's answer, not the tick's; never branch the simulation on it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `backend` | `MotionBackend` | `"none"` | — | — | — | gpu once the weights are resident; none before, or when they failed. |
| `reason` | `string` | `""` | — | — | — | Why the backend is none: loading, the reason a load failed, or no GPU adapter (empty for gpu). |
| `modelBytes` | `number` | `0` | — | `bytes` | — | The size of cooked/motion/model.ogb, once read (0 before). |

```ogt
struct MotionCapabilities {
  doc "Where the motion model runs and what it knows. Presentation: a frame's answer, not the tick's; never branch the simulation on it."
  field backend MotionBackend "gpu once the weights are resident; none before, or when they failed."
  field reason String "Why the backend is none: loading, the reason a load failed, or no GPU adapter (empty for gpu)."
  field modelBytes f64 "The size of cooked/motion/model.ogb, once read (0 before)." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 28898</sub>

## Column kinds

Each is a factory, `MotionGoalKind(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-MotionGoalKind"></a>`MotionGoalKind` | `MotionGoalKind` | `Uint32Array` | 1 | A MotionGoalKind field: the variant's index. |

```ogt
column MotionGoalKind {
  doc "A MotionGoalKind field: the variant's index."
  array Uint32Array
  stride 1
  value MotionGoalKind
}
```

<sub>`api.ogt` line 28792</sub>
