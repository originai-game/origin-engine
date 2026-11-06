<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="fn-shake"></a>

### `og.camera.shake`

Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared, so 0.1-0.2 is a light hit, 0.25-0.4 a medium one, 0.5-0.8 an explosion or a boss slam; 1 is almost never right (nauseating). Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick.

Also known as: screen shake, camera shake, trauma, Cinemachine impulse

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
  doc "Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared, so 0.1-0.2 is a light hit, 0.25-0.4 a medium one, 0.5-0.8 an explosion or a boss slam; 1 is almost never right (nauseating). Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick."
  aliases "screen shake, camera shake, trauma, Cinemachine impulse"
  param world World "The world."
  param trauma f32 "Trauma to add, 0 to 1."
  returns Result.Void
}
```

<sub>`api.ogt` line 3768</sub>
