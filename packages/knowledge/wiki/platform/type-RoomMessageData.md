<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-RoomMessageData"></a>

### `RoomMessageData` (struct)

A message another peer sent: what og.RoomMessage announced.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `message` | `number` | `0` | — | — | — | Its number (og.RoomMessage.message): 1 for the first message this engine received, counting up. |
| `from` | `number` | `0` | — | — | — | The sender's peer number. |
| `tick` | `number` | `0` | — | — | — | The tick it was delivered at. |
| `data` | `unknown` | `null` | — | — | — | What it carries, when it was sent as a JSON value; null when it was sent as bytes. |
| `bytes` | `Uint8Array \| null` | `null` | — | — | — | What it carries, when it was sent as bytes. |

```ogt
struct RoomMessageData {
  doc "A message another peer sent: what og.RoomMessage announced."
  field message u32 "Its number (og.RoomMessage.message): 1 for the first message this engine received, counting up."
  field from u32 "The sender's peer number."
  field tick u32 "The tick it was delivered at."
  field data Json "What it carries, when it was sent as a JSON value; null when it was sent as bytes."
  field bytes Option.Bytes "What it carries, when it was sent as bytes."
}
```

<sub>Module `platform` line 519</sub>
