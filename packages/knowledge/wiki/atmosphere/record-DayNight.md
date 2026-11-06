<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-DayNight"></a>

### `og.DayNight` (resource)

A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the sky's colours step every `step` minutes of game time while its sun moves smoothly, and the image-based light and the reflections' sky follow the sky as it is drawn (clouds, lightning and all), captured a slice a frame. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`.

Token `og.DayNight`, registered under the schema name `og.DayNight`; its value type is `DayNight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Run the cycle. |
| `start` | `number` | `9` | 0 … 24 | — | tunable | The time of day at tick 0, in hours (6 sunrise, 12 noon, 18 sunset). |
| `speed` | `number` | `0.05` | 0 … 24 | — | tunable | Game hours per simulated second (0.05: a day in 8 minutes; 0 holds the time). |
| `tilt` | `number` | `25` | 0 … 89 | `deg` | tunable | How far the sun's path leans from overhead. |
| `azimuth` | `number` | `0` | — | `deg` | tunable | Where the sun rises, turned about +y from +x. |
| `lightIntensity` | `number` | `1` | 0 … 10 | — | tunable | A multiplier on the sun and moon light. |
| `tintFog` | `boolean` | `true` | — | — | — | Turn the og.Fog colour towards the horizon's. |
| `step` | `number` | `10` | 1 … 120 | — | tunable | Game minutes between steps of the sky's colours (the image-based light follows the sky as drawn). |
| `moonSize` | `number` | `1.6` | 0 … 20 | `deg` | tunable | The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon). |
| `moonPhase` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter. |

```ogt
resource DayNight {
  doc "A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the sky's colours step every `step` minutes of game time while its sun moves smoothly, and the image-based light and the reflections' sky follow the sky as it is drawn (clouds, lightning and all), captured a slice a frame. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`."
  field enabled bool "Run the cycle."
  field start f32 "The time of day at tick 0, in hours (6 sunrise, 12 noon, 18 sunset)." {
    default 9
    min 0
    max 24
    tunable true
  }
  field speed f32 "Game hours per simulated second (0.05: a day in 8 minutes; 0 holds the time)." {
    default 0.05
    min 0
    max 24
    tunable true
  }
  field tilt f32 "How far the sun's path leans from overhead." {
    default 25
    min 0
    max 89
    unit deg
    tunable true
  }
  field azimuth f32 "Where the sun rises, turned about +y from +x." {
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
  field step f32 "Game minutes between steps of the sky's colours (the image-based light follows the sky as drawn)." {
    default 10
    min 1
    max 120
    tunable true
  }
  field moonSize f32 "The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon)." {
    default 1.6
    min 0
    max 20
    unit deg
    tunable true
  }
  field moonPhase f32 "How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
}
```

<sub>`api.ogt` line 10749</sub>
