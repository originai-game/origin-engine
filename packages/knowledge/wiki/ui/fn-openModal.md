<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openModal"></a>

### `og.ui.openModal`

Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`.

```ts
og.ui.openModal(id: string, open: boolean, desc?: ModalDesc): ModalResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it should be open. |
| `desc` | `ModalDesc` | yes | Title, width and dismissal rules. |

**Returns** `ModalResult`.

Binding symbol `og_ui_open_modal` · publish builds: kept

```ogt
fn openModal {
  doc "Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`."
  param id String "Its id."
  param open bool "Whether it should be open."
  param desc ModalDesc "Title, width and dismissal rules."
  returns ModalResult
}
```

<sub>Module `ui` line 1260</sub>
