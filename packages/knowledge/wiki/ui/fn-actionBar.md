<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-actionBar"></a>

### `og.ui.actionBar`

What an action bar component reads: the actions the top layer's document lists (`actions { confirm "Buy" }`), each with its hint and its glyph on the device used last.

```ts
og.ui.actionBar(world: World): readonly UiActionHint[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly UiActionHint[]`.

Binding symbol `og_ui_action_bar` · publish builds: kept

```ogt
fn actionBar {
  doc "What an action bar component reads: the actions the top layer's document lists (`actions { confirm \"Buy\" }`), each with its hint and its glyph on the device used last."
  param world World "The world."
  returns List.UiActionHint
}
```

<sub>Module `ui` line 4343</sub>
