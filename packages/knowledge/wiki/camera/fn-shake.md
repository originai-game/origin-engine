<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="fn-shake"></a>

### `og.camera.shake`

Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared. Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick.

See `camera §4`.

Also known as: screen shake, camera shake, screen trembles when hit, hit shake, trauma, Cinemachine impulse

```ts
og.camera.shake(world: World, trauma: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `trauma` | `number` | no | Trauma to add, 0 to 1. |

**Returns** `Result<void>`.

Binding symbol `og_camera_shake` · publish builds: kept

```ogt
fn shake {
  doc "Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared. Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick."
  aliases "screen shake, camera shake, screen trembles when hit, hit shake, trauma, Cinemachine impulse"
  guide "camera §4"
  param world World "The world."
  param trauma f32 "Trauma to add, 0 to 1."
  returns Result.Void
}
```

<sub>Module `camera` line 560</sub>
