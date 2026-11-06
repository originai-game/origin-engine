<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-flash"></a>

### `og.tween.flash`

Flash the entity's og.Sprite, its og.Mesh, or its og.Model (a glTF instance: this instance only, the others sharing its materials stay as they are). Cancels its earlier flash.

```ts
og.tween.flash(world: World, desc?: FlashDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FlashDesc` | yes | The entity and the flash. |

**Returns** `Result<Tween>`: Refused for an entity with none of og.Sprite, og.Mesh and og.Model.

Binding symbol `og_tween_flash` · publish builds: kept

```ogt
fn flash {
  doc "Flash the entity's og.Sprite, its og.Mesh, or its og.Model (a glTF instance: this instance only, the others sharing its materials stay as they are). Cancels its earlier flash."
  param world World "The world."
  param desc FlashDesc "The entity and the flash."
  returns Result.Tween "Refused for an entity with none of og.Sprite, og.Mesh and og.Model."
}
```

<sub>`api.ogt` line 12491</sub>
