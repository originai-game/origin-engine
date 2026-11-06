<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-roomSend"></a>

### `og.platform.roomSend`

Send a message to every other peer in the room, or to one: a JSON value, or bytes (a Uint8Array). It arrives as og.RoomMessage, after every earlier message of this peer.

```ts
og.platform.roomSend(world: World, data: Uint8Array | unknown, to?: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `data` | `Uint8Array \| unknown` | no | What to send: a Uint8Array, or any JSON value; at most 64 KiB encoded. |
| `to` | `number` | yes | The one peer to send it to; omitted is every other peer. |

**Returns** `Result<void>`: Refused when in no room, for a message over 64 KiB, or a peer not in the room.

Binding symbol `og_platform_room_send` · publish builds: kept

```ogt
fn roomSend {
  doc "Send a message to every other peer in the room, or to one: a JSON value, or bytes (a Uint8Array). It arrives as og.RoomMessage, after every earlier message of this peer."
  param world World "The world."
  param data OneOf.Bytes.Json "What to send: a Uint8Array, or any JSON value; at most 64 KiB encoded."
  param to u32 "The one peer to send it to; omitted is every other peer." {
    optional true
  }
  returns Result.Void "Refused when in no room, for a message over 64 KiB, or a peer not in the room."
}
```

<sub>`api.ogt` line 15481</sub>
