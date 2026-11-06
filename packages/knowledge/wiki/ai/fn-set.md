<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

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

<sub>`api.ogt` line 14345</sub>
