<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="type-LockstepSeat"></a>

### `LockstepSeat` (struct)

One peer's hold on a seat.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seat` | `number` | `0` | — | — | — | The seat. |
| `peer` | `number` | `0` | — | — | — | The peer that holds (or held) it. |
| `since` | `number` | `0` | — | `ticks` | — | The first tick of its input. |
| `until` | `number \| null` | `null` | — | `ticks` | — | The last tick of its input, once it was dropped. |
| `left` | `boolean` | `false` | — | — | — | Whether it was dropped because it left the room (else it stalled). |

```ogt
struct LockstepSeat {
  doc "One peer's hold on a seat."
  field seat u32 "The seat."
  field peer u32 "The peer that holds (or held) it."
  field since f64 "The first tick of its input." {
    unit ticks
  }
  field until Option.f64 "The last tick of its input, once it was dropped." {
    unit ticks
  }
  field left bool "Whether it was dropped because it left the room (else it stalled)."
}
```

<sub>`api.ogt` line 26397</sub>
