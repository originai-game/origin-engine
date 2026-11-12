<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Weather"></a>

### `og.Weather` (resource)

A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation.

Token `og.Weather`, registered under the schema name `og.Weather`; its value type is `Weather`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | 0 … 4 | — | label “Kind” | WEATHER_NONE 0, WEATHER_RAIN 1, WEATHER_SNOW 2, WEATHER_DUST 3, WEATHER_MIST 4. |
| `intensity` | `number` | `1` | 0 … 2 | — | tunable | How heavy: scales the particle rate, the fog and the dimming. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | A wind of the weather's own on the particles, added to og.Wind's. |
| `radius` | `number` | `18` | 1 … 200 | `m` | tunable | The size of the region around the camera the particles fill. |
| `tint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Multiplies the particles' colour. |
| `dim` | `number` | `1` | 0 … 1 | `ratio` | tunable | How much of the preset's dimming of the sunlight applies. |
| `layer` | `number` | `50` | — | — | — | With an orthographic camera (2D), the sprite layer the particles draw in. |

```ogt
resource Weather {
  doc "A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation."
  field kind u32 "WEATHER_NONE 0, WEATHER_RAIN 1, WEATHER_SNOW 2, WEATHER_DUST 3, WEATHER_MIST 4." {
    min 0
    max 4
    label "Kind"
  }
  field intensity f32 "How heavy: scales the particle rate, the fog and the dimming." {
    default 1
    min 0
    max 2
    tunable true
  }
  field wind Vector3 "A wind of the weather's own on the particles, added to og.Wind's." {
    unit mps
    tunable true
  }
  field radius f32 "The size of the region around the camera the particles fill." {
    default 18
    min 1
    max 200
    unit m
    tunable true
  }
  field tint Color "Multiplies the particles' colour." {
    default 1 1 1
  }
  field dim f32 "How much of the preset's dimming of the sunlight applies." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layer i32 "With an orthographic camera (2D), the sprite layer the particles draw in." {
    default 50
  }
}
```

<sub>Module `atmosphere` line 531</sub>
