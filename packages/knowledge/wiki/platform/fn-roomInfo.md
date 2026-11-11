<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-roomInfo"></a>

### `og.platform.roomInfo`

The room this engine is in, and what carries rooms here.

```ts
og.platform.roomInfo(world: World): RoomInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `RoomInfo`.

Binding symbol `og_platform_room_info` · publish builds: kept

```ogt
fn roomInfo {
  doc "The room this engine is in, and what carries rooms here."
  param world World "The world."
  returns RoomInfo
}
```

<sub>Module `platform` line 606</sub>
