<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="record-PlayerInput"></a>

### `og.PlayerInput` (component)

One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout.

Token `og.PlayerInput`, registered under the schema name `og.PlayerInput`; its value type is `PlayerInput`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `player` | `number` | `0` | — | — | — | The seat whose input this entity carries (0 to og.Lockstep.players - 1); the game sets it when it spawns the player. |
| `buttons` | `number` | `0` | — | — | — | The buttons held, one bit each, as the game numbers them (bit 0 fire, bit 1 dash, ...). |
| `x` | `number` | `0` | -1 … 1 | — | — | The move axis, left -1 to right 1. |
| `y` | `number` | `0` | -1 … 1 | — | — | The move axis, back -1 to forward 1. |
| `aimX` | `number` | `0` | — | — | — | A second axis (aim, a cursor's world x), as the game uses it. |
| `aimY` | `number` | `0` | — | — | — | A second axis (aim, a cursor's world y), as the game uses it. |
| `tick` | `number` | `0` | — | — | read only | The tick these values are for: the tick now, once the engine wrote them. |

```ogt
component PlayerInput {
  doc "One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout."
  field player u32 "The seat whose input this entity carries (0 to og.Lockstep.players - 1); the game sets it when it spawns the player."
  field buttons u32 "The buttons held, one bit each, as the game numbers them (bit 0 fire, bit 1 dash, ...)."
  field x f32 "The move axis, left -1 to right 1." {
    min -1
    max 1
  }
  field y f32 "The move axis, back -1 to forward 1." {
    min -1
    max 1
  }
  field aimX f32 "A second axis (aim, a cursor's world x), as the game uses it."
  field aimY f32 "A second axis (aim, a cursor's world y), as the game uses it."
  field tick u32 "The tick these values are for: the tick now, once the engine wrote them." {
    readonly true
  }
}
```

<sub>`api.ogt` line 26323</sub>
