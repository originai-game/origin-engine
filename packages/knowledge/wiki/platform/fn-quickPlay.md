<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-quickPlay"></a>

### `og.platform.quickPlay`

Quick play: wait to be matched with other players asking for the same mode and number of players, then play in a room of their own. The answer is og.RoomJoined once the room has filled (every matched peer hears it at once, numbered in the order they asked), or ok false with `timeout` after timeoutMs. roomLeave stops waiting (og.RoomJoined with `cancelled`). Natively `og relay` matches the engines connected to it (`og run --room ws://127.0.0.1:<port>/`); on the web the platform's matchmaking does (platform ask 21: until then the Player answers `unavailable`).

```ts
og.platform.quickPlay(world: World, desc?: QuickPlayDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `QuickPlayDesc` | yes | The mode, the players, how long to wait and this player's name. |

**Returns** `Result<void>`: Refused with no room host, while in or joining a room, or for a malformed mode or name.

Binding symbol `og_platform_quick_play` · publish builds: kept

```ogt
fn quickPlay {
  doc "Quick play: wait to be matched with other players asking for the same mode and number of players, then play in a room of their own. The answer is og.RoomJoined once the room has filled (every matched peer hears it at once, numbered in the order they asked), or ok false with `timeout` after timeoutMs. roomLeave stops waiting (og.RoomJoined with `cancelled`). Natively `og relay` matches the engines connected to it (`og run --room ws://127.0.0.1:<port>/`); on the web the platform's matchmaking does (platform ask 21: until then the Player answers `unavailable`)."
  param world World "The world."
  param desc QuickPlayDesc "The mode, the players, how long to wait and this player's name." {
    optional true
  }
  returns Result.Void "Refused with no room host, while in or joining a room, or for a malformed mode or name."
}
```

<sub>`api.ogt` line 15467</sub>
