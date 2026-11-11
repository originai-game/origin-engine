<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-QuickPlayDesc"></a>

### `QuickPlayDesc` (desc struct)

Quick play: what this engine waits to be matched for (og.platform.quickPlay).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mode` | `string` | `""` | — | — | — | The mode to play (at most 32 of letters, digits, `.`, `_` and `-`): only engines asking for the same mode and players are matched. Empty is `default`. |
| `players` | `number` | `2` | 1 … 16 | — | — | How many play together: the room is made once this many wait (set og.Lockstep.players to the same). |
| `timeoutMs` | `number` | `30000` | 0 … 600000 | `ms` | — | How long to wait for the others, in milliseconds; then og.RoomJoined says `timeout`. 0 waits until roomLeave. |
| `name` | `string` | `""` | — | — | — | This player's name, as the other peers see it (og.platform.roomPeers); empty for none. |

```ogt
struct QuickPlayDesc {
  doc "Quick play: what this engine waits to be matched for (og.platform.quickPlay)."
  desc true
  field mode String "The mode to play (at most 32 of letters, digits, `.`, `_` and `-`): only engines asking for the same mode and players are matched. Empty is `default`."
  field players u32 "How many play together: the room is made once this many wait (set og.Lockstep.players to the same)." {
    default 2
    min 1
    max 16
  }
  field timeoutMs u32 "How long to wait for the others, in milliseconds; then og.RoomJoined says `timeout`. 0 waits until roomLeave." {
    default 30000
    min 0
    max 600000
    unit ms
  }
  field name String "This player's name, as the other peers see it (og.platform.roomPeers); empty for none."
}
```

<sub>Module `platform` line 484</sub>
