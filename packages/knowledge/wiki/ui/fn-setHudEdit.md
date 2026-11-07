<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setHudEdit"></a>

### `og.ui.setHudEdit`

Turns the HUD edit mode on or off (docs/ui.md §8.4; kept until changed). While it is on, elements with ElementProps.customize stop answering their own presses and show their outlines: a press selects one and a drag moves it, two touches or the wheel scale it, and with no virtual cursor pad 0's left stick moves the selected one. Build og.ui.hudLayoutEditor for steps, Next, Reset and Done.

```ts
og.ui.setHudEdit(on: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `on` | `boolean` | no | On or off. |

**Returns** `void`.

Binding symbol `og_ui_set_hud_edit` · publish builds: kept

```ogt
fn setHudEdit {
  doc "Turns the HUD edit mode on or off (docs/ui.md §8.4; kept until changed). While it is on, elements with ElementProps.customize stop answering their own presses and show their outlines: a press selects one and a drag moves it, two touches or the wheel scale it, and with no virtual cursor pad 0's left stick moves the selected one. Build og.ui.hudLayoutEditor for steps, Next, Reset and Done."
  param on bool "On or off."
  returns Void
}
```

<sub>Module `ui` line 1627</sub>
