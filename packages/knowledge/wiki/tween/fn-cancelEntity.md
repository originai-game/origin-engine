<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-cancelEntity"></a>

### `og.tween.cancelEntity`

Stop every tween of an entity (no events), say before starting a new reaction.

```ts
og.tween.cancelEntity(world: World, entity: Entity): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity. |

**Returns** `number`: How many stopped.

Binding symbol `og_tween_cancel_entity` · publish builds: kept

```ogt
fn cancelEntity {
  doc "Stop every tween of an entity (no events), say before starting a new reaction."
  param world World "The world."
  param entity Entity "The entity."
  returns u32 "How many stopped."
}
```

<sub>`api.ogt` line 12527</sub>
