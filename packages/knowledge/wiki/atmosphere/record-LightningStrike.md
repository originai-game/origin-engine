<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-LightningStrike"></a>

### `og.LightningStrike` (event)

Lightning struck, in the tick og.weather drew it (og.WeatherState.lightning): read in the systems after og.weather that tick, or the next tick's first.

Token `og.LightningStrike`, registered under the schema name `og.LightningStrike`; its value type is `LightningStrike`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where it landed: in the disc of og.WeatherState.lightningRadius around lightningCenter, at its height. |
| `strike` | `number` | `0` | — | — | — | Which strike it is, counting from 1 (og.WeatherState.strikes after it). |

```ogt
event LightningStrike {
  doc "Lightning struck, in the tick og.weather drew it (og.WeatherState.lightning): read in the systems after og.weather that tick, or the next tick's first."
  field position Vector3 "Where it landed: in the disc of og.WeatherState.lightningRadius around lightningCenter, at its height." {
    worldspace world
    unit m
  }
  field strike u32 "Which strike it is, counting from 1 (og.WeatherState.strikes after it)."
}
```

<sub>Module `atmosphere` line 550</sub>
