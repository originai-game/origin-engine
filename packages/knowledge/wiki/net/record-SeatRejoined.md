<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="record-SeatRejoined"></a>

### `og.SeatRejoined` (event)

A seat held before is taken again, by its returning player (og.Lockstep.rejoin) or a newcomer: sent at the start of the first tick of its new holder's input, the same tick on every peer (hashed).

Token `og.SeatRejoined`, registered under the schema name `og.SeatRejoined`; its value type is `SeatRejoined`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seat` | `number` | `0` | — | — | — | The seat. |
| `peer` | `number` | `0` | — | — | — | The peer that holds it now. |
| `tick` | `number` | `0` | — | — | — | The first tick of its input (og.Time.tick now). |

```ogt
event SeatRejoined {
  doc "A seat held before is taken again, by its returning player (og.Lockstep.rejoin) or a newcomer: sent at the start of the first tick of its new holder's input, the same tick on every peer (hashed)."
  field seat u32 "The seat."
  field peer u32 "The peer that holds it now."
  field tick u32 "The first tick of its input (og.Time.tick now)."
}
```

<sub>`api.ogt` line 26226</sub>
