<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openPanel"></a>

### `og.ui.openPanel`

Open a card, closed by close(): og.ui.panel without the callback.

```ts
og.ui.openPanel(id: string, desc?: PanelDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PanelDesc` | yes | Its layout and look. |

**Returns** `void`.

Binding symbol `og_ui_open_panel` · publish builds: kept

```ogt
fn openPanel {
  doc "Open a card, closed by close(): og.ui.panel without the callback."
  param id String "Its id."
  param desc PanelDesc "Its layout and look."
  returns Void
}
```

<sub>Module `ui` line 601</sub>
