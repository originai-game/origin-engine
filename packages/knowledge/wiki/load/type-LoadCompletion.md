<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-LoadCompletion"></a>

### `LoadCompletion` (struct)

When an uncooked load was put in place: the journal a replay applies again (og.load.completions).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `request` | `number` | `0` | — | — | — | The load's number: 0 for the world's first uncooked load, then one more for each. |
| `tick` | `number` | `0` | — | `ticks` | — | The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed. |
| `ok` | `boolean` | `false` | — | — | — | Whether it was ready (else failed). |

```ogt
struct LoadCompletion {
  doc "When an uncooked load was put in place: the journal a replay applies again (og.load.completions)."
  field request u32 "The load's number: 0 for the world's first uncooked load, then one more for each."
  field tick f64 "The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed." {
    unit ticks
  }
  field ok bool "Whether it was ready (else failed)."
}
```

<sub>Module `load` line 542</sub>
