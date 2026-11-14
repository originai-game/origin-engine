<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Aurora"></a>

### `og.Aurora` (resource)

The aurora: curtains of light in the upper atmosphere raymarched in the procedural sky (after Lawlor and Genetti 2011), each a sheet of falling electrons along the auroral oval, folded by travelling waves and curled by a Kelvin-Helmholtz instability, glowing by height as the electrons' energy deposits (green oxygen near 114 km, a nitrogen border below, red oxygen above 200 km; Whiter et al. 2023). Emission only, under the clouds; invisible against a daylight sky. It follows the tick's clock. Drawn over the procedural sky. Visual only.

See `render §7`.

Token `og.Aurora`, registered under the schema name `og.Aurora`; its value type is `Aurora`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the aurora. |
| `intensity` | `number` | `1` | 0 … 20 | — | tunable | How bright the display is: 1 a bright display against a night sky, 0.3 a faint one. |
| `activity` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | 0 quiet, smooth arcs; 0.5 folding, rayed curtains; 1 a substorm's breakup, curled and fragmented, moving fast. |
| `curtains` | `number` | `3` | 1 … 6 | — | — | How many arcs, one behind another. |
| `direction` | `number` | `90` | — | `deg` | tunable | Where the arcs lie, turned about +y from +x (the convention of og.DayNight's azimuth). |
| `distance` | `number` | `150000` | 0 … 1500000 | `m` | tunable | How far the first arc is from the world's origin over the ground (0: overhead). |
| `spread` | `number` | `70000` | ≥ 0 | `m` | tunable | The distance between successive arcs. |
| `energy` | `number` | `2` | 0.3 … 20 | — | tunable | The falling electrons' characteristic energy in keV: about 2 for a typical display; under 1 a high, red-topped aurora; over 5 a low one with a pink-violet lower border. |
| `speed` | `number` | `1` | 0 … 20 | — | tunable | How fast the folds travel and the rays drift (1: as observed). |
| `seed` | `number` | `0` | — | — | — | The display's seed: its curtains' shapes and their curls. |
| `greenColor` | `Color` | `{ r: 0.29, g: 1, b: 0 }` | — | — | — | The oxygen green line's colour (557.7 nm): by default its spectral colour, a yellow-green; a camera's photograph renders it bluer (0.1 1 0.55). |
| `redColor` | `Color` | `{ r: 1, g: 0, b: 0.06 }` | — | — | — | The oxygen red line's colour (630.0 nm), of the high tops. |
| `nitrogenColor` | `Color` | `{ r: 0.57, g: 0, b: 1 }` | — | — | — | The nitrogen bands' colour (427.8 nm and the red first positive band), of the lower border: violet to pink. |

```ogt
resource Aurora {
  doc "The aurora: curtains of light in the upper atmosphere raymarched in the procedural sky (after Lawlor and Genetti 2011), each a sheet of falling electrons along the auroral oval, folded by travelling waves and curled by a Kelvin-Helmholtz instability, glowing by height as the electrons' energy deposits (green oxygen near 114 km, a nitrogen border below, red oxygen above 200 km; Whiter et al. 2023). Emission only, under the clouds; invisible against a daylight sky. It follows the tick's clock. Drawn over the procedural sky. Visual only."
  guide "render §7"
  field enabled bool "Draw the aurora."
  field intensity f32 "How bright the display is: 1 a bright display against a night sky, 0.3 a faint one." {
    default 1
    min 0
    max 20
    tunable true
  }
  field activity f32 "0 quiet, smooth arcs; 0.5 folding, rayed curtains; 1 a substorm's breakup, curled and fragmented, moving fast." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field curtains u32 "How many arcs, one behind another." {
    default 3
    min 1
    max 6
  }
  field direction f32 "Where the arcs lie, turned about +y from +x (the convention of og.DayNight's azimuth)." {
    default 90
    unit deg
    tunable true
  }
  field distance f32 "How far the first arc is from the world's origin over the ground (0: overhead)." {
    default 150000
    min 0
    max 1500000
    unit m
    tunable true
  }
  field spread f32 "The distance between successive arcs." {
    default 70000
    min 0
    unit m
    tunable true
  }
  field energy f32 "The falling electrons' characteristic energy in keV: about 2 for a typical display; under 1 a high, red-topped aurora; over 5 a low one with a pink-violet lower border." {
    default 2
    min 0.3
    max 20
    tunable true
  }
  field speed f32 "How fast the folds travel and the rays drift (1: as observed)." {
    default 1
    min 0
    max 20
    tunable true
  }
  field seed u32 "The display's seed: its curtains' shapes and their curls."
  field greenColor Color "The oxygen green line's colour (557.7 nm): by default its spectral colour, a yellow-green; a camera's photograph renders it bluer (0.1 1 0.55)." {
    default 0.29 1 0
  }
  field redColor Color "The oxygen red line's colour (630.0 nm), of the high tops." {
    default 1 0 0.06
  }
  field nitrogenColor Color "The nitrogen bands' colour (427.8 nm and the red first positive band), of the lower border: violet to pink." {
    default 0.57 0 1
  }
}
```

<sub>Module `atmosphere` line 358</sub>
