<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-info"></a>

### `og.destruction.info`

How much of a destructible stands.

```ts
og.destruction.info(world: World, entity: Entity): Result<DestructibleInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.Destructible. |

**Returns** `Result<DestructibleInfo>`: Refused for an entity without one, or one whose model is not loaded or has no fracture.

Binding symbol `og_destruction_info` · publish builds: kept

```ogt
fn info {
  doc "How much of a destructible stands."
  param world World "The world."
  param entity Entity "An entity with og.Destructible."
  returns Result.DestructibleInfo "Refused for an entity without one, or one whose model is not loaded or has no fracture."
}
```

<sub>Module `destruction` line 287</sub>
