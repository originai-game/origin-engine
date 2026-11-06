<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ai` module

Enemy and NPC AI as data: behaviour trees and state machines in `.ogt` files of `type behaviour`, perception (sight cones checked against physics occlusion, hearing of spatial sounds and game noises) and a blackboard per agent.

An entity with an og.AiAgent (and an og.Transform) thinks in the og.ai system before the game's systems: perception first, then its tree or machine, whose running action moves it through its og.NavAgent (og.nav plans and walks after the game's systems) or runs one of the game's commands (GameDesc.commands, with the agent's entity handle as its first argument) before the next tick.

Conditions read blackboard keys (declared, set by actions or og.ai.set, or built in: self, position, sees, heard, target, targetPosition, targetDistance, lastSeen, sinceSeen, heardAt, sinceHeard, elapsed), the agent's component fields (`Health.hp`) and resource fields (`Game.alarm`), and compare them with a literal or with another key or field read each time (`check Foe.hp below Tuning.fleeHp`), so thresholds live in a tuning resource. og.ai.restart starts a behaviour afresh. Deterministic; memories and blackboards are in snapshots. `dump_ai:<entity>` writes an agent's state, running node and blackboard.

- **TypeScript**: functions are the namespace `og.ai`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module ai {
  doc "Enemy and NPC AI as data: behaviour trees and state machines in `.ogt` files of `type behaviour`, perception (sight cones checked against physics occlusion, hearing of spatial sounds and game noises) and a blackboard per agent."
  doc "An entity with an og.AiAgent (and an og.Transform) thinks in the og.ai system before the game's systems: perception first, then its tree or machine, whose running action moves it through its og.NavAgent (og.nav plans and walks after the game's systems) or runs one of the game's commands (GameDesc.commands, with the agent's entity handle as its first argument) before the next tick."
  doc "Conditions read blackboard keys (declared, set by actions or og.ai.set, or built in: self, position, sees, heard, target, targetPosition, targetDistance, lastSeen, sinceSeen, heardAt, sinceHeard, elapsed), the agent's component fields (`Health.hp`) and resource fields (`Game.alarm`), and compare them with a literal or with another key or field read each time (`check Foe.hp below Tuning.fleeHp`), so thresholds live in a tuning resource. og.ai.restart starts a behaviour afresh. Deterministic; memories and blackboards are in snapshots. `dump_ai:<entity>` writes an agent's state, running node and blackboard."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 14132</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Behaviour`](ai/type-Behaviour.md) | handle |  | A loaded behaviour tree or state machine. |
| [`Behaviour()`](#column-Behaviour) | column kind |  | A Behaviour handle field. |
| [`AI_RUNNING`](#const-AI_RUNNING) | constant |  | AiAgent.status: the tree or state is still running. |
| [`AI_SUCCESS`](#const-AI_SUCCESS) | constant |  | AiAgent.status: the tree succeeded on the last think. |
| [`AI_FAILURE`](#const-AI_FAILURE) | constant |  | AiAgent.status: the tree failed on the last think. |
| [`AI_INTERRUPTED`](#const-AI_INTERRUPTED) | constant |  | AiActionStopped.result: the action stopped before the game finished it (another branch won, the state changed, the agent restarted). |
| [`og.AiAgent`](ai/record-AiAgent.md) | component |  | An entity that thinks with a behaviour. |
| [`og.Perception`](ai/record-Perception.md) | component |  | What an agent sees and hears. |
| [`og.Perceivable`](ai/record-Perceivable.md) | component |  | Something agents of other teams can see. |
| [`og.AiChanged`](ai/record-AiChanged.md) | event |  | An agent's state or running node changed (systems read it the tick after). |
| [`og.AiActionStarted`](ai/record-AiActionStarted.md) | event |  | A game action (`action <name> args` in a behaviour) started: play it (a reload, a door, a line of dialogue) and call og.ai.finish when it is over; until then the node runs. |
| [`og.AiActionStopped`](ai/record-AiActionStopped.md) | event |  | A game action stopped: the game finished it (og.ai.finish), or the behaviour moved on first (stop playing it). |
| [`BehaviourInfo`](ai/type-BehaviourInfo.md) | struct |  | A loaded behaviour. |
| [`NoiseDesc`](ai/type-NoiseDesc.md) | desc struct |  | A noise agents may hear next tick. |
| [`AiDescription`](ai/type-AiDescription.md) | struct |  | What an agent is doing. |
| [`og.ai.load`](ai/fn-load.md) | function | `(world: World, path: string): Result<BehaviourInfo>` | A cooked behaviour (an `.ogt` of `type behaviour`, validated by og cook); its `use` subtrees are read from their own cooked assets. |
| [`og.ai.parse`](ai/fn-parse.md) | function | `(world: World, text: string): Result<BehaviourInfo>` | A behaviour from text, now (tests and generated behaviours; assets go through og cook). |
| [`og.ai.get`](ai/fn-get.md) | function | `(world: World, entity: Entity, key: string): unknown` | A blackboard value of an agent as of its last think. |
| [`og.ai.set`](ai/fn-set.md) | function | `(world: World, entity: Entity, key: string, value: unknown): Result<void>` | Set a blackboard value of an agent: a number, a boolean, a point `{x, y, z}` or an entity handle `{entity}`. |
| [`og.ai.restart`](ai/fn-restart.md) | function | `(world: World, entity: Entity): Result<void>` | Start an agent's behaviour afresh at its next think, as a new behaviour would: a machine in its initial state (its enter actions run again), a tree at its root, every timer and cooldown reset, the bl… |
| [`og.ai.finish`](ai/fn-finish.md) | function | `(world: World, entity: Entity, success: boolean, action?: string): boolean` | End an agent's running game actions (og.AiActionStarted): each answers success or failure at the agent's next think and sends og.AiActionStopped. |
| [`og.ai.noise`](ai/fn-noise.md) | function | `(world: World, desc?: NoiseDesc): Result<void>` | Make a noise agents may hear next tick (a gunshot, a door, a thrown stone). |
| [`og.ai.describe`](ai/fn-describe.md) | function | `(world: World, entity: Entity): AiDescription \| null` | What an agent is doing: its state, the node it runs and the path to it, its status, its blackboard and its target. |

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

<sub>`api.ogt` line 14139</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-AI_RUNNING"></a>`AI_RUNNING` | `number` | `1` | AiAgent.status: the tree or state is still running. |
| <a id="const-AI_SUCCESS"></a>`AI_SUCCESS` | `number` | `2` | AiAgent.status: the tree succeeded on the last think. |
| <a id="const-AI_FAILURE"></a>`AI_FAILURE` | `number` | `3` | AiAgent.status: the tree failed on the last think. |
| <a id="const-AI_INTERRUPTED"></a>`AI_INTERRUPTED` | `number` | `4` | AiActionStopped.result: the action stopped before the game finished it (another branch won, the state changed, the agent restarted). |

```ogt
const AI_RUNNING u32 1 "AiAgent.status: the tree or state is still running."
```

<sub>`api.ogt` line 14146</sub>

```ogt
const AI_SUCCESS u32 2 "AiAgent.status: the tree succeeded on the last think."
```

<sub>`api.ogt` line 14147</sub>

```ogt
const AI_FAILURE u32 3 "AiAgent.status: the tree failed on the last think."
```

<sub>`api.ogt` line 14148</sub>

```ogt
const AI_INTERRUPTED u32 4 "AiActionStopped.result: the action stopped before the game finished it (another branch won, the state changed, the agent restarted)."
```

<sub>`api.ogt` line 14149</sub>
