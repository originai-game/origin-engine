<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-DayNight"></a>

### `og.DayNight` (resource)

A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and moon, the DirectionalLight drawn (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the sky, its LUT and the light share that frame's time. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given; the probe's `get` answers what is drawn.

See `render §7`.

Token `og.DayNight`, registered under the schema name `og.DayNight`; its value type is `DayNight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Run the cycle. |
| `start` | `number` | `9` | 0 … 24 | — | tunable | The time of day at tick 0, in hours (6 sunrise, 12 noon, 18 sunset). An f64, as og.atmosphere.timeOfDay answers, so an hour written here reads back as written. |
| `speed` | `number` | `0.05` | 0 … 24 | — | tunable | Game hours per simulated second (0.05: a day in 8 minutes; 0 holds the time). |
| `tilt` | `number` | `25` | 0 … 89 | `deg` | tunable | How far the sun's path leans from overhead. Not used while an enabled og.Season sets the sun's path. |
| `azimuth` | `number` | `90` | — | `deg` | tunable | The heading the sun rises at: about +Y from +Z towards +X, so 0 rises in +Z and 90 in +X. Its path leans by tilt towards the heading azimuth + 90. |
| `lightIntensity` | `number` | `1` | 0 … 10 | — | tunable | A multiplier on the sun and moon light. |
| `tintFog` | `boolean` | `true` | — | — | — | Turn the og.Fog colour towards the horizon's. |
| `moonSize` | `number` | `1.6` | 0 … 20 | `deg` | tunable | The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon). |
| `moonPhase` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter. The moonlight follows it: a quarter moon gives about 0.38 of the full moon's light, a new moon none (Jensen et al. 2001). |

```ogt
resource DayNight {
  doc "A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and moon, the DirectionalLight drawn (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the sky, its LUT and the light share that frame's time. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given; the probe's `get` answers what is drawn."
  guide "render §7"
  field enabled bool "Run the cycle."
  field start f64 "The time of day at tick 0, in hours (6 sunrise, 12 noon, 18 sunset). An f64, as og.atmosphere.timeOfDay answers, so an hour written here reads back as written." {
    default 9
    min 0
    max 24
    tunable true
  }
  field speed f64 "Game hours per simulated second (0.05: a day in 8 minutes; 0 holds the time)." {
    default 0.05
    min 0
    max 24
    tunable true
  }
  field tilt f32 "How far the sun's path leans from overhead. Not used while an enabled og.Season sets the sun's path." {
    default 25
    min 0
    max 89
    unit deg
    tunable true
  }
  field azimuth f32 "The heading the sun rises at: about +Y from +Z towards +X, so 0 rises in +Z and 90 in +X. Its path leans by tilt towards the heading azimuth + 90." {
    default 90
    unit deg
    tunable true
  }
  field lightIntensity f32 "A multiplier on the sun and moon light." {
    default 1
    min 0
    max 10
    tunable true
  }
  field tintFog bool "Turn the og.Fog colour towards the horizon's." {
    default true
  }
  field moonSize f32 "The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon)." {
    default 1.6
    min 0
    max 20
    unit deg
    tunable true
  }
  field moonPhase f32 "How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter. The moonlight follows it: a quarter moon gives about 0.38 of the full moon's light, a new moon none (Jensen et al. 2001)." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
}
```

<sub>Module `atmosphere` line 196</sub>
