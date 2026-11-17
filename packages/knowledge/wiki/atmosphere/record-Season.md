<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Season"></a>

### `og.Season` (resource)

The time of year: a calendar date, a latitude and a climate. With og.DayNight it sets where the sun rises, how high it climbs and how long the day is; og.atmosphere.season gives the date, the season, the temperature and how far the leaves and the grass are through their year. The date is start plus the simulated seconds times speed, wrapped to 365 days, the same on every run and after a restore. Registered the first time a game names it: a world that never does keeps its type table and hash.

See `seasons §1`.

Token `og.Season`, registered under the schema name `og.Season`; its value type is `Season`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Run the year. |

#### Calendar

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `start` | `number` | `172` | 0 … 365 | — | tunable | The day of the year at tick 0, counted from New Year's midnight: SEASON_SPRING 79, SEASON_SUMMER 172, SEASON_AUTUMN 265 and SEASON_WINTER 355 are the northern equinoxes and solstices. |
| `speed` | `number` | `0` | 0 … 365 | — | tunable | Days per simulated second (0 holds the date; 0.05: a season in half an hour). |
| `latitude` | `number` | `45` | -90 … 90 | `deg` | tunable | Degrees north of the equator. South of it (negative) the same date is the opposite season and the noon sun stands in the north. |
| `change` | `number` | `0` | ≥ 0 | `s` | tunable | Seconds a jump of one season (a quarter of the year) takes to play when start or speed changes: the date drawn and read runs forward through the days between, or back for a jump of under 30 days. 0 cuts to the new date. See `seasons §2`. |

#### Sun

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sun` | `boolean` | `true` | — | — | — | With og.DayNight enabled, the sun rises, climbs and sets where and when it does on this date at this latitude: DayNight.tilt is not used, its azimuth stays the heading of due east, and its hours are solar time (12 the sun's highest). False keeps DayNight's own path. See `seasons §3`. |

#### Climate

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `temperature` | `number` | `10` | -60 … 60 | — | tunable | The year's mean air temperature, in °C. |
| `temperatureRange` | `number` | `20` | 0 … 80 | — | tunable | The warmest month's mean temperature less the coldest's, in °C; the warmest day comes 30 days after the summer solstice. 0: the same all year. |
| `dayRange` | `number` | `8` | 0 … 40 | — | tunable | The afternoon's temperature (at 15 h) less the dawn's, in °C. |

#### State

Written by og.season; read them, never write them.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `day` | `number` | `-1` | — | — | read only | The calendar's date as og.season last followed it, 0 to 365: on its way there while a jump plays over `change`; negative before the first tick and while the year is off. og.atmosphere.season's `day` is the date drawn and read (written by og.season). |

```ogt
resource Season {
  doc "The time of year: a calendar date, a latitude and a climate. With og.DayNight it sets where the sun rises, how high it climbs and how long the day is; og.atmosphere.season gives the date, the season, the temperature and how far the leaves and the grass are through their year. The date is start plus the simulated seconds times speed, wrapped to 365 days, the same on every run and after a restore. Registered the first time a game names it: a world that never does keeps its type table and hash."
  guide "seasons §1"
  field enabled bool "Run the year."
  group "Calendar"
  field start f32 "The day of the year at tick 0, counted from New Year's midnight: SEASON_SPRING 79, SEASON_SUMMER 172, SEASON_AUTUMN 265 and SEASON_WINTER 355 are the northern equinoxes and solstices." {
    default 172
    min 0
    max 365
    tunable true
  }
  field speed f32 "Days per simulated second (0 holds the date; 0.05: a season in half an hour)." {
    min 0
    max 365
    tunable true
  }
  field latitude f32 "Degrees north of the equator. South of it (negative) the same date is the opposite season and the noon sun stands in the north." {
    default 45
    min -90
    max 90
    unit deg
    tunable true
  }
  field change f32 "Seconds a jump of one season (a quarter of the year) takes to play when start or speed changes: the date drawn and read runs forward through the days between, or back for a jump of under 30 days. 0 cuts to the new date." {
    guide "seasons §2"
    min 0
    unit s
    tunable true
  }
  group "Sun"
  field sun bool "With og.DayNight enabled, the sun rises, climbs and sets where and when it does on this date at this latitude: DayNight.tilt is not used, its azimuth stays the heading of due east, and its hours are solar time (12 the sun's highest). False keeps DayNight's own path." {
    guide "seasons §3"
    default true
  }
  group "Climate"
  field temperature f32 "The year's mean air temperature, in °C." {
    default 10
    min -60
    max 60
    tunable true
  }
  field temperatureRange f32 "The warmest month's mean temperature less the coldest's, in °C; the warmest day comes 30 days after the summer solstice. 0: the same all year." {
    default 20
    min 0
    max 80
    tunable true
  }
  field dayRange f32 "The afternoon's temperature (at 15 h) less the dawn's, in °C." {
    default 8
    min 0
    max 40
    tunable true
  }
  group "State" "Written by og.season; read them, never write them."
  field day f32 "The calendar's date as og.season last followed it, 0 to 365: on its way there while a jump plays over `change`; negative before the first tick and while the year is off. og.atmosphere.season's `day` is the date drawn and read (written by og.season)." {
    readonly true
    default -1
  }
}
```

<sub>Module `atmosphere` line 254</sub>
