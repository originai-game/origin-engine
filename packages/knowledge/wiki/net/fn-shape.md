<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-shape"></a>

### `og.net.shape`

Give the room a game's own input shape: a component of the game's in place of og.PlayerInput, and commands (an RTS's orders, a card played, a unit built) that act on one tick only, never repeated by a prediction. Call it in setup or start, outside a system and before the room starts, the same on every peer: a peer whose shape differs is refused the room's start. Without a call, og.PlayerInput and no commands.

```ts
og.net.shape(world: World, desc?: NetShapeDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NetShapeDesc` | yes | The input component, the command component and the most commands per tick. |

**Returns** `Result<void>`: Refused inside a system, while lockstep runs, or for a component that is not a game's own with a u32 `player` field.

Binding symbol `og_net_shape` · publish builds: kept

```ogt
fn shape {
  doc "Give the room a game's own input shape: a component of the game's in place of og.PlayerInput, and commands (an RTS's orders, a card played, a unit built) that act on one tick only, never repeated by a prediction. Call it in setup or start, outside a system and before the room starts, the same on every peer: a peer whose shape differs is refused the room's start. Without a call, og.PlayerInput and no commands."
  param world World "The world."
  param desc NetShapeDesc "The input component, the command component and the most commands per tick."
  returns Result.Void "Refused inside a system, while lockstep runs, or for a component that is not a game's own with a u32 `player` field."
}
```

<sub>Module `net` line 203</sub>
