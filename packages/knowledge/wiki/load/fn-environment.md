<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environment"></a>

### `og.load.environment`

A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it) or a KTX2 cube map (six square faces +X -X +Y -Y +Z -Z with rows from the top; RGBA16F, RGB9E5, BC6H and the other HDR formats, decoded at cook time), or `faces` with six `.hdr`/`.exr` images in that order; `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it; `prefiltered true` takes a KTX2 cube's own mip chain as those levels instead (one already GGX-prefiltered by cmgen, IBLBaker or the glTF IBL sampler: level k of L at roughness k / (L - 1)), at the file's own size, without filtering it again. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path.

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
  doc "A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it) or a KTX2 cube map (six square faces +X -X +Y -Y +Z -Z with rows from the top; RGBA16F, RGB9E5, BC6H and the other HDR formats, decoded at cook time), or `faces` with six `.hdr`/`.exr` images in that order; `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it; `prefiltered true` takes a KTX2 cube's own mip chain as those levels instead (one already GGX-prefiltered by cmgen, IBLBaker or the glTF IBL sampler: level k of L at roughness k / (L - 1)), at the file's own size, without filtering it again. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `env/studio.ogt`."
  returns Result.EnvironmentMap "Refused for a path not in the bundle (the reason says to run og cook) or not an environment map."
}
```

<sub>Module `load` line 258</sub>
