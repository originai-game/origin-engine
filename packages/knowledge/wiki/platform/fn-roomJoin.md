<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-roomJoin"></a>

### `og.platform.roomJoin`

Join a room by its id. The answer is og.RoomJoined (with ok false and `not_found` or `full` when it cannot); then og.RoomPeerJoined and og.RoomPeerLeft as others come and go.

Also known as: join lobby, join match, join session

```ts
og.platform.roomJoin(world: World, room: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `room` | `string` | no | The room's id: 1 to 64 of letters, digits, `.`, `_` and `-`. |

**Returns** `Result<void>`: Refused with no room host, while in or joining a room, or for a malformed id.

Binding symbol `og_platform_room_join` · publish builds: kept

```ogt
fn roomJoin {
  doc "Join a room by its id. The answer is og.RoomJoined (with ok false and `not_found` or `full` when it cannot); then og.RoomPeerJoined and og.RoomPeerLeft as others come and go."
  aliases "join lobby, join match, join session"
  param world World "The world."
  param room String "The room's id: 1 to 64 of letters, digits, `.`, `_` and `-`."
  returns Result.Void "Refused with no room host, while in or joining a room, or for a malformed id."
}
```

<sub>`api.ogt` line 15444</sub>
