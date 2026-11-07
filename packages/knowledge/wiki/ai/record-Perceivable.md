<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="record-Perceivable"></a>

### `og.Perceivable` (component)

Something agents of other teams can see.

Token `og.Perceivable`, registered under the schema name `og.Perceivable`; its value type is `Perceivable`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `team` | `number` | `1` | — | — | — | Its team (agents of the same team ignore it). |
| `eyeHeight` | `number` | `0` | -10 … 10 | `m` | — | Agents look at a point this far above its position (+y). |

```ogt
component Perceivable {
  doc "Something agents of other teams can see."
  field team u32 "Its team (agents of the same team ignore it)." {
    default 1
  }
  field eyeHeight f32 "Agents look at a point this far above its position (+y)." {
    min -10
    max 10
    unit m
  }
}
```

<sub>Module `ai` line 121</sub>
