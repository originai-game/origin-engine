<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environmentSunRotation"></a>

### `og.load.environmentSunRotation`

The Environment.rotation that puts a cooked map's sun at `azimuth` around a view direction, so a sky is turned by where its sun should be ("light from behind the camera's left") instead of by a hand-tuned angle. The answer is exact for the map's extracted sun (og.load.environmentSun at rotation 0); its elevation is the map's, reported in `elevation`.

```ts
og.load.environmentSunRotation(world: World, map: EnvironmentMap, view: Vector3, azimuth: number): Result<EnvironmentSunTurn>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the map. |
| `map` | `EnvironmentMap` | no | A map og.load.environment answered, cooked with its sun extracted. |
| `view` | `Vector3` | no | The direction the view looks, such as the camera's forward; only its horizontal part counts. |
| `azimuth` | `number` | no | Where the sun stands around the view, in radians from straight ahead and positive to the view's right: 0 ahead (the subject backlit), pi behind the viewer (the subject lit from the front), -pi/2 to its left, -3pi/4 behind its left shoulder. |

**Returns** `Result<EnvironmentSunTurn>`: Refused when og.load.environmentSun refuses the map, when the map holds no sun (its cook found none: the cook's report says why), when the sun stands straight overhead (no turn moves it), or when `view` is vertical.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn environmentSunRotation {
  doc "The Environment.rotation that puts a cooked map's sun at `azimuth` around a view direction, so a sky is turned by where its sun should be (\"light from behind the camera's left\") instead of by a hand-tuned angle. The answer is exact for the map's extracted sun (og.load.environmentSun at rotation 0); its elevation is the map's, reported in `elevation`."
  host false
  param world World "The world whose renderer owns the map."
  param map EnvironmentMap "A map og.load.environment answered, cooked with its sun extracted."
  param view Vector3 "The direction the view looks, such as the camera's forward; only its horizontal part counts."
  param azimuth f32 "Where the sun stands around the view, in radians from straight ahead and positive to the view's right: 0 ahead (the subject backlit), pi behind the viewer (the subject lit from the front), -pi/2 to its left, -3pi/4 behind its left shoulder."
  returns Result.EnvironmentSunTurn "Refused when og.load.environmentSun refuses the map, when the map holds no sun (its cook found none: the cook's report says why), when the sun stands straight overhead (no turn moves it), or when `view` is vertical."
}
```

<sub>Module `load` line 348</sub>
