<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-hudLayoutEditor"></a>

### `og.ui.hudLayoutEditor`

The HUD edit mode's panel (docs/ui.md §8.4): the selected element's key, size and opacity steps, Next (the next element: how keys and pads choose one), Reset (the selected one, or all with none selected) and Done, which turns the edit mode off. Build it where it fits, while og.ui.hudEdit().on; presses on it edit nothing behind it. Its children are `Title`, `Selected`, `Size/Less`, `Size/More`, `Opacity/Less`, `Opacity/More`, `Actions/Next`, `Actions/Reset`, `Actions/Done`.

```ts
og.ui.hudLayoutEditor(id: string, desc?: HudEditorDesc): HudEditorResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `HudEditorDesc` | yes | Its labels. |

**Returns** `HudEditorResult`.

Binding symbol `og_ui_hud_layout_editor` · publish builds: kept

```ogt
fn hudLayoutEditor {
  doc "The HUD edit mode's panel (docs/ui.md §8.4): the selected element's key, size and opacity steps, Next (the next element: how keys and pads choose one), Reset (the selected one, or all with none selected) and Done, which turns the edit mode off. Build it where it fits, while og.ui.hudEdit().on; presses on it edit nothing behind it. Its children are `Title`, `Selected`, `Size/Less`, `Size/More`, `Opacity/Less`, `Opacity/More`, `Actions/Next`, `Actions/Reset`, `Actions/Done`."
  param id String "Its id."
  param desc HudEditorDesc "Its labels."
  returns HudEditorResult
}
```

<sub>`api.ogt` line 17479</sub>
