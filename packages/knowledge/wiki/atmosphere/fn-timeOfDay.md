<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="fn-timeOfDay"></a>

### `og.atmosphere.timeOfDay`

The DayNight clock at the current tick, in hours from 0 to 24 (its start when the cycle is off): for lamps that light at dusk and shops that close at night. Inside a system, declare og.DayNight among its reads.

```ts
og.atmosphere.timeOfDay(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`.

Binding symbol `og_atmosphere_time_of_day` · publish builds: kept

```ogt
fn timeOfDay {
  doc "The DayNight clock at the current tick, in hours from 0 to 24 (its start when the cycle is off): for lamps that light at dusk and shops that close at night. Inside a system, declare og.DayNight among its reads."
  param world World "The world."
  returns f32
}
```

<sub>Module `atmosphere` line 661</sub>
