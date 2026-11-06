<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `anim` module

Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics.

Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none.

Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names; a rig with other names or proportions plays it retargeted (og.anim.retarget at run time, or a model's `animation { library }` import setting at cook time). Assets are not world state: a hot reload registers them again.

- **TypeScript**: functions are the namespace `og.anim`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module anim {
  doc "Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics."
  doc "Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none."
  doc "Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names; a rig with other names or proportions plays it retargeted (og.anim.retarget at run time, or a model's `animation { library }` import setting at cook time). Assets are not world state: a hot reload registers them again."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 11468</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`SkeletonAsset`](anim/type-SkeletonAsset.md) | handle |  | A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices). |
| [`ClipAsset`](anim/type-ClipAsset.md) | handle |  | An animation clip, resampled at the cook's rate, with its markers. |
| [`GraphAsset`](anim/type-GraphAsset.md) | handle |  | A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles. |
| [`SkeletonAsset()`](#column-SkeletonAsset) | column kind |  | A SkeletonAsset handle field. |
| [`ClipAsset()`](#column-ClipAsset) | column kind |  | A ClipAsset handle field. |
| [`GraphAsset()`](#column-GraphAsset) | column kind |  | A GraphAsset handle field. |
| [`RootMotion`](anim/type-RootMotion.md) | enum |  | What the root joint's motion does to the entity. |
| [`RootMotion()`](#column-RootMotion) | column kind |  | A RootMotion field: the variant's index. |
| [`BlendshapeSource`](anim/type-BlendshapeSource.md) | enum |  | Who decides blendshape weights. |
| [`BlendshapeSource()`](#column-BlendshapeSource) | column kind |  | A BlendshapeSource field: the variant's index. |
| [`IkSolver`](anim/type-IkSolver.md) | enum |  | How an IK chain is solved. |
| [`IkSolver()`](#column-IkSolver) | column kind |  | An IkSolver field: the variant's index. |
| [`IkSpace`](anim/type-IkSpace.md) | enum |  | What an IK chain's target and pole are relative to. |
| [`IkSpace()`](#column-IkSpace) | column kind |  | An IkSpace field: the variant's index. |
| [`og.Skeleton`](anim/record-Skeleton.md) | component |  | A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. |
| [`og.CrowdAnimation`](anim/record-CrowdAnimation.md) | component |  | Plays a clip on a model on the GPU, for crowds: put it on an entity with og.Model (and no og.Skeleton, which og.anim would pose on the CPU instead) and the model is drawn skinned by `clip` baked into… |
| [`og.AnimationPlayer`](anim/record-AnimationPlayer.md) | component |  | Plays one clip on a skeleton. |
| [`og.AnimationGraph`](anim/record-AnimationGraph.md) | component |  | Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param`… |
| [`og.Blendshapes`](anim/record-Blendshapes.md) | component |  | Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets, up to 64 targets (w0...w7, then `influences`). |
| [`og.IkChain`](anim/record-IkChain.md) | component |  | An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. |
| [`og.LookAt`](anim/record-LookAt.md) | component |  | An aim constraint on a skeleton, applied after its clips, graph and IK chains: `joint` turns so its `axis` points at `target` and its `upAxis` leans towards `up` (three.js Object3D.lookAt: +Z at the … |
| [`og.JointLimit`](anim/record-JointLimit.md) | component |  | How far a joint may turn from its rest rotation when og.anim's solvers (og.IkChain, og.LookAt, og.FootLock) turn it: a swing of `axis` within a cone and a twist about it within a range. |
| [`og.FootLock`](anim/record-FootLock.md) | component |  | Foot locking: while the playing clips plant `foot` (within 5 mm of its lowest in each clip, the notion retargeting uses, weighed across blends and cross-fades: planted when the planting clips hold at… |
| [`SpringSpace`](anim/type-SpringSpace.md) | enum |  | Where a spring bone's tails move. |
| [`SpringSpace()`](#column-SpringSpace) | column kind |  | A SpringSpace field: the variant's index. |
| [`og.SpringBone`](anim/record-SpringBone.md) | component |  | A spring (jiggle) bone chain, three-vrm's VRMSpringBone: `joint` and its `bones` - 1 parents swing after their parents' motion. |
| [`SpringColliderShape`](anim/type-SpringColliderShape.md) | enum |  | The shape of a spring collider. |
| [`SpringColliderShape()`](#column-SpringColliderShape) | column kind |  | A SpringColliderShape field: the variant's index. |
| [`og.SpringCollider`](anim/record-SpringCollider.md) | component |  | A collider spring bones are kept out of, three-vrm's VRMSpringBoneCollider: a sphere or a capsule riding on a joint of a skeleton (a head, a chest, an upper arm), so hair, ears and tails do not swing… |
| [`og.AnimationEvent`](anim/record-AnimationEvent.md) | event |  | A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). |
| [`ClipInfo`](anim/type-ClipInfo.md) | struct |  | One clip of a loaded animation. |
| [`Animation`](anim/type-Animation.md) | struct |  | What og.load.animation registered from a model's `#anim` blob. |
| [`GraphDesc`](anim/type-GraphDesc.md) | desc struct |  | A blend-tree graph to register. |
| [`og.anim.graph`](anim/fn-graph.md) | function | `(world: World, desc?: GraphDesc): Result<GraphAsset>` | Register a blend-tree graph for AnimationGraph.asset. |
| [`Rig`](anim/type-Rig.md) | handle |  | A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow, the standard bones it names outright and, with `footHeight`, how high its ankles sta… |
| [`Rig()`](#column-Rig) | column kind |  | A Rig handle field (0 is none): a humanoid map og.load.rig answered. |
| [`HumanBone`](anim/type-HumanBone.md) | struct |  | One standard humanoid bone of a skeleton. |
| [`Humanoid`](anim/type-Humanoid.md) | struct |  | Which node of a skeleton is each standard humanoid bone (what retargeting maps). |
| [`HumanoidDesc`](anim/type-HumanoidDesc.md) | desc struct |  | A skeleton and, optionally, the rig that says how its bones are named. |
| [`og.anim.humanoid`](anim/fn-humanoid.md) | function | `(world: World, desc?: HumanoidDesc): Result<Humanoid>` | The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it. |
| [`RetargetDesc`](anim/type-RetargetDesc.md) | desc struct |  | A clip made for one skeleton, to play on another (other proportions, rest pose or bone names). |
| [`og.anim.retarget`](anim/fn-retarget.md) | function | `(world: World, desc?: RetargetDesc): Result<ClipInfo>` | A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames… |
| [`MirrorDesc`](anim/type-MirrorDesc.md) | desc struct |  | A clip to play on the other side of the skeleton it was made for. |
| [`og.anim.mirror`](anim/fn-mirror.md) | function | `(world: World, desc?: MirrorDesc): Result<ClipInfo>` | A new clip that plays `clip` on the other side: every frame reflected in the skeleton's x = 0 plane, each node taking its counterpart's motion (a left swing becomes the right's), rotations moved in m… |
| [`FlipbookSheet`](anim/type-FlipbookSheet.md) | handle |  | A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook. |
| [`FlipbookSheet()`](#column-FlipbookSheet) | column kind |  | A FlipbookSheet handle field. |
| [`FlipMode`](anim/type-FlipMode.md) | enum |  | How a flipbook clip runs past its last frame. |
| [`og.Flipbook`](anim/record-Flipbook.md) | component |  | Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or, for an `array true` sheet, the frame's l… |
| [`og.FlipbookEvent`](anim/record-FlipbookEvent.md) | event |  | A flipbook entered a frame its clip names in an `event` line (`event 2 "step"`): footsteps, the frame a hit lands, a pickup's sparkle. |
| [`FlipbookClipInfo`](anim/type-FlipbookClipInfo.md) | struct |  | One clip of a loaded flipbook sheet. |
| [`FlipbookInfo`](anim/type-FlipbookInfo.md) | struct |  | What og.load.flipbook registered. |
| [`og.anim.clip`](anim/fn-clip.md) | function | `(info: FlipbookInfo, name: string): number` | The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, "walk_down")`. |
| [`og.anim.clip`](anim/fn-clip.md) | function | `(info: Animation, name: string): ClipAsset` | A loaded model's clip by name, for AnimationPlayer.clip: `og.anim.clip(og.load.animation(world, "models/slime.glb").value, "chase")`, the same lookup as a flipbook's (Animation.clips by ClipInfo.name… |
| [`og.anim.clip`](anim/fn-clip.md) | function | `(info: Rig2DInfo, name: string): number` | The index of a 2D rig's clip by name, for Skeleton2D.clip: `og.anim.clip(og.load.rig2d(world, "rigs/hero.ogt").value, "walk")`. |
| [`Rig2D`](anim/type-Rig2D.md) | handle |  | A 2D rig (a cooked `.ogt` of `type rig2d`): bones, slots, region, mesh, bounding-box and child-armature attachments, IK constraints, child armatures and clips, from og.load.rig2d. |
| [`Rig2D()`](#column-Rig2D) | column kind |  | A Rig2D handle field. |
| [`og.Skeleton2D`](anim/record-Skeleton2D.md) | component |  | Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its s… |
| [`og.Skeleton2DEvent`](anim/record-Skeleton2DEvent.md) | event |  | A 2D skeleton's clip, or the clip a child armature in one of its slots plays, passed one of its events (`event 0.2 "step"` in the rig) in the last tick: footsteps, the frame a swing lands. |
| [`og.IkTarget2D`](anim/record-IkTarget2D.md) | component |  | Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge, a tail's tip. |
| [`Rig2DClipInfo`](anim/type-Rig2DClipInfo.md) | struct |  | One clip of a loaded 2D rig. |
| [`Rig2DInfo`](anim/type-Rig2DInfo.md) | struct |  | What og.load.rig2d registered, or og.anim.rig2dInfo reads back from a handle. |
| [`og.anim.rig2dInfo`](anim/fn-rig2dInfo.md) | function | `(world: World, rig: Rig2D): Result<Rig2DInfo>` | What a loaded 2D rig holds: its bones, slots, IK constraints and clips, by the handle Skeleton2D.rig keeps (og.load.rig2d answers the same on load). |
| [`Skeleton2DBox`](anim/type-Skeleton2DBox.md) | struct |  | A bounding box a 2D skeleton's pose shows: a `box` attachment of its rig (or of a child armature in one of its slots) while its slot shows it. |
| [`og.anim.skeleton2dBoxes`](anim/fn-skeleton2dBoxes.md) | function | `(world: World, entity: Entity): Result<readonly Skeleton2DBox[]>` | The bounding boxes an og.Skeleton2D's pose showed when og.skeleton2d last ran (after the game's systems, so a system reads the previous tick's): hurt and hit boxes a clip switches on and off with its… |

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

<sub>`api.ogt` line 11477</sub>

```ogt
column ClipAsset {
  doc "A ClipAsset handle field."
  array Uint32Array
  stride 2
  value ClipAsset
}
```

<sub>`api.ogt` line 11484</sub>

```ogt
column GraphAsset {
  doc "A GraphAsset handle field."
  array Uint32Array
  stride 2
  value GraphAsset
}
```

<sub>`api.ogt` line 11491</sub>

```ogt
column RootMotion {
  doc "A RootMotion field: the variant's index."
  array Uint32Array
  stride 1
  value RootMotion
}
```

<sub>`api.ogt` line 11505</sub>

```ogt
column BlendshapeSource {
  doc "A BlendshapeSource field: the variant's index."
  array Uint32Array
  stride 1
  value BlendshapeSource
}
```

<sub>`api.ogt` line 11518</sub>

```ogt
column IkSolver {
  doc "An IkSolver field: the variant's index."
  array Uint32Array
  stride 1
  value IkSolver
}
```

<sub>`api.ogt` line 11531</sub>

```ogt
column IkSpace {
  doc "An IkSpace field: the variant's index."
  array Uint32Array
  stride 1
  value IkSpace
}
```

<sub>`api.ogt` line 11544</sub>

```ogt
column SpringSpace {
  doc "A SpringSpace field: the variant's index."
  array Uint32Array
  stride 1
  value SpringSpace
}
```

<sub>`api.ogt` line 11902</sub>

```ogt
column SpringColliderShape {
  doc "A SpringColliderShape field: the variant's index."
  array Uint32Array
  stride 1
  value SpringColliderShape
}
```

<sub>`api.ogt` line 11975</sub>

```ogt
column Rig {
  doc "A Rig handle field (0 is none): a humanoid map og.load.rig answered."
  array Uint32Array
  stride 2
  value Rig
}
```

<sub>`api.ogt` line 12053</sub>

```ogt
column FlipbookSheet {
  doc "A FlipbookSheet handle field."
  array Uint32Array
  stride 2
  value FlipbookSheet
}
```

<sub>`api.ogt` line 12122</sub>

```ogt
column Rig2D {
  doc "A Rig2D handle field."
  array Uint32Array
  stride 2
  value Rig2D
}
```

<sub>`api.ogt` line 12224</sub>
