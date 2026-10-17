<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `time` module

Simulation time. Game code never reads the wall clock; it reads Time.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module time {
  doc "Simulation time. Game code never reads the wall clock; it reads Time."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 541</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Time`](#record-Time) | resource | The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers. |

## Components, resources and events

<a id="record-Time"></a>

### `og.Time` (resource)

The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers.

Token `og.Time`, registered under the schema name `og.Time`; its value type is `Time`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | `ticks` | read only | Fixed steps taken since load. |
| `dt` | `number` | `0` | — | `s` | read only | The length of one tick: 1 / rate. |
| `elapsed` | `number` | `0` | — | `s` | read only | Simulation time since load: tick * dt. |
| `rate` | `number` | `0` | — | `hz` | read only | The tick rate, from GameDesc.tick. |

```ogt
resource Time {
  doc "The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers."
  field tick u32 "Fixed steps taken since load." {
    readonly true
    unit ticks
  }
  field dt f32 "The length of one tick: 1 / rate." {
    readonly true
    unit s
  }
  field elapsed f64 "Simulation time since load: tick * dt." {
    readonly true
    unit s
  }
  field rate u32 "The tick rate, from GameDesc.tick." {
    readonly true
    unit hz
  }
}
```

<sub>`api.ogt` line 544</sub>
