<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="type-NetShapeDesc"></a>

### `NetShapeDesc` (desc struct)

A game's own input shape (og.net.shape): the record each seat's input fills in place of og.PlayerInput, and the commands a seat gives on single ticks (an RTS's orders).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `input` | `AnyComponent \| null` | `null` | — | — | — | The component every seat's input fills, one entity per player, in place of og.PlayerInput: a game's own with a u32 `player` field (the seat it carries, set by the game) and any others (a u32 `tick`, when it has one, is written as og.PlayerInput's is). og.net.submitInput gives this peer's. Null keeps og.PlayerInput. |
| `command` | `AnyComponent \| null` | `null` | — | — | — | The component whose values are the commands (og.net.command, read with og.net.commands): a game's own with a u32 `player` field, which the engine sets to the seat that gave the command. Used as a record shape only: no entity needs to carry it. Null: no commands. |
| `maxCommands` | `number` | `32` | 1 … 256 | — | — | The most commands one seat gives for one tick; og.net.command refuses one more. |

```ogt
struct NetShapeDesc {
  doc "A game's own input shape (og.net.shape): the record each seat's input fills in place of og.PlayerInput, and the commands a seat gives on single ticks (an RTS's orders)."
  desc true
  field input Option.AnyComponent "The component every seat's input fills, one entity per player, in place of og.PlayerInput: a game's own with a u32 `player` field (the seat it carries, set by the game) and any others (a u32 `tick`, when it has one, is written as og.PlayerInput's is). og.net.submitInput gives this peer's. Null keeps og.PlayerInput."
  field command Option.AnyComponent "The component whose values are the commands (og.net.command, read with og.net.commands): a game's own with a u32 `player` field, which the engine sets to the seat that gave the command. Used as a record shape only: no entity needs to carry it. Null: no commands."
  field maxCommands u32 "The most commands one seat gives for one tick; og.net.command refuses one more." {
    default 32
    min 1
    max 256
  }
}
```

<sub>`api.ogt` line 26370</sub>
