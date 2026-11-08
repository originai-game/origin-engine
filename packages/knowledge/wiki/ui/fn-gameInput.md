<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-gameInput"></a>

### `og.ui.gameInput`

Gameplay takes input this tick: no layer whose input is `menu` is on the stack. Input goes to the top layer that takes it, once per tick.

```ts
og.ui.gameInput(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_ui_game_input` · publish builds: kept

```ogt
fn gameInput {
  doc "Gameplay takes input this tick: no layer whose input is `menu` is on the stack. Input goes to the top layer that takes it, once per tick."
  param world World "The world."
  returns bool
}
```

<sub>Module `ui` line 4286</sub>
