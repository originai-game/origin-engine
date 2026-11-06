<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Fog"></a>

### `og.Fog` (resource)

Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ramp from start to full fog at end; the two apply together, and density defaults to 0.02, so setting only end still adds exponential fog (set density 0 for the ramp alone). Height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged.

Token `og.Fog`, registered under the schema name `og.Fog`; its value type is `Fog`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the fog. |
| `color` | `Color` | `{ r: 0.62, g: 0.68, b: 0.76 }` | — | — | — | The colour the fog tends to (linear, before exposure). |
| `density` | `number` | `0.02` | 0 … 2 | — | tunable | Exponential distance fog per metre past start, with or without end's ramp (0: none). |
| `start` | `number` | `0` | ≥ 0 | `m` | tunable | The distance from the camera where distance fog starts. |
| `end` | `number` | `0` | ≥ 0 | `m` | tunable | The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp). |
| `heightDensity` | `number` | `0` | 0 … 10 | — | tunable | Height fog at and below height (0: none). |
| `height` | `number` | `0` | — | `m` | tunable, worldspace `world y` | The world height below which height fog is densest. |
| `falloff` | `number` | `0.5` | 0.0001 … 20 | — | tunable | How fast height fog thins above height, per metre. |
| `sunColor` | `Color` | `{ r: 1, g: 0.86, b: 0.62 }` | — | — | — | The fog's colour looking into the sun (forward scattering). |
| `sunPower` | `number` | `8` | 0 … 256 | — | tunable | How tight the sun's glow in the fog is (higher: smaller). |
| `maxOpacity` | `number` | `1` | 0 … 1 | `ratio` | tunable | The most the fog hides. |
| `sky` | `number` | `1` | 0 … 1 | `ratio` | tunable | How much of the fog covers the sky (0: the sky stays clear). |
| `volumetric` | `boolean` | `false` | — | — | — | Light the fog as a participating medium: within volumetricDistance of the camera the fog (density per metre, thickened below height by heightDensity and falloff, as above) scatters the sun through its shadow cascades, so light shafts fall through windows and between trees and move with the sun, and every point, spot and rect light it is lit by (up to RenderCapabilities.volumetricLights at each sample); it dims what lies behind it by the same amounts. RenderCapabilities.volumetricFog names the method (a froxel grid). Opaque surfaces and the sky are seen through the volume; blended surfaces, particles and sprites take the plain distance and height fog of the same medium. Past volumetricDistance nothing more is added. |
| `scattering` | `number` | `0.8` | 0 … 4 | — | tunable | How much of the light the fog takes from a ray it scatters (its albedo, tinted by color): 0 dims without glowing, 1 sends all of it on, above 1 brightens the shafts for effect. |
| `anisotropy` | `number` | `0.6` | -0.95 … 0.95 | — | tunable | Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light. |
| `volumetricDistance` | `number` | `48` | 4 … 400 | `m` | — | How far from the camera the volume reaches. |

```ogt
resource Fog {
  doc "Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ramp from start to full fog at end; the two apply together, and density defaults to 0.02, so setting only end still adds exponential fog (set density 0 for the ramp alone). Height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged."
  field enabled bool "Draw the fog."
  field color Color "The colour the fog tends to (linear, before exposure)." {
    default 0.62 0.68 0.76
  }
  field density f32 "Exponential distance fog per metre past start, with or without end's ramp (0: none)." {
    default 0.02
    min 0
    max 2
    tunable true
  }
  field start f32 "The distance from the camera where distance fog starts." {
    min 0
    unit m
    tunable true
  }
  field end f32 "The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp)." {
    min 0
    unit m
    tunable true
  }
  field heightDensity f32 "Height fog at and below height (0: none)." {
    min 0
    max 10
    tunable true
  }
  field height f32 "The world height below which height fog is densest." {
    worldspace world y
    unit m
    tunable true
  }
  field falloff f32 "How fast height fog thins above height, per metre." {
    default 0.5
    min 0.0001
    max 20
    tunable true
  }
  field sunColor Color "The fog's colour looking into the sun (forward scattering)." {
    default 1 0.86 0.62
  }
  field sunPower f32 "How tight the sun's glow in the fog is (higher: smaller)." {
    default 8
    min 0
    max 256
    tunable true
  }
  field maxOpacity f32 "The most the fog hides." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field sky f32 "How much of the fog covers the sky (0: the sky stays clear)." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field volumetric bool "Light the fog as a participating medium: within volumetricDistance of the camera the fog (density per metre, thickened below height by heightDensity and falloff, as above) scatters the sun through its shadow cascades, so light shafts fall through windows and between trees and move with the sun, and every point, spot and rect light it is lit by (up to RenderCapabilities.volumetricLights at each sample); it dims what lies behind it by the same amounts. RenderCapabilities.volumetricFog names the method (a froxel grid). Opaque surfaces and the sky are seen through the volume; blended surfaces, particles and sprites take the plain distance and height fog of the same medium. Past volumetricDistance nothing more is added."
  field scattering f32 "How much of the light the fog takes from a ray it scatters (its albedo, tinted by color): 0 dims without glowing, 1 sends all of it on, above 1 brightens the shafts for effect." {
    default 0.8
    min 0
    max 4
    tunable true
  }
  field anisotropy f32 "Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light." {
    default 0.6
    min -0.95
    max 0.95
    tunable true
  }
  field volumetricDistance f32 "How far from the camera the volume reaches." {
    default 48
    min 4
    max 400
    unit m
  }
}
```

<sub>Module `atmosphere` line 11</sub>
