<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Clouds"></a>

### `og.Clouds` (resource)

Clouds in the procedural sky: a volumetric layer of cumulus raymarched through a curved shell over the planet, lit by the first DirectionalLight (shadowed within themselves, with multiple scattering: bright tops, dark bases, silver edges against the sun) and the sky's light, faded into the sky with distance; and a thin, streaked high layer (cirrus) above. The wind carries them with the tick's clock, so they look the same on every run and after a restore. They are drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky; alone they bring the built-in procedural sky with them) and dim the sun and the sky's light by their cover (a broken sky hides the sun a share of the time; an overcast deck hides it from every surface, leaving the deck's flat grey light). Visual only: nothing in the simulation reads them.

Token `og.Clouds`, registered under the schema name `og.Clouds`; its value type is `Clouds`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the clouds. |
| `coverage` | `number` | `0.45` | 0 … 1 | `ratio` | tunable | How much of the sky the cumulus covers: 0 none, 0.3 fair weather, 0.6 broken, 1 overcast. |
| `density` | `number` | `0.04` | 0 … 0.5 | — | tunable | How thick they are per metre (0.04: cumulus; 0.01: thin and translucent; 0.08: dense storm cloud). |
| `base` | `number` | `1500` | ≥ 0 | `m` | tunable | The height of the cloud base above the world's y 0. |
| `thickness` | `number` | `1800` | ≥ 0 | `m` | tunable | How tall the layer is (0: no cumulus, only the high layer). |
| `scale` | `number` | `9000` | 500 … 200000 | `m` | tunable | The size of the largest cloud shapes. |
| `wind` | `Vector3` | `{ x: 12, y: 0.4, z: 4 }` | — | `mps` | tunable | How fast the wind carries them (x and z; y slowly turns their shapes over). |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Their albedo: white for fair-weather clouds, a grey for rain clouds. |
| `detail` | `number` | `0.35` | 0 … 1 | `ratio` | tunable | How much fine noise erodes their edges (0: smooth blobs, 1: wispy). |
| `layer` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | The high thin layer's cover (0: none). |
| `layerAltitude` | `number` | `8000` | ≥ 0 | `m` | tunable | The high layer's height (kept above the cumulus). |
| `steps` | `number` | `48` | 8 … 256 | — | — | Samples along each view ray through the cumulus: more is smoother and costs more. |

```ogt
resource Clouds {
  doc "Clouds in the procedural sky: a volumetric layer of cumulus raymarched through a curved shell over the planet, lit by the first DirectionalLight (shadowed within themselves, with multiple scattering: bright tops, dark bases, silver edges against the sun) and the sky's light, faded into the sky with distance; and a thin, streaked high layer (cirrus) above. The wind carries them with the tick's clock, so they look the same on every run and after a restore. They are drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky; alone they bring the built-in procedural sky with them) and dim the sun and the sky's light by their cover (a broken sky hides the sun a share of the time; an overcast deck hides it from every surface, leaving the deck's flat grey light). Visual only: nothing in the simulation reads them."
  field enabled bool "Draw the clouds."
  field coverage f32 "How much of the sky the cumulus covers: 0 none, 0.3 fair weather, 0.6 broken, 1 overcast." {
    default 0.45
    min 0
    max 1
    unit ratio
    tunable true
  }
  field density f32 "How thick they are per metre (0.04: cumulus; 0.01: thin and translucent; 0.08: dense storm cloud)." {
    default 0.04
    min 0
    max 0.5
    tunable true
  }
  field base f32 "The height of the cloud base above the world's y 0." {
    default 1500
    min 0
    unit m
    tunable true
  }
  field thickness f32 "How tall the layer is (0: no cumulus, only the high layer)." {
    default 1800
    min 0
    unit m
    tunable true
  }
  field scale f32 "The size of the largest cloud shapes." {
    default 9000
    min 500
    max 200000
    unit m
    tunable true
  }
  field wind Vector3 "How fast the wind carries them (x and z; y slowly turns their shapes over)." {
    default 12 0.4 4
    unit mps
    tunable true
  }
  field color Color "Their albedo: white for fair-weather clouds, a grey for rain clouds." {
    default 1 1 1
  }
  field detail f32 "How much fine noise erodes their edges (0: smooth blobs, 1: wispy)." {
    default 0.35
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layer f32 "The high thin layer's cover (0: none)." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layerAltitude f32 "The high layer's height (kept above the cumulus)." {
    default 8000
    min 0
    unit m
    tunable true
  }
  field steps u32 "Samples along each view ray through the cumulus: more is smoother and costs more." {
    default 48
    min 8
    max 256
  }
}
```

<sub>`api.ogt` line 10842</sub>
