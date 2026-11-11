<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="record-Terrain"></a>

### `og.Terrain` (component)

Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to.

Token `og.Terrain`, registered under the schema name `og.Terrain`; its value type is `Terrain`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `visible` | `boolean` | `true` | — | — | — | Draw it; false keeps it only as the ground its og.TerrainScatter copies stand on (nothing else sees it). |
| `castShadows` | `boolean` | `true` | — | — | — | Cast the directional and local lights' shadows (it always receives them). |
| `horizonShadows` | `boolean` | `true` | — | — | — | Shade it by its own horizon, baked by the cook: past the sun's shadow cascades a valley behind a ridge falls into its shade (to the horizon), and everywhere the sky a valley's walls hide lights it less. |
| `lodSpacing` | `number` | `0` | 0 … 64 | `m` | tunable | The finest clipmap level's spacing (0: the terrain file's, or the samples' spacing). |
| `lodRing` | `number` | `0` | 0 … 256 | — | — | Quads per side of every clipmap level at least (0: the terrain file's, else 64, or 256 for a terrain whose layers displace it; 16 to 256, rounded to a multiple of 4). Levels grow to 128 quads where a coarser level's height error would show (`lodError`). See `world §2`. |
| `lodError` | `number` | `0` | 0 … 64 | — | tunable | Pixels of height error a coarser clipmap level may show before the finer one reaches further out (0: 1): a peak that is large on screen keeps its shape at any distance. Larger draws fewer vertices. |
| `displacementDistance` | `number` | `40` | 0 … 1000 | `m` | tunable | Metres from the camera within which layers with a `displacement` raise and lower the drawn ground, fading out over the last 40%. Near the camera the clipmap's levels then grow finer and denser to carry the relief (`lodSpacing` and `lodRing` above 0 decide instead). 0: no displacement. See `world §2`. |

```ogt
component Terrain {
  doc "Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field visible bool "Draw it; false keeps it only as the ground its og.TerrainScatter copies stand on (nothing else sees it)." {
    default true
  }
  field castShadows bool "Cast the directional and local lights' shadows (it always receives them)." {
    default true
  }
  field horizonShadows bool "Shade it by its own horizon, baked by the cook: past the sun's shadow cascades a valley behind a ridge falls into its shade (to the horizon), and everywhere the sky a valley's walls hide lights it less." {
    default true
  }
  field lodSpacing f32 "The finest clipmap level's spacing (0: the terrain file's, or the samples' spacing)." {
    min 0
    max 64
    unit m
    tunable true
  }
  field lodRing u32 "Quads per side of every clipmap level at least (0: the terrain file's, else 64, or 256 for a terrain whose layers displace it; 16 to 256, rounded to a multiple of 4). Levels grow to 128 quads where a coarser level's height error would show (`lodError`)." {
    guide "world §2"
    min 0
    max 256
  }
  field lodError f32 "Pixels of height error a coarser clipmap level may show before the finer one reaches further out (0: 1): a peak that is large on screen keeps its shape at any distance. Larger draws fewer vertices." {
    min 0
    max 64
    tunable true
  }
  field displacementDistance f32 "Metres from the camera within which layers with a `displacement` raise and lower the drawn ground, fading out over the last 40%. Near the camera the clipmap's levels then grow finer and denser to carry the relief (`lodSpacing` and `lodRing` above 0 decide instead). 0: no displacement." {
    guide "world §2"
    default 40
    min 0
    max 1000
    unit m
    tunable true
  }
}
```

<sub>Module `terrain` line 32</sub>
