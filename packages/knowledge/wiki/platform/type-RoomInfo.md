<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-RoomInfo"></a>

### `RoomInfo` (struct)

The room this engine is in.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `state` | `string` | `""` | — | — | — | `none` (in no room), `joining` (roomCreate, roomJoin or quickPlay asked, not answered yet: quick play waiting for its match) or `in`. |
| `room` | `string` | `""` | — | — | — | The room's id; empty in none. |
| `host` | `string` | `""` | — | — | — | What carries rooms here: `relay` (natively, `og run --room` through `og relay`), `player` (the platform's rooms through the Player, on the web) or `none` (roomCreate and roomJoin are refused). |
| `peer` | `number` | `0` | — | — | — | This engine's peer number; 0 in none. |
| `peers` | `readonly RoomPeer[]` | `[]` | — | — | — | Every peer present, this one included, by number. |
| `link` | `string` | `""` | — | — | — | The room a Play link to a room (`?room=<id>` on the play page) named: the engine joined it at load (og.RoomJoined answers, as for roomJoin); empty without one. |

```ogt
struct RoomInfo {
  doc "The room this engine is in."
  field state String "`none` (in no room), `joining` (roomCreate, roomJoin or quickPlay asked, not answered yet: quick play waiting for its match) or `in`."
  field room String "The room's id; empty in none."
  field host String "What carries rooms here: `relay` (natively, `og run --room` through `og relay`), `player` (the platform's rooms through the Player, on the web) or `none` (roomCreate and roomJoin are refused)."
  field peer u32 "This engine's peer number; 0 in none."
  field peers List.RoomPeer "Every peer present, this one included, by number."
  field link String "The room a Play link to a room (`?room=<id>` on the play page) named: the engine joined it at load (og.RoomJoined answers, as for roomJoin); empty without one."
}
```

<sub>Module `platform` line 510</sub>
