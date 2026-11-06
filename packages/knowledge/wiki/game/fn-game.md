<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="fn-game"></a>

### `og.game`

Declare the game. The bundle's default export is its result.

```ts
og.game(desc?: GameDesc): Game
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `GameDesc` | yes | Tick, scenes, systems and commands. |

**Returns** `Game`.

Binding symbol `og_game_game` · publish builds: kept

```ogt
fn game {
  doc "Declare the game. The bundle's default export is its result."
  param desc GameDesc "Tick, scenes, systems and commands."
  returns Game
}
```

<sub>Module `game` line 59</sub>
