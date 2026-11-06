<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertCost"></a>

### `AssertCost` (struct)

What the default invariants cost, measured outside the tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fields` | `number` | `0` | — | — | — | The reflected fields they check. |
| `ticks` | `number` | `0` | — | — | — | The ticks they checked in this report. |
| `values` | `number` | `0` | — | — | — | The values they read in this report. |
| `totalMs` | `number` | `0` | — | `ms` | — | Their time in this report. |
| `meanUs` | `number` | `0` | — | — | — | Their mean time a tick, in microseconds. |
| `maxUs` | `number` | `0` | — | — | — | Their slowest tick, in microseconds. |

```ogt
struct AssertCost {
  doc "What the default invariants cost, measured outside the tick."
  field fields u32 "The reflected fields they check."
  field ticks u32 "The ticks they checked in this report."
  field values f64 "The values they read in this report."
  field totalMs f64 "Their time in this report." {
    unit ms
  }
  field meanUs f64 "Their mean time a tick, in microseconds."
  field maxUs f64 "Their slowest tick, in microseconds."
}
```

<sub>`api.ogt` line 25930</sub>
