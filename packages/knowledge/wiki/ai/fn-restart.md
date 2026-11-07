<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="fn-restart"></a>

### `og.ai.restart`

Start an agent's behaviour afresh at its next think, as a new behaviour would: a machine in its initial state (its entry actions run again), a tree at its root, every timer and cooldown reset, the blackboard back to the declared keys, wander's centre where it then stands, and what it saw and heard forgotten. Actions without a speed keep the speed it first started with. For a new run or a respawn; an agent that has not thought yet starts afresh anyway.

```ts
og.ai.restart(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The agent. |

**Returns** `Result<void>`: Refused for a world without og.ai.

Binding symbol `og_ai_restart` · publish builds: kept

```ogt
fn restart {
  doc "Start an agent's behaviour afresh at its next think, as a new behaviour would: a machine in its initial state (its entry actions run again), a tree at its root, every timer and cooldown reset, the blackboard back to the declared keys, wander's centre where it then stands, and what it saw and heard forgotten. Actions without a speed keep the speed it first started with. For a new run or a respawn; an agent that has not thought yet starts afresh anyway."
  param world World "The world."
  param entity Entity "The agent."
  returns Result.Void "Refused for a world without og.ai."
}
```

<sub>Module `ai` line 226</sub>
