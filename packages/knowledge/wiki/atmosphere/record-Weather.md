<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Weather"></a>

### `og.Weather` (resource)

A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence), the cloud cover rain and snow fall from (og.Clouds closed to it, or a deck of the weather's own under a procedural sky) and the light under it, dimmed once. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation.

Token `og.Weather`, registered under the schema name `og.Weather`; its value type is `Weather`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | 0 … 4 | — | label “Kind” | WEATHER_NONE 0, WEATHER_RAIN 1, WEATHER_SNOW 2, WEATHER_DUST 3, WEATHER_MIST 4. |
| `intensity` | `number` | `1` | 0 … 2 | — | tunable | How heavy: the share of the particles born, the fog, the cover and the dimming. The weather drawn follows it over `change`, keeping the particles in the air. |
| `change` | `number` | `20` | 0 … 600 | `s` | tunable | How long the weather takes to follow a change of kind or intensity: the drawn intensity moves by 1 in this many seconds, a new kind building as the one before thins out. The first tick takes the weather as it is; 0 switches at once. |
| `drawnKind` | `number` | `0` | — | — | read only | The kind drawn now, which follows `kind` (written by the tick). |
| `drawnIntensity` | `number` | `-1` | — | — | read only | The intensity drawn now, which follows `intensity` over `change`; -1 before the first tick (written by the tick). |
| `fadingKind` | `number` | `0` | — | — | read only | The kind thinning out after a switch, 0 when none is (written by the tick). |
| `fadingIntensity` | `number` | `0` | — | — | read only | How much of fadingKind is still drawn (written by the tick). |
| `settling` | `number` | `0` | — | `s` | read only | Seconds the faded kind's last particles still fall once fadingIntensity reaches 0 (written by the tick). |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | A wind of the weather's own on the particles, added to og.Wind's. |
| `radius` | `number` | `18` | 1 … 200 | `m` | tunable | The size of the region around the camera the particles fill. Rain's streaks fill at most its nearest 10 m: farther drops are thinner than a pixel, and its fog draws them. |
| `tint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Multiplies the particles' colour. |
| `dim` | `number` | `1` | 0 … 1 | `ratio` | tunable | How much of the preset's cloud cover and dimming of the sunlight applies. |
| `layer` | `number` | `50` | — | — | — | With an orthographic camera (2D), the sprite layer the particles draw in. |

```ogt
resource Weather {
  doc "A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence), the cloud cover rain and snow fall from (og.Clouds closed to it, or a deck of the weather's own under a procedural sky) and the light under it, dimmed once. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation."
  field kind u32 "WEATHER_NONE 0, WEATHER_RAIN 1, WEATHER_SNOW 2, WEATHER_DUST 3, WEATHER_MIST 4." {
    min 0
    max 4
    label "Kind"
  }
  field intensity f32 "How heavy: the share of the particles born, the fog, the cover and the dimming. The weather drawn follows it over `change`, keeping the particles in the air." {
    default 1
    min 0
    max 2
    tunable true
  }
  field change f32 "How long the weather takes to follow a change of kind or intensity: the drawn intensity moves by 1 in this many seconds, a new kind building as the one before thins out. The first tick takes the weather as it is; 0 switches at once." {
    default 20
    min 0
    max 600
    unit s
    tunable true
  }
  field drawnKind u32 "The kind drawn now, which follows `kind` (written by the tick)." {
    readonly true
  }
  field drawnIntensity f32 "The intensity drawn now, which follows `intensity` over `change`; -1 before the first tick (written by the tick)." {
    default -1
    readonly true
  }
  field fadingKind u32 "The kind thinning out after a switch, 0 when none is (written by the tick)." {
    readonly true
  }
  field fadingIntensity f32 "How much of fadingKind is still drawn (written by the tick)." {
    readonly true
  }
  field settling f32 "Seconds the faded kind's last particles still fall once fadingIntensity reaches 0 (written by the tick)." {
    readonly true
    unit s
  }
  field wind Vector3 "A wind of the weather's own on the particles, added to og.Wind's." {
    unit mps
    tunable true
  }
  field radius f32 "The size of the region around the camera the particles fill. Rain's streaks fill at most its nearest 10 m: farther drops are thinner than a pixel, and its fog draws them." {
    default 18
    min 1
    max 200
    unit m
    tunable true
  }
  field tint Color "Multiplies the particles' colour." {
    default 1 1 1
  }
  field dim f32 "How much of the preset's cloud cover and dimming of the sunlight applies." {
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

<sub>Module `atmosphere` line 629</sub>
