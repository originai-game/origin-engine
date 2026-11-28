<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environment"></a>

### `og.load.environment`

A prefiltered environment map for Environment.map, from a TS file.environment declaration (an equirectangular HDR/EXR or sRGB image, a KTX2 cube map, or six faces). The cook produces five GGX levels and nine diffuse irradiance harmonics; loading only uploads them. One in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path.

```ts
og.load.environment(world: World, path: string): Result<EnvironmentMap>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The canonical content id, such as `base/env/studio`. |

**Returns** `Result<EnvironmentMap>`: Refused for a content ID not declared in the bundle or not an environment map.

Binding symbol `og_load_environment` · publish builds: kept

```ogt
fn environment {
  doc "A prefiltered environment map for Environment.map, from a TS file.environment declaration (an equirectangular HDR/EXR or sRGB image, a KTX2 cube map, or six faces). The cook produces five GGX levels and nine diffuse irradiance harmonics; loading only uploads them. One in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The canonical content id, such as `base/env/studio`."
  returns Result.EnvironmentMap "Refused for a content ID not declared in the bundle or not an environment map."
}
```

<sub>Module `load` line 313</sub>
