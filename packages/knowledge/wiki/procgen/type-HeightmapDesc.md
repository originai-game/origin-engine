<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-HeightmapDesc"></a>

### `HeightmapDesc` (desc struct)

Deterministic procedural terrain heightmap, erosion, rivers and biome control map generator (matching `og cook`'s terrain generator bit for bit).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector2` | `{ x: 256, y: 256 }` | — | `m` | — | World extent along x and z. |
| `columns` | `number` | `257` | 2 … 4097 | — | — | Samples along x. |
| `rows` | `number` | `257` | 2 … 4097 | — | — | Samples along z. |
| `heights` | `Float32Array \| null` | `null` | — | — | — | Optional row-major metres (columns × rows). Skips noise/mask and applies cook's erosion, thermal, terraces, shore, rivers and biomes without normalising/clipping. Input terraces anchor at zero. |
| `channels` | `readonly TerrainChannelDesc[]` | `[]` | — | — | — | Explicit valley/river polylines; no inferred branches. Intersections take the lower carved bed. |
| `preserve` | `Float32Array \| null` | `null` | — | — | — | Optional columns × rows weights in 0..1. After all carving, blend toward the original base: 0 free, 1 bit-exact original. This is a final shape constraint, not an impermeable erosion boundary. |
| `preserveBorder` | `number` | `0` | ≤ 4097 | — | — | Width in samples of the boundary pinned to the original base, overriding preserve weights. A band covering the whole field pins every sample. |
| `minHeight` | `number` | `0` | — | `m` | — | Lowest base height before erosion. |
| `maxHeight` | `number` | `48` | — | `m` | — | Highest base height before erosion. |
| `seed` | `number` | `1` | — | — | — | Deterministic seed. |
| `octaves` | `number` | `6` | 1 … 12 | — | — | Fractal noise octaves. |
| `frequency` | `number` | `3` | 0.001 … 10000 | — | — | Base noise periods across the terrain. |
| `lacunarity` | `number` | `2` | 1 … 8 | — | — | Frequency multiplier per octave. |
| `gain` | `number` | `0.5` | 0 … 1 | — | — | Amplitude multiplier per octave. |
| `warp` | `number` | `0` | 0 … 100 | — | — | Domain warp strength (0: none). |
| `mask` | `HeightmapMask` | `"none"` | — | — | — | Macro terrain shaping mask. |
| `maskStrength` | `number` | `0.4` | 0 … 1 | — | — | Island coast falloff width (0..1) or ridge blend factor (0..1). |
| `erosion` | `HydraulicErosionDesc \| null` | `null` | — | — | — | Optional hydraulic droplet erosion pass. |
| `thermal` | `ThermalErosionDesc \| null` | `null` | — | — | — | Optional thermal talus erosion pass. |
| `terraceStep` | `number` | `0` | 0 … 1000000 | `m` | — | Terrace step height in metres (0: disabled). |
| `terraceBlend` | `number` | `0.4` | 0 … 1 | — | — | Smooth fraction of a terrace step, matching terrain.generate.terraces.blend. |
| `seaLevel` | `number \| null` | `null` | — | `m` | — | Optional sea level height for coastal shore flattening and river outlets. |
| `shore` | `number` | `2` | 0 … 1000000 | `m` | — | Shore flattening margin above sea level. |
| `rivers` | `RiverCarveDesc \| null` | `null` | — | — | — | Optional drainage river carving pass. |
| `moisture` | `MoistureDesc` | `{}` | — | — | — | Moisture settings for biome painting. |
| `biomes` | `readonly BiomeRuleDesc[]` | `[]` | — | — | — | Biome control-map rules (empty omits the control map). |

```ogt
struct HeightmapDesc {
  doc "Deterministic procedural terrain heightmap, erosion, rivers and biome control map generator (matching `og cook`'s terrain generator bit for bit)."
  desc true
  field size Vector2 "World extent along x and z." {
    default 256 256
    unit m
  }
  field columns u32 "Samples along x." {
    default 257
    min 2
    max 4097
  }
  field rows u32 "Samples along z." {
    default 257
    min 2
    max 4097
  }
  field heights Option.Float32Array "Optional row-major metres (columns × rows). Skips noise/mask and applies cook's erosion, thermal, terraces, shore, rivers and biomes without normalising/clipping. Input terraces anchor at zero."
  field channels List.TerrainChannelDesc "Explicit valley/river polylines; no inferred branches. Intersections take the lower carved bed."
  field preserve Option.Float32Array "Optional columns × rows weights in 0..1. After all carving, blend toward the original base: 0 free, 1 bit-exact original. This is a final shape constraint, not an impermeable erosion boundary."
  field preserveBorder u32 "Width in samples of the boundary pinned to the original base, overriding preserve weights. A band covering the whole field pins every sample." {
    max 4097
  }
  field minHeight f32 "Lowest base height before erosion." {
    default 0
    unit m
  }
  field maxHeight f32 "Highest base height before erosion." {
    default 48
    unit m
  }
  field seed u32 "Deterministic seed." {
    default 1
  }
  field octaves u32 "Fractal noise octaves." {
    default 6
    min 1
    max 12
  }
  field frequency f32 "Base noise periods across the terrain." {
    default 3
    min 0.001
    max 10000
  }
  field lacunarity f32 "Frequency multiplier per octave." {
    default 2
    min 1
    max 8
  }
  field gain f32 "Amplitude multiplier per octave." {
    default 0.5
    min 0
    max 1
  }
  field warp f32 "Domain warp strength (0: none)." {
    min 0
    max 100
  }
  field mask HeightmapMask "Macro terrain shaping mask."
  field maskStrength f32 "Island coast falloff width (0..1) or ridge blend factor (0..1)." {
    default 0.4
    min 0
    max 1
  }
  field erosion Option.HydraulicErosionDesc "Optional hydraulic droplet erosion pass."
  field thermal Option.ThermalErosionDesc "Optional thermal talus erosion pass."
  field terraceStep f32 "Terrace step height in metres (0: disabled)." {
    min 0
    max 1000000
    unit m
  }
  field terraceBlend f32 "Smooth fraction of a terrace step, matching terrain.generate.terraces.blend." {
    default 0.4
    min 0
    max 1
  }
  field seaLevel Option.f32 "Optional sea level height for coastal shore flattening and river outlets." {
    unit m
  }
  field shore f32 "Shore flattening margin above sea level." {
    default 2
    min 0
    max 1000000
    unit m
  }
  field rivers Option.RiverCarveDesc "Optional drainage river carving pass."
  field moisture MoistureDesc "Moisture settings for biome painting."
  field biomes List.BiomeRuleDesc "Biome control-map rules (empty omits the control map)."
}
```

<sub>Module `procgen` line 499</sub>
