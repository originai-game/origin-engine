<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `motion` module

Generated motion for humanoids: the engine's motion model (a 17.6M-parameter student distilled from NVIDIA MotionBricks, og-motion) moves a character between the goals the game gives it, on the GPU. Give an entity with an og.Skeleton (a humanoid rig og.anim.humanoid recognises, or one an og.MotionAgent.rig names) an og.MotionAgent, then queue goals: og.motion.pose (a pose of the model's library where the character stands), og.motion.place (a pose at a place, facing a heading), og.motion.face (turn to a heading) and og.motion.walkTo (walk or run to a point). og.motion.poses lists the library: idle, walk, run, jump, crouch, wave, cheer.

**Presentation only.** The generated pose is drawn, never simulated (docs/design.md §2.4): it replaces og.anim's joint palette when the frame is drawn, and nothing a tick reads (og.anim's pose, its joint queries, root motion, physics, the world hash) depends on it, so a run is deterministic on every GPU. The model never moves the character: og.Transform is its root, always. With og.MotionAgent.drive the og.motion system moves the Transform to the goals itself with fixed math inside the tick (walkTo at its speed along a straight line, place over its duration, face at turnSpeed), and the generated pose is re-rooted onto the Transform, its planted feet absorbing the difference; without drive the game moves the Transform (its own physics, path or controller) and the character's motion follows it (walking, running or standing by how fast it moves). No model root motion ever reaches the simulation.

**Goals are the game's.** A goal is current from the tick it starts until it ends by fixed rules (walkTo when the mover arrives, pose, place and face after their duration), then the next queued one starts; og.motion.status answers the queue and the current goal, never the model's progress, so game code may branch on it. og.motion.presentation reports visual readiness for a host cover; game logic must not branch on it. The probe's `dump_motion` also reports readiness, generated frames, batches and timings.

**The model.** Its weights (35 MB in f16) ship inside @origingame/engine; `og cook` writes them as an optional file, cooked/motion/model.ogb, only for a project that uses og.motion (the manifest names it), and the engine reads and uploads it in the background as soon as the bundle is open, before any agent needs it (natively on a thread of its own; on the web a fetch after the boot bundle, never in it). The model runs on the GPU beside the renderer (in the engine Worker on the web): one batch for every agent at once, submitted in slices of about 3 ms of GPU time a frame, so a batch never costs a frame its vsync, and decoded a few characters a frame. Until the weights are resident, or when they fail to load, og.motion.capabilities answers backend none and each agent is drawn as og.anim poses it: its og.AnimationPlayer or og.AnimationGraph is the fallback, so give it an idle or walk clip.

