<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-roomCreate"></a>

### `og.platform.roomCreate`

Create a room and join it. The answer is og.RoomJoined, with the room's id to share (natively the relay names it `room-N`; on the web the platform does).

```ts
og.platform.roomCreate(world: World, desc?: RoomDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RoomDesc` | yes | Its name and capacity. |

**Returns** `Result<void>`: Refused with no room host (og.platform.roomInfo's host `none`), while in or joining a room (roomLeave first), or for a malformed name.

Binding symbol `og_platform_room_create` · publish builds: kept

```ogt
fn roomCreate {
  doc "Create a room and join it. The answer is og.RoomJoined, with the room's id to share (natively the relay names it `room-N`; on the web the platform does)."
  param world World "The world."
  param desc RoomDesc "Its name and capacity." {
    optional true
  }
  returns Result.Void "Refused with no room host (og.platform.roomInfo's host `none`), while in or joining a room (roomLeave first), or for a malformed name."
}
```

<sub>`api.ogt` line 15457</sub>
