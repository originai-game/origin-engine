<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-roomPeers"></a>

### `og.platform.roomPeers`

The peers in this engine's room, this one included, by number; empty in none.

```ts
og.platform.roomPeers(world: World): readonly RoomPeer[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly RoomPeer[]`.

Binding symbol `og_platform_room_peers` · publish builds: kept

```ogt
fn roomPeers {
  doc "The peers in this engine's room, this one included, by number; empty in none."
  param world World "The world."
  returns List.RoomPeer
}
```

<sub>Module `platform` line 600</sub>
