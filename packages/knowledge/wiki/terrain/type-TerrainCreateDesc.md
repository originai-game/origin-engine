<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainCreateDesc"></a>

### `TerrainCreateDesc` (desc struct)

Create a terrain directly at run time from a heightmap Float32Array (for example from og.procgen.heightmap or og.procgen.noiseGrid), with optional biome control map, holes, material layers and height-field collision. Drawn, sampled, edited, stamped and collided with exactly like a cooked terrain.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector2` | `{ x: 64, y: 64 }` | — | `m` | — | World extent along x and z. |
| `columns` | `number` | `65` | 2 … 4096 | — | — | Height samples along x (at least 2). |
| `rows` | `number` | `65` | 2 … 4096 | — | — | Height samples along z (at least 2). |
| `heights` | `Float32Array` | — | — | — | — | Row-major heights in metres (`columns * rows` values, -z to +z, -x to +x). |
| `control` | `Uint8Array \| null` | `null` | — | — | — | Optional RGBA8 control map pages (`controlWidth * controlHeight * 4 * pages` bytes, 4 layers per page). |
| `controlWidth` | `number` | `0` | ≤ 4096 | — | — | Control map width in texels (0 defaults to `columns`). |
| `controlHeight` | `number` | `0` | ≤ 4096 | — | — | Control map height in texels (0 defaults to `rows`). |
| `holes` | `Uint8Array \| null` | `null` | — | — | — | Optional per-cell hole mask (`(columns - 1) * (rows - 1)` bytes; non-zero cuts a hole). |
| `layers` | `readonly TerrainLayerDesc[]` | `[]` | — | — | — | Material layers (up to 16; empty creates one default neutral layer). |
| `heightBlend` | `number` | `0.15` | 0 … 1 | — | — | Height-blend transition window between layers (0: linear weights). |
| `seaLevel` | `number` | `0` | — | `m` | — | Reference sea level height reported in TerrainInfo.seaLevel. |
| `collision` | `boolean` | `true` | — | — | — | Register a physics height-field collider under TerrainInfo.geometry. |
| `castShadows` | `boolean` | `true` | — | — | — | Whether the terrain casts shadows by default. |

```ogt
struct TerrainCreateDesc {
  doc "Create a terrain directly at run time from a heightmap Float32Array (for example from og.procgen.heightmap or og.procgen.noiseGrid), with optional biome control map, holes, material layers and height-field collision. Drawn, sampled, edited, stamped and collided with exactly like a cooked terrain."
  desc true
  field size Vector2 "World extent along x and z." {
    default 64 64
    unit m
  }
  field columns u32 "Height samples along x (at least 2)." {
    default 65
    min 2
    max 4096
  }
  field rows u32 "Height samples along z (at least 2)." {
    default 65
    min 2
    max 4096
  }
  field heights Float32Array "Row-major heights in metres (`columns * rows` values, -z to +z, -x to +x)."
  field control Option.Uint8Array "Optional RGBA8 control map pages (`controlWidth * controlHeight * 4 * pages` bytes, 4 layers per page)."
  field controlWidth u32 "Control map width in texels (0 defaults to `columns`)." {
    max 4096
  }
  field controlHeight u32 "Control map height in texels (0 defaults to `rows`)." {
    max 4096
  }
  field holes Option.Uint8Array "Optional per-cell hole mask (`(columns - 1) * (rows - 1)` bytes; non-zero cuts a hole)."
  field layers List.TerrainLayerDesc "Material layers (up to 16; empty creates one default neutral layer)."
  field heightBlend f32 "Height-blend transition window between layers (0: linear weights)." {
    default 0.15
    min 0
    max 1
  }
  field seaLevel f32 "Reference sea level height reported in TerrainInfo.seaLevel." {
    unit m
  }
  field collision bool "Register a physics height-field collider under TerrainInfo.geometry." {
    default true
  }
  field castShadows bool "Whether the terrain casts shadows by default." {
    default true
  }
}
```

<sub>Module `terrain` line 658</sub>
