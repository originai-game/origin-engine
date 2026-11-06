<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `motion` module

Generated motion for humanoids: the engine's motion model (a 17.6M-parameter student distilled from NVIDIA MotionBricks, og-motion) moves a character between the goals the game gives it, on the GPU. Give an entity with an og.Skeleton (a humanoid rig og.anim.humanoid recognises, or one an og.MotionAgent.rig names) an og.MotionAgent, then queue goals: og.motion.pose (a pose of the model's library where the character stands), og.motion.place (a pose at a place, facing a heading), og.motion.face (turn to a heading), og.motion.walkTo (walk or run to a point) and og.motion.jump (a jump whose arc the simulation owns). og.motion.poses lists the library: idle, walk, run, jump, crouch, wave, cheer.

**Presentation only.** The generated pose is drawn, never simulated (docs/design.md §2.4): it replaces og.anim's joint palette when the frame is drawn, and nothing a tick reads (og.anim's pose, its joint queries, root motion, physics, the world hash) depends on it, so a run is deterministic on every GPU. The model never moves the character: og.Transform is its root, always. With og.MotionAgent.drive the og.motion system moves the Transform to the goals itself with fixed math inside the tick (walkTo at its speed along a straight line, place over its duration, face at turnSpeed), and the generated pose is re-rooted onto the Transform, its planted feet absorbing the difference; without drive the game moves the Transform (its own physics, path or controller) and the character's motion follows it (walking, running or standing by how fast it moves). No model root motion ever reaches the simulation.

**Goals are the game's.** A goal is current from the tick it starts until it ends by fixed rules (walkTo when the mover arrives, pose, place and face after their duration), then the next queued one starts; og.motion.status answers the queue and the current goal, never the model's progress, so game code may branch on it. og.motion.presentation reports visual readiness for a host cover; game logic must not branch on it. The probe's `dump_motion` also reports readiness, generated frames, batches and timings.

**The model.** Its weights (35 MB in f16) ship inside @origingame/engine; `og cook` writes them as an optional file, cooked/motion/model.ogb, only for a project that uses og.motion (the manifest names it), and the engine reads and uploads it in the background as soon as the bundle is open, before any agent needs it (natively on a thread of its own; on the web a fetch after the boot bundle, never in it). The model runs on the GPU beside the renderer (in the engine Worker on the web): one batch for every agent at once, submitted in slices of about 3 ms of GPU time a frame, so a batch never costs a frame its vsync, and decoded a few characters a frame. Until the weights are resident, or when they fail to load, og.motion.capabilities answers backend none and each agent is drawn as og.anim poses it: its og.AnimationPlayer or og.AnimationGraph is the fallback, so give it an idle or walk clip.

- **TypeScript**: functions are the namespace `og.motion`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module motion {
  doc "Generated motion for humanoids: the engine's motion model (a 17.6M-parameter student distilled from NVIDIA MotionBricks, og-motion) moves a character between the goals the game gives it, on the GPU. Give an entity with an og.Skeleton (a humanoid rig og.anim.humanoid recognises, or one an og.MotionAgent.rig names) an og.MotionAgent, then queue goals: og.motion.pose (a pose of the model's library where the character stands), og.motion.place (a pose at a place, facing a heading), og.motion.face (turn to a heading), og.motion.walkTo (walk or run to a point) and og.motion.jump (a jump whose arc the simulation owns). og.motion.poses lists the library: idle, walk, run, jump, crouch, wave, cheer."
  doc "**Presentation only.** The generated pose is drawn, never simulated (docs/design.md §2.4): it replaces og.anim's joint palette when the frame is drawn, and nothing a tick reads (og.anim's pose, its joint queries, root motion, physics, the world hash) depends on it, so a run is deterministic on every GPU. The model never moves the character: og.Transform is its root, always. With og.MotionAgent.drive the og.motion system moves the Transform to the goals itself with fixed math inside the tick (walkTo at its speed along a straight line, place over its duration, face at turnSpeed), and the generated pose is re-rooted onto the Transform, its planted feet absorbing the difference; without drive the game moves the Transform (its own physics, path or controller) and the character's motion follows it (walking, running or standing by how fast it moves). No model root motion ever reaches the simulation."
  doc "**Goals are the game's.** A goal is current from the tick it starts until it ends by fixed rules (walkTo when the mover arrives, pose, place and face after their duration), then the next queued one starts; og.motion.status answers the queue and the current goal, never the model's progress, so game code may branch on it. og.motion.presentation reports visual readiness for a host cover; game logic must not branch on it. The probe's `dump_motion` also reports readiness, generated frames, batches and timings."
  doc "**The model.** Its weights (35 MB in f16) ship inside @origingame/engine; `og cook` writes them as an optional file, cooked/motion/model.ogb, only for a project that uses og.motion (the manifest names it), and the engine reads and uploads it in the background as soon as the bundle is open, before any agent needs it (natively on a thread of its own; on the web a fetch after the boot bundle, never in it). The model runs on the GPU beside the renderer (in the engine Worker on the web): one batch for every agent at once, submitted in slices of about 3 ms of GPU time a frame, so a batch never costs a frame its vsync, and decoded a few characters a frame. Until the weights are resident, or when they fail to load, og.motion.capabilities answers backend none and each agent is drawn as og.anim poses it: its og.AnimationPlayer or og.AnimationGraph is the fallback, so give it an idle or walk clip."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 26763</sub>

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
| [`ReactionDesc`](motion/type-ReactionDesc.md) | desc struct |  | An authored humanoid reaction over locomotion. |
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
| [`og.motion.capabilities`](motion/fn-capabilities.md) | function | `(world: World): MotionCapabilities` | Where the model runs: gpu once its weights are resident, none before (loading) or when they failed to load. |
| [`og.motion.presentation`](motion/fn-presentation.md) | function | `(world: World, entity: Entity): Result<MotionPresentationStatus>` | Presentation-only readiness on both hosts. |
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

<sub>`api.ogt` line 26779</sub>
