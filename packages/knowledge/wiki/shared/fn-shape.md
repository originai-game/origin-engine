<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-shape"></a>

### `og.shared.shape`

Set what the game shares besides the transform: its avatar component and its event component. Call it outside a system before og.shared.join; its fingerprint is checked when joining, so every client of a world needs the same shape. Without a call, the transform alone and no events.

See `shared-worlds §2`.

```ts
og.shared.shape(world: World, desc?: SharedShapeDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SharedShapeDesc` | yes | The avatar component and the event component. |

**Returns** `Result<void>`: Refused inside a system, during a session, or for a component whose packed bytes are too many.

Binding symbol `og_shared_shape` · publish builds: kept

```ogt
fn shape {
  doc "Set what the game shares besides the transform: its avatar component and its event component. Call it outside a system before og.shared.join; its fingerprint is checked when joining, so every client of a world needs the same shape. Without a call, the transform alone and no events."
  guide "shared-worlds §2"
  param world World "The world."
  param desc SharedShapeDesc "The avatar component and the event component."
  returns Result.Void "Refused inside a system, during a session, or for a component whose packed bytes are too many."
}
```

<sub>Module `shared` line 162</sub>
