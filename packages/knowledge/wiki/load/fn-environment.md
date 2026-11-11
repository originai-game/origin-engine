<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environment"></a>

### `og.load.environment`

A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment` (an equirectangular .hdr or .exr, a KTX2 cube map, or six faces). The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) and nine harmonics of diffuse irradiance, so loading only uploads it. One in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path.

See `loading §4`.

```ts
og.load.environment(world: World, path: string): Result<EnvironmentMap>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `env/studio.ogt`. |

**Returns** `Result<EnvironmentMap>`: Refused for a path not in the bundle (the reason says to run og cook) or not an environment map.

Binding symbol `og_load_environment` · publish builds: kept

```ogt
fn environment {
  doc "A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment` (an equirectangular .hdr or .exr, a KTX2 cube map, or six faces). The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) and nine harmonics of diffuse irradiance, so loading only uploads it. One in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path."
  guide "loading §4"
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `env/studio.ogt`."
  returns Result.EnvironmentMap "Refused for a path not in the bundle (the reason says to run og cook) or not an environment map."
}
```

<sub>Module `load` line 273</sub>
