<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `anim` module

Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics.

Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none.

Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names; a rig with other names or proportions plays it retargeted (og.anim.retarget at run time, or a model's `animation { library }` import setting at cook time). Assets are not world state: a hot reload registers them again.

- **TypeScript**: functions are the namespace `og.anim`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module anim {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 7777</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`SkeletonAsset`](#type-SkeletonAsset) | handle | A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices). |
| [`ClipAsset`](#type-ClipAsset) | handle | An animation clip, resampled at the cook's rate, with its markers. |
| [`GraphAsset`](#type-GraphAsset) | handle | A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles. |
| [`SkeletonAsset()`](#column-SkeletonAsset) | column kind | A SkeletonAsset handle field. |
| [`ClipAsset()`](#column-ClipAsset) | column kind | A ClipAsset handle field. |
| [`GraphAsset()`](#column-GraphAsset) | column kind | A GraphAsset handle field. |
| [`RootMotion`](#type-RootMotion) | enum | What the root joint's motion does to the entity. |
| [`RootMotion()`](#column-RootMotion) | column kind | A RootMotion field: the variant's index. |
| [`BlendshapeSource`](#type-BlendshapeSource) | enum | Who decides blendshape weights. |
| [`BlendshapeSource()`](#column-BlendshapeSource) | column kind | A BlendshapeSource field: the variant's index. |
| [`IkSolver`](#type-IkSolver) | enum | How an IK chain is solved. |
| [`IkSolver()`](#column-IkSolver) | column kind | An IkSolver field: the variant's index. |
| [`IkSpace`](#type-IkSpace) | enum | What an IK chain's target and pole are relative to. |
| [`IkSpace()`](#column-IkSpace) | column kind | An IkSpace field: the variant's index. |
| [`og.Skeleton`](#record-Skeleton) | component | A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block. |
| [`og.AnimationPlayer`](#record-AnimationPlayer) | component | Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick). |
| [`og.AnimationGraph`](#record-AnimationGraph) | component | Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param` is p1, ...). |
| [`og.Blendshapes`](#record-Blendshapes) | component | Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets. With `source` clips the playing clips' weight channels write them; with `component` game code does. |
| [`og.IkChain`](#record-IkChain) | component | An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order. |
| [`og.LookAt`](#record-LookAt) | component | An aim constraint on a skeleton, applied after its clips, graph and IK chains: `joint` turns so its `axis` points at `target` and its `upAxis` leans towards `up` (three.js Object3D.lookAt: +Z at the target, +Y up). The direction is held within maxYaw and maxPitch of the animated forward; with `bones` above 1 the joint's parents share the turn. Put it on its own entity (a head, the eyes, a turret's barrel); constraints apply in query order, and og.JointLimit clamps the joints it turns. |
| [`og.JointLimit`](#record-JointLimit) | component | How far a joint may turn from its rest rotation when og.anim's solvers (og.IkChain, og.LookAt, og.FootLock) turn it: a swing of `axis` within a cone and a twist about it within a range. A hinge (a knee, an elbow) is `axis` along the hinge, `swing` 0 (or a little play) and the bend range as twistMin...twistMax. Put it on its own entity, one per joint; the animated pose itself is never clamped. |
| [`og.FootLock`](#record-FootLock) | component | Foot locking: while the playing clips plant `foot` (within 5 mm of its lowest in each clip, the notion retargeting uses, weighed across blends and cross-fades: planted when the planting clips hold at least half the pose), the foot stays where it touched the ground, through clip blends and root motion, its leg bent by two-bone IK and the foot keeping its animated orientation; when the clips lift it, the foot rejoins its animation over `release`. Put it on its own entity, one per foot; locks apply after root motion and before og.IkChain, in query order, and og.JointLimit clamps the leg. The lock is world state (the `og.anim` state block). |
| [`SpringSpace`](#type-SpringSpace) | enum | Where a spring bone's tails move. |
| [`SpringSpace()`](#column-SpringSpace) | column kind | A SpringSpace field: the variant's index. |
| [`og.SpringBone`](#record-SpringBone) | component | A spring (jiggle) bone chain, three-vrm's VRMSpringBone: `joint` and its `bones` - 1 parents swing after their parents' motion. Each joint's tail (the next joint of the chain; for the last joint its first child, or its parent bone continued at the same length) is a point pulled towards where the animated pose puts it by a damped spring (`frequency`, `damping`), under `gravity`, held at the bone's length, and the joint turns to point at it. Applied after og.IkChain and og.LookAt, top joint first; put it on its own entity (a tail, an ear, a strand of hair, a single jiggling node), one per chain. Deterministic under the fixed tick; the tails' positions and velocities are world state (the `og.anim` state block), so a restore continues a swing exactly. A changed chain (another skeleton, length or space) restarts at the animated pose. |
| [`SpringColliderShape`](#type-SpringColliderShape) | enum | The shape of a spring collider. |
| [`SpringColliderShape()`](#column-SpringColliderShape) | column kind | A SpringColliderShape field: the variant's index. |
| [`og.SpringCollider`](#record-SpringCollider) | component | A collider spring bones are kept out of, three-vrm's VRMSpringBoneCollider: a sphere or a capsule riding on a joint of a skeleton (a head, a chest, an upper arm), so hair, ears and tails do not swing through the body. Each tick, after a tail of an og.SpringBone of the same skeleton has stepped and before its joint turns, the tail is pushed out of every collider whose `layers` meet the spring's `colliders` (in query order) to `radius` plus the spring's `radius`, keeping its bone's length. Put it on its own entity, one per shape; colliders follow the pose as it is when a chain is swung, so one on a joint an earlier chain swung follows that swing. |
| [`og.AnimationEvent`](#record-AnimationEvent) | event | A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers. |
| [`ClipInfo`](#type-ClipInfo) | struct | One clip of a loaded animation. |
| [`Animation`](#type-Animation) | struct | What og.load.animation registered from a model's `#anim` blob. |
| [`GraphDesc`](#type-GraphDesc) | desc struct | A blend-tree graph to register. |
| [`og.anim.graph`](#fn-graph) | function | Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`. |
| [`Rig`](#type-Rig) | handle | A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow, the standard bones it names outright and, with `footHeight`, how high its ankles stand above the ground. |
| [`Rig()`](#column-Rig) | column kind | A Rig handle field (0 is none): a humanoid map og.load.rig answered. |
| [`HumanBone`](#type-HumanBone) | struct | One standard humanoid bone of a skeleton. |
| [`Humanoid`](#type-Humanoid) | struct | Which node of a skeleton is each standard humanoid bone (what retargeting maps). |
| [`HumanoidDesc`](#type-HumanoidDesc) | desc struct | A skeleton and, optionally, the rig that says how its bones are named. |
| [`og.anim.humanoid`](#fn-humanoid) | function | The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it. |
| [`RetargetDesc`](#type-RetargetDesc) | desc struct | A clip made for one skeleton, to play on another (other proportions, rest pose or bone names). |
| [`og.anim.retarget`](#fn-retarget) | function | A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree; fingers turn from their hand, so a hand pose holds whatever the arm does), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled, on the ground each rig's `footHeight` names (else its rest pose's foot heights). The target's twist bones (unmapped nodes named `twist`, or Rigify's `DEF-….001`, on an upper or lower arm or leg) take a share of their segment's twist by how far along it they sit. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time. |
| [`MirrorDesc`](#type-MirrorDesc) | desc struct | A clip to play on the other side of the skeleton it was made for. |
| [`og.anim.mirror`](#fn-mirror) | function | A new clip that plays `clip` on the other side: every frame reflected in the skeleton's x = 0 plane, each node taking its counterpart's motion (a left swing becomes the right's), rotations moved in model space so left and right bone frames need not mirror each other, root motion's sideways travel and turns flipped. Assumes a symmetric rest pose; a sided node without a counterpart keeps its rest pose. Same name, duration and markers (markers keep their names). Mirroring twice gives the clip back. A clip known ahead needs no call: a graph's `clip` line with `mirror true` plays it mirrored, and the cook's `animation { mirror "<clip>" }` writes the mirrored clip into the model. |
| [`FlipbookSheet`](#type-FlipbookSheet) | handle | A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook. |
| [`FlipbookSheet()`](#column-FlipbookSheet) | column kind | A FlipbookSheet handle field. |
| [`FlipMode`](#type-FlipMode) | enum | How a flipbook clip runs past its last frame. |
| [`og.Flipbook`](#record-Flipbook) | component | Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or, for an `array true` sheet, the frame's layer of its texture array whole (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems. |
| [`og.FlipbookEvent`](#record-FlipbookEvent) | event | A flipbook entered a frame its clip names in an `event` line (`event 2 "step"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events. |
| [`FlipbookClipInfo`](#type-FlipbookClipInfo) | struct | One clip of a loaded flipbook sheet. |
| [`FlipbookInfo`](#type-FlipbookInfo) | struct | What og.load.flipbook registered. |
| [`og.anim.clip`](#fn-clip) | function | The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, "walk_down")`. -1 when the sheet has no such clip. |
| [`og.anim.clip`](#fn-clip) | function | A loaded model's clip by name, for AnimationPlayer.clip: `og.anim.clip(og.load.animation(world, "models/slime.glb").value, "chase")`, the same lookup as a flipbook's (Animation.clips by ClipInfo.name). A skinned glTF's clips are its animations' names; an animated `.vox`'s are `frames` or its `voxClip` names. 0 (plays nothing) when the model has no such clip. |
| [`og.anim.clip`](#fn-clip) | function | The index of a 2D rig's clip by name, for Skeleton2D.clip: `og.anim.clip(og.load.rig2d(world, "rigs/hero.ogt").value, "walk")`. -1 when the rig has no such clip. |
| [`Rig2D`](#type-Rig2D) | handle | A 2D rig (a cooked `.ogt` of `type rig2d`): bones, slots, region, mesh, bounding-box and child-armature attachments, IK constraints, child armatures and clips, from og.load.rig2d. |
| [`Rig2D()`](#column-Rig2D) | column kind | A Rig2D handle field. |
| [`og.Skeleton2D`](#record-Skeleton2D) | component | Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its slots, in their order, as textured triangles of the sprite batch at `layer` (region attachments as quads, mesh attachments weighted to bones and deformed by the clip), lit by og.Lighting2D when the layer is in its range, as a sprite there is. The rig's pixels are `unit` to a world unit and +y is up; the entity's og.Transform places, turns (about z) and scales it. |
| [`og.Skeleton2DEvent`](#record-Skeleton2DEvent) | event | A 2D skeleton's clip, or the clip a child armature in one of its slots plays, passed one of its events (`event 0.2 "step"` in the rig) in the last tick: footsteps, the frame a swing lands. Read in the tick after, like every event; a clip started at 0 fires its events at 0, a looping clip fires each once per pass. |
| [`og.IkTarget2D`](#record-IkTarget2D) | component | Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge, a tail's tip. A constraint is a chain of any length: one bone points at the target, two are solved exactly (bending the way the rig says), three or more by FABRIK from the animated pose, curling the way the rig says. Put it on its own entity, one per constraint; og.skeleton2d writes `reached` and `error`, and the drawn pose solves the same way. |
| [`Rig2DClipInfo`](#type-Rig2DClipInfo) | struct | One clip of a loaded 2D rig. |
| [`Rig2DInfo`](#type-Rig2DInfo) | struct | What og.load.rig2d registered, or og.anim.rig2dInfo reads back from a handle. |
| [`og.anim.rig2dInfo`](#fn-rig2dInfo) | function | What a loaded 2D rig holds: its bones, slots, IK constraints and clips, by the handle Skeleton2D.rig keeps (og.load.rig2d answers the same on load). |
| [`Skeleton2DBox`](#type-Skeleton2DBox) | struct | A bounding box a 2D skeleton's pose shows: a `box` attachment of its rig (or of a child armature in one of its slots) while its slot shows it. |
| [`og.anim.skeleton2dBoxes`](#fn-skeleton2dBoxes) | function | The bounding boxes an og.Skeleton2D's pose showed when og.skeleton2d last ran (after the game's systems, so a system reads the previous tick's): hurt and hit boxes a clip switches on and off with its slots' attachments, to test against points or other shapes. Empty for a rig without boxes or when no slot shows one. They are part of the og.skeleton2d state block, so a restored snapshot answers what it saved. |

## Functions

<a id="fn-graph"></a>

### `og.anim.graph`

Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`.

```ts
og.anim.graph(world: World, desc?: GraphDesc): Result<GraphAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the graph. |
| `desc` | `GraphDesc` | yes | The text and the clips it may name. |

**Returns** `Result<GraphAsset>`: Refused for a parse error (the reason names the line), a clip name `clips` lacks, or an invalid graph.

Binding symbol `og_anim_graph` · publish builds: kept

```ogt
fn graph {
  doc "Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`."
  param world World "The world whose animation owns the graph."
  param desc GraphDesc "The text and the clips it may name."
  returns Result.GraphAsset "Refused for a parse error (the reason names the line), a clip name `clips` lacks, or an invalid graph."
}
```

<sub>`api.ogt` line 8316</sub>

<a id="fn-humanoid"></a>

### `og.anim.humanoid`

The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it.

```ts
og.anim.humanoid(world: World, desc?: HumanoidDesc): Result<Humanoid>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the skeleton. |
| `desc` | `HumanoidDesc` | yes | The skeleton and its rig. |

**Returns** `Result<Humanoid>`: Refused for an unregistered skeleton or rig, or a rig naming a node the skeleton lacks.

Binding symbol `og_anim_humanoid` · publish builds: kept

```ogt
fn humanoid {
  doc "The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it."
  param world World "The world whose animation owns the skeleton."
  param desc HumanoidDesc "The skeleton and its rig."
  returns Result.Humanoid "Refused for an unregistered skeleton or rig, or a rig naming a node the skeleton lacks."
}
```

<sub>`api.ogt` line 8351</sub>

<a id="fn-retarget"></a>

### `og.anim.retarget`

A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree; fingers turn from their hand, so a hand pose holds whatever the arm does), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled, on the ground each rig's `footHeight` names (else its rest pose's foot heights). The target's twist bones (unmapped nodes named `twist`, or Rigify's `DEF-….001`, on an upper or lower arm or leg) take a share of their segment's twist by how far along it they sit. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time.

```ts
og.anim.retarget(world: World, desc?: RetargetDesc): Result<ClipInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the skeletons and the clip. |
| `desc` | `RetargetDesc` | yes | The clip, the two skeletons and their rigs. |

**Returns** `Result<ClipInfo>`: The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip, skeleton or rig, a skeleton without recognisable hips (name them in a `type rig` file), or a rig naming a missing node.

Binding symbol `og_anim_retarget` · publish builds: kept

```ogt
fn retarget {
  doc "A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree; fingers turn from their hand, so a hand pose holds whatever the arm does), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled, on the ground each rig's `footHeight` names (else its rest pose's foot heights). The target's twist bones (unmapped nodes named `twist`, or Rigify's `DEF-….001`, on an upper or lower arm or leg) take a share of their segment's twist by how far along it they sit. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time."
  param world World "The world whose animation owns the skeletons and the clip."
  param desc RetargetDesc "The clip, the two skeletons and their rigs."
  returns Result.ClipInfo "The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip, skeleton or rig, a skeleton without recognisable hips (name them in a `type rig` file), or a rig naming a missing node."
}
```

<sub>`api.ogt` line 8371</sub>

<a id="fn-mirror"></a>

### `og.anim.mirror`

A new clip that plays `clip` on the other side: every frame reflected in the skeleton's x = 0 plane, each node taking its counterpart's motion (a left swing becomes the right's), rotations moved in model space so left and right bone frames need not mirror each other, root motion's sideways travel and turns flipped. Assumes a symmetric rest pose; a sided node without a counterpart keeps its rest pose. Same name, duration and markers (markers keep their names). Mirroring twice gives the clip back. A clip known ahead needs no call: a graph's `clip` line with `mirror true` plays it mirrored, and the cook's `animation { mirror "<clip>" }` writes the mirrored clip into the model.

```ts
og.anim.mirror(world: World, desc?: MirrorDesc): Result<ClipInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the skeleton and the clip. |
| `desc` | `MirrorDesc` | yes | The clip and its skeleton. |

**Returns** `Result<ClipInfo>`: The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip or skeleton.

Binding symbol `og_anim_mirror` · publish builds: kept

```ogt
fn mirror {
  doc "A new clip that plays `clip` on the other side: every frame reflected in the skeleton's x = 0 plane, each node taking its counterpart's motion (a left swing becomes the right's), rotations moved in model space so left and right bone frames need not mirror each other, root motion's sideways travel and turns flipped. Assumes a symmetric rest pose; a sided node without a counterpart keeps its rest pose. Same name, duration and markers (markers keep their names). Mirroring twice gives the clip back. A clip known ahead needs no call: a graph's `clip` line with `mirror true` plays it mirrored, and the cook's `animation { mirror \"<clip>\" }` writes the mirrored clip into the model."
  param world World "The world whose animation owns the skeleton and the clip."
  param desc MirrorDesc "The clip and its skeleton."
  returns Result.ClipInfo "The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip or skeleton."
}
```

<sub>`api.ogt` line 8385</sub>

<a id="fn-clip"></a>

### `og.anim.clip`

The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, "walk_down")`. -1 when the sheet has no such clip.

```ts
og.anim.clip(info: FlipbookInfo, name: string): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `info` | `FlipbookInfo` | no | What og.load.flipbook answered. |
| `name` | `string` | no | The clip's name. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn clip {
  doc "The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, \"walk_down\")`. -1 when the sheet has no such clip."
  host false
  param info FlipbookInfo "What og.load.flipbook answered."
  param name String "The clip's name."
  returns i32
}
```

<sub>`api.ogt` line 8470</sub>

<a id="fn-clip-2"></a>

### `og.anim.clip`

A loaded model's clip by name, for AnimationPlayer.clip: `og.anim.clip(og.load.animation(world, "models/slime.glb").value, "chase")`, the same lookup as a flipbook's (Animation.clips by ClipInfo.name). A skinned glTF's clips are its animations' names; an animated `.vox`'s are `frames` or its `voxClip` names. 0 (plays nothing) when the model has no such clip.

```ts
og.anim.clip(info: Animation, name: string): ClipAsset
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `info` | `Animation` | no | What og.load.animation answered. |
| `name` | `string` | no | The clip's name. |

**Returns** `ClipAsset`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn clip {
  doc "A loaded model's clip by name, for AnimationPlayer.clip: `og.anim.clip(og.load.animation(world, \"models/slime.glb\").value, \"chase\")`, the same lookup as a flipbook's (Animation.clips by ClipInfo.name). A skinned glTF's clips are its animations' names; an animated `.vox`'s are `frames` or its `voxClip` names. 0 (plays nothing) when the model has no such clip."
  host false
  param info Animation "What og.load.animation answered."
  param name String "The clip's name."
  returns ClipAsset
}
```

<sub>`api.ogt` line 8478</sub>

<a id="fn-clip-3"></a>

### `og.anim.clip`

The index of a 2D rig's clip by name, for Skeleton2D.clip: `og.anim.clip(og.load.rig2d(world, "rigs/hero.ogt").value, "walk")`. -1 when the rig has no such clip.

```ts
og.anim.clip(info: Rig2DInfo, name: string): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `info` | `Rig2DInfo` | no | What og.load.rig2d or og.anim.rig2dInfo answered. |
| `name` | `string` | no | The clip's name. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn clip {
  doc "The index of a 2D rig's clip by name, for Skeleton2D.clip: `og.anim.clip(og.load.rig2d(world, \"rigs/hero.ogt\").value, \"walk\")`. -1 when the rig has no such clip."
  host false
  param info Rig2DInfo "What og.load.rig2d or og.anim.rig2dInfo answered."
  param name String "The clip's name."
  returns i32
}
```

<sub>`api.ogt` line 8486</sub>

<a id="fn-rig2dInfo"></a>

### `og.anim.rig2dInfo`

What a loaded 2D rig holds: its bones, slots, IK constraints and clips, by the handle Skeleton2D.rig keeps (og.load.rig2d answers the same on load).

```ts
og.anim.rig2dInfo(world: World, rig: Rig2D): Result<Rig2DInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the rig. |
| `rig` | `Rig2D` | no | The rig. |

**Returns** `Result<Rig2DInfo>`: Refused for a handle og.load.rig2d did not answer.

Binding symbol `og_anim_rig2d_info` · publish builds: kept

```ogt
fn rig2dInfo {
  doc "What a loaded 2D rig holds: its bones, slots, IK constraints and clips, by the handle Skeleton2D.rig keeps (og.load.rig2d answers the same on load)."
  param world World "The world whose animation owns the rig."
  param rig Rig2D "The rig."
  returns Result.Rig2DInfo "Refused for a handle og.load.rig2d did not answer."
}
```

<sub>`api.ogt` line 8596</sub>

<a id="fn-skeleton2dBoxes"></a>

### `og.anim.skeleton2dBoxes`

The bounding boxes an og.Skeleton2D's pose showed when og.skeleton2d last ran (after the game's systems, so a system reads the previous tick's): hurt and hit boxes a clip switches on and off with its slots' attachments, to test against points or other shapes. Empty for a rig without boxes or when no slot shows one. They are part of the og.skeleton2d state block, so a restored snapshot answers what it saved.

```ts
og.anim.skeleton2dBoxes(world: World, entity: Entity): Result<readonly Skeleton2DBox[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation poses the skeleton. |
| `entity` | `Entity` | no | The entity with the og.Skeleton2D. |

**Returns** `Result<readonly Skeleton2DBox[]>`: Refused for an entity og.skeleton2d has not posed: no og.Skeleton2D with a loaded rig, or before its first tick.

Binding symbol `og_anim_skeleton2d_boxes` · publish builds: kept

```ogt
fn skeleton2dBoxes {
  doc "The bounding boxes an og.Skeleton2D's pose showed when og.skeleton2d last ran (after the game's systems, so a system reads the previous tick's): hurt and hit boxes a clip switches on and off with its slots' attachments, to test against points or other shapes. Empty for a rig without boxes or when no slot shows one. They are part of the og.skeleton2d state block, so a restored snapshot answers what it saved."
  param world World "The world whose animation poses the skeleton."
  param entity Entity "The entity with the og.Skeleton2D."
  returns Result.List.Skeleton2DBox "Refused for an entity og.skeleton2d has not posed: no og.Skeleton2D with a loaded rig, or before its first tick."
}
```

<sub>`api.ogt` line 8610</sub>

## Components, resources and events

<a id="record-Skeleton"></a>

### `og.Skeleton` (component)

A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block.

Token `og.Skeleton`, registered under the schema name `og.Skeleton`; its value type is `Skeleton`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `SkeletonAsset` | `0` | — | — | — | The skeleton asset (og.load.animation's `skeleton`); 0 is none. |
| `skin` | `number` | `0` | ≥ 0 | — | — | Which of the asset's skins the joint palette is for (glTF skin index). |
| `rootMotion` | `RootMotion` | `"off"` | — | — | — | Root motion extraction: off (animate in place), planar (xz travel and yaw move og.Transform) or full. |
| `rootJoint` | `number` | `0` | ≥ 0 | — | — | The joint whose motion is extracted (an index into the skin's joints; 0 is its first joint, usually the hips). |
| `jointCount` | `number` | `0` | — | — | read only | Written: the number of joints in the palette (0 until the first tick). |

```ogt
component Skeleton {
  doc "A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block."
  field asset SkeletonAsset "The skeleton asset (og.load.animation's `skeleton`); 0 is none."
  field skin u32 "Which of the asset's skins the joint palette is for (glTF skin index)." {
    min 0
  }
  field rootMotion RootMotion "Root motion extraction: off (animate in place), planar (xz travel and yaw move og.Transform) or full."
  field rootJoint u32 "The joint whose motion is extracted (an index into the skin's joints; 0 is its first joint, usually the hips)." {
    min 0
  }
  field jointCount u32 "Written: the number of joints in the palette (0 until the first tick)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 7861</sub>

<a id="record-AnimationPlayer"></a>

### `og.AnimationPlayer` (component)

Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick).

Token `og.AnimationPlayer`, registered under the schema name `og.AnimationPlayer`; its value type is `AnimationPlayer`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | The clip asset; 0 plays nothing. |
| `time` | `number` | `0` | ≥ 0 | `s` | — | The playback position; written every tick, set it to seek (no markers fire across a seek). |
| `speed` | `number` | `1` | -10 … 10 | — | tunable | Playback rate (1 = as authored, negative plays backwards). |
| `loop` | `boolean` | `true` | — | — | — | Wrap at the end of the clip; otherwise hold the last frame and set `finished`. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the pose this player decides (0 = none, 1 = all). |
| `fade` | `number` | `0.2` | 0 … 10 | `s` | tunable | Cross-fade duration when `clip` changes. |
| `stepFps` | `number` | `0` | 0 … 60 | — | tunable | Stepped playback: each pose is held for 1/stepFps seconds, the clip sampled at its time floored to that step (12 is stop motion "on twos", 8 on threes), the same on every tick rate; 0 samples every tick. `time`, markers and root motion stay continuous; a one-shot still ends on its last frame. |
| `finished` | `boolean` | `false` | — | — | read only | Written: a non-looping clip reached its end. |

```ogt
component AnimationPlayer {
  doc "Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick)."
  field clip ClipAsset "The clip asset; 0 plays nothing."
  field time f32 "The playback position; written every tick, set it to seek (no markers fire across a seek)." {
    min 0
    unit s
  }
  field speed f32 "Playback rate (1 = as authored, negative plays backwards)." {
    default 1
    min -10
    max 10
    tunable true
  }
  field loop bool "Wrap at the end of the clip; otherwise hold the last frame and set `finished`." {
    default true
  }
  field weight f32 "How much of the pose this player decides (0 = none, 1 = all)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field fade f32 "Cross-fade duration when `clip` changes." {
    default 0.2
    min 0
    max 10
    unit s
    tunable true
  }
  field stepFps f32 "Stepped playback: each pose is held for 1/stepFps seconds, the clip sampled at its time floored to that step (12 is stop motion \"on twos\", 8 on threes), the same on every tick rate; 0 samples every tick. `time`, markers and root motion stay continuous; a one-shot still ends on its last frame." {
    min 0
    max 60
    tunable true
  }
  field finished bool "Written: a non-looping clip reached its end." {
    readonly true
  }
}
```

<sub>`api.ogt` line 7876</sub>

<a id="record-AnimationGraph"></a>

### `og.AnimationGraph` (component)

Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param` is p1, ...).

Token `og.AnimationGraph`, registered under the schema name `og.AnimationGraph`; its value type is `AnimationGraph`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `GraphAsset` | `0` | — | — | — | The graph asset; 0 is none. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the pose the graph decides over the rest pose. |
| `speed` | `number` | `1` | 0 … 10 | — | tunable | A playback-rate multiplier for every layer. |
| `p0` | `number` | `0` | — | — | tunable | Parameter 0 (the graph's first `param`). |
| `p1` | `number` | `0` | — | — | tunable | Parameter 1. |
| `p2` | `number` | `0` | — | — | tunable | Parameter 2. |
| `p3` | `number` | `0` | — | — | tunable | Parameter 3. |
| `p4` | `number` | `0` | — | — | tunable | Parameter 4. |
| `p5` | `number` | `0` | — | — | tunable | Parameter 5. |
| `p6` | `number` | `0` | — | — | tunable | Parameter 6. |
| `p7` | `number` | `0` | — | — | tunable | Parameter 7. |
| `state` | `number` | `0` | — | — | read only | Written: the base layer's current state (its index in the graph file). |
| `stateTime` | `number` | `0` | — | `s` | read only | Written: seconds since the base layer entered `state`. |

```ogt
component AnimationGraph {
  doc "Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param` is p1, ...)."
  field asset GraphAsset "The graph asset; 0 is none."
  field weight f32 "How much of the pose the graph decides over the rest pose." {
    default 1
    min 0
    max 1
    tunable true
  }
  field speed f32 "A playback-rate multiplier for every layer." {
    default 1
    min 0
    max 10
    tunable true
  }
  field p0 f32 "Parameter 0 (the graph's first `param`)." {
    tunable true
  }
  field p1 f32 "Parameter 1." {
    tunable true
  }
  field p2 f32 "Parameter 2." {
    tunable true
  }
  field p3 f32 "Parameter 3." {
    tunable true
  }
  field p4 f32 "Parameter 4." {
    tunable true
  }
  field p5 f32 "Parameter 5." {
    tunable true
  }
  field p6 f32 "Parameter 6." {
    tunable true
  }
  field p7 f32 "Parameter 7." {
    tunable true
  }
  field state u32 "Written: the base layer's current state (its index in the graph file)." {
    readonly true
  }
  field stateTime f32 "Written: seconds since the base layer entered `state`." {
    unit s
    readonly true
  }
}
```

<sub>`api.ogt` line 7915</sub>

<a id="record-Blendshapes"></a>

### `og.Blendshapes` (component)

Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets. With `source` clips the playing clips' weight channels write them; with `component` game code does.

Token `og.Blendshapes`, registered under the schema name `og.Blendshapes`; its value type is `Blendshapes`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `source` | `BlendshapeSource` | `"component"` | — | — | — | Who decides the weights: component (game code writes w0...w7) or clips (the animation writes them). |
| `mesh` | `number` | `0` | ≥ 0 | — | — | The glTF mesh index whose targets the weights drive (0 is the first mesh). |
| `w0` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 0. |
| `w1` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 1. |
| `w2` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 2. |
| `w3` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 3. |
| `w4` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 4. |
| `w5` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 5. |
| `w6` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 6. |
| `w7` | `number` | `0` | -1 … 2 | — | tunable | Weight of target 7. |

```ogt
component Blendshapes {
  doc "Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets. With `source` clips the playing clips' weight channels write them; with `component` game code does."
  field source BlendshapeSource "Who decides the weights: component (game code writes w0...w7) or clips (the animation writes them)."
  field mesh u32 "The glTF mesh index whose targets the weights drive (0 is the first mesh)." {
    min 0
  }
  field w0 f32 "Weight of target 0." {
    min -1
    max 2
    tunable true
  }
  field w1 f32 "Weight of target 1." {
    min -1
    max 2
    tunable true
  }
  field w2 f32 "Weight of target 2." {
    min -1
    max 2
    tunable true
  }
  field w3 f32 "Weight of target 3." {
    min -1
    max 2
    tunable true
  }
  field w4 f32 "Weight of target 4." {
    min -1
    max 2
    tunable true
  }
  field w5 f32 "Weight of target 5." {
    min -1
    max 2
    tunable true
  }
  field w6 f32 "Weight of target 6." {
    min -1
    max 2
    tunable true
  }
  field w7 f32 "Weight of target 7." {
    min -1
    max 2
    tunable true
  }
}
```

<sub>`api.ogt` line 7963</sub>

<a id="record-IkChain"></a>

### `og.IkChain` (component)

An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order.

Token `og.IkChain`, registered under the schema name `og.IkChain`; its value type is `IkChain`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton this chain bends. |
| `tip` | `number` | `0` | ≥ 0 | — | — | The end joint (an index into the skin's joints): the foot or the hand. |
| `bones` | `number` | `2` | 1 … 32 | — | — | How many bones above the tip turn (two-bone always uses 2). |
| `solver` | `IkSolver` | `"twoBone"` | — | — | — | twoBone (analytic, with a pole) or fabrik (iterative, any length). |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the tip should be, in `space`: for `world`, the frame of the skeleton entity's og.Transform (its parent's frame when it has a parent, not true world). |
| `pole` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend. |
| `space` | `IkSpace` | `"world"` | — | — | — | What `target` and `pole` are relative to: world (the frame the skeleton entity's og.Transform is in, so its parent's frame for a child entity, not true world) or model. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the solved rotation is applied (0 = the animated pose, 1 = fully solved). |
| `iterations` | `number` | `10` | 1 … 64 | — | — | FABRIK iterations at most. |
| `tolerance` | `number` | `0.001` | ≥ 0 | `m` | — | The tip distance (model units) at which the target counts as reached. |
| `reached` | `boolean` | `false` | — | — | read only | Written: the tip is within `tolerance` of the target after solving. |
| `error` | `number` | `0` | — | `m` | read only | Written: the tip's distance from the target after solving, in model units. |

```ogt
component IkChain {
  doc "An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order."
  field skeleton Entity "The entity with the og.Skeleton this chain bends."
  field tip u32 "The end joint (an index into the skin's joints): the foot or the hand." {
    min 0
  }
  field bones u32 "How many bones above the tip turn (two-bone always uses 2)." {
    default 2
    min 1
    max 32
  }
  field solver IkSolver "twoBone (analytic, with a pole) or fabrik (iterative, any length)."
  field target Vector3 "Where the tip should be, in `space`: for `world`, the frame of the skeleton entity's og.Transform (its parent's frame when it has a parent, not true world)." {
    unit m
  }
  field pole Vector3 "A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend." {
    unit m
  }
  field space IkSpace "What `target` and `pole` are relative to: world (the frame the skeleton entity's og.Transform is in, so its parent's frame for a child entity, not true world) or model."
  field weight f32 "How much of the solved rotation is applied (0 = the animated pose, 1 = fully solved)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field iterations u32 "FABRIK iterations at most." {
    default 10
    min 1
    max 64
  }
  field tolerance f32 "The tip distance (model units) at which the target counts as reached." {
    default 0.001
    min 0
    unit m
  }
  field reached bool "Written: the tip is within `tolerance` of the target after solving." {
    readonly true
  }
  field error f32 "Written: the tip's distance from the target after solving, in model units." {
    unit m
    readonly true
  }
}
```

<sub>`api.ogt` line 8011</sub>

<a id="record-LookAt"></a>

### `og.LookAt` (component)

An aim constraint on a skeleton, applied after its clips, graph and IK chains: `joint` turns so its `axis` points at `target` and its `upAxis` leans towards `up` (three.js Object3D.lookAt: +Z at the target, +Y up). The direction is held within maxYaw and maxPitch of the animated forward; with `bones` above 1 the joint's parents share the turn. Put it on its own entity (a head, the eyes, a turret's barrel); constraints apply in query order, and og.JointLimit clamps the joints it turns.

Token `og.LookAt`, registered under the schema name `og.LookAt`; its value type is `LookAt`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton this constraint turns. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The joint that aims (an index into the skin's joints): the head, an eye. |
| `bones` | `number` | `1` | 1 … 8 | — | — | How many joints share the turn: `joint` and its parents (3 spreads a look over neck, chest and spine), each parent taking an even share and `joint` the rest. |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The point to look at, in `space`. |
| `space` | `IkSpace` | `"world"` | — | — | — | What `target` and `up` are relative to: world (the frame the skeleton entity's og.Transform is in) or model. |
| `axis` | `Vector3` | `{ x: 0, y: 0, z: 1 }` | — | — | — | The joint's forward, in its own frame: the axis turned onto the target. |
| `upAxis` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The joint's up, in its own frame: yaw turns about it, pitch towards it, and it rolls towards `up`. |
| `up` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The up hint, a direction in `space` the joint's `upAxis` leans to (three.js Object3D.up); the zero vector keeps the animated roll. |
| `maxYaw` | `number` | `3.1416` | 0 … 3.1416 | `rad` | tunable | The largest turn about `upAxis`, left or right of the animated forward. |
| `maxPitch` | `number` | `1.5708` | 0 … 1.5708 | `rad` | tunable | The largest turn towards or away from `upAxis`, from the animated forward. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the turn is applied (0 = the animated pose, 1 = aimed). |
| `clamped` | `boolean` | `false` | — | — | read only | Written: the target lay outside maxYaw or maxPitch, so the joint aims at the nearest direction inside them. |

```ogt
component LookAt {
  doc "An aim constraint on a skeleton, applied after its clips, graph and IK chains: `joint` turns so its `axis` points at `target` and its `upAxis` leans towards `up` (three.js Object3D.lookAt: +Z at the target, +Y up). The direction is held within maxYaw and maxPitch of the animated forward; with `bones` above 1 the joint's parents share the turn. Put it on its own entity (a head, the eyes, a turret's barrel); constraints apply in query order, and og.JointLimit clamps the joints it turns."
  field skeleton Entity "The entity with the og.Skeleton this constraint turns."
  field joint u32 "The joint that aims (an index into the skin's joints): the head, an eye." {
    min 0
  }
  field bones u32 "How many joints share the turn: `joint` and its parents (3 spreads a look over neck, chest and spine), each parent taking an even share and `joint` the rest." {
    default 1
    min 1
    max 8
  }
  field target Vector3 "The point to look at, in `space`." {
    unit m
  }
  field space IkSpace "What `target` and `up` are relative to: world (the frame the skeleton entity's og.Transform is in) or model."
  field axis Vector3 "The joint's forward, in its own frame: the axis turned onto the target." {
    default 0 0 1
  }
  field upAxis Vector3 "The joint's up, in its own frame: yaw turns about it, pitch towards it, and it rolls towards `up`." {
    default 0 1 0
  }
  field up Vector3 "The up hint, a direction in `space` the joint's `upAxis` leans to (three.js Object3D.up); the zero vector keeps the animated roll." {
    default 0 1 0
  }
  field maxYaw f32 "The largest turn about `upAxis`, left or right of the animated forward." {
    default 3.1416
    min 0
    max 3.1416
    unit rad
    tunable true
  }
  field maxPitch f32 "The largest turn towards or away from `upAxis`, from the animated forward." {
    default 1.5708
    min 0
    max 1.5708
    unit rad
    tunable true
  }
  field weight f32 "How much of the turn is applied (0 = the animated pose, 1 = aimed)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field clamped bool "Written: the target lay outside maxYaw or maxPitch, so the joint aims at the nearest direction inside them." {
    readonly true
  }
}
```

<sub>`api.ogt` line 8055</sub>

<a id="record-JointLimit"></a>

### `og.JointLimit` (component)

How far a joint may turn from its rest rotation when og.anim's solvers (og.IkChain, og.LookAt, og.FootLock) turn it: a swing of `axis` within a cone and a twist about it within a range. A hinge (a knee, an elbow) is `axis` along the hinge, `swing` 0 (or a little play) and the bend range as twistMin...twistMax. Put it on its own entity, one per joint; the animated pose itself is never clamped.

Token `og.JointLimit`, registered under the schema name `og.JointLimit`; its value type is `JointLimit`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose joint this limits. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The limited joint (an index into the skin's joints). |
| `axis` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The twist axis in the joint's rest frame: along the bone for a ball joint, along the hinge for a hinge. |
| `swing` | `number` | `3.1416` | 0 … 3.1416 | `rad` | tunable | The cone half-angle the axis may swing away from its rest direction (0 locks it: a hinge; pi is free). |
| `twistMin` | `number` | `-3.1416` | -3.1416 … 3.1416 | `rad` | tunable | The least twist about `axis` from rest (a hinge's bend one way). |
| `twistMax` | `number` | `3.1416` | -3.1416 … 3.1416 | `rad` | tunable | The most twist about `axis` from rest (a hinge's bend the other way). |

```ogt
component JointLimit {
  doc "How far a joint may turn from its rest rotation when og.anim's solvers (og.IkChain, og.LookAt, og.FootLock) turn it: a swing of `axis` within a cone and a twist about it within a range. A hinge (a knee, an elbow) is `axis` along the hinge, `swing` 0 (or a little play) and the bend range as twistMin...twistMax. Put it on its own entity, one per joint; the animated pose itself is never clamped."
  field skeleton Entity "The entity with the og.Skeleton whose joint this limits."
  field joint u32 "The limited joint (an index into the skin's joints)." {
    min 0
  }
  field axis Vector3 "The twist axis in the joint's rest frame: along the bone for a ball joint, along the hinge for a hinge." {
    default 0 1 0
  }
  field swing f32 "The cone half-angle the axis may swing away from its rest direction (0 locks it: a hinge; pi is free)." {
    default 3.1416
    min 0
    max 3.1416
    unit rad
    tunable true
  }
  field twistMin f32 "The least twist about `axis` from rest (a hinge's bend one way)." {
    default -3.1416
    min -3.1416
    max 3.1416
    unit rad
    tunable true
  }
  field twistMax f32 "The most twist about `axis` from rest (a hinge's bend the other way)." {
    default 3.1416
    min -3.1416
    max 3.1416
    unit rad
    tunable true
  }
}
```

<sub>`api.ogt` line 8104</sub>

<a id="record-FootLock"></a>

### `og.FootLock` (component)

Foot locking: while the playing clips plant `foot` (within 5 mm of its lowest in each clip, the notion retargeting uses, weighed across blends and cross-fades: planted when the planting clips hold at least half the pose), the foot stays where it touched the ground, through clip blends and root motion, its leg bent by two-bone IK and the foot keeping its animated orientation; when the clips lift it, the foot rejoins its animation over `release`. Put it on its own entity, one per foot; locks apply after root motion and before og.IkChain, in query order, and og.JointLimit clamps the leg. The lock is world state (the `og.anim` state block).

Token `og.FootLock`, registered under the schema name `og.FootLock`; its value type is `FootLock`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose foot this holds. |
| `foot` | `number` | `0` | ≥ 0 | — | — | The foot joint (an index into the skin's joints); the two bones above it are the leg that bends. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the lock is applied (0 = the animated foot, 1 = held). |
| `release` | `number` | `0.15` | 0 … 2 | `s` | tunable | Seconds the foot takes to rejoin its animation once lifted (0: at once). |
| `maxDistance` | `number` | `0.5` | ≥ 0 | `m` | tunable | Farther than this from where the animation puts the foot, the lock lets go (a leg cannot stretch). |
| `planted` | `boolean` | `false` | — | — | read only | Written: the playing clips plant the foot this tick. |
| `locked` | `boolean` | `false` | — | — | read only | Written: the foot is held at its lock this tick. |

```ogt
component FootLock {
  doc "Foot locking: while the playing clips plant `foot` (within 5 mm of its lowest in each clip, the notion retargeting uses, weighed across blends and cross-fades: planted when the planting clips hold at least half the pose), the foot stays where it touched the ground, through clip blends and root motion, its leg bent by two-bone IK and the foot keeping its animated orientation; when the clips lift it, the foot rejoins its animation over `release`. Put it on its own entity, one per foot; locks apply after root motion and before og.IkChain, in query order, and og.JointLimit clamps the leg. The lock is world state (the `og.anim` state block)."
  field skeleton Entity "The entity with the og.Skeleton whose foot this holds."
  field foot u32 "The foot joint (an index into the skin's joints); the two bones above it are the leg that bends." {
    min 0
  }
  field weight f32 "How much of the lock is applied (0 = the animated foot, 1 = held)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field release f32 "Seconds the foot takes to rejoin its animation once lifted (0: at once)." {
    default 0.15
    min 0
    max 2
    unit s
    tunable true
  }
  field maxDistance f32 "Farther than this from where the animation puts the foot, the lock lets go (a leg cannot stretch)." {
    default 0.5
    min 0
    unit m
    tunable true
  }
  field planted bool "Written: the playing clips plant the foot this tick." {
    readonly true
  }
  field locked bool "Written: the foot is held at its lock this tick." {
    readonly true
  }
}
```

<sub>`api.ogt` line 8136</sub>

<a id="record-SpringBone"></a>

### `og.SpringBone` (component)

A spring (jiggle) bone chain, three-vrm's VRMSpringBone: `joint` and its `bones` - 1 parents swing after their parents' motion. Each joint's tail (the next joint of the chain; for the last joint its first child, or its parent bone continued at the same length) is a point pulled towards where the animated pose puts it by a damped spring (`frequency`, `damping`), under `gravity`, held at the bone's length, and the joint turns to point at it. Applied after og.IkChain and og.LookAt, top joint first; put it on its own entity (a tail, an ear, a strand of hair, a single jiggling node), one per chain. Deterministic under the fixed tick; the tails' positions and velocities are world state (the `og.anim` state block), so a restore continues a swing exactly. A changed chain (another skeleton, length or space) restarts at the animated pose.

Token `og.SpringBone`, registered under the schema name `og.SpringBone`; its value type is `SpringBone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose joints this swings. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The chain's last joint (an index into the skin's joints): the tip of a tail, an ear. |
| `bones` | `number` | `1` | 1 … 16 | — | — | How many joints swing: `joint` and its parents (1 is the joint alone). |
| `frequency` | `number` | `3` | 0 … 30 | `hz` | tunable | How fast a tail springs back to its animated place: the spring's natural frequency (0: no spring, it only hangs and swings). |
| `damping` | `number` | `0.3` | 0 … 2 | — | tunable | The damping ratio: 0 swings on, 1 settles without overshooting (critical damping). |
| `tipFrequency` | `number` | `1` | 0 … 4 | — | tunable | The last joint's frequency as a multiple of `frequency`, the joints between in proportion along the chain (1: every joint alike; 0.25: the tip springs back four times slower than the top, a soft strand on a stiff root). |
| `tipDamping` | `number` | `1` | 0 … 4 | — | tunable | The last joint's damping ratio as a multiple of `damping`, the joints between in proportion along the chain (1: every joint alike). |
| `gravity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | — | An acceleration on every tail, in `space` (three-vrm's gravityDir times gravityPower): (0, -9.8, 0) makes a chain droop. |
| `space` | `SpringSpace` | `"world"` | — | — | — | Where the tails move: world (the frame the skeleton entity's og.Transform is in, so the chain lags behind the entity's motion) or model (only the animation moves it). |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the swing is applied (0 = the animated pose, 1 = the spring's). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | The tails' radius against og.SpringCollider (three-vrm's hitRadius): a tail stays this far outside every collider it meets (0: a point). |
| `colliders` | `number` | `4294967295` | — | — | — | The og.SpringCollider layers the tails are pushed out of, as bits: a collider on the same skeleton counts when its `layers` meet these. |
| `lag` | `number` | `0` | — | `m` | read only | Written: the farthest a tail is from its animated place this tick. |

```ogt
component SpringBone {
  doc "A spring (jiggle) bone chain, three-vrm's VRMSpringBone: `joint` and its `bones` - 1 parents swing after their parents' motion. Each joint's tail (the next joint of the chain; for the last joint its first child, or its parent bone continued at the same length) is a point pulled towards where the animated pose puts it by a damped spring (`frequency`, `damping`), under `gravity`, held at the bone's length, and the joint turns to point at it. Applied after og.IkChain and og.LookAt, top joint first; put it on its own entity (a tail, an ear, a strand of hair, a single jiggling node), one per chain. Deterministic under the fixed tick; the tails' positions and velocities are world state (the `og.anim` state block), so a restore continues a swing exactly. A changed chain (another skeleton, length or space) restarts at the animated pose."
  field skeleton Entity "The entity with the og.Skeleton whose joints this swings."
  field joint u32 "The chain's last joint (an index into the skin's joints): the tip of a tail, an ear." {
    min 0
  }
  field bones u32 "How many joints swing: `joint` and its parents (1 is the joint alone)." {
    default 1
    min 1
    max 16
  }
  field frequency f32 "How fast a tail springs back to its animated place: the spring's natural frequency (0: no spring, it only hangs and swings)." {
    default 3
    min 0
    max 30
    unit hz
    tunable true
  }
  field damping f32 "The damping ratio: 0 swings on, 1 settles without overshooting (critical damping)." {
    default 0.3
    min 0
    max 2
    tunable true
  }
  field tipFrequency f32 "The last joint's frequency as a multiple of `frequency`, the joints between in proportion along the chain (1: every joint alike; 0.25: the tip springs back four times slower than the top, a soft strand on a stiff root)." {
    default 1
    min 0
    max 4
    tunable true
  }
  field tipDamping f32 "The last joint's damping ratio as a multiple of `damping`, the joints between in proportion along the chain (1: every joint alike)." {
    default 1
    min 0
    max 4
    tunable true
  }
  field gravity Vector3 "An acceleration on every tail, in `space` (three-vrm's gravityDir times gravityPower): (0, -9.8, 0) makes a chain droop." {
    unit mps2
  }
  field space SpringSpace "Where the tails move: world (the frame the skeleton entity's og.Transform is in, so the chain lags behind the entity's motion) or model (only the animation moves it)."
  field weight f32 "How much of the swing is applied (0 = the animated pose, 1 = the spring's)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field radius f32 "The tails' radius against og.SpringCollider (three-vrm's hitRadius): a tail stays this far outside every collider it meets (0: a point)." {
    min 0
    unit m
    tunable true
  }
  field colliders u32 "The og.SpringCollider layers the tails are pushed out of, as bits: a collider on the same skeleton counts when its `layers` meet these." {
    default 4294967295
  }
  field lag f32 "Written: the farthest a tail is from its animated place this tick." {
    readonly true
    unit m
  }
}
```

<sub>`api.ogt` line 8182</sub>

<a id="record-SpringCollider"></a>

### `og.SpringCollider` (component)

A collider spring bones are kept out of, three-vrm's VRMSpringBoneCollider: a sphere or a capsule riding on a joint of a skeleton (a head, a chest, an upper arm), so hair, ears and tails do not swing through the body. Each tick, after a tail of an og.SpringBone of the same skeleton has stepped and before its joint turns, the tail is pushed out of every collider whose `layers` meet the spring's `colliders` (in query order) to `radius` plus the spring's `radius`, keeping its bone's length. Put it on its own entity, one per shape; colliders follow the pose as it is when a chain is swung, so one on a joint an earlier chain swung follows that swing.

Token `og.SpringCollider`, registered under the schema name `og.SpringCollider`; its value type is `SpringCollider`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose spring bones this keeps out. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The joint the collider rides on (an index into the skin's joints). |
| `shape` | `SpringColliderShape` | `"sphere"` | — | — | — | A sphere about `offset`, or a capsule from `offset` to `tail`. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sphere's centre, or the capsule's first end, in the joint's frame. |
| `tail` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The capsule's other end, in the joint's frame (a sphere ignores it). |
| `radius` | `number` | `0.1` | ≥ 0 | `m` | tunable | The radius, in metres of the skeleton's model space (the joint's own scale does not apply). |
| `layers` | `number` | `1` | — | — | — | The layers this collider is on, as bits (a spring bone's `colliders` picks them). |

```ogt
component SpringCollider {
  doc "A collider spring bones are kept out of, three-vrm's VRMSpringBoneCollider: a sphere or a capsule riding on a joint of a skeleton (a head, a chest, an upper arm), so hair, ears and tails do not swing through the body. Each tick, after a tail of an og.SpringBone of the same skeleton has stepped and before its joint turns, the tail is pushed out of every collider whose `layers` meet the spring's `colliders` (in query order) to `radius` plus the spring's `radius`, keeping its bone's length. Put it on its own entity, one per shape; colliders follow the pose as it is when a chain is swung, so one on a joint an earlier chain swung follows that swing."
  field skeleton Entity "The entity with the og.Skeleton whose spring bones this keeps out."
  field joint u32 "The joint the collider rides on (an index into the skin's joints)." {
    min 0
  }
  field shape SpringColliderShape "A sphere about `offset`, or a capsule from `offset` to `tail`."
  field offset Vector3 "The sphere's centre, or the capsule's first end, in the joint's frame." {
    unit m
  }
  field tail Vector3 "The capsule's other end, in the joint's frame (a sphere ignores it)." {
    unit m
  }
  field radius f32 "The radius, in metres of the skeleton's model space (the joint's own scale does not apply)." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field layers u32 "The layers this collider is on, as bits (a spring bone's `colliders` picks them)." {
    default 1
  }
}
```

<sub>`api.ogt` line 8255</sub>

<a id="record-AnimationEvent"></a>

### `og.AnimationEvent` (event)

A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers.

Token `og.AnimationEvent`, registered under the schema name `og.AnimationEvent`; its value type is `AnimationEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The skeleton entity. |
| `clip` | `ClipAsset` | `0` | — | — | — | The clip asset the marker belongs to. |
| `name` | `string` | `""` | — | — | — | The marker's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | The marker's time in the clip. |
| `weight` | `number` | `0` | — | — | — | The clip's blend weight when it fired. |

```ogt
event AnimationEvent {
  doc "A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers."
  field entity Entity "The skeleton entity."
  field clip ClipAsset "The clip asset the marker belongs to."
  field name String "The marker's name (at most 63 bytes)."
  field time f32 "The marker's time in the clip." {
    unit s
  }
  field weight f32 "The clip's blend weight when it fired."
}
```

<sub>`api.ogt` line 8279</sub>

<a id="record-Flipbook"></a>

### `og.Flipbook` (component)

Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or, for an `array true` sheet, the frame's layer of its texture array whole (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems.

Changing clip restarts at the clip's first frame (or at `time`, written in the same tick); writing a different time is a seek. Frame events are og.FlipbookEvent. The playback position is world state (the og.flipbook state block).

Token `og.Flipbook`, registered under the schema name `og.Flipbook`; its value type is `Flipbook`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sheet` | `FlipbookSheet` | `0` | — | — | — | The sheet (og.load.flipbook's `sheet`); 0 is none. |
| `clip` | `number` | `0` | ≥ 0 | — | — | The clip: an index into FlipbookInfo.clips (og.anim.clip finds one by name). |
| `speed` | `number` | `1` | 0 … 10 | — | tunable | The playback rate: 1 as authored, 2 twice as fast, 0 holds the frame. |
| `playing` | `boolean` | `true` | — | — | — | Whether time advances. |
| `frame` | `number` | `0` | — | — | read only | Written: the clip frame shown (an index into the clip's frames). |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Seconds into the current cycle; written every tick, set it to seek (no events fire across a seek). |
| `finished` | `boolean` | `false` | — | — | read only | Written: a `once` clip reached and holds its last frame. |
| `loops` | `number` | `0` | — | — | read only | Written: cycles completed by a looping or ping-pong clip. |

```ogt
component Flipbook {
  doc "Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or, for an `array true` sheet, the frame's layer of its texture array whole (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems."
  doc "Changing clip restarts at the clip's first frame (or at `time`, written in the same tick); writing a different time is a seek. Frame events are og.FlipbookEvent. The playback position is world state (the og.flipbook state block)."
  field sheet FlipbookSheet "The sheet (og.load.flipbook's `sheet`); 0 is none."
  field clip u32 "The clip: an index into FlipbookInfo.clips (og.anim.clip finds one by name)." {
    min 0
  }
  field speed f32 "The playback rate: 1 as authored, 2 twice as fast, 0 holds the frame." {
    default 1
    min 0
    max 10
    tunable true
  }
  field playing bool "Whether time advances." {
    default true
  }
  field frame u32 "Written: the clip frame shown (an index into the clip's frames)." {
    readonly true
  }
  field time f32 "Seconds into the current cycle; written every tick, set it to seek (no events fire across a seek)." {
    min 0
    unit s
  }
  field finished bool "Written: a `once` clip reached and holds its last frame." {
    readonly true
  }
  field loops u32 "Written: cycles completed by a looping or ping-pong clip." {
    readonly true
  }
}
```

<sub>`api.ogt` line 8408</sub>

<a id="record-FlipbookEvent"></a>

### `og.FlipbookEvent` (event)

A flipbook entered a frame its clip names in an `event` line (`event 2 "step"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events.

Token `og.FlipbookEvent`, registered under the schema name `og.FlipbookEvent`; its value type is `FlipbookEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Flipbook. |
| `clip` | `number` | `0` | — | — | — | The clip index. |
| `frame` | `number` | `0` | — | — | — | The clip frame entered. |
| `name` | `string` | `""` | — | — | — | The event's name (at most 63 bytes). |

```ogt
event FlipbookEvent {
  doc "A flipbook entered a frame its clip names in an `event` line (`event 2 \"step\"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events."
  field entity Entity "The entity with the og.Flipbook."
  field clip u32 "The clip index."
  field frame u32 "The clip frame entered."
  field name String "The event's name (at most 63 bytes)."
}
```

<sub>`api.ogt` line 8439</sub>

<a id="record-Skeleton2D"></a>

### `og.Skeleton2D` (component)

Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its slots, in their order, as textured triangles of the sprite batch at `layer` (region attachments as quads, mesh attachments weighted to bones and deformed by the clip), lit by og.Lighting2D when the layer is in its range, as a sprite there is. The rig's pixels are `unit` to a world unit and +y is up; the entity's og.Transform places, turns (about z) and scales it.

Changing `clip` restarts at 0 (or at `time`, written in the same tick); writing a different time is a seek and fires no events. The pose is a function of these fields, the Transform and the og.IkTarget2D on it; the playback position is world state (the og.skeleton2d state block).

Token `og.Skeleton2D`, registered under the schema name `og.Skeleton2D`; its value type is `Skeleton2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rig` | `Rig2D` | `0` | — | — | — | The rig (og.load.rig2d's `rig`); 0 is none. |
| `clip` | `number` | `0` | ≥ 0 | — | — | The clip: an index into Rig2DInfo.clips (og.anim.clip finds one by name). |
| `time` | `number` | `0` | ≥ 0 | `s` | — | Seconds into the clip; written every tick, set it to seek. |
| `speed` | `number` | `1` | -10 … 10 | — | tunable | The playback rate: 1 as authored, 2 twice as fast, negative backwards. |
| `loop` | `boolean` | `true` | — | — | — | Wrap at the clip's end; otherwise hold the last pose and set `finished`. Rig2DInfo.clips says what the rig's author meant. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror the rig left to right about the entity's position (a character facing left). |
| `layer` | `number` | `0` | — | — | — | The sprite layer it draws in, sorted with the sprites there (after them when they share the layer). |
| `skin` | `number` | `0` | ≥ 0 | — | — | The skin it wears: an index into Rig2DInfo.skins (0 is the rig's default; `rig.skins.indexOf("armor")` finds one by name). A skin shows its own attachment wherever the setup pose or a clip shows the default's; a slot the skin leaves alone shows the default's. Change it any tick: the pose is drawn from it. |
| `finished` | `boolean` | `false` | — | — | read only | Written: a clip that does not loop reached its end (or its start, played backwards). |

```ogt
component Skeleton2D {
  doc "Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its slots, in their order, as textured triangles of the sprite batch at `layer` (region attachments as quads, mesh attachments weighted to bones and deformed by the clip), lit by og.Lighting2D when the layer is in its range, as a sprite there is. The rig's pixels are `unit` to a world unit and +y is up; the entity's og.Transform places, turns (about z) and scales it."
  doc "Changing `clip` restarts at 0 (or at `time`, written in the same tick); writing a different time is a seek and fires no events. The pose is a function of these fields, the Transform and the og.IkTarget2D on it; the playback position is world state (the og.skeleton2d state block)."
  field rig Rig2D "The rig (og.load.rig2d's `rig`); 0 is none."
  field clip u32 "The clip: an index into Rig2DInfo.clips (og.anim.clip finds one by name)." {
    min 0
  }
  field time f32 "Seconds into the clip; written every tick, set it to seek." {
    min 0
    unit s
  }
  field speed f32 "The playback rate: 1 as authored, 2 twice as fast, negative backwards." {
    default 1
    min -10
    max 10
    tunable true
  }
  field loop bool "Wrap at the clip's end; otherwise hold the last pose and set `finished`. Rig2DInfo.clips says what the rig's author meant." {
    default true
  }
  field flipX bool "Mirror the rig left to right about the entity's position (a character facing left)."
  field layer i32 "The sprite layer it draws in, sorted with the sprites there (after them when they share the layer)."
  field skin u32 "The skin it wears: an index into Rig2DInfo.skins (0 is the rig's default; `rig.skins.indexOf(\"armor\")` finds one by name). A skin shows its own attachment wherever the setup pose or a clip shows the default's; a slot the skin leaves alone shows the default's. Change it any tick: the pose is drawn from it." {
    min 0
  }
  field finished bool "Written: a clip that does not loop reached its end (or its start, played backwards)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 8503</sub>

<a id="record-Skeleton2DEvent"></a>

### `og.Skeleton2DEvent` (event)

A 2D skeleton's clip, or the clip a child armature in one of its slots plays, passed one of its events (`event 0.2 "step"` in the rig) in the last tick: footsteps, the frame a swing lands. Read in the tick after, like every event; a clip started at 0 fires its events at 0, a looping clip fires each once per pass.

Token `og.Skeleton2DEvent`, registered under the schema name `og.Skeleton2DEvent`; its value type is `Skeleton2DEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton2D. |
| `clip` | `number` | `0` | — | — | — | The clip index. |
| `name` | `string` | `""` | — | — | — | The event's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | The event's time in the clip. |
| `armature` | `string` | `""` | — | — | — | Empty for the skeleton's own clip; for an event of a child armature shown in one of its slots, that armature's name (`clip` and `time` are then the child's clip and its time): a flame's crackle apart from the keeper's footsteps (at most 63 bytes). |

```ogt
event Skeleton2DEvent {
  doc "A 2D skeleton's clip, or the clip a child armature in one of its slots plays, passed one of its events (`event 0.2 \"step\"` in the rig) in the last tick: footsteps, the frame a swing lands. Read in the tick after, like every event; a clip started at 0 fires its events at 0, a looping clip fires each once per pass."
  field entity Entity "The entity with the og.Skeleton2D."
  field clip u32 "The clip index."
  field name String "The event's name (at most 63 bytes)."
  field time f32 "The event's time in the clip." {
    unit s
  }
  field armature String "Empty for the skeleton's own clip; for an event of a child armature shown in one of its slots, that armature's name (`clip` and `time` are then the child's clip and its time): a flame's crackle apart from the keeper's footsteps (at most 63 bytes)."
}
```

<sub>`api.ogt` line 8533</sub>

<a id="record-IkTarget2D"></a>

### `og.IkTarget2D` (component)

Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge, a tail's tip. A constraint is a chain of any length: one bone points at the target, two are solved exactly (bending the way the rig says), three or more by FABRIK from the animated pose, curling the way the rig says. Put it on its own entity, one per constraint; og.skeleton2d writes `reached` and `error`, and the drawn pose solves the same way.

Token `og.IkTarget2D`, registered under the schema name `og.IkTarget2D`; its value type is `IkTarget2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton2D. |
| `constraint` | `number` | `0` | ≥ 0 | — | — | The constraint: an index into Rig2DInfo.constraints. |
| `target` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Where the chain's tip should be, in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent). |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the solve applies (0 is the animated pose, 1 fully solved), times the constraint's own weight (its `weight` line, or its clip's IK keys). |
| `tolerance` | `number` | `0.01` | ≥ 0 | `m` | — | The distance at which the target counts as reached. |
| `reached` | `boolean` | `false` | — | — | read only | Written: the tip is within `tolerance` of the target after solving. |
| `error` | `number` | `0` | — | `m` | read only | Written: the tip's distance from the target after solving (for a one-bone constraint, how far the target lies off the bone's line). |

```ogt
component IkTarget2D {
  doc "Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge, a tail's tip. A constraint is a chain of any length: one bone points at the target, two are solved exactly (bending the way the rig says), three or more by FABRIK from the animated pose, curling the way the rig says. Put it on its own entity, one per constraint; og.skeleton2d writes `reached` and `error`, and the drawn pose solves the same way."
  field skeleton Entity "The entity with the og.Skeleton2D."
  field constraint u32 "The constraint: an index into Rig2DInfo.constraints." {
    min 0
  }
  field target Vector2 "Where the chain's tip should be, in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent)." {
    unit m
  }
  field weight f32 "How much of the solve applies (0 is the animated pose, 1 fully solved), times the constraint's own weight (its `weight` line, or its clip's IK keys)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field tolerance f32 "The distance at which the target counts as reached." {
    default 0.01
    min 0
    unit m
  }
  field reached bool "Written: the tip is within `tolerance` of the target after solving." {
    readonly true
  }
  field error f32 "Written: the tip's distance from the target after solving (for a one-bone constraint, how far the target lies off the bone's line)." {
    unit m
    readonly true
  }
}
```

<sub>`api.ogt` line 8544</sub>

## Types

<a id="type-SkeletonAsset"></a>

### `SkeletonAsset` (handle)

A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle SkeletonAsset "A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices)."
```

<sub>`api.ogt` line 7783</sub>

<a id="type-ClipAsset"></a>

### `ClipAsset` (handle)

An animation clip, resampled at the cook's rate, with its markers.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle ClipAsset "An animation clip, resampled at the cook's rate, with its markers."
```

<sub>`api.ogt` line 7784</sub>

<a id="type-GraphAsset"></a>

### `GraphAsset` (handle)

A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle GraphAsset "A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles."
```

<sub>`api.ogt` line 7785</sub>

<a id="type-RootMotion"></a>

### `RootMotion` (enum)

What the root joint's motion does to the entity.

A string: one of the variants below. The zero value is `"off"`.

| Variant | Doc |
|---|---|
| `"off"` | The root animates in place; the entity does not move. |
| `"planar"` | The root's horizontal (xz) travel and yaw move og.Transform. |
| `"full"` | The root's whole translation and rotation move og.Transform. |

```ogt
enum RootMotion {
  doc "What the root joint's motion does to the entity."
  variant off "The root animates in place; the entity does not move."
  variant planar "The root's horizontal (xz) travel and yaw move og.Transform."
  variant full "The root's whole translation and rotation move og.Transform."
}
```

<sub>`api.ogt` line 7808</sub>

<a id="type-BlendshapeSource"></a>

### `BlendshapeSource` (enum)

Who decides blendshape weights.

A string: one of the variants below. The zero value is `"component"`.

| Variant | Doc |
|---|---|
| `"component"` | Game code writes w0...w7. |
| `"clips"` | The playing clips' weight channels write them. |

```ogt
enum BlendshapeSource {
  doc "Who decides blendshape weights."
  variant component "Game code writes w0...w7."
  variant clips "The playing clips' weight channels write them."
}
```

<sub>`api.ogt` line 7822</sub>

<a id="type-IkSolver"></a>

### `IkSolver` (enum)

How an IK chain is solved.

A string: one of the variants below. The zero value is `"twoBone"`.

| Variant | Doc |
|---|---|
| `"twoBone"` | Analytic two-bone (thigh, shin, foot), bending towards the pole. |
| `"fabrik"` | FABRIK over `bones` bones, iteratively. |

```ogt
enum IkSolver {
  doc "How an IK chain is solved."
  variant twoBone "Analytic two-bone (thigh, shin, foot), bending towards the pole."
  variant fabrik "FABRIK over `bones` bones, iteratively."
}
```

<sub>`api.ogt` line 7835</sub>

<a id="type-IkSpace"></a>

### `IkSpace` (enum)

What an IK chain's target and pole are relative to.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | Positions in the frame the skeleton entity's og.Transform is written in (its position, rotation and scale are undone): true world for a skeleton entity without a parent, its parent's frame for a child (a body under a character capsule takes targets relative to the capsule; transform world points by the parent's inverse yourself). |
| `"model"` | The skeleton's model space. |

```ogt
enum IkSpace {
  doc "What an IK chain's target and pole are relative to."
  variant world "Positions in the frame the skeleton entity's og.Transform is written in (its position, rotation and scale are undone): true world for a skeleton entity without a parent, its parent's frame for a child (a body under a character capsule takes targets relative to the capsule; transform world points by the parent's inverse yourself)."
  variant model "The skeleton's model space."
}
```

<sub>`api.ogt` line 7848</sub>

<a id="type-SpringSpace"></a>

### `SpringSpace` (enum)

Where a spring bone's tails move.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | The frame the skeleton entity's og.Transform is written in: the chain lags behind the entity's motion (walking, root motion, a jump). |
| `"model"` | The skeleton's model space: only the animation moves the chain (a teleport or a vehicle carrying the entity does not). |

```ogt
enum SpringSpace {
  doc "Where a spring bone's tails move."
  variant world "The frame the skeleton entity's og.Transform is written in: the chain lags behind the entity's motion (walking, root motion, a jump)."
  variant model "The skeleton's model space: only the animation moves the chain (a teleport or a vehicle carrying the entity does not)."
}
```

<sub>`api.ogt` line 8169</sub>

<a id="type-SpringColliderShape"></a>

### `SpringColliderShape` (enum)

The shape of a spring collider.

A string: one of the variants below. The zero value is `"sphere"`.

| Variant | Doc |
|---|---|
| `"sphere"` | A ball of `radius` about `offset`. |
| `"capsule"` | A capsule of `radius` about the segment from `offset` to `tail`. |

```ogt
enum SpringColliderShape {
  doc "The shape of a spring collider."
  variant sphere "A ball of `radius` about `offset`."
  variant capsule "A capsule of `radius` about the segment from `offset` to `tail`."
}
```

<sub>`api.ogt` line 8242</sub>

<a id="type-ClipInfo"></a>

### `ClipInfo` (struct)

One clip of a loaded animation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | Its handle, what AnimationPlayer.clip takes. |
| `name` | `string` | `""` | — | — | — | The glTF animation's name. |
| `duration` | `number` | `0` | — | `s` | — | Its length. |
| `markers` | `readonly string[]` | `[]` | — | — | — | Its markers' names, in time order. |

```ogt
struct ClipInfo {
  doc "One clip of a loaded animation."
  field clip ClipAsset "Its handle, what AnimationPlayer.clip takes."
  field name String "The glTF animation's name."
  field duration f32 "Its length." {
    unit s
  }
  field markers List.String "Its markers' names, in time order."
}
```

<sub>`api.ogt` line 8290</sub>

<a id="type-Animation"></a>

### `Animation` (struct)

What og.load.animation registered from a model's `#anim` blob.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The rig, what Skeleton.asset takes. |
| `clips` | `readonly ClipInfo[]` | `[]` | — | — | — | Its clips, in glTF order. |
| `nodes` | `readonly string[]` | `[]` | — | — | — | The rig's node names, in topological order. |
| `skins` | `readonly number[]` | `[]` | — | — | — | Each skin's joint count, in glTF order. |
| `morphTargets` | `number` | `0` | — | — | — | Morph targets across the model's meshes. |

```ogt
struct Animation {
  doc "What og.load.animation registered from a model's `#anim` blob."
  field skeleton SkeletonAsset "The rig, what Skeleton.asset takes."
  field clips List.ClipInfo "Its clips, in glTF order."
  field nodes List.String "The rig's node names, in topological order."
  field skins List.u32 "Each skin's joint count, in glTF order."
  field morphTargets u32 "Morph targets across the model's meshes."
}
```

<sub>`api.ogt` line 8300</sub>

<a id="type-GraphDesc"></a>

### `GraphDesc` (desc struct)

A blend-tree graph to register.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The graph's `.ogt` text (`type animgraph`, `version 1`; og-game §9 has every line). |
| `clips` | `readonly ClipInfo[]` | `[]` | — | — | — | The clips a `clip <motion> "<name>"` line may name, by ClipInfo.name: usually og.load.animation's `clips`. |

```ogt
struct GraphDesc {
  doc "A blend-tree graph to register."
  desc true
  field text String "The graph's `.ogt` text (`type animgraph`, `version 1`; og-game §9 has every line)."
  field clips List.ClipInfo "The clips a `clip <motion> \"<name>\"` line may name, by ClipInfo.name: usually og.load.animation's `clips`."
}
```

<sub>`api.ogt` line 8309</sub>

<a id="type-Rig"></a>

### `Rig` (handle)

A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow, the standard bones it names outright and, with `footHeight`, how high its ankles stand above the ground.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Rig "A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow, the standard bones it names outright and, with `footHeight`, how high its ankles stand above the ground."
```

<sub>`api.ogt` line 8323</sub>

<a id="type-HumanBone"></a>

### `HumanBone` (struct)

One standard humanoid bone of a skeleton.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bone` | `string` | `""` | — | — | — | The standard bone: hips, spine, chest, upperChest, neck, head, then leftShoulder, leftUpperArm, leftLowerArm, leftHand, the right arm, leftUpperLeg, leftLowerLeg, leftFoot, leftToes and the right leg, then each hand's fingers (VRM 1.0 names): leftThumbMetacarpal, leftThumbProximal, leftThumbDistal, leftIndexProximal, leftIndexIntermediate, leftIndexDistal, the same for Middle, Ring and Little, and the right hand's. |
| `node` | `string` | `""` | — | — | — | The skeleton node that is that bone. |

```ogt
struct HumanBone {
  doc "One standard humanoid bone of a skeleton."
  field bone String "The standard bone: hips, spine, chest, upperChest, neck, head, then leftShoulder, leftUpperArm, leftLowerArm, leftHand, the right arm, leftUpperLeg, leftLowerLeg, leftFoot, leftToes and the right leg, then each hand's fingers (VRM 1.0 names): leftThumbMetacarpal, leftThumbProximal, leftThumbDistal, leftIndexProximal, leftIndexIntermediate, leftIndexDistal, the same for Middle, Ring and Little, and the right hand's."
  field node String "The skeleton node that is that bone."
}
```

<sub>`api.ogt` line 8332</sub>

<a id="type-Humanoid"></a>

### `Humanoid` (struct)

Which node of a skeleton is each standard humanoid bone (what retargeting maps).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `string` | `""` | — | — | — | The naming scheme the bones were read in: mixamo, vrm, unity, rigify, ue or blender; custom when a rig named them, none when nothing matched. |
| `bones` | `readonly HumanBone[]` | `[]` | — | — | — | The bones found, in the standard order (hips first). |

```ogt
struct Humanoid {
  doc "Which node of a skeleton is each standard humanoid bone (what retargeting maps)."
  field scheme String "The naming scheme the bones were read in: mixamo, vrm, unity, rigify, ue or blender; custom when a rig named them, none when nothing matched."
  field bones List.HumanBone "The bones found, in the standard order (hips first)."
}
```

<sub>`api.ogt` line 8338</sub>

<a id="type-HumanoidDesc"></a>

### `HumanoidDesc` (desc struct)

A skeleton and, optionally, the rig that says how its bones are named.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The skeleton (og.load.animation's `skeleton`). |
| `rig` | `Rig` | `0` | — | — | — | A humanoid map from og.load.rig; 0 detects the bones from the node names (Mixamo, VRM, Unity, Rigify, Unreal or Blender names). |

```ogt
struct HumanoidDesc {
  doc "A skeleton and, optionally, the rig that says how its bones are named."
  desc true
  field skeleton SkeletonAsset "The skeleton (og.load.animation's `skeleton`)."
  field rig Rig "A humanoid map from og.load.rig; 0 detects the bones from the node names (Mixamo, VRM, Unity, Rigify, Unreal or Blender names)."
}
```

<sub>`api.ogt` line 8344</sub>

<a id="type-RetargetDesc"></a>

### `RetargetDesc` (desc struct)

A clip made for one skeleton, to play on another (other proportions, rest pose or bone names).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | The clip, made for `source`. |
| `source` | `SkeletonAsset` | `0` | — | — | — | The skeleton the clip was made for (og.load.animation's `skeleton`: a character's, or an animation library's). |
| `target` | `SkeletonAsset` | `0` | — | — | — | The skeleton that will play it. |
| `sourceRig` | `Rig` | `0` | — | — | — | How the source's bones are named (og.load.rig); 0 detects them. |
| `targetRig` | `Rig` | `0` | — | — | — | How the target's bones are named; 0 detects them. |
| `footIk` | `boolean` | `true` | — | — | — | Solve the legs so a planted foot stays planted on the target's ground (and the hips lower when a leg cannot reach). |

```ogt
struct RetargetDesc {
  doc "A clip made for one skeleton, to play on another (other proportions, rest pose or bone names)."
  desc true
  field clip ClipAsset "The clip, made for `source`."
  field source SkeletonAsset "The skeleton the clip was made for (og.load.animation's `skeleton`: a character's, or an animation library's)."
  field target SkeletonAsset "The skeleton that will play it."
  field sourceRig Rig "How the source's bones are named (og.load.rig); 0 detects them."
  field targetRig Rig "How the target's bones are named; 0 detects them."
  field footIk bool "Solve the legs so a planted foot stays planted on the target's ground (and the hips lower when a leg cannot reach)." {
    default true
  }
}
```

<sub>`api.ogt` line 8358</sub>

<a id="type-MirrorDesc"></a>

### `MirrorDesc` (desc struct)

A clip to play on the other side of the skeleton it was made for.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | The clip, made for `skeleton`. |
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The skeleton the clip was made for (og.load.animation's `skeleton`); its left and right nodes are paired by name, the side tokens every rig naming scheme writes swapped (LeftArm and RightArm, thigh.L and thigh.R, J_Bip_L_ and J_Bip_R_, calf_l and calf_r). |

```ogt
struct MirrorDesc {
  doc "A clip to play on the other side of the skeleton it was made for."
  desc true
  field clip ClipAsset "The clip, made for `skeleton`."
  field skeleton SkeletonAsset "The skeleton the clip was made for (og.load.animation's `skeleton`); its left and right nodes are paired by name, the side tokens every rig naming scheme writes swapped (LeftArm and RightArm, thigh.L and thigh.R, J_Bip_L_ and J_Bip_R_, calf_l and calf_r)."
}
```

<sub>`api.ogt` line 8378</sub>

<a id="type-FlipbookSheet"></a>

### `FlipbookSheet` (handle)

A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle FlipbookSheet "A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook."
```

<sub>`api.ogt` line 8392</sub>

<a id="type-FlipMode"></a>

### `FlipMode` (enum)

How a flipbook clip runs past its last frame.

A string: one of the variants below. The zero value is `"loop"`.

| Variant | Doc |
|---|---|
| `"loop"` | Back to the first frame. |
| `"pingPong"` | Back and forth, without repeating the end frames. |
| `"once"` | Hold the last frame and set Flipbook.finished. |

```ogt
enum FlipMode {
  doc "How a flipbook clip runs past its last frame."
  variant loop "Back to the first frame."
  variant pingPong "Back and forth, without repeating the end frames."
  variant once "Hold the last frame and set Flipbook.finished."
}
```

<sub>`api.ogt` line 8401</sub>

<a id="type-FlipbookClipInfo"></a>

### `FlipbookClipInfo` (struct)

One clip of a loaded flipbook sheet.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `number` | `0` | — | — | — | Its index, what Flipbook.clip takes. |
| `name` | `string` | `""` | — | — | — | Its name in the sheet. |
| `frames` | `number` | `0` | — | — | — | How many frames. |
| `duration` | `number` | `0` | — | `s` | — | Seconds of one cycle (a ping-pong cycle goes there and back). |
| `mode` | `FlipMode` | `"loop"` | — | — | — | How it runs past its end. |
| `events` | `readonly string[]` | `[]` | — | — | — | Its events' names, in frame order. |

```ogt
struct FlipbookClipInfo {
  doc "One clip of a loaded flipbook sheet."
  field clip u32 "Its index, what Flipbook.clip takes."
  field name String "Its name in the sheet."
  field frames u32 "How many frames."
  field duration f32 "Seconds of one cycle (a ping-pong cycle goes there and back)." {
    unit s
  }
  field mode FlipMode "How it runs past its end."
  field events List.String "Its events' names, in frame order."
}
```

<sub>`api.ogt` line 8447</sub>

<a id="type-FlipbookInfo"></a>

### `FlipbookInfo` (struct)

What og.load.flipbook registered.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sheet` | `FlipbookSheet` | `0` | — | — | — | What Flipbook.sheet takes. |
| `texture` | `Texture` | `0` | — | — | — | The sheet's texture (loaded with it; sampled nearest unless the sheet says `filter linear`); 0 for an `array true` sheet. |
| `array` | `TextureArray` | `0` | — | — | — | An `array true` sheet's texture array (its frames as layers, loaded with it); 0 otherwise. |
| `clips` | `readonly FlipbookClipInfo[]` | `[]` | — | — | — | Its clips, in sheet order. |
| `frames` | `number` | `0` | — | — | — | Frames in the sheet. |
| `frameWidth` | `number` | `0` | — | — | — | The first frame's width in pixels. |
| `frameHeight` | `number` | `0` | — | — | — | The first frame's height in pixels. |

```ogt
struct FlipbookInfo {
  doc "What og.load.flipbook registered."
  field sheet FlipbookSheet "What Flipbook.sheet takes."
  field texture Texture "The sheet's texture (loaded with it; sampled nearest unless the sheet says `filter linear`); 0 for an `array true` sheet."
  field array TextureArray "An `array true` sheet's texture array (its frames as layers, loaded with it); 0 otherwise."
  field clips List.FlipbookClipInfo "Its clips, in sheet order."
  field frames u32 "Frames in the sheet."
  field frameWidth u32 "The first frame's width in pixels."
  field frameHeight u32 "The first frame's height in pixels."
}
```

<sub>`api.ogt` line 8459</sub>

<a id="type-Rig2D"></a>

### `Rig2D` (handle)

A 2D rig (a cooked `.ogt` of `type rig2d`): bones, slots, region, mesh, bounding-box and child-armature attachments, IK constraints, child armatures and clips, from og.load.rig2d.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Rig2D "A 2D rig (a cooked `.ogt` of `type rig2d`): bones, slots, region, mesh, bounding-box and child-armature attachments, IK constraints, child armatures and clips, from og.load.rig2d."
```

<sub>`api.ogt` line 8494</sub>

<a id="type-Rig2DClipInfo"></a>

### `Rig2DClipInfo` (struct)

One clip of a loaded 2D rig.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `number` | `0` | — | — | — | Its index, what Skeleton2D.clip takes. |
| `name` | `string` | `""` | — | — | — | Its name in the rig. |
| `duration` | `number` | `0` | — | `s` | — | Its length. |
| `loop` | `boolean` | `false` | — | — | — | Whether the rig's author meant it to loop (its `loop` line): copy it into Skeleton2D.loop. |
| `events` | `readonly string[]` | `[]` | — | — | — | Its events' names, in time order. |

```ogt
struct Rig2DClipInfo {
  doc "One clip of a loaded 2D rig."
  field clip u32 "Its index, what Skeleton2D.clip takes."
  field name String "Its name in the rig."
  field duration f32 "Its length." {
    unit s
  }
  field loop bool "Whether the rig's author meant it to loop (its `loop` line): copy it into Skeleton2D.loop."
  field events List.String "Its events' names, in time order."
}
```

<sub>`api.ogt` line 8573</sub>

<a id="type-Rig2DInfo"></a>

### `Rig2DInfo` (struct)

What og.load.rig2d registered, or og.anim.rig2dInfo reads back from a handle.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rig` | `Rig2D` | `0` | — | — | — | What Skeleton2D.rig takes. |
| `texture` | `Texture` | `0` | — | — | — | The rig's texture (loaded with it). |
| `unit` | `number` | `0` | — | — | — | Pixels of the rig per world unit. |
| `bones` | `readonly string[]` | `[]` | — | — | — | The bones' names, parents first. |
| `slots` | `readonly string[]` | `[]` | — | — | — | The slots' names, in draw order. |
| `constraints` | `readonly string[]` | `[]` | — | — | — | The IK constraints' names: an index here is IkTarget2D.constraint. |
| `skins` | `readonly string[]` | `[]` | — | — | — | Its skins' names, `default` (the rig's own attachments) first: an index here is Skeleton2D.skin. |
| `clips` | `readonly Rig2DClipInfo[]` | `[]` | — | — | — | Its clips, in file order. |

```ogt
struct Rig2DInfo {
  doc "What og.load.rig2d registered, or og.anim.rig2dInfo reads back from a handle."
  field rig Rig2D "What Skeleton2D.rig takes."
  field texture Texture "The rig's texture (loaded with it)."
  field unit f32 "Pixels of the rig per world unit."
  field bones List.String "The bones' names, parents first."
  field slots List.String "The slots' names, in draw order."
  field constraints List.String "The IK constraints' names: an index here is IkTarget2D.constraint."
  field skins List.String "Its skins' names, `default` (the rig's own attachments) first: an index here is Skeleton2D.skin."
  field clips List.Rig2DClipInfo "Its clips, in file order."
}
```

<sub>`api.ogt` line 8584</sub>

<a id="type-Skeleton2DBox"></a>

### `Skeleton2DBox` (struct)

A bounding box a 2D skeleton's pose shows: a `box` attachment of its rig (or of a child armature in one of its slots) while its slot shows it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slot` | `string` | `""` | — | — | — | The slot showing it (`slot/child-slot` inside a child armature). |
| `name` | `string` | `""` | — | — | — | The box's name in the rig. |
| `points` | `readonly Vector2[]` | `[]` | — | — | — | Its polygon, moved by its bones (IK included), in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent). |

```ogt
struct Skeleton2DBox {
  doc "A bounding box a 2D skeleton's pose shows: a `box` attachment of its rig (or of a child armature in one of its slots) while its slot shows it."
  field slot String "The slot showing it (`slot/child-slot` inside a child armature)."
  field name String "The box's name in the rig."
  field points List.Vector2 "Its polygon, moved by its bones (IK included), in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent)."
}
```

<sub>`api.ogt` line 8603</sub>

## Column kinds

Each is a factory, `SkeletonAsset(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-SkeletonAsset"></a>`SkeletonAsset` | `SkeletonAsset` | `Uint32Array` | 2 | A SkeletonAsset handle field. |
| <a id="column-ClipAsset"></a>`ClipAsset` | `ClipAsset` | `Uint32Array` | 2 | A ClipAsset handle field. |
| <a id="column-GraphAsset"></a>`GraphAsset` | `GraphAsset` | `Uint32Array` | 2 | A GraphAsset handle field. |
| <a id="column-RootMotion"></a>`RootMotion` | `RootMotion` | `Uint32Array` | 1 | A RootMotion field: the variant's index. |
| <a id="column-BlendshapeSource"></a>`BlendshapeSource` | `BlendshapeSource` | `Uint32Array` | 1 | A BlendshapeSource field: the variant's index. |
| <a id="column-IkSolver"></a>`IkSolver` | `IkSolver` | `Uint32Array` | 1 | An IkSolver field: the variant's index. |
| <a id="column-IkSpace"></a>`IkSpace` | `IkSpace` | `Uint32Array` | 1 | An IkSpace field: the variant's index. |
| <a id="column-SpringSpace"></a>`SpringSpace` | `SpringSpace` | `Uint32Array` | 1 | A SpringSpace field: the variant's index. |
| <a id="column-SpringColliderShape"></a>`SpringColliderShape` | `SpringColliderShape` | `Uint32Array` | 1 | A SpringColliderShape field: the variant's index. |
| <a id="column-Rig"></a>`Rig` | `Rig` | `Uint32Array` | 2 | A Rig handle field (0 is none): a humanoid map og.load.rig answered. |
| <a id="column-FlipbookSheet"></a>`FlipbookSheet` | `FlipbookSheet` | `Uint32Array` | 2 | A FlipbookSheet handle field. |
| <a id="column-Rig2D"></a>`Rig2D` | `Rig2D` | `Uint32Array` | 2 | A Rig2D handle field. |

```ogt
column SkeletonAsset {
  doc "A SkeletonAsset handle field."
  array Uint32Array
  stride 2
  value SkeletonAsset
}
```

<sub>`api.ogt` line 7787</sub>

```ogt
column ClipAsset {
  doc "A ClipAsset handle field."
  array Uint32Array
  stride 2
  value ClipAsset
}
```

<sub>`api.ogt` line 7794</sub>

```ogt
column GraphAsset {
  doc "A GraphAsset handle field."
  array Uint32Array
  stride 2
  value GraphAsset
}
```

<sub>`api.ogt` line 7801</sub>

```ogt
column RootMotion {
  doc "A RootMotion field: the variant's index."
  array Uint32Array
  stride 1
  value RootMotion
}
```

<sub>`api.ogt` line 7815</sub>

```ogt
column BlendshapeSource {
  doc "A BlendshapeSource field: the variant's index."
  array Uint32Array
  stride 1
  value BlendshapeSource
}
```

<sub>`api.ogt` line 7828</sub>

```ogt
column IkSolver {
  doc "An IkSolver field: the variant's index."
  array Uint32Array
  stride 1
  value IkSolver
}
```

<sub>`api.ogt` line 7841</sub>

```ogt
column IkSpace {
  doc "An IkSpace field: the variant's index."
  array Uint32Array
  stride 1
  value IkSpace
}
```

<sub>`api.ogt` line 7854</sub>

```ogt
column SpringSpace {
  doc "A SpringSpace field: the variant's index."
  array Uint32Array
  stride 1
  value SpringSpace
}
```

<sub>`api.ogt` line 8175</sub>

```ogt
column SpringColliderShape {
  doc "A SpringColliderShape field: the variant's index."
  array Uint32Array
  stride 1
  value SpringColliderShape
}
```

<sub>`api.ogt` line 8248</sub>

```ogt
column Rig {
  doc "A Rig handle field (0 is none): a humanoid map og.load.rig answered."
  array Uint32Array
  stride 2
  value Rig
}
```

<sub>`api.ogt` line 8325</sub>

```ogt
column FlipbookSheet {
  doc "A FlipbookSheet handle field."
  array Uint32Array
  stride 2
  value FlipbookSheet
}
```

<sub>`api.ogt` line 8394</sub>

```ogt
column Rig2D {
  doc "A Rig2D handle field."
  array Uint32Array
  stride 2
  value Rig2D
}
```

<sub>`api.ogt` line 8496</sub>
