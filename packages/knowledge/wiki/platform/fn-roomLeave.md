<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-roomLeave"></a>

### `og.platform.roomLeave`

Leave the room: og.RoomPeerLeft with this engine's own peer number answers, and the other peers hear it leave after its last message. While joining (quick play waiting for its match) it stops waiting: og.RoomJoined answers with ok false and `cancelled`.

```ts
og.platform.roomLeave(world: World): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<void>`: Refused when in no room and not joining one.

Binding symbol `og_platform_room_leave` · publish builds: kept

```ogt
fn roomLeave {
  doc "Leave the room: og.RoomPeerLeft with this engine's own peer number answers, and the other peers hear it leave after its last message. While joining (quick play waiting for its match) it stops waiting: og.RoomJoined answers with ok false and `cancelled`."
  param world World "The world."
  returns Result.Void "Refused when in no room and not joining one."
}
```

<sub>`api.ogt` line 15396</sub>
