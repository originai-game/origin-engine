<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="record-RoomMessage"></a>

### `og.RoomMessage` (event)

Another peer's message arrived; read it with og.platform.roomMessage. Messages from one peer arrive in the order it sent them. While lockstep runs (og.Lockstep) a message is this peer's host input alone: a system that changes the world from one makes it differ from the other peers' (og.Desync); carry what all must see in og.PlayerInput.

Token `og.RoomMessage`, registered under the schema name `og.RoomMessage`; its value type is `RoomMessage`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | — | — | The sender's peer number. |
| `message` | `number` | `0` | — | — | — | The message's number, for og.platform.roomMessage. |
| `binary` | `boolean` | `false` | — | — | — | Whether it was sent as bytes (else as a JSON value). |

```ogt
event RoomMessage {
  doc "Another peer's message arrived; read it with og.platform.roomMessage. Messages from one peer arrive in the order it sent them. While lockstep runs (og.Lockstep) a message is this peer's host input alone: a system that changes the world from one makes it differ from the other peers' (og.Desync); carry what all must see in og.PlayerInput."
  field from u32 "The sender's peer number."
  field message u32 "The message's number, for og.platform.roomMessage."
  field binary bool "Whether it was sent as bytes (else as a JSON value)."
}
```

<sub>`api.ogt` line 15450</sub>
