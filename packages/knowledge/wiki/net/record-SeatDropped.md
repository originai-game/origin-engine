<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="record-SeatDropped"></a>

### `og.SeatDropped` (event)

A seat's player was dropped: it left the room, or stalled past og.Lockstep.stallTimeout. Sent at the start of the first tick the seat is empty, the same tick on every peer (a hashed event: the confirmed world holds it); from then on its og.PlayerInput is the zero value until og.SeatRejoined.

Token `og.SeatDropped`, registered under the schema name `og.SeatDropped`; its value type is `SeatDropped`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seat` | `number` | `0` | — | — | — | The seat, og.PlayerInput.player. |
| `peer` | `number` | `0` | — | — | — | The peer that held it. |
| `tick` | `number` | `0` | — | — | — | The first tick the seat is empty (og.Time.tick now). |
| `left` | `boolean` | `false` | — | — | — | Whether the peer left the room (else it stalled and may come back). |

```ogt
event SeatDropped {
  doc "A seat's player was dropped: it left the room, or stalled past og.Lockstep.stallTimeout. Sent at the start of the first tick the seat is empty, the same tick on every peer (a hashed event: the confirmed world holds it); from then on its og.PlayerInput is the zero value until og.SeatRejoined."
  field seat u32 "The seat, og.PlayerInput.player."
  field peer u32 "The peer that held it."
  field tick u32 "The first tick the seat is empty (og.Time.tick now)."
  field left bool "Whether the peer left the room (else it stalled and may come back)."
}
```

<sub>Module `net` line 93</sub>
