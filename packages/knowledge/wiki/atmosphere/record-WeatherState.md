<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-WeatherState"></a>

### `og.WeatherState` (resource)

The weather's effect on the world, simulated in the tick from og.Weather so gameplay can read it: how wet the ground is, how much standing water, snow and dust lie on it, and the lightning strikes, drawn from a stream forked from og.Rng (`fork("og.weather")`) and kept here, so replays, restores and every host strike alike. Registered the first time a game names it: a world that never does keeps its type table and hash. The renderer draws the same values on every lit surface (StandardMaterialDesc.weathering opts a material out); rendering never writes it.

See `render §7`.

Token `og.WeatherState`, registered under the schema name `og.WeatherState`; its value type is `WeatherState`.

#### Wetness

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `soakTime` | `number` | `60` | ≥ 0.1 | `s` | tunable | Seconds of rain at intensity 1 to soak dry ground (wetness 0 to 1). |
| `dryTime` | `number` | `300` | ≥ 0.1 | `s` | tunable | Seconds for soaked ground to dry once the rain stops. |
| `puddleTime` | `number` | `120` | ≥ 0.1 | `s` | tunable | Seconds of rain at intensity 1 on soaked ground to fill the puddles (puddles 0 to 1). |
| `drainTime` | `number` | `600` | ≥ 0.1 | `s` | tunable | Seconds for full puddles to drain once the rain stops. |

