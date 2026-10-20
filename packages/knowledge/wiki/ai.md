<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ai` module

Enemy and NPC AI as data: behaviour trees and state machines in `.ogt` files of `type behaviour` (crates/og-ai/README.md has the grammar), perception (sight cones checked against physics occlusion, hearing of spatial sounds and game noises) and a blackboard per agent.

An entity with an og.AiAgent (and an og.Transform) thinks in the og.ai system before the game's systems: perception first, then its tree or machine, whose running action moves it through its og.NavAgent (og.nav plans and walks after the game's systems) or runs one of the game's commands (GameDesc.commands, with the agent's entity handle as its first argument) before the next tick.

Conditions read blackboard keys (declared, set by actions or og.ai.set, or built in: self, position, sees, heard, target, targetPosition, targetDistance, lastSeen, sinceSeen, heardAt, sinceHeard, elapsed), the agent's component fields (`Health.hp`) and resource fields (`Game.alarm`). Deterministic; memories and blackboards are in snapshots. `dump_ai:<entity>` writes an agent's state, running node and blackboard.

- **TypeScript**: functions are the namespace `og.ai`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-ai`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module ai {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 6989</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Behaviour`](#type-Behaviour) | handle | A loaded behaviour tree or state machine. |
| [`Behaviour()`](#column-Behaviour) | column kind | A Behaviour handle field. |
| [`AI_RUNNING`](#const-AI_RUNNING) | constant | AiAgent.status: the tree or state is still running. |
| [`AI_SUCCESS`](#const-AI_SUCCESS) | constant | AiAgent.status: the tree succeeded on the last think. |
| [`AI_FAILURE`](#const-AI_FAILURE) | constant | AiAgent.status: the tree failed on the last think. |
| [`og.AiAgent`](#record-AiAgent) | component | An entity that thinks with a behaviour. Give it an og.NavAgent to move and an og.Perception to see and hear. |
| [`og.Perception`](#record-Perception) | component | What an agent sees and hears. It sees the nearest og.Perceivable of another team within `sight`, inside the `fov` cone around `forward` (or the way it last moved), unless a collider is in the way; it hears a spatial sound that starts (og.SoundStarted) or a game noise (og.ai.noise) within `hearing` times its loudness. |
| [`og.Perceivable`](#record-Perceivable) | component | Something agents of other teams can see. |
| [`og.AiChanged`](#record-AiChanged) | event | An agent's state or running node changed (systems read it the tick after). |
| [`BehaviourInfo`](#type-BehaviourInfo) | struct | A loaded behaviour. |
| [`NoiseDesc`](#type-NoiseDesc) | desc struct | A noise agents may hear next tick. |
| [`AiDescription`](#type-AiDescription) | struct | What an agent is doing. |
| [`og.ai.load`](#fn-load) | function | A cooked behaviour (an `.ogt` of `type behaviour`, validated by og cook). |
| [`og.ai.parse`](#fn-parse) | function | A behaviour from text, now (tests and generated behaviours; assets go through og cook). |
| [`og.ai.get`](#fn-get) | function | A blackboard value of an agent as of its last think. |
| [`og.ai.set`](#fn-set) | function | Set a blackboard value of an agent: a number, a boolean, a point `{x, y, z}` or an entity handle `{entity}`. |
| [`og.ai.noise`](#fn-noise) | function | Make a noise agents may hear next tick (a gunshot, a door, a thrown stone). |
| [`og.ai.describe`](#fn-describe) | function | What an agent is doing: its state, the node it runs and the path to it, its status, its blackboard and its target. |

## Functions

<a id="fn-load"></a>

### `og.ai.load`

A cooked behaviour (an `.ogt` of `type behaviour`, validated by og cook).

```ts
og.ai.load(world: World, path: string): Result<BehaviourInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The behaviour asset, such as `ai/slime.ogt`. |

**Returns** `Result<BehaviourInfo>`: Refused for a path that is not a cooked behaviour.

Binding symbol `og_ai_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked behaviour (an `.ogt` of `type behaviour`, validated by og cook)."
  param world World "The world."
  param path String "The behaviour asset, such as `ai/slime.ogt`."
  returns Result.BehaviourInfo "Refused for a path that is not a cooked behaviour."
}
```

<sub>`api.ogt` line 7160</sub>

<a id="fn-parse"></a>

### `og.ai.parse`

A behaviour from text, now (tests and generated behaviours; assets go through og cook).

```ts
og.ai.parse(world: World, text: string): Result<BehaviourInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `text` | `string` | no | The behaviour's `.ogt` text. |

**Returns** `Result<BehaviourInfo>`: Refused with the first error's line and the fix.

Binding symbol `og_ai_parse` · publish builds: kept

```ogt
fn parse {
  doc "A behaviour from text, now (tests and generated behaviours; assets go through og cook)."
  param world World "The world."
  param text String "The behaviour's `.ogt` text."
  returns Result.BehaviourInfo "Refused with the first error's line and the fix."
}
```

<sub>`api.ogt` line 7167</sub>

<a id="fn-get"></a>

### `og.ai.get`

A blackboard value of an agent as of its last think.

```ts
og.ai.get(world: World, entity: Entity, key: string): unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The agent. |
| `key` | `string` | no | The key. |

**Returns** `unknown`: A number, a boolean, a point `{x, y, z}`, an entity handle, or null for an unknown key or an agent that has not thought.

Binding symbol `og_ai_get` · publish builds: kept

```ogt
fn get {
  doc "A blackboard value of an agent as of its last think."
  param world World "The world."
  param entity Entity "The agent."
  param key String "The key."
  returns Json "A number, a boolean, a point `{x, y, z}`, an entity handle, or null for an unknown key or an agent that has not thought."
}
```

<sub>`api.ogt` line 7174</sub>

<a id="fn-set"></a>

### `og.ai.set`

Set a blackboard value of an agent: a number, a boolean, a point `{x, y, z}` or an entity handle `{entity}`.

```ts
og.ai.set(world: World, entity: Entity, key: string, value: unknown): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The agent. |
| `key` | `string` | no | The key; built-in keys are refused. |
| `value` | `unknown` | no | The value. |

**Returns** `Result<void>`: Refused for an agent that has not thought yet, a built-in key or another kind of value.

Binding symbol `og_ai_set` · publish builds: kept

```ogt
fn set {
  doc "Set a blackboard value of an agent: a number, a boolean, a point `{x, y, z}` or an entity handle `{entity}`."
  param world World "The world."
  param entity Entity "The agent."
  param key String "The key; built-in keys are refused."
  param value Json "The value."
  returns Result.Void "Refused for an agent that has not thought yet, a built-in key or another kind of value."
}
```

<sub>`api.ogt` line 7182</sub>

<a id="fn-noise"></a>

### `og.ai.noise`

Make a noise agents may hear next tick (a gunshot, a door, a thrown stone).

```ts
og.ai.noise(world: World, desc?: NoiseDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NoiseDesc` | yes | Where and how loud. |

**Returns** `Result<void>`: Refused for a world without og.ai.

Binding symbol `og_ai_noise` · publish builds: kept

```ogt
fn noise {
  doc "Make a noise agents may hear next tick (a gunshot, a door, a thrown stone)."
  param world World "The world."
  param desc NoiseDesc "Where and how loud."
  returns Result.Void "Refused for a world without og.ai."
}
```

<sub>`api.ogt` line 7191</sub>

<a id="fn-describe"></a>

### `og.ai.describe`

What an agent is doing: its state, the node it runs and the path to it, its status, its blackboard and its target.

```ts
og.ai.describe(world: World, entity: Entity): AiDescription | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The agent. |

**Returns** `AiDescription | null`: Null for an entity that has not thought.

Binding symbol `og_ai_describe` · publish builds: kept

```ogt
fn describe {
  doc "What an agent is doing: its state, the node it runs and the path to it, its status, its blackboard and its target."
  param world World "The world."
  param entity Entity "The agent."
  returns Option.AiDescription "Null for an entity that has not thought."
}
```

<sub>`api.ogt` line 7198</sub>

## Components, resources and events

<a id="record-AiAgent"></a>

### `og.AiAgent` (component)

An entity that thinks with a behaviour. Give it an og.NavAgent to move and an og.Perception to see and hear.

Token `og.AiAgent`, registered under the schema name `og.AiAgent`; its value type is `AiAgent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `behaviour` | `Behaviour` | `0` | — | — | — | The behaviour it runs (og.ai.load); 0 runs nothing. |
| `active` | `boolean` | `true` | — | — | — | Thinks while true. |
| `thinkEvery` | `number` | `1` | 1 … 600 | `ticks` | — | Thinks every this many ticks (1: every tick); agents are staggered by their order. |
| `status` | `number` | `0` | — | — | read only | AI_RUNNING, AI_SUCCESS or AI_FAILURE after the last think (0 before the first; written by og.ai). |
| `state` | `number` | `0` | — | — | read only | The state machine's state, an index into BehaviourInfo.states (written by og.ai). |
| `node` | `number` | `0` | — | — | read only | The node it ran last, an index into BehaviourInfo.nodes (written by og.ai). |
| `changes` | `number` | `0` | — | — | read only | Times the state or the running node changed (written by og.ai). |

```ogt
component AiAgent {
  doc "An entity that thinks with a behaviour. Give it an og.NavAgent to move and an og.Perception to see and hear."
  field behaviour Behaviour "The behaviour it runs (og.ai.load); 0 runs nothing."
  field active bool "Thinks while true." {
    default true
  }
  field thinkEvery u32 "Thinks every this many ticks (1: every tick); agents are staggered by their order." {
    default 1
    min 1
    max 600
    unit ticks
  }
  field status u32 "AI_RUNNING, AI_SUCCESS or AI_FAILURE after the last think (0 before the first; written by og.ai)." {
    readonly true
  }
  field state u32 "The state machine's state, an index into BehaviourInfo.states (written by og.ai)." {
    readonly true
  }
  field node u32 "The node it ran last, an index into BehaviourInfo.nodes (written by og.ai)." {
    readonly true
  }
  field changes u32 "Times the state or the running node changed (written by og.ai)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 7008</sub>

<a id="record-Perception"></a>

### `og.Perception` (component)

What an agent sees and hears. It sees the nearest og.Perceivable of another team within `sight`, inside the `fov` cone around `forward` (or the way it last moved), unless a collider is in the way; it hears a spatial sound that starts (og.SoundStarted) or a game noise (og.ai.noise) within `hearing` times its loudness.

Token `og.Perception`, registered under the schema name `og.Perception`; its value type is `Perception`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sight` | `number` | `8` | 0 … 500 | `m` | tunable | How far it sees. |
| `fov` | `number` | `120` | 0 … 360 | `deg` | tunable | The width of its view cone (360: all round). |
| `eyeHeight` | `number` | `0` | -10 … 10 | `m` | — | Its eyes are this far above its position (+y). |
| `occlude` | `boolean` | `true` | — | — | — | Colliders hide what is behind them. |
| `occluders` | `number` | `0` | — | — | — | Collision layers that block sight (0: every layer). |
| `hearing` | `number` | `6` | 0 … 500 | `m` | tunable | How far it hears a sound of loudness 1 (a source's volume). |
| `memory` | `number` | `3` | 0 … 600 | `s` | tunable | How long a lost target stays its target. |
| `team` | `number` | `0` | — | — | — | Its team: it perceives og.Perceivable entities of other teams. |
| `forward` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where it looks; zero: the way it last moved (before it moves, the way its Transform faces: +Z in 3D, +X in 2D). |
| `target` | `Entity` | `0` | — | — | read only | What it sees or remembers seeing (written by og.ai). |
| `sees` | `boolean` | `false` | — | — | read only | It sees the target now (written by og.ai). |
| `lastSeen` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only | Where it last saw the target (written by og.ai). |
| `sinceSeen` | `number` | `1000000` | — | `s` | read only | Seconds since it last saw a target (written by og.ai; 1e6 before the first time). |
| `heard` | `boolean` | `false` | — | — | read only | It heard something this tick (written by og.ai). |
| `heardAt` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only | Where it last heard something (written by og.ai). |
| `sinceHeard` | `number` | `1000000` | — | `s` | read only | Seconds since it last heard something (written by og.ai; 1e6 before the first time). |

```ogt
component Perception {
  doc "What an agent sees and hears. It sees the nearest og.Perceivable of another team within `sight`, inside the `fov` cone around `forward` (or the way it last moved), unless a collider is in the way; it hears a spatial sound that starts (og.SoundStarted) or a game noise (og.ai.noise) within `hearing` times its loudness."
  field sight f32 "How far it sees." {
    default 8
    min 0
    max 500
    unit m
    tunable true
  }
  field fov f32 "The width of its view cone (360: all round)." {
    default 120
    min 0
    max 360
    unit deg
    tunable true
  }
  field eyeHeight f32 "Its eyes are this far above its position (+y)." {
    min -10
    max 10
    unit m
  }
  field occlude bool "Colliders hide what is behind them." {
    default true
  }
  field occluders u32 "Collision layers that block sight (0: every layer)."
  field hearing f32 "How far it hears a sound of loudness 1 (a source's volume)." {
    default 6
    min 0
    max 500
    unit m
    tunable true
  }
  field memory f32 "How long a lost target stays its target." {
    default 3
    min 0
    max 600
    unit s
    tunable true
  }
  field team u32 "Its team: it perceives og.Perceivable entities of other teams."
  field forward Vector3 "Where it looks; zero: the way it last moved (before it moves, the way its Transform faces: +Z in 3D, +X in 2D)."
  field target Entity "What it sees or remembers seeing (written by og.ai)." {
    readonly true
  }
  field sees bool "It sees the target now (written by og.ai)." {
    readonly true
  }
  field lastSeen Vector3 "Where it last saw the target (written by og.ai)." {
    unit m
    readonly true
  }
  field sinceSeen f32 "Seconds since it last saw a target (written by og.ai; 1e6 before the first time)." {
    default 1000000
    unit s
    readonly true
  }
  field heard bool "It heard something this tick (written by og.ai)." {
    readonly true
  }
  field heardAt Vector3 "Where it last heard something (written by og.ai)." {
    unit m
    readonly true
  }
  field sinceHeard f32 "Seconds since it last heard something (written by og.ai; 1e6 before the first time)." {
    default 1000000
    unit s
    readonly true
  }
}
```

<sub>`api.ogt` line 7034</sub>

<a id="record-Perceivable"></a>

### `og.Perceivable` (component)

Something agents of other teams can see.

Token `og.Perceivable`, registered under the schema name `og.Perceivable`; its value type is `Perceivable`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `team` | `number` | `1` | — | — | — | Its team (agents of the same team ignore it). |
| `eyeHeight` | `number` | `0` | -10 … 10 | `m` | — | Agents look at a point this far above its position (+y). |

```ogt
component Perceivable {
  doc "Something agents of other teams can see."
  field team u32 "Its team (agents of the same team ignore it)." {
    default 1
  }
  field eyeHeight f32 "Agents look at a point this far above its position (+y)." {
    min -10
    max 10
    unit m
  }
}
```

<sub>`api.ogt` line 7104</sub>

<a id="record-AiChanged"></a>

### `og.AiChanged` (event)

An agent's state or running node changed (systems read it the tick after).

Token `og.AiChanged`, registered under the schema name `og.AiChanged`; its value type is `AiChanged`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The agent. |
| `state` | `number` | `0` | — | — | — | Its state now. |
| `node` | `number` | `0` | — | — | — | The node it runs now. |
| `status` | `number` | `0` | — | — | — | AI_RUNNING, AI_SUCCESS or AI_FAILURE. |

```ogt
event AiChanged {
  doc "An agent's state or running node changed (systems read it the tick after)."
  field entity Entity "The agent."
  field state u32 "Its state now."
  field node u32 "The node it runs now."
  field status u32 "AI_RUNNING, AI_SUCCESS or AI_FAILURE."
}
```

<sub>`api.ogt` line 7116</sub>

## Types

<a id="type-Behaviour"></a>

### `Behaviour` (handle)

A loaded behaviour tree or state machine.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Behaviour "A loaded behaviour tree or state machine."
```

<sub>`api.ogt` line 6995</sub>

<a id="type-BehaviourInfo"></a>

### `BehaviourInfo` (struct)

A loaded behaviour.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `behaviour` | `Behaviour` | `0` | — | — | — | What AiAgent.behaviour takes. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `string` | `""` | — | — | — | tree or machine. |
| `states` | `readonly string[]` | `[]` | — | — | — | A machine's states, by index (AiAgent.state). |
| `nodes` | `readonly string[]` | `[]` | — | — | — | Its nodes' names, by index (AiAgent.node). |
| `keys` | `readonly string[]` | `[]` | — | — | — | Its declared blackboard keys. |

```ogt
struct BehaviourInfo {
  doc "A loaded behaviour."
  field behaviour Behaviour "What AiAgent.behaviour takes."
  field name String "Its name."
  field kind String "tree or machine."
  field states List.String "A machine's states, by index (AiAgent.state)."
  field nodes List.String "Its nodes' names, by index (AiAgent.node)."
  field keys List.String "Its declared blackboard keys."
}
```

<sub>`api.ogt` line 7124</sub>

<a id="type-NoiseDesc"></a>

### `NoiseDesc` (desc struct)

A noise agents may hear next tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `loudness` | `number` | `1` | 0 … 100 | — | — | How loud: heard within Perception.hearing times this. |

```ogt
struct NoiseDesc {
  doc "A noise agents may hear next tick."
  desc true
  field position Vector3 "Where it is." {
    unit m
  }
  field loudness f32 "How loud: heard within Perception.hearing times this." {
    default 1
    min 0
    max 100
  }
}
```

<sub>`api.ogt` line 7134</sub>

<a id="type-AiDescription"></a>

### `AiDescription` (struct)

What an agent is doing.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `behaviour` | `string` | `""` | — | — | — | Its behaviour's name. |
| `kind` | `string` | `""` | — | — | — | tree or machine. |
| `state` | `string` | `""` | — | — | — | A machine's state (empty for a tree). |
| `node` | `string` | `""` | — | — | — | The node it ran last, such as `chase`. |
| `path` | `string` | `""` | — | — | — | The path to it from the root, such as `selector > sequence chase > chase`. |
| `status` | `string` | `""` | — | — | — | running, success or failure. |
| `blackboard` | `unknown` | `null` | — | — | — | Its blackboard, keys to numbers, booleans, points and entity handles. |
| `sees` | `boolean` | `false` | — | — | — | It sees its target. |
| `target` | `Entity` | `0` | — | — | — | Its target (0: none). |

```ogt
struct AiDescription {
  doc "What an agent is doing."
  field behaviour String "Its behaviour's name."
  field kind String "tree or machine."
  field state String "A machine's state (empty for a tree)."
  field node String "The node it ran last, such as `chase`."
  field path String "The path to it from the root, such as `selector > sequence chase > chase`."
  field status String "running, success or failure."
  field blackboard Json "Its blackboard, keys to numbers, booleans, points and entity handles."
  field sees bool "It sees its target."
  field target Entity "Its target (0: none)."
}
```

<sub>`api.ogt` line 7147</sub>

## Column kinds

Each is a factory, `Behaviour(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Behaviour"></a>`Behaviour` | `Behaviour` | `Uint32Array` | 2 | A Behaviour handle field. |

```ogt
column Behaviour {
  doc "A Behaviour handle field."
  array Uint32Array
  stride 2
  value Behaviour
}
```

<sub>`api.ogt` line 6997</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-AI_RUNNING"></a>`AI_RUNNING` | `number` | `1` | AiAgent.status: the tree or state is still running. |
| <a id="const-AI_SUCCESS"></a>`AI_SUCCESS` | `number` | `2` | AiAgent.status: the tree succeeded on the last think. |
| <a id="const-AI_FAILURE"></a>`AI_FAILURE` | `number` | `3` | AiAgent.status: the tree failed on the last think. |

```ogt
const AI_RUNNING u32 1 "AiAgent.status: the tree or state is still running."
```

<sub>`api.ogt` line 7004</sub>

```ogt
const AI_SUCCESS u32 2 "AiAgent.status: the tree succeeded on the last think."
```

<sub>`api.ogt` line 7005</sub>

```ogt
const AI_FAILURE u32 3 "AiAgent.status: the tree failed on the last think."
```

<sub>`api.ogt` line 7006</sub>
