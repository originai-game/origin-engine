<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

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

<sub>Module `ai` line 250</sub>
