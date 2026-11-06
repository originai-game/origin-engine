<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="record-RoomPeerLeft"></a>

### `og.RoomPeerLeft` (event)

A peer left this engine's room (it left, closed, or lost its connection), after every message it sent; or, with peer equal to this engine's own number, this engine is in no room any more (roomLeave, or the host lost the room).

Token `og.RoomPeerLeft`, registered under the schema name `og.RoomPeerLeft`; its value type is `RoomPeerLeft`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `peer` | `number` | `0` | — | — | — | Its peer number. |
| `peers` | `number` | `0` | — | — | — | How many peers are in the room now (0 once this engine left). |

```ogt
event RoomPeerLeft {
  doc "A peer left this engine's room (it left, closed, or lost its connection), after every message it sent; or, with peer equal to this engine's own number, this engine is in no room any more (roomLeave, or the host lost the room)."
  field peer u32 "Its peer number."
  field peers u32 "How many peers are in the room now (0 once this engine left)."
}
```

<sub>`api.ogt` line 15422</sub>
