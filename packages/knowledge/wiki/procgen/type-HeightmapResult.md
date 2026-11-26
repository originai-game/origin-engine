<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-HeightmapResult"></a>

### `HeightmapResult` (struct)

Generated procedural heightfield and optional biome control map, ready for og.terrain.create.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `columns` | `number` | `0` | — | — | — | Samples along x. |
| `rows` | `number` | `0` | — | — | — | Samples along z. |
| `minHeight` | `number` | `0` | — | `m` | — | Lowest generated height. |
| `maxHeight` | `number` | `0` | — | `m` | — | Highest generated height. |
| `riverLength` | `number` | `0` | — | `m` | — | Total length of carved river channels. |
| `heights` | `Float32Array` | — | — | — | — | Row-major heights in metres (`columns * rows`). |
| `control` | `Uint8Array \| null` | `null` | — | — | — | RGBA8 biome control-map pages (`columns * rows * 4 * pages`), or null when no biomes were given. |

```ogt
struct HeightmapResult {
  doc "Generated procedural heightfield and optional biome control map, ready for og.terrain.create."
  field columns u32 "Samples along x."
  field rows u32 "Samples along z."
  field minHeight f32 "Lowest generated height." {
    unit m
  }
  field maxHeight f32 "Highest generated height." {
    unit m
  }
  field riverLength f32 "Total length of carved river channels." {
    unit m
  }
  field heights Float32Array "Row-major heights in metres (`columns * rows`)."
  field control Option.Uint8Array "RGBA8 biome control-map pages (`columns * rows * 4 * pages`), or null when no biomes were given."
}
```

<sub>Module `procgen` line 589</sub>
