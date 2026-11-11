<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="record-RoomJoined"></a>

### `og.RoomJoined` (event)

roomCreate, roomJoin or quickPlay finished, or the host put this engine in a room (`og run --room`, a Play link to a room): sent at the start of the tick after the host answered. With ok false the engine is in no room and `reason` says why.

Token `og.RoomJoined`, registered under the schema name `og.RoomJoined`; its value type is `RoomJoined`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ok` | `boolean` | `false` | — | — | — | Whether this engine is in the room now. |
| `room` | `string` | `""` | — | — | — | The room's id (at most 63 bytes); empty when not ok. |
| `peer` | `number` | `0` | — | — | — | This engine's peer number. |
| `peers` | `number` | `0` | — | — | — | How many peers are in the room, this one included. |
| `reason` | `string` | `""` | — | — | — | Why it failed: `unavailable` (no rooms here), `not_found`, `full`, `invalid`, `offline`, `timeout` (quick play matched nobody in time) or `cancelled` (roomLeave while joining), then a colon and the host's words; empty when ok. |

```ogt
event RoomJoined {
  doc "roomCreate, roomJoin or quickPlay finished, or the host put this engine in a room (`og run --room`, a Play link to a room): sent at the start of the tick after the host answered. With ok false the engine is in no room and `reason` says why."
  field ok bool "Whether this engine is in the room now."
  field room String "The room's id (at most 63 bytes); empty when not ok."
  field peer u32 "This engine's peer number."
  field peers u32 "How many peers are in the room, this one included."
  field reason String "Why it failed: `unavailable` (no rooms here), `not_found`, `full`, `invalid`, `offline`, `timeout` (quick play matched nobody in time) or `cancelled` (roomLeave while joining), then a colon and the host's words; empty when ok."
}
```

<sub>Module `platform` line 529</sub>
