<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `anim` module

Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics.

Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none.

Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names. Assets are not world state: a hot reload registers them again.

- **TypeScript**: functions are the namespace `og.anim`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module anim {
  doc "Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics."
  doc "Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none."
  doc "Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names. Assets are not world state: a hot reload registers them again."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 2858</sub>

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

<sub>`api.ogt` line 3167</sub>

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

<sub>`api.ogt` line 2941</sub>

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

<sub>`api.ogt` line 2956</sub>

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

<sub>`api.ogt` line 2990</sub>

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

<sub>`api.ogt` line 3038</sub>

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

<sub>`api.ogt` line 3086</sub>

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

<sub>`api.ogt` line 3130</sub>

## Types

<a id="type-SkeletonAsset"></a>

### `SkeletonAsset` (handle)

A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle SkeletonAsset "A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices)."
```

<sub>`api.ogt` line 2863</sub>

<a id="type-ClipAsset"></a>

### `ClipAsset` (handle)

An animation clip, resampled at the cook's rate, with its markers.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle ClipAsset "An animation clip, resampled at the cook's rate, with its markers."
```

<sub>`api.ogt` line 2864</sub>

<a id="type-GraphAsset"></a>

### `GraphAsset` (handle)

A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle GraphAsset "A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles."
```

<sub>`api.ogt` line 2865</sub>

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

<sub>`api.ogt` line 2888</sub>

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

<sub>`api.ogt` line 2902</sub>

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

<sub>`api.ogt` line 2915</sub>

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

<sub>`api.ogt` line 2928</sub>

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

<sub>`api.ogt` line 3141</sub>

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

<sub>`api.ogt` line 3151</sub>

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

<sub>`api.ogt` line 3160</sub>

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

```ogt
column SkeletonAsset {
  doc "A SkeletonAsset handle field."
  array Uint32Array
  stride 2
  value SkeletonAsset
}
```

<sub>`api.ogt` line 2867</sub>

```ogt
column ClipAsset {
  doc "A ClipAsset handle field."
  array Uint32Array
  stride 2
  value ClipAsset
}
```

<sub>`api.ogt` line 2874</sub>

```ogt
column GraphAsset {
  doc "A GraphAsset handle field."
  array Uint32Array
  stride 2
  value GraphAsset
}
```

<sub>`api.ogt` line 2881</sub>

```ogt
column RootMotion {
  doc "A RootMotion field: the variant's index."
  array Uint32Array
  stride 1
  value RootMotion
}
```

<sub>`api.ogt` line 2895</sub>

```ogt
column BlendshapeSource {
  doc "A BlendshapeSource field: the variant's index."
  array Uint32Array
  stride 1
  value BlendshapeSource
}
```

<sub>`api.ogt` line 2908</sub>

```ogt
column IkSolver {
  doc "An IkSolver field: the variant's index."
  array Uint32Array
  stride 1
  value IkSolver
}
```

<sub>`api.ogt` line 2921</sub>

```ogt
column IkSpace {
  doc "An IkSpace field: the variant's index."
  array Uint32Array
  stride 1
  value IkSpace
}
```

<sub>`api.ogt` line 2934</sub>
