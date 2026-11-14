<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-LightVolume"></a>

### `og.LightVolume` (component)

A box of baked light probes (an irradiance volume): og bake lightmaps traces the light arriving from the scene at a grid of points spacing metres apart inside it. Every surface drawn inside it without a lightmap takes its ambient light from the probes instead of the sky's, fading out over og.Volume.fade past the box's faces.

The box is the og.Volume on its entity, as the world's axes bound it; without one it has no box and nothing is baked. Unbaked, it lights nothing and og doctor --project names the scene.

See `render §4`.

Token `og.LightVolume`, registered under the schema name `og.LightVolume`; its value type is `LightVolume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `spacing` | `number` | `1` | 0.1 … 16 | `m` | — | The most distance between neighbouring probes (the box is divided evenly, with probes on its faces; at most 64 a side). |

```ogt
component LightVolume {
  doc "A box of baked light probes (an irradiance volume): og bake lightmaps traces the light arriving from the scene at a grid of points spacing metres apart inside it. Every surface drawn inside it without a lightmap takes its ambient light from the probes instead of the sky's, fading out over og.Volume.fade past the box's faces."
  doc "The box is the og.Volume on its entity, as the world's axes bound it; without one it has no box and nothing is baked. Unbaked, it lights nothing and og doctor --project names the scene."
  guide "render §4"
  field spacing f32 "The most distance between neighbouring probes (the box is divided evenly, with probes on its faces; at most 64 a side)." {
    default 1
    min 0.1
    max 16
    unit m
  }
}
```

<sub>Module `render` line 4185</sub>
