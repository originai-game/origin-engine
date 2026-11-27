<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environmentSun"></a>

### `og.load.environmentSun`

The sun a cooked environment map holds apart from its image (sun: 'extract' in file.environment): the direction, colour and intensity to give a DirectionalLight, so the sun is counted once, casts shadows and leaves no speckle in rough reflections. An image's sun left in the map lights without shadows, and a DirectionalLight beside it adds the sun a second time.

See `loading §4`.

```ts
og.load.environmentSun(world: World, map: EnvironmentMap, rotation?: number, intensity?: number): Result<EnvironmentSun>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the map. |
| `map` | `EnvironmentMap` | no | A map og.load.environment answered. |
| `rotation` | `number` | yes | The Environment.rotation in radians about +Y; applied here, not by the game. Default `0`. |
| `intensity` | `number` | yes | The Environment.intensity multiplier, applied here to the cooked sun, in the same scale as the map lighting. Default `1`. |

**Returns** `Result<EnvironmentSun>`: Refused for a handle that is not an environment map, or one whose streaming group is not resident yet.

Binding symbol `og_load_environment_sun` · publish builds: kept

```ogt
fn environmentSun {
  doc "The sun a cooked environment map holds apart from its image (sun: 'extract' in file.environment): the direction, colour and intensity to give a DirectionalLight, so the sun is counted once, casts shadows and leaves no speckle in rough reflections. An image's sun left in the map lights without shadows, and a DirectionalLight beside it adds the sun a second time."
  guide "loading §4"
  param world World "The world whose renderer owns the map."
  param map EnvironmentMap "A map og.load.environment answered."
  param rotation f32 "The Environment.rotation in radians about +Y; applied here, not by the game." {
    default 0
  }
  param intensity f32 "The Environment.intensity multiplier, applied here to the cooked sun, in the same scale as the map lighting." {
    default 1
  }
  returns Result.EnvironmentSun "Refused for a handle that is not an environment map, or one whose streaming group is not resident yet."
}
```

<sub>Module `load` line 342</sub>
