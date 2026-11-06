<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-now"></a>

### `og.platform.now`

The host's wall-clock time, UTC milliseconds since 1970, as of this tick: what the host last told the engine (when it attached, and with every answer, applied at a tick's start like the answers) moved on by the ticks since. The one wall-clock read game code may make: it is host input, so a replay with the same answers gives the same values. Use it to know how long the game was closed (a save's time against a launch's), never to time play (that is og.Time).

Also known as: real time, wall clock, date, server time, calendar day, daily reward clock

```ts
og.platform.now(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: UTC milliseconds; 0 until a host has told the time (`none`).

Binding symbol `og_platform_now` · publish builds: kept

```ogt
fn now {
  doc "The host's wall-clock time, UTC milliseconds since 1970, as of this tick: what the host last told the engine (when it attached, and with every answer, applied at a tick's start like the answers) moved on by the ticks since. The one wall-clock read game code may make: it is host input, so a replay with the same answers gives the same values. Use it to know how long the game was closed (a save's time against a launch's), never to time play (that is og.Time)."
  aliases "real time, wall clock, date, server time, calendar day, daily reward clock"
  param world World "The world."
  returns f64 "UTC milliseconds; 0 until a host has told the time (`none`)."
}
```

<sub>`api.ogt` line 15357</sub>