- **TypeScript**: functions are the namespace `og.motion`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-anim`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module motion {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 28204</sub>

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
| [`LookAtDesc`](#type-LookAtDesc) | desc struct | A timed look over locomotion: spine, neck and head share the turn, never the hips or legs. An omitted target releases the look over blend seconds. |
| [`ReactionDesc`](#type-ReactionDesc) | desc struct | An authored humanoid reaction over locomotion. Entry, a held interval, and exit come from a cooked type motion-action asset. Timing and strength belong to the tick; the generated pose is presentation only. |
| [`MotionStatus`](#type-MotionStatus) | struct | An agent's goals, as the game set them: the current goal and how many wait. Deterministic: the same on every GPU, the model's progress never shows here. |
| [`MotionPresentationState`](#type-MotionPresentationState) | enum | Presentation progress, dependent on GPU timing, never simulation state. |
| [`MotionPresentationStatus`](#type-MotionPresentationStatus) | struct | Visual readiness only, outside snapshots and hashes. |
| [`MotionCapabilities`](#type-MotionCapabilities) | struct | Where the motion model runs and what it knows. Presentation: a frame's answer, not the tick's; never branch the simulation on it. |
| [`og.motion.pose`](#fn-pose) | function | Queue a library pose where the character stands, after its other goals. |
| [`og.motion.place`](#fn-place) | function | Queue a library pose at a place and heading; needs og.MotionAgent.drive (the mover takes the Transform there). |
| [`og.motion.face`](#fn-face) | function | Queue a turn to a heading where the character stands; needs og.MotionAgent.drive. |
| [`og.motion.walkTo`](#fn-walkTo) | function | Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive. The mover travels in a straight line up to speed, brakes at deceleration, and turns at turnSpeed. With a CharacterController it writes the controller's move input before physics; collision and ground height remain the controller's. The goal ends on actual arrival and completion of arrivalHeading, when supplied. |
| [`og.motion.clear`](#fn-clear) | function | Drop the agent's goals, the current one included: the character holds where it is. |
| [`og.motion.lookAt`](#fn-lookAt) | function | Replace the timed look overlay without interrupting locomotion. Empty desc releases it. State is snapshotted and advances only on ticks; the turn is applied after generated motion or the animation fallback, on both hosts. |
| [`og.motion.react`](#fn-react) | function | Replace the authored reaction overlay without discarding locomotion goals. Empty desc releases it. A motion-action asset names a humanoid clip and rig, hold interval and optional head-local hand contact. The hand is solved after the look so it stays on the hood. |
| [`og.motion.status`](#fn-status) | function | The agent's current goal and queue. |
| [`og.motion.capabilities`](#fn-capabilities) | function | Where the model runs: gpu once its weights are resident, none before (loading) or when they failed to load. Asking starts the load when no agent has yet. |
| [`og.motion.presentation`](#fn-presentation) | function | Presentation-only readiness on both hosts. Game logic must not branch on this value: it depends on GPU timing and is outside snapshots and hashes. A host may keep its page or shell cover up while the agent remains in the world and warms, then lift the cover on ready or failed (showing reason). Never write a component, gate a goal or advance a tick from this answer. ready requires generated frames and their palette for the current rendered frame; authored fallback does not count. Available in the probe's dump_motion too. |
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

<sub>`api.ogt` line 28417</sub>

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

<sub>`api.ogt` line 28425</sub>

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

<sub>`api.ogt` line 28433</sub>

<a id="fn-walkTo"></a>

### `og.motion.walkTo`

Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive. The mover travels in a straight line up to speed, brakes at deceleration, and turns at turnSpeed. With a CharacterController it writes the controller's move input before physics; collision and ground height remain the controller's. The goal ends on actual arrival and completion of arrivalHeading, when supplied.

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
  doc "Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive. The mover travels in a straight line up to speed, brakes at deceleration, and turns at turnSpeed. With a CharacterController it writes the controller's move input before physics; collision and ground height remain the controller's. The goal ends on actual arrival and completion of arrivalHeading, when supplied."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc WalkGoalDesc "Where, how fast, and the pose on arrival."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive, or a position that is not finite."
}
```

<sub>`api.ogt` line 28441</sub>

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

<sub>`api.ogt` line 28449</sub>

<a id="fn-lookAt"></a>

### `og.motion.lookAt`

Replace the timed look overlay without interrupting locomotion. Empty desc releases it. State is snapshotted and advances only on ticks; the turn is applied after generated motion or the animation fallback, on both hosts.

