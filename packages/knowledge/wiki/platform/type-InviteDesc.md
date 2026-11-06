<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-InviteDesc"></a>

### `InviteDesc` (desc struct)

An invite to this game.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `to` | `string` | `""` | — | — | — | The friend's player id (og.platform.friends); empty: the host offers an invite link through the share sheet. |
| `message` | `string` | `""` | — | — | — | What the invite says, at most 120 characters; empty: the host's own words. |
| `payload` | `unknown` | `null` | — | — | — | What the game hands the invited player (a room id, a level): any JSON, at most 1 KiB encoded; their og.platform.invites carries it back. |

```ogt
struct InviteDesc {
  doc "An invite to this game."
  desc true
  field to String "The friend's player id (og.platform.friends); empty: the host offers an invite link through the share sheet."
  field message String "What the invite says, at most 120 characters; empty: the host's own words."
  field payload Json "What the game hands the invited player (a room id, a level): any JSON, at most 1 KiB encoded; their og.platform.invites carries it back."
}
```

<sub>`api.ogt` line 15122</sub>
