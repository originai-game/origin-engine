<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Fog"></a>

### `og.Fog` (resource)

Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ramp from start to full fog at end; the two apply together, and density defaults to 0.02, so setting only end still adds exponential fog (set density 0 for the ramp alone). Height fog is densest below height and thins by falloff per metre above it. Plain fog turns towards sunColor looking into the sun; volumetric fog scatters the actual lights with anisotropy's phase. The overlay is never fogged.

See `render §7`.

Token `og.Fog`, registered under the schema name `og.Fog`; its value type is `Fog`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the fog. |
| `color` | `Color` | `{ r: 0.62, g: 0.68, b: 0.76 }` | — | — | — | The colour the fog tends to (linear, before exposure). |
| `density` | `number` | `0.02` | 0 … 2 | — | tunable | Exponential distance fog per metre past start, with or without end's ramp (0: none). |
| `start` | `number` | `0` | ≥ 0 | `m` | tunable | The distance from the camera where distance fog starts, also with volumetric on. Height fog and FogVolume are independent of start. |
| `end` | `number` | `0` | ≥ 0 | `m` | tunable | The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp). |
| `heightDensity` | `number` | `0` | 0 … 10 | — | tunable | Height fog at and below height (0: none). |
| `height` | `number` | `0` | — | `m` | tunable, worldspace `world y` | The world height below which height fog is densest. |
| `falloff` | `number` | `0.5` | 0.0001 … 20 | — | tunable | How fast height fog thins above height, per metre. |
| `sunColor` | `Color` | `{ r: 1, g: 0.86, b: 0.62 }` | — | — | — | Plain fog's colour looking into the sun. Ignored by volumetric fog, which scatters the actual lights' colours; anisotropy controls its angular spread. |
| `sunPower` | `number` | `8` | 0 … 256 | — | tunable | How tight plain fog's sun-colour glow is (higher: smaller). Ignored by volumetric fog: use anisotropy for its Henyey-Greenstein angular spread. |
| `maxOpacity` | `number` | `1` | 0 … 1 | `ratio` | tunable | The most the global distance and height fog together hide, also with volumetric on. Does not limit independent FogVolume media. |
| `sky` | `number` | `1` | 0 … 1 | `ratio` | tunable | How much fog covers the sky and clouds (0: clear). Below a cumulus base, or without cumulus, the lower sky stands for distant land and takes land fog. Above a cumulus base it is cloud sea and keeps this share. |
| `volumetric` | `boolean` | `false` | — | — | — | Light the fog as a medium: unlit, it tends to color as the plain fog does; over that it (density, thickened below height) scatters the sun through its cascades and the point, spot and rect lights (up to RenderCapabilities.volumetricLights a sample). Blended surfaces and sprites take the plain fog. See `render §7`. |
| `scattering` | `number` | `0.8` | 0 … 4 | — | tunable | How much of the sun's and the lamps' light the fog scatters (its albedo, tinted by color): 0 leaves the plain fog's haze without shafts or glow, 1 sends all of that light on, above 1 brightens the shafts for effect. |
| `anisotropy` | `number` | `0.6` | -0.95 … 0.95 | — | tunable | Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light. |
| `volumetricDistance` | `number` | `48` | 4 … 400 | `m` | — | How far from the camera the fog shows shafts and the local lights; past it the fog goes on in its colour, with the sun's light in it unshadowed. |

```ogt
resource Fog {
  doc "Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ramp from start to full fog at end; the two apply together, and density defaults to 0.02, so setting only end still adds exponential fog (set density 0 for the ramp alone). Height fog is densest below height and thins by falloff per metre above it. Plain fog turns towards sunColor looking into the sun; volumetric fog scatters the actual lights with anisotropy's phase. The overlay is never fogged."
  guide "render §7"
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
  field start f32 "The distance from the camera where distance fog starts, also with volumetric on. Height fog and FogVolume are independent of start." {
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
  field sunColor Color "Plain fog's colour looking into the sun. Ignored by volumetric fog, which scatters the actual lights' colours; anisotropy controls its angular spread." {
    default 1 0.86 0.62
  }
  field sunPower f32 "How tight plain fog's sun-colour glow is (higher: smaller). Ignored by volumetric fog: use anisotropy for its Henyey-Greenstein angular spread." {
    default 8
    min 0
    max 256
    tunable true
  }
  field maxOpacity f32 "The most the global distance and height fog together hide, also with volumetric on. Does not limit independent FogVolume media." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field sky f32 "How much fog covers the sky and clouds (0: clear). Below a cumulus base, or without cumulus, the lower sky stands for distant land and takes land fog. Above a cumulus base it is cloud sea and keeps this share." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field volumetric bool "Light the fog as a medium: unlit, it tends to color as the plain fog does; over that it (density, thickened below height) scatters the sun through its cascades and the point, spot and rect lights (up to RenderCapabilities.volumetricLights a sample). Blended surfaces and sprites take the plain fog." {
    guide "render §7"
  }
  field scattering f32 "How much of the sun's and the lamps' light the fog scatters (its albedo, tinted by color): 0 leaves the plain fog's haze without shafts or glow, 1 sends all of that light on, above 1 brightens the shafts for effect." {
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
  field volumetricDistance f32 "How far from the camera the fog shows shafts and the local lights; past it the fog goes on in its colour, with the sun's light in it unshadowed." {
    default 48
    min 4
    max 400
    unit m
  }
}
```

<sub>Module `atmosphere` line 15</sub>