```ts
og.motion.lookAt(world: World, entity: Entity, desc?: LookAtDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent and humanoid og.Skeleton. |
| `desc` | `LookAtDesc` | yes | Target, limits and timing. |

**Returns** `Result<void>`: Refused for a missing agent, skeleton or mapped head/spine, or invalid inputs.

Binding symbol `og_motion_look_at` · publish builds: kept

```ogt
fn lookAt {
  doc "Replace the timed look overlay without interrupting locomotion. Empty desc releases it. State is snapshotted and advances only on ticks; the turn is applied after generated motion or the animation fallback, on both hosts."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent and humanoid og.Skeleton."
  param desc LookAtDesc "Target, limits and timing."
  returns Result.Void "Refused for a missing agent, skeleton or mapped head/spine, or invalid inputs."
}
```

<sub>`api.ogt` line 28456</sub>

<a id="fn-react"></a>

### `og.motion.react`

Replace the authored reaction overlay without discarding locomotion goals. Empty desc releases it. A motion-action asset names a humanoid clip and rig, hold interval and optional head-local hand contact. The hand is solved after the look so it stays on the hood.

```ts
og.motion.react(world: World, entity: Entity, desc?: ReactionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent and humanoid og.Skeleton. |
| `desc` | `ReactionDesc` | yes | Action, direction, strength and timing. |

**Returns** `Result<void>`: Refused for a missing agent, skeleton, action or required bones, an invalid hold interval, or invalid inputs.

Binding symbol `og_motion_react` · publish builds: kept

```ogt
fn react {
  doc "Replace the authored reaction overlay without discarding locomotion goals. Empty desc releases it. A motion-action asset names a humanoid clip and rig, hold interval and optional head-local hand contact. The hand is solved after the look so it stays on the hood."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent and humanoid og.Skeleton."
  param desc ReactionDesc "Action, direction, strength and timing."
  returns Result.Void "Refused for a missing agent, skeleton, action or required bones, an invalid hold interval, or invalid inputs."
}
```

<sub>`api.ogt` line 28464</sub>

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

<sub>`api.ogt` line 28472</sub>

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

<sub>`api.ogt` line 28479</sub>

<a id="fn-presentation"></a>

### `og.motion.presentation`

Presentation-only readiness on both hosts. Game logic must not branch on this value: it depends on GPU timing and is outside snapshots and hashes. A host may keep its page or shell cover up while the agent remains in the world and warms, then lift the cover on ready or failed (showing reason). Never write a component, gate a goal or advance a tick from this answer. ready requires generated frames and their palette for the current rendered frame; authored fallback does not count. Available in the probe's dump_motion too.

```ts
og.motion.presentation(world: World, entity: Entity): Result<MotionPresentationStatus>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |

**Returns** `Result<MotionPresentationStatus>`: Refused for an entity without og.MotionAgent.

Binding symbol `og_motion_presentation` · publish builds: kept

```ogt
fn presentation {
  doc "Presentation-only readiness on both hosts. Game logic must not branch on this value: it depends on GPU timing and is outside snapshots and hashes. A host may keep its page or shell cover up while the agent remains in the world and warms, then lift the cover on ready or failed (showing reason). Never write a component, gate a goal or advance a tick from this answer. ready requires generated frames and their palette for the current rendered frame; authored fallback does not count. Available in the probe's dump_motion too."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  returns Result.MotionPresentationStatus "Refused for an entity without og.MotionAgent."
}
```

<sub>`api.ogt` line 28485</sub>

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

<sub>`api.ogt` line 28492</sub>

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

<sub>`api.ogt` line 28233</sub>

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

<sub>`api.ogt` line 28211</sub>

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

<sub>`api.ogt` line 28227</sub>

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

<sub>`api.ogt` line 28253</sub>

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

<sub>`api.ogt` line 28264</sub>

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

<sub>`api.ogt` line 28281</sub>

<a id="type-WalkGoalDesc"></a>

### `WalkGoalDesc` (desc struct)

Walk or run to a point (og.motion.walkTo); needs og.MotionAgent.drive.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where to go (y is ignored). |
| `speed` | `number` | `1.4` | 0.1 … 10 | `mps` | — | How fast: the motion walks below 2.5 m/s and runs above. |
| `pose` | `string` | `""` | — | — | — | The pose taken on arrival; empty is `idle`. |
| `arrivalHeading` | `number` | `0` | — | `rad` | — | Facing after arrival; omitted keeps the travel heading. The goal ends only after this turn completes. |
| `deceleration` | `number` | `3` | ≥ 0 | — | — | Braking near the target; 0 uses 3 m/s². A CharacterController owns ground height and collision when present; otherwise motion keeps the Transform's height. |

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
  field arrivalHeading f32 "Facing after arrival; omitted keeps the travel heading. The goal ends only after this turn completes." {
    unit rad
  }
  field deceleration f32 "Braking near the target; 0 uses 3 m/s². A CharacterController owns ground height and collision when present; otherwise motion keeps the Transform's height." {
    default 3
    min 0
  }
}
```

<sub>`api.ogt` line 28294</sub>

<a id="type-LookAtDesc"></a>

### `LookAtDesc` (desc struct)

A timed look over locomotion: spine, neck and head share the turn, never the hips or legs. An omitted target releases the look over blend seconds.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | World point to look at; omitted releases the current look. |
| `duration` | `number` | `2` | ≥ 0 | `s` | — | Seconds held after blending in, before blending out. |
| `blend` | `number` | `0.2` | ≥ 0 | `s` | — | Seconds to enter and leave; 0 is immediate. |
| `maxYaw` | `number` | `1.92` | 0 … 3.141593 | `rad` | — | Maximum yaw from the animated forward. |
| `maxPitch` | `number` | `0.6` | 0 … 1.570796 | `rad` | — | Maximum pitch from the animated forward. |
| `weight` | `number` | `1` | 0 … 1 | — | — | Contribution of the look. |

```ogt
struct LookAtDesc {
  doc "A timed look over locomotion: spine, neck and head share the turn, never the hips or legs. An omitted target releases the look over blend seconds."
  desc true
  field target Vector3 "World point to look at; omitted releases the current look." {
    unit m
  }
  field duration f32 "Seconds held after blending in, before blending out." {
    default 2
    min 0
    unit s
  }
  field blend f32 "Seconds to enter and leave; 0 is immediate." {
    default 0.2
    min 0
    unit s
  }
  field maxYaw f32 "Maximum yaw from the animated forward." {
    default 1.92
    min 0
    max 3.141593
    unit rad
  }
  field maxPitch f32 "Maximum pitch from the animated forward." {
    default 0.6
    min 0
    max 1.570796
    unit rad
  }
  field weight f32 "Contribution of the look." {
    default 1
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 28316</sub>

<a id="type-ReactionDesc"></a>

### `ReactionDesc` (desc struct)

An authored humanoid reaction over locomotion. Entry, a held interval, and exit come from a cooked type motion-action asset. Timing and strength belong to the tick; the generated pose is presentation only.

**Action asset.** An .ogt starts with `type motion-action`, `version 1`, then `---`. Its body has `source "anims/brace.glb"`, `clip "brace"`, optional `rig "rigs/meshy.ogt"`, `holdStart 0.5` and `holdEnd 1.5`, each on its own line. Paths are relative to assets/. The hold must lie inside the clip (0 <= holdStart < holdEnd < clip duration), with matching poses at both ends for looping. Use a stationary root, +Z facing the reaction source, entry before the hold, and recovery after it. A 30 Hz clip is recommended; the cook also accepts other sample rates.

**Hood contact.** Optional `hand right` (or left) and `handTarget x y z` attach that wrist to an offset from the head during the hold. Supply the offset in metres along the source head bone's local axes, independent of the GLB's unit scale. Retargeting adjusts it to the target's proportions; IK follows the final head pose after lookAt. The clip owns the stance step and lean: the engine does not synthesize an untrained action. Keep provenance and licensing beside the source asset.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The motion-action asset path, such as actions/brace.ogt. Empty releases the current reaction. |
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | World direction the character reacts toward (toward the wind source for a brace); zero keeps the character's facing. |
| `strength` | `number` | `1` | 0 … 1 | — | — | Contribution of the authored reaction; 0 leaves locomotion unchanged. |
| `duration` | `number` | `1` | ≥ 0 | `s` | — | Seconds to hold between the action's holdStart and holdEnd, after its entry; then play its exit. |
| `blend` | `number` | `0.2` | ≥ 0 | `s` | — | Seconds to blend in and out; 0 is immediate. |

```ogt
struct ReactionDesc {
  doc "An authored humanoid reaction over locomotion. Entry, a held interval, and exit come from a cooked type motion-action asset. Timing and strength belong to the tick; the generated pose is presentation only."
  doc "**Action asset.** An .ogt starts with `type motion-action`, `version 1`, then `---`. Its body has `source \"anims/brace.glb\"`, `clip \"brace\"`, optional `rig \"rigs/meshy.ogt\"`, `holdStart 0.5` and `holdEnd 1.5`, each on its own line. Paths are relative to assets/. The hold must lie inside the clip (0 <= holdStart < holdEnd < clip duration), with matching poses at both ends for looping. Use a stationary root, +Z facing the reaction source, entry before the hold, and recovery after it. A 30 Hz clip is recommended; the cook also accepts other sample rates."
  doc "**Hood contact.** Optional `hand right` (or left) and `handTarget x y z` attach that wrist to an offset from the head during the hold. Supply the offset in metres along the source head bone's local axes, independent of the GLB's unit scale. Retargeting adjusts it to the target's proportions; IK follows the final head pose after lookAt. The clip owns the stance step and lean: the engine does not synthesize an untrained action. Keep provenance and licensing beside the source asset."
  desc true
  field action String "The motion-action asset path, such as actions/brace.ogt. Empty releases the current reaction."
  field direction Vector3 "World direction the character reacts toward (toward the wind source for a brace); zero keeps the character's facing."
  field strength f32 "Contribution of the authored reaction; 0 leaves locomotion unchanged." {
    default 1
    min 0
    max 1
  }
  field duration f32 "Seconds to hold between the action's holdStart and holdEnd, after its entry; then play its exit." {
    default 1
    min 0
    unit s
  }
  field blend f32 "Seconds to blend in and out; 0 is immediate." {
    default 0.2
    min 0
    unit s
  }
}
```

<sub>`api.ogt` line 28351</sub>

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
| `looking` | `boolean` | `false` | — | — | — | A look is active, including its entry or release. |
| `reacting` | `boolean` | `false` | — | — | — | A reaction is active, including its entry or release. |

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
  field looking bool "A look is active, including its entry or release."
  field reacting bool "A reaction is active, including its entry or release."
}
```

<sub>`api.ogt` line 28375</sub>

<a id="type-MotionPresentationState"></a>

### `MotionPresentationState` (enum)

Presentation progress, dependent on GPU timing, never simulation state.

A string: one of the variants below. The zero value is `"loading"`.

| Variant | Doc |
|---|---|
| `"loading"` | Waiting for the model or renderer. |
| `"warming"` | Weights are resident; this agent has no generated palette yet. |
| `"ready"` | Generated frames and their palette are ready for the current rendered frame; authored fallback never counts. |
| `"failed"` | Loading, inference or this agent's retarget failed; reason explains why. |

```ogt
enum MotionPresentationState {
  doc "Presentation progress, dependent on GPU timing, never simulation state."
  variant loading "Waiting for the model or renderer."
  variant warming "Weights are resident; this agent has no generated palette yet."
  variant ready "Generated frames and their palette are ready for the current rendered frame; authored fallback never counts."
  variant failed "Loading, inference or this agent's retarget failed; reason explains why."
}
```

<sub>`api.ogt` line 28394</sub>

<a id="type-MotionPresentationStatus"></a>

### `MotionPresentationStatus` (struct)

Visual readiness only, outside snapshots and hashes.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `state` | `MotionPresentationState` | `"loading"` | — | — | — | The agent's presentation progress. |
| `reason` | `string` | `""` | — | — | — | Why it is loading, warming or failed; empty when ready. |

```ogt
struct MotionPresentationStatus {
  doc "Visual readiness only, outside snapshots and hashes."
  field state MotionPresentationState "The agent's presentation progress."
  field reason String "Why it is loading, warming or failed; empty when ready."
}
```

<sub>`api.ogt` line 28402</sub>

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

<sub>`api.ogt` line 28408</sub>

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

<sub>`api.ogt` line 28220</sub>
