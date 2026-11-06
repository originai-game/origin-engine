<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-damage"></a>

### `og.destruction.damage`

Damage a destructible at a point: every bond, anchor and chunk within reach takes the amount, all of it within innerRadius and falling to nothing at radius. Applied by og.destruction after the game's systems this tick (in the order called), so a bond broken now splits the prop before this tick's physics step.

```ts
og.destruction.damage(world: World, entity: Entity, desc?: DamageDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.Destructible, or one of its chunks. |
| `desc` | `DamageDesc` | yes | Where, how far and how much. |

**Returns** `Result<void>`: Refused for an entity that is neither, or a point that is not finite.

Binding symbol `og_destruction_damage` · publish builds: kept

```ogt
fn damage {
  doc "Damage a destructible at a point: every bond, anchor and chunk within reach takes the amount, all of it within innerRadius and falling to nothing at radius. Applied by og.destruction after the game's systems this tick (in the order called), so a bond broken now splits the prop before this tick's physics step."
  param world World "The world."
  param entity Entity "An entity with og.Destructible, or one of its chunks."
  param desc DamageDesc "Where, how far and how much."
  returns Result.Void "Refused for an entity that is neither, or a point that is not finite."
}
```

<sub>`api.ogt` line 27359</sub>
