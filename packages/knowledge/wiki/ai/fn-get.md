<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

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

<sub>`api.ogt` line 14391</sub>
