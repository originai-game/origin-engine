<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `seasons` guide: Seasons

The time of year as one resource: what og.Season's date, latitude and climate decide, how to change the season in play, and how the sun follows the date. `render §7` covers the sky, the clouds and the weather the season sits beside.

The API: `og api show atmosphere`. og.Season is off until a game enables
it, and a world that never names it is unchanged.

<a id="guide-seasons-1"></a>
### `seasons §1` The year

- A date is a day of the year, 0 to 365 from New Year's midnight. It is
  `start` plus the simulated time times `speed`, so a save, a replay and
  every host are on the same day at the same tick.
- Hold the date (`speed: 0`) for a level set in one season. Run it for a
  farm or a survival game: 0.05 days a second is a season in half an hour,
  and a year in two hours. A day of og.DayNight and a day of og.Season are
  separate clocks: a fast day with a slow year is usual, and nothing ties
  365 sunsets to one year.
- `latitude` decides how much the year changes. Near the equator the days
  stay near 12 hours and the sun passes overhead; at 45 to 55 the summer
  day is twice the winter's; past 66.5 the sun does not set in midsummer
  or rise in midwinter. South of the equator the same date is the opposite
  season: `SEASON_SUMMER` at -35 is midwinter.
- The climate (`temperature`, `temperatureRange`, `dayRange`) sets when
  things happen, not only how warm it is. Leaves open as the season's mean
  temperature rises through 8 °C and turn as it falls through 12 °C; grass
  grows above about 5 °C. The defaults (10 °C, a range of 20) bud in the
  second week of April and turn in the second week of October. A mean of
  16 with a range of 12 is a mild coast with a long green year; a mean of
  2 with a range of 28 a short northern summer. A climate whose coldest
  month stays above 8 °C never drops its leaves, and one whose warmest
  stays below it never opens them.
- og.atmosphere.season answers gameplay's questions from the same numbers
  the frame is drawn with: `Math.floor(info.season)` is 0 spring, 1 summer,
  2 autumn or 3 winter where the player stands; `temperature` below 0 is
  frost (snow instead of rain, ice on a pond); `sunrise` and `sunset` are
  hours on og.atmosphere.timeOfDay's clock, so a lamp lights when
  `timeOfDay` passes `sunset` whatever the date.
- In a playtest, `get:og.Season` answers the same facts beside the
  record's fields: `day` is the date drawn, and the air's temperature is
  `temperatureNow` (the record's `temperature` is the year's mean).
- The engine does not choose a season's weather. Pick og.Weather's kind
  from the temperature and the game's own rules.

<a id="guide-seasons-2"></a>
### `seasons §2` Changing the season

- Write `start` (or `speed`) to change the date. With `change` 0 the next
  frame is the new date: right for loading a level, wrong in front of the
  player.
- With `change` above 0 the date drawn and read runs through the days
  between, a quarter of the year every `change` seconds: 10 to 20 s reads
  as time passing, under 4 s as a flicker of light. The run goes forward,
  as a year does: summer to spring passes through autumn and winter and
  takes three times `change`. A jump of under 30 days back is taken
  backward.
- While it runs, og.atmosphere.season's `day` is the date in between and
  everything else follows it, so gameplay and the frame agree. The game's
  own writes stay simple: set `start`, and read `day` when it matters
  whether the change has arrived.
- A world starts on its date: the first tick never plays a change.

<a id="guide-seasons-3"></a>
### `seasons §3` The sun

- With og.DayNight enabled and `sun` true, the sun's path is the date's
  at the latitude. og.DayNight's `hours` are solar time: 12 is always the
  sun's highest, and sunrise and sunset move apart in summer and together
  in winter. `DayNight.tilt` is not used; its `azimuth` is still the
  heading of due east, so a scene laid out for a sun rising over the sea
  keeps it there on the equinoxes, and the sunrise swings to the north of
  that in summer and to the south in winter (north of the equator).
- Set the scene by the low sun, not the high one. At 47 the midwinter noon
  sun is under 20 degrees up: shadows are five times as long as in June,
  a valley or a street may get no sun all day, and the light is warm from
  sunrise to sunset. Check the winter frame before fixing where buildings
  and trees stand.
- A scene that wants today's fixed path keeps `sun: false`: the season
  then changes the temperature and what follows it, never the light.
- At an equinox with `latitude` equal to `DayNight.tilt`, the path is the
  one og.DayNight draws without a season.
