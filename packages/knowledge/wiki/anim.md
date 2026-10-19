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

<sub>`api.ogt` line 4576</sub>

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
| [`og.AnimationEvent`](#record-AnimationEvent) | event | A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers. |
| [`ClipInfo`](#type-ClipInfo) | struct | One clip of a loaded animation. |
| [`Animation`](#type-Animation) | struct | What og.load.animation registered from a model's `#anim` blob. |
| [`GraphDesc`](#type-GraphDesc) | desc struct | A blend-tree graph to register. |
| [`og.anim.graph`](#fn-graph) | function | Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`. |
| [`Rig`](#type-Rig) | handle | A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow and the standard bones it names outright. |
| [`HumanBone`](#type-HumanBone) | struct | One standard humanoid bone of a skeleton. |
| [`Humanoid`](#type-Humanoid) | struct | Which node of a skeleton is each standard humanoid bone (what retargeting maps). |
| [`HumanoidDesc`](#type-HumanoidDesc) | desc struct | A skeleton and, optionally, the rig that says how its bones are named. |
| [`og.anim.humanoid`](#fn-humanoid) | function | The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it. |
| [`RetargetDesc`](#type-RetargetDesc) | desc struct | A clip made for one skeleton, to play on another (other proportions, rest pose or bone names). |
| [`og.anim.retarget`](#fn-retarget) | function | A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time. |
| [`FlipbookSheet`](#type-FlipbookSheet) | handle | A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook. |
| [`FlipbookSheet()`](#column-FlipbookSheet) | column kind | A FlipbookSheet handle field. |
| [`FlipMode`](#type-FlipMode) | enum | How a flipbook clip runs past its last frame. |
| [`og.Flipbook`](#record-Flipbook) | component | Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems. |
| [`og.FlipbookEvent`](#record-FlipbookEvent) | event | A flipbook entered a frame its clip names in an `event` line (`event 2 "step"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events. |
| [`FlipbookClipInfo`](#type-FlipbookClipInfo) | struct | One clip of a loaded flipbook sheet. |
| [`FlipbookInfo`](#type-FlipbookInfo) | struct | What og.load.flipbook registered. |
| [`og.anim.clip`](#fn-clip) | function | The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, "walk_down")`. -1 when the sheet has no such clip. |

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

<sub>`api.ogt` line 4886</sub>

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

<sub>`api.ogt` line 4914</sub>

<a id="fn-retarget"></a>

### `og.anim.retarget`

A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time.

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
  doc "A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time."
  param world World "The world whose animation owns the skeletons and the clip."
  param desc RetargetDesc "The clip, the two skeletons and their rigs."
  returns Result.ClipInfo "The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip, skeleton or rig, a skeleton without recognisable hips (name them in a `type rig` file), or a rig naming a missing node."
}
```

<sub>`api.ogt` line 4934</sub>

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

<sub>`api.ogt` line 5018</sub>

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

<sub>`api.ogt` line 4660</sub>

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
  field finished bool "Written: a non-looping clip reached its end." {
    readonly true
  }
}
```

<sub>`api.ogt` line 4675</sub>

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

<sub>`api.ogt` line 4709</sub>

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

<sub>`api.ogt` line 4757</sub>

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
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the tip should be. |
| `pole` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend. |
| `space` | `IkSpace` | `"world"` | — | — | — | What `target` and `pole` are relative to: world (the skeleton entity's og.Transform applies) or model. |
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
  field target Vector3 "Where the tip should be." {
    unit m
  }
  field pole Vector3 "A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend." {
    unit m
  }
  field space IkSpace "What `target` and `pole` are relative to: world (the skeleton entity's og.Transform applies) or model."
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

<sub>`api.ogt` line 4805</sub>

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

<sub>`api.ogt` line 4849</sub>

<a id="record-Flipbook"></a>

### `og.Flipbook` (component)

Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems.

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
  doc "Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems."
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

<sub>`api.ogt` line 4957</sub>

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

<sub>`api.ogt` line 4988</sub>

## Types

<a id="type-SkeletonAsset"></a>

### `SkeletonAsset` (handle)

A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle SkeletonAsset "A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices)."
```

<sub>`api.ogt` line 4582</sub>

<a id="type-ClipAsset"></a>

### `ClipAsset` (handle)

An animation clip, resampled at the cook's rate, with its markers.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle ClipAsset "An animation clip, resampled at the cook's rate, with its markers."
```

<sub>`api.ogt` line 4583</sub>

<a id="type-GraphAsset"></a>

### `GraphAsset` (handle)

A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle GraphAsset "A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles."
```

<sub>`api.ogt` line 4584</sub>

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

<sub>`api.ogt` line 4607</sub>

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

<sub>`api.ogt` line 4621</sub>

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

<sub>`api.ogt` line 4634</sub>

<a id="type-IkSpace"></a>

### `IkSpace` (enum)

What an IK chain's target and pole are relative to.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | World positions: the skeleton entity's og.Transform applies. |
| `"model"` | The skeleton's model space. |

```ogt
enum IkSpace {
  doc "What an IK chain's target and pole are relative to."
  variant world "World positions: the skeleton entity's og.Transform applies."
  variant model "The skeleton's model space."
}
```

<sub>`api.ogt` line 4647</sub>

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

<sub>`api.ogt` line 4860</sub>

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

<sub>`api.ogt` line 4870</sub>

<a id="type-GraphDesc"></a>

### `GraphDesc` (desc struct)

A blend-tree graph to register.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The graph's `.ogt` text (`type animgraph`, `version 1`; the grammar is og-anim's README). |
| `clips` | `readonly ClipInfo[]` | `[]` | — | — | — | The clips a `clip <motion> "<name>"` line may name, by ClipInfo.name: usually og.load.animation's `clips`. |

```ogt
struct GraphDesc {
  doc "A blend-tree graph to register."
  desc true
  field text String "The graph's `.ogt` text (`type animgraph`, `version 1`; the grammar is og-anim's README)."
  field clips List.ClipInfo "The clips a `clip <motion> \"<name>\"` line may name, by ClipInfo.name: usually og.load.animation's `clips`."
}
```

<sub>`api.ogt` line 4879</sub>

<a id="type-Rig"></a>

### `Rig` (handle)

A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow and the standard bones it names outright.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Rig "A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow and the standard bones it names outright."
```

<sub>`api.ogt` line 4893</sub>

<a id="type-HumanBone"></a>

### `HumanBone` (struct)

One standard humanoid bone of a skeleton.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bone` | `string` | `""` | — | — | — | The standard bone: hips, spine, chest, upperChest, neck, head, then leftShoulder, leftUpperArm, leftLowerArm, leftHand, the right arm, leftUpperLeg, leftLowerLeg, leftFoot, leftToes and the right leg. |
| `node` | `string` | `""` | — | — | — | The skeleton node that is that bone. |

```ogt
struct HumanBone {
  doc "One standard humanoid bone of a skeleton."
  field bone String "The standard bone: hips, spine, chest, upperChest, neck, head, then leftShoulder, leftUpperArm, leftLowerArm, leftHand, the right arm, leftUpperLeg, leftLowerLeg, leftFoot, leftToes and the right leg."
  field node String "The skeleton node that is that bone."
}
```

<sub>`api.ogt` line 4895</sub>

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

<sub>`api.ogt` line 4901</sub>

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

<sub>`api.ogt` line 4907</sub>

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

<sub>`api.ogt` line 4921</sub>

<a id="type-FlipbookSheet"></a>

### `FlipbookSheet` (handle)

A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle FlipbookSheet "A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook."
```

<sub>`api.ogt` line 4941</sub>

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

<sub>`api.ogt` line 4950</sub>

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

<sub>`api.ogt` line 4996</sub>

<a id="type-FlipbookInfo"></a>

### `FlipbookInfo` (struct)

What og.load.flipbook registered.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sheet` | `FlipbookSheet` | `0` | — | — | — | What Flipbook.sheet takes. |
| `texture` | `Texture` | `0` | — | — | — | The sheet's texture (loaded with it; sampled nearest unless the sheet says `filter linear`). |
| `clips` | `readonly FlipbookClipInfo[]` | `[]` | — | — | — | Its clips, in sheet order. |
| `frames` | `number` | `0` | — | — | — | Frames in the sheet. |
| `frameWidth` | `number` | `0` | — | — | — | The first frame's width in pixels. |
| `frameHeight` | `number` | `0` | — | — | — | The first frame's height in pixels. |

```ogt
struct FlipbookInfo {
  doc "What og.load.flipbook registered."
  field sheet FlipbookSheet "What Flipbook.sheet takes."
  field texture Texture "The sheet's texture (loaded with it; sampled nearest unless the sheet says `filter linear`)."
  field clips List.FlipbookClipInfo "Its clips, in sheet order."
  field frames u32 "Frames in the sheet."
  field frameWidth u32 "The first frame's width in pixels."
  field frameHeight u32 "The first frame's height in pixels."
}
```

<sub>`api.ogt` line 5008</sub>

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
| <a id="column-FlipbookSheet"></a>`FlipbookSheet` | `FlipbookSheet` | `Uint32Array` | 2 | A FlipbookSheet handle field. |

```ogt
column SkeletonAsset {
  doc "A SkeletonAsset handle field."
  array Uint32Array
  stride 2
  value SkeletonAsset
}
```

<sub>`api.ogt` line 4586</sub>

```ogt
column ClipAsset {
  doc "A ClipAsset handle field."
  array Uint32Array
  stride 2
  value ClipAsset
}
```

<sub>`api.ogt` line 4593</sub>

```ogt
column GraphAsset {
  doc "A GraphAsset handle field."
  array Uint32Array
  stride 2
  value GraphAsset
}
```

<sub>`api.ogt` line 4600</sub>

```ogt
column RootMotion {
  doc "A RootMotion field: the variant's index."
  array Uint32Array
  stride 1
  value RootMotion
}
```

<sub>`api.ogt` line 4614</sub>

```ogt
column BlendshapeSource {
  doc "A BlendshapeSource field: the variant's index."
  array Uint32Array
  stride 1
  value BlendshapeSource
}
```

<sub>`api.ogt` line 4627</sub>

```ogt
column IkSolver {
  doc "An IkSolver field: the variant's index."
  array Uint32Array
  stride 1
  value IkSolver
}
```

<sub>`api.ogt` line 4640</sub>

```ogt
column IkSpace {
  doc "An IkSpace field: the variant's index."
  array Uint32Array
  stride 1
  value IkSpace
}
```

<sub>`api.ogt` line 4653</sub>

```ogt
column FlipbookSheet {
  doc "A FlipbookSheet handle field."
  array Uint32Array
  stride 2
  value FlipbookSheet
}
```

<sub>`api.ogt` line 4943</sub>
