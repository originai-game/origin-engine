<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-LightVolume"></a>

### `og.LightVolume` (component)

A box of baked light probes (an irradiance volume): og bake lightmaps traces the light arriving from the scene (off static models, the sky, emissive surfaces) at a grid of points spacing metres apart inside it, as first-order spherical harmonics. Every surface drawn inside it without a lightmap of its own takes its ambient light from the probes instead of the sky's, blended at the box's edge over one spacing.

The box is size metres centred on the entity, axis-aligned in world space (its rotation and scale are ignored). Unbaked, it lights nothing and og doctor --project names the scene.

See `render §4`.

Token `og.LightVolume`, registered under the schema name `og.LightVolume`; its value type is `LightVolume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector3` | `{ x: 4, y: 4, z: 4 }` | — | `m` | — | The box's extent along world x, y and z. |
| `spacing` | `number` | `1` | 0.1 … 16 | `m` | — | The most distance between neighbouring probes (the box is divided evenly, with probes on its faces; at most 64 a side). |

```ogt
component LightVolume {
  doc "A box of baked light probes (an irradiance volume): og bake lightmaps traces the light arriving from the scene (off static models, the sky, emissive surfaces) at a grid of points spacing metres apart inside it, as first-order spherical harmonics. Every surface drawn inside it without a lightmap of its own takes its ambient light from the probes instead of the sky's, blended at the box's edge over one spacing."
  doc "The box is size metres centred on the entity, axis-aligned in world space (its rotation and scale are ignored). Unbaked, it lights nothing and og doctor --project names the scene."
  guide "render §4"
  field size Vector3 "The box's extent along world x, y and z." {
    default 4 4 4
    unit m
  }
  field spacing f32 "The most distance between neighbouring probes (the box is divided evenly, with probes on its faces; at most 64 a side)." {
    default 1
    min 0.1
    max 16
    unit m
  }
}
```

<sub>Module `render` line 4128</sub>
