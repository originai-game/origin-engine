<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="fn-punch"></a>

### `og.camera.punch`

A zoom punch: the view narrows by `amount` and springs back by Camera.punchDecay. Visual only.

See `camera §4`.

```ts
og.camera.punch(world: World, amount: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `amount` | `number` | no | The fraction to zoom in (negative zooms out), -0.9 to 4. |

**Returns** `Result<void>`.

Binding symbol `og_camera_punch` · publish builds: kept

```ogt
fn punch {
  doc "A zoom punch: the view narrows by `amount` and springs back by Camera.punchDecay. Visual only."
  guide "camera §4"
  param world World "The world."
  param amount f32 "The fraction to zoom in (negative zooms out), -0.9 to 4."
  returns Result.Void
}
```

<sub>Module `camera` line 570</sub>
