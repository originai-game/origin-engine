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
| `lodRing` | `number` | `0` | 0 … 256 | — | — | Quads per side of every clipmap level at least (0: the terrain file's, else 64, or 256 for a terrain whose layers displace it; 16 to 256, rounded to a multiple of 4). The levels grow to 128 quads where a coarser level's height error would show (`lodError`); 256 draws every level that large, at about 3 ms more on an M4 Pro. |
| `lodError` | `number` | `0` | 0 … 64 | — | tunable | Pixels of height error a coarser clipmap level may show before the finer one reaches further out (0: 1): a peak that is large on screen keeps its shape at any distance. Larger draws fewer vertices. |
| `displacementDistance` | `number` | `40` | 0 … 1000 | `m` | tunable | Metres from the camera within which layers with a `displacement` raise and lower the drawn ground by their height maps, fading out over the last 40%. Near the camera the clipmap's levels then grow finer than the samples (down to a 128th of the finest displacing layer's `tile`, a 128th of a sample at most; `lodSpacing` above 0 decides instead) and denser (256 quads a side unless `lodRing` says) to carry the relief: a sett's joint and arris are drawn by vertices, not stepped across a grid cell. Each corner is evaluated once a frame for every pass (T4's cobbled crest: 6.5 ms of GPU a frame on an M4 Pro against 3.8 with the default 64-quad levels). 0: no displacement. |

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
  field lodRing u32 "Quads per side of every clipmap level at least (0: the terrain file's, else 64, or 256 for a terrain whose layers displace it; 16 to 256, rounded to a multiple of 4). The levels grow to 128 quads where a coarser level's height error would show (`lodError`); 256 draws every level that large, at about 3 ms more on an M4 Pro." {
    min 0
    max 256
  }
  field lodError f32 "Pixels of height error a coarser clipmap level may show before the finer one reaches further out (0: 1): a peak that is large on screen keeps its shape at any distance. Larger draws fewer vertices." {
    min 0
    max 64
    tunable true
  }
  field displacementDistance f32 "Metres from the camera within which layers with a `displacement` raise and lower the drawn ground by their height maps, fading out over the last 40%. Near the camera the clipmap's levels then grow finer than the samples (down to a 128th of the finest displacing layer's `tile`, a 128th of a sample at most; `lodSpacing` above 0 decides instead) and denser (256 quads a side unless `lodRing` says) to carry the relief: a sett's joint and arris are drawn by vertices, not stepped across a grid cell. Each corner is evaluated once a frame for every pass (T4's cobbled crest: 6.5 ms of GPU a frame on an M4 Pro against 3.8 with the default 64-quad levels). 0: no displacement." {
    default 40
    min 0
    max 1000
    unit m
    tunable true
  }
}
```

<sub>`api.ogt` line 24809</sub>
