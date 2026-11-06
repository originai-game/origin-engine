<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-fracture"></a>

### `og.destruction.fracture`

Break every bond of a destructible at once (this tick, as damage): its anchored chunks stay, the rest fall.

```ts
og.destruction.fracture(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.Destructible. |

**Returns** `Result<void>`: Refused for an entity without one.

Binding symbol `og_destruction_fracture` · publish builds: kept

```ogt
fn fracture {
  doc "Break every bond of a destructible at once (this tick, as damage): its anchored chunks stay, the rest fall."
  param world World "The world."
  param entity Entity "An entity with og.Destructible."
  returns Result.Void "Refused for an entity without one."
}
```

<sub>`api.ogt` line 27324</sub>
