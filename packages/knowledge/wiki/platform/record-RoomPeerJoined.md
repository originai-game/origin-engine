<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="record-RoomPeerJoined"></a>

### `og.RoomPeerJoined` (event)

Another peer joined this engine's room.

Token `og.RoomPeerJoined`, registered under the schema name `og.RoomPeerJoined`; its value type is `RoomPeerJoined`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `peer` | `number` | `0` | — | — | — | Its peer number. |
| `peers` | `number` | `0` | — | — | — | How many peers are in the room now. |

```ogt
event RoomPeerJoined {
  doc "Another peer joined this engine's room."
  field peer u32 "Its peer number."
  field peers u32 "How many peers are in the room now."
}
```

<sub>Module `platform` line 538</sub>
