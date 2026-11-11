<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `motion` module

Generated motion for humanoids: the engine's motion model moves a character between the game's goals, on the GPU. Give a humanoid og.Skeleton an og.MotionAgent, then queue goals (og.motion.walkTo, og.motion.jump, og.motion.pose, ...).

Presentation only (docs/design.md §2.4): the generated pose replaces og.anim's joint palette when drawn, and nothing a tick reads depends on it. og.Transform is always the root. og.motion.status answers the goals by fixed rules; game logic never branches on og.motion.presentation. Until the weights are resident, og.anim's player or graph draws the agent.

See `animation §10`.

- **TypeScript**: functions are the namespace `og.motion`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module motion {
  doc "Generated motion for humanoids: the engine's motion model moves a character between the game's goals, on the GPU. Give a humanoid og.Skeleton an og.MotionAgent, then queue goals (og.motion.walkTo, og.motion.jump, og.motion.pose, ...)."
  doc "Presentation only (docs/design.md §2.4): the generated pose replaces og.anim's joint palette when drawn, and nothing a tick reads depends on it. og.Transform is always the root. og.motion.status answers the goals by fixed rules; game logic never branches on og.motion.presentation. Until the weights are resident, og.anim's player or graph draws the agent."
  # declarations follow, each on its own page
```

<sub>Module `motion` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`MotionGoalKind`](motion/type-MotionGoalKind.md) | enum |  | What a goal asks for. |
| [`MotionGoalKind()`](#column-MotionGoalKind) | column kind |  | A MotionGoalKind field: the variant's index. |
| [`MotionBackend`](motion/type-MotionBackend.md) | enum |  | Where the motion model runs. |
| [`og.MotionAgent`](motion/record-MotionAgent.md) | component |  | A humanoid the motion model animates: the entity needs an og.Skeleton whose asset is a humanoid rig (og.anim.humanoid recognises its bone names, or `rig` names them). |
| [`PoseGoalDesc`](motion/type-PoseGoalDesc.md) | desc struct |  | A pose of the model's library where the character stands (og.motion.pose). |
| [`PlaceGoalDesc`](motion/type-PlaceGoalDesc.md) | desc struct |  | A pose of the library at a place (og.motion.place): the character's hips over `position`, facing `heading`. |
| [`FaceGoalDesc`](motion/type-FaceGoalDesc.md) | desc struct |  | Turn to a heading where the character stands (og.motion.face). |
| [`WalkGoalDesc`](motion/type-WalkGoalDesc.md) | desc struct |  | Walk or run to a point (og.motion.walkTo); needs og.MotionAgent.drive. |
| [`JumpGoalDesc`](motion/type-JumpGoalDesc.md) | desc struct |  | A jump (og.motion.jump); needs og.MotionAgent.drive. |
| [`LookAtDesc`](motion/type-LookAtDesc.md) | desc struct |  | A timed look over locomotion: spine, neck and head share the turn, never the hips or legs. |
| [`ReactionDesc`](motion/type-ReactionDesc.md) | desc struct |  | An authored humanoid reaction over locomotion: entry, a held interval and exit from a cooked `type motion-action` asset (source, clip, optional rig, holdStart and holdEnd, optional hand and handTarge… |
| [`MotionStatus`](motion/type-MotionStatus.md) | struct |  | An agent's goals, as the game set them: the current goal and how many wait. |
| [`MotionPresentationState`](motion/type-MotionPresentationState.md) | enum |  | Presentation progress, dependent on GPU timing, never simulation state. |
| [`MotionPresentationStatus`](motion/type-MotionPresentationStatus.md) | struct |  | Visual readiness only, outside snapshots and hashes. |
| [`MotionCapabilities`](motion/type-MotionCapabilities.md) | struct |  | Where the motion model runs and what it knows. |
| [`og.motion.pose`](motion/fn-pose.md) | function | `(world: World, entity: Entity, desc?: PoseGoalDesc): Result<void>` | Queue a library pose where the character stands, after its other goals. |
| [`og.motion.place`](motion/fn-place.md) | function | `(world: World, entity: Entity, desc?: PlaceGoalDesc): Result<void>` | Queue a library pose at a place and heading; needs og.MotionAgent.drive (the mover takes the Transform there). |
| [`og.motion.face`](motion/fn-face.md) | function | `(world: World, entity: Entity, desc?: FaceGoalDesc): Result<void>` | Queue a turn to a heading where the character stands; needs og.MotionAgent.drive. |
| [`og.motion.walkTo`](motion/fn-walkTo.md) | function | `(world: World, entity: Entity, desc?: WalkGoalDesc): Result<void>` | Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive. |
| [`og.motion.jump`](motion/fn-jump.md) | function | `(world: World, entity: Entity, desc?: JumpGoalDesc): Result<void>` | Queue a jump after the agent's other goals; needs og.MotionAgent.drive. |
| [`og.motion.clear`](motion/fn-clear.md) | function | `(world: World, entity: Entity): Result<void>` | Drop the agent's goals, the current one included: the character holds where it is. |
| [`og.motion.lookAt`](motion/fn-lookAt.md) | function | `(world: World, entity: Entity, desc?: LookAtDesc): Result<void>` | Replace the timed look overlay without interrupting locomotion. |
| [`og.motion.react`](motion/fn-react.md) | function | `(world: World, entity: Entity, desc?: ReactionDesc): Result<void>` | Replace the authored reaction overlay without discarding locomotion goals. |
| [`og.motion.status`](motion/fn-status.md) | function | `(world: World, entity: Entity): Result<MotionStatus>` | The agent's current goal and queue. |
| [`og.motion.capabilities`](motion/fn-capabilities.md) | function | `(world: World): MotionCapabilities` | Where the model runs: gpu (or cpu with the opt-in) once its weights are resident, none before (loading) or when they failed to load. |
| [`og.motion.setBackend`](motion/fn-setBackend.md) | function | `(world: World, backend: MotionBackend): Result<void>` | Where the motion model runs: gpu (the default, beside the renderer, cheapest per character) or cpu (across the lanes, each generation spread over the ticks before it is due, for a few characters). |
| [`og.motion.presentation`](motion/fn-presentation.md) | function | `(world: World, entity: Entity): Result<MotionPresentationStatus>` | Presentation-only readiness on both hosts, for a host's cover: ready once generated frames and their palette exist for the current rendered frame (authored fallback does not count), or failed with a … |
| [`og.motion.poses`](motion/fn-poses.md) | function | `(): readonly string[]` | The model's pose library, by name: idle, walk, run, jump, crouch, wave, cheer. |

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

<sub>Module `motion` line 16</sub>