#### Snow

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `snowfall` | `number` | `0.01` | ≥ 0 | `m` | tunable | How much snow settles per minute of snowfall at intensity 1. |
| `melt` | `number` | `0.004` | ≥ 0 | `m` | tunable | How much snow melts per minute when none falls (three times as fast in rain). |
| `snowCover` | `number` | `0.08` | ≥ 0.001 | `m` | tunable | The snow depth at which it covers everything that can hold it (the renderer's full cover). |

#### Dust

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dustTime` | `number` | `600` | ≥ 0.1 | `s` | tunable | Seconds of a dust storm at intensity 1 to cover what faces up with dust and sand (dust 0 to 1). It stays when the storm ends; rain washes it off. |
| `washTime` | `number` | `90` | ≥ 0.1 | `s` | tunable | Seconds of rain at intensity 1 to wash full dust off. |
| `dustColor` | `Color` | `{ r: 0.62, g: 0.5, b: 0.36 }` | — | — | — | The colour of the settled dust and sand (linear): pale ochre by default; a red desert's or a grey quarry's. |

#### Lightning

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lightning` | `number` | `0` | 0 … 600 | — | tunable | Lightning strikes per minute on average, at random ticks (0: none), whatever the kind of weather. |
| `lightningCenter` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | The centre of the area strikes land in (its y is the ground's height there). |
| `lightningRadius` | `number` | `300` | ≥ 0 | `m` | tunable | The radius of that area: strikes land evenly over the disc. |

#### Shelter

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shelterSize` | `number` | `64` | 0 … 1024 | `m` | tunable | The side of the square around the camera in which roofs, overhangs and canopies keep what is under them dry and free of snow, with a soft edge at the drip line; past it every surface is open to the sky. 0: nothing shelters. Drawn only: the tick's wetness, puddles and snowDepth are the open ground's. See `render §7`. |

#### State

Written by og.weather; read them, never write them.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wetness` | `number` | `0` | — | `ratio` | read only | How wet the ground is, 0 dry to 1 soaked (written by og.weather). |
| `puddles` | `number` | `0` | — | `ratio` | read only | How far standing water has spread, 0 none to 1 (written by og.weather). |
| `dust` | `number` | `0` | — | `ratio` | read only | How much dust and sand has settled, 0 none to 1 covering what faces up (written by og.weather). |
| `snowDepth` | `number` | `0` | — | `m` | read only | The depth of lying snow (written by og.weather). The drawn terrain rises by it on level ground, up to 10 cm, once the cover has built up; only drawn: collision, og.terrain.sample, picking and what stands on the ground keep the terrain's own height. |
| `strikes` | `number` | `0` | — | — | read only | Lightning strikes so far (written by og.weather). |
| `lastStrike` | `number` | `0` | — | — | read only | The tick of the latest strike (0 before the first; written by og.weather). |
| `strikePosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | Where the latest strike landed (written by og.weather). |
| `stream` | `number` | `0` | — | — | read only | The lightning's random state, forked from og.Rng the first tick (written by og.weather). |

```ogt
resource WeatherState {
  doc "The weather's effect on the world, simulated in the tick from og.Weather so gameplay can read it: how wet the ground is, how much standing water, snow and dust lie on it, and the lightning strikes, drawn from a stream forked from og.Rng (`fork(\"og.weather\")`) and kept here, so replays, restores and every host strike alike. Registered the first time a game names it: a world that never does keeps its type table and hash. The renderer draws the same values on every lit surface (StandardMaterialDesc.weathering opts a material out); rendering never writes it."
  guide "render §7"
  group "Wetness"
  field soakTime f32 "Seconds of rain at intensity 1 to soak dry ground (wetness 0 to 1)." {
    default 60
    min 0.1
    unit s
    tunable true
  }
  field dryTime f32 "Seconds for soaked ground to dry once the rain stops." {
    default 300
    min 0.1
    unit s
    tunable true
  }
  field puddleTime f32 "Seconds of rain at intensity 1 on soaked ground to fill the puddles (puddles 0 to 1)." {
    default 120
    min 0.1
    unit s
    tunable true
  }
  field drainTime f32 "Seconds for full puddles to drain once the rain stops." {
    default 600
    min 0.1
    unit s
    tunable true
  }
  group "Snow"
  field snowfall f32 "How much snow settles per minute of snowfall at intensity 1." {
    default 0.01
    min 0
    unit m
    tunable true
  }
  field melt f32 "How much snow melts per minute when none falls (three times as fast in rain)." {
    default 0.004
    min 0
    unit m
    tunable true
  }
  field snowCover f32 "The snow depth at which it covers everything that can hold it (the renderer's full cover)." {
    default 0.08
    min 0.001
    unit m
    tunable true
  }
  group "Dust"
  field dustTime f32 "Seconds of a dust storm at intensity 1 to cover what faces up with dust and sand (dust 0 to 1). It stays when the storm ends; rain washes it off." {
    default 600
    min 0.1
    unit s
    tunable true
  }
  field washTime f32 "Seconds of rain at intensity 1 to wash full dust off." {
    default 90
    min 0.1
    unit s
    tunable true
  }
  field dustColor Color "The colour of the settled dust and sand (linear): pale ochre by default; a red desert's or a grey quarry's." {
    default 0.62 0.5 0.36
  }
  group "Lightning"
  field lightning f32 "Lightning strikes per minute on average, at random ticks (0: none), whatever the kind of weather." {
    min 0
    max 600
    tunable true
  }
  field lightningCenter Vector3 "The centre of the area strikes land in (its y is the ground's height there)." {
    worldspace world
    unit m
  }
  field lightningRadius f32 "The radius of that area: strikes land evenly over the disc." {
    default 300
    min 0
    unit m
    tunable true
  }
  group "Shelter"
  field shelterSize f32 "The side of the square around the camera in which roofs, overhangs and canopies keep what is under them dry and free of snow, with a soft edge at the drip line; past it every surface is open to the sky. 0: nothing shelters. Drawn only: the tick's wetness, puddles and snowDepth are the open ground's." {
    guide "render §7"
    default 64
    min 0
    max 1024
    unit m
    tunable true
  }
  group "State" "Written by og.weather; read them, never write them."
  field wetness f32 "How wet the ground is, 0 dry to 1 soaked (written by og.weather)." {
    readonly true
    unit ratio
  }
  field puddles f32 "How far standing water has spread, 0 none to 1 (written by og.weather)." {
    readonly true
    unit ratio
  }
  field dust f32 "How much dust and sand has settled, 0 none to 1 covering what faces up (written by og.weather)." {
    readonly true
    unit ratio
  }
  field snowDepth f32 "The depth of lying snow (written by og.weather). The drawn terrain rises by it on level ground, up to 10 cm, once the cover has built up; only drawn: collision, og.terrain.sample, picking and what stands on the ground keep the terrain's own height." {
    readonly true
    unit m
  }
  field strikes u32 "Lightning strikes so far (written by og.weather)." {
    readonly true
  }
  field lastStrike u32 "The tick of the latest strike (0 before the first; written by og.weather)." {
    readonly true
  }
  field strikePosition Vector3 "Where the latest strike landed (written by og.weather)." {
    worldspace world
    readonly true
    unit m
  }
  field stream u32 "The lightning's random state, forked from og.Rng the first tick (written by og.weather)." {
    readonly true
  }
}
```

<sub>Module `atmosphere` line 422</sub>
