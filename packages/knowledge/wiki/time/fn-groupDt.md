<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`time`](../time.md) · [index](../index.md)

<a id="fn-groupDt"></a>

### `og.time.groupDt`

This tick's scaled length for an entity: scaledDt times its time group's scale (scaledDt for an entity in no group). Move an entity's own gameplay by it so a slowed group slows everything it does.

```ts
og.time.groupDt(world: World, entity: Entity): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity. |

**Returns** `number`.

Binding symbol `og_time_group_dt` · publish builds: kept

```ogt
fn groupDt {
  doc "This tick's scaled length for an entity: scaledDt times its time group's scale (scaledDt for an entity in no group). Move an entity's own gameplay by it so a slowed group slows everything it does."
  param world World "The world."
  param entity Entity "The entity."
  returns f32
}
```

<sub>Module `time` line 80</sub>
