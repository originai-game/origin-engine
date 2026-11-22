<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="type-SeasonInfo"></a>

### `SeasonInfo` (struct)

The time of year now (og.atmosphere.season): the date, the sun's day and the climate's facts for it, each a function of og.Season's fields and the tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `day` | `number` | `0` | — | — | — | The date drawn and read: days from New Year's midnight, 0 to 365 (og.Season.day: the calendar's date, or on its way there after a jump). |
| `season` | `number` | `0` | — | — | — | The local season, from 0 up to 4 and never 4: its whole part is the season (0 spring, 1 summer, 2 autumn, 3 winter, each from its equinox or solstice, in either hemisphere), its fraction how far through it. |
| `declination` | `number` | `0` | — | `deg` | — | How far north of the equator the sun stands overhead: +23.4 at the northern summer solstice, 0 at the equinoxes. |
| `sunrise` | `number` | `0` | — | — | — | The hour of solar time the sun rises, as og.atmosphere.timeOfDay counts hours (0 where it does not set that day, 12 where it does not rise). |
| `sunset` | `number` | `0` | — | — | — | The hour of solar time the sun sets (24 where it does not set that day, 12 where it does not rise). |
| `dayLength` | `number` | `0` | — | — | — | Hours from sunrise to sunset, 0 to 24. |
| `temperature` | `number` | `0` | — | — | — | The air's temperature now, in °C: the season's with the day's swing at og.DayNight's hour (the day's mean without an enabled og.DayNight). |
| `leafOut` | `number` | `0` | — | `ratio` | — | How far deciduous leaves have opened: 0 at the bud, 1 in full leaf and through autumn until the next bud. |
| `leafTurn` | `number` | `0` | — | `ratio` | — | How far they have turned to their autumn colour. |
| `leafFall` | `number` | `0` | — | `ratio` | — | The share of them that has fallen: a tree carries leafOut times (1 - leafFall) of its leaves. |
| `grassGrowth` | `number` | `0` | — | `ratio` | — | How far the grass has grown, 0 dormant in the cold to 1 full grown. |

```ogt
struct SeasonInfo {
  doc "The time of year now (og.atmosphere.season): the date, the sun's day and the climate's facts for it, each a function of og.Season's fields and the tick."
  field day f32 "The date drawn and read: days from New Year's midnight, 0 to 365 (og.Season.day: the calendar's date, or on its way there after a jump)."
  field season f32 "The local season, from 0 up to 4 and never 4: its whole part is the season (0 spring, 1 summer, 2 autumn, 3 winter, each from its equinox or solstice, in either hemisphere), its fraction how far through it."
  field declination f32 "How far north of the equator the sun stands overhead: +23.4 at the northern summer solstice, 0 at the equinoxes." {
    unit deg
  }
  field sunrise f32 "The hour of solar time the sun rises, as og.atmosphere.timeOfDay counts hours (0 where it does not set that day, 12 where it does not rise)."
  field sunset f32 "The hour of solar time the sun sets (24 where it does not set that day, 12 where it does not rise)."
  field dayLength f32 "Hours from sunrise to sunset, 0 to 24."
  field temperature f32 "The air's temperature now, in °C: the season's with the day's swing at og.DayNight's hour (the day's mean without an enabled og.DayNight)."
  field leafOut f32 "How far deciduous leaves have opened: 0 at the bud, 1 in full leaf and through autumn until the next bud." {
    unit ratio
  }
  field leafTurn f32 "How far they have turned to their autumn colour." {
    unit ratio
  }
  field leafFall f32 "The share of them that has fallen: a tree carries leafOut times (1 - leafFall) of its leaves." {
    unit ratio
  }
  field grassGrowth f32 "How far the grass has grown, 0 dormant in the cold to 1 full grown." {
    unit ratio
  }
}
```

<sub>Module `atmosphere` line 769</sub>
