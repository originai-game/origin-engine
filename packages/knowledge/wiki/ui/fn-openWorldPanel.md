<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openWorldPanel"></a>

### `og.ui.openWorldPanel`

Open a world panel, closed by close(): its children are an ordinary UI (buttons, lists, fields) laid out at width by height and drawn into desc.target; a press on its quad reaches its controls. One render target per panel.

```ts
og.ui.openWorldPanel(id: string, desc?: WorldPanelDesc): PanelInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id; the probe path is the id alone. |
| `desc` | `WorldPanelDesc` | yes | Its target, size and quad. |

**Returns** `PanelInfo`.

Binding symbol `og_ui_open_world_panel` · publish builds: kept

```ogt
fn openWorldPanel {
  doc "Open a world panel, closed by close(): its children are an ordinary UI (buttons, lists, fields) laid out at width by height and drawn into desc.target; a press on its quad reaches its controls. One render target per panel."
  param id String "Its id; the probe path is the id alone."
  param desc WorldPanelDesc "Its target, size and quad."
  returns PanelInfo
}
```

<sub>Module `ui` line 7564</sub>
