<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-roomMessage"></a>

### `og.platform.roomMessage`

A message og.RoomMessage announced: who sent it and what it carries. The engine keeps the last 256.

```ts
og.platform.roomMessage(world: World, message: number): RoomMessageData | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `message` | `number` | no | og.RoomMessage.message. |

**Returns** `RoomMessageData | null`: Null for a number never given, or a message older than the last 256.

Binding symbol `og_platform_room_message` · publish builds: kept

```ogt
fn roomMessage {
  doc "A message og.RoomMessage announced: who sent it and what it carries. The engine keeps the last 256."
  param world World "The world."
  param message u32 "og.RoomMessage.message."
  returns Option.RoomMessageData "Null for a number never given, or a message older than the last 256."
}
```

<sub>Module `platform` line 611</sub>
