<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="fn-kick"></a>

### `og.camera.kick`

A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. Visual only.

See `camera §4`.

```ts
og.camera.kick(world: World, direction: Vector3, distance: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `direction` | `Vector3` | no | The direction to push the view (normalised here; zero does nothing). |
| `distance` | `number` | no | How far, in world units. |

**Returns** `Result<void>`.

Binding symbol `og_camera_kick` · publish builds: kept

```ogt
fn kick {
  doc "A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. Visual only."
  guide "camera §4"
  param world World "The world."
  param direction Vector3 "The direction to push the view (normalised here; zero does nothing)."
  param distance f32 "How far, in world units."
  returns Result.Void
}
```

<sub>Module `camera` line 578</sub>
