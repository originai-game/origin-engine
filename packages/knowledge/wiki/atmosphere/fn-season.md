<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="fn-season"></a>

### `og.atmosphere.season`

The time of year at the current tick, from og.Season: its defaults in a world that has none, the date at its start while it is disabled. For lamps that light at sunset, snow that should fall instead of rain, crops and festivals. Inside a system, declare og.Season among its reads, and og.DayNight for the hour's temperature.

See `seasons §1`.

```ts
og.atmosphere.season(world: World): SeasonInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `SeasonInfo`.

Binding symbol `og_atmosphere_season` · publish builds: kept

```ogt
fn season {
  doc "The time of year at the current tick, from og.Season: its defaults in a world that has none, the date at its start while it is disabled. For lamps that light at sunset, snow that should fall instead of rain, crops and festivals. Inside a system, declare og.Season among its reads, and og.DayNight for the hour's temperature."
  guide "seasons §1"
  param world World "The world."
  returns SeasonInfo
}
```

<sub>Module `atmosphere` line 788</sub>
