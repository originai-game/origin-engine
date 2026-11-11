<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainLayerDesc"></a>

### `TerrainLayerDesc` (desc struct)

One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Multiplies the layer's texture (linear). |
| `roughness` | `number` | `0.9` | 0.04 … 1 | — | — | Its roughness. |
| `tile` | `number` | `4` | 0.01 … 1000 | `m` | — | Metres per repeat of its textures. |
| `triplanar` | `boolean` | `false` | — | — | — | Sample by three projections blended by the normal (for cliffs). `projectSlope` above 0 decides instead. |
| `projectSlope` | `number` | `0` | 0 … 90 | `deg` | — | The slope past which the layer is sampled by the two projections facing the surface most (biplanar: no projection seen edge-on, so a steep face does not streak) instead of from above, faded over 10° around it; below it, from above. 0: `triplanar` decides. |
| `antiTiling` | `number` | `0` | 0 … 1 | `ratio` | — | How far the texture's repeat is broken: each point blends three hexagonal tiles of it, each turned (by up to half a turn at 1) and moved by its own hash, so no period shows across a field. 0: the plain repeat. |
| `heightContrast` | `number` | `0` | 0 … 1 | `ratio` | — | How sharp the height blend (the terrain file's `heightBlend`) is at this layer's edges: 0 the default window, 1 ten times narrower (a crisp rim). |
| `macroVariation` | `number` | `0` | 0 … 1 | `ratio` | — | A slow drift of brightness and hue over the ground and, with distance, the texture at a quarter of its frequency blended in, so a field of one layer does not read as one texture from afar. 0: none. |
| `displacement` | `number` | `0` | 0 … 4 | `m` | — | Metres the drawn ground rises and falls by the layer's height map (its albedo's alpha, about its middle) within og.Terrain's `displacementDistance`, where the layer shows and the ground is not steep. Only drawn: collision, og.terrain.sample and picking keep the height field. 0: flat. See `world §2`. |
| `cellVariation` | `number` | `0` | 0 … 1 | `ratio` | — | How much each cell of the layer's pattern takes its own tone: its normal map's alpha names the cell (0 to 1; opaque for none), and at 1 each cell's brightness and warmth move by up to 25 % and 20 %. 0: none. See `world §2`. |
| `normalStrength` | `number` | `1` | 0 … 4 | — | — | How strongly its normal map bends the surface. |
| `slopeMin` | `number` | `0` | 0 … 90 | `deg` | — | The flattest slope it shows on. |
| `slopeMax` | `number` | `90` | 0 … 90 | `deg` | — | The steepest. |
| `slopeFade` | `number` | `4` | 0 … 90 | `deg` | — | How far past its slopes it fades out. |
| `altitudeMin` | `number` | `-1000000` | — | `m` | — | The lowest height above the terrain's origin it shows at. |
| `altitudeMax` | `number` | `1000000` | — | `m` | — | The highest. |
| `altitudeFade` | `number` | `4` | ≥ 0 | `m` | — | How far past its heights it fades out. |

```ogt
struct TerrainLayerDesc {
  doc "One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised."
  desc true
  field color Color "Multiplies the layer's texture (linear)." {
    default 1 1 1
  }
  field roughness f32 "Its roughness." {
    default 0.9
    min 0.04
    max 1
  }
  field tile f32 "Metres per repeat of its textures." {
    default 4
    min 0.01
    max 1000
    unit m
  }
  field triplanar bool "Sample by three projections blended by the normal (for cliffs). `projectSlope` above 0 decides instead."
  field projectSlope f32 "The slope past which the layer is sampled by the two projections facing the surface most (biplanar: no projection seen edge-on, so a steep face does not streak) instead of from above, faded over 10° around it; below it, from above. 0: `triplanar` decides." {
    min 0
    max 90
    unit deg
  }
  field antiTiling f32 "How far the texture's repeat is broken: each point blends three hexagonal tiles of it, each turned (by up to half a turn at 1) and moved by its own hash, so no period shows across a field. 0: the plain repeat." {
    min 0
    max 1
    unit ratio
  }
  field heightContrast f32 "How sharp the height blend (the terrain file's `heightBlend`) is at this layer's edges: 0 the default window, 1 ten times narrower (a crisp rim)." {
    min 0
    max 1
    unit ratio
  }
  field macroVariation f32 "A slow drift of brightness and hue over the ground and, with distance, the texture at a quarter of its frequency blended in, so a field of one layer does not read as one texture from afar. 0: none." {
    min 0
    max 1
    unit ratio
  }
  field displacement f32 "Metres the drawn ground rises and falls by the layer's height map (its albedo's alpha, about its middle) within og.Terrain's `displacementDistance`, where the layer shows and the ground is not steep. Only drawn: collision, og.terrain.sample and picking keep the height field. 0: flat." {
    guide "world §2"
    min 0
    max 4
    unit m
  }
  field cellVariation f32 "How much each cell of the layer's pattern takes its own tone: its normal map's alpha names the cell (0 to 1; opaque for none), and at 1 each cell's brightness and warmth move by up to 25 % and 20 %. 0: none." {
    guide "world §2"
    min 0
    max 1
    unit ratio
  }
  field normalStrength f32 "How strongly its normal map bends the surface." {
    default 1
    min 0
    max 4
  }
  field slopeMin f32 "The flattest slope it shows on." {
    min 0
    max 90
    unit deg
  }
  field slopeMax f32 "The steepest." {
    default 90
    min 0
    max 90
    unit deg
  }
  field slopeFade f32 "How far past its slopes it fades out." {
    default 4
    min 0
    max 90
    unit deg
  }
  field altitudeMin f32 "The lowest height above the terrain's origin it shows at." {
    default -1000000
    unit m
  }
  field altitudeMax f32 "The highest." {
    default 1000000
    unit m
  }
  field altitudeFade f32 "How far past its heights it fades out." {
    default 4
    min 0
    unit m
  }
}
```

<sub>Module `terrain` line 553</sub>
