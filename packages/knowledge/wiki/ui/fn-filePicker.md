<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-filePicker"></a>

### `og.ui.filePicker`

Whether this host opens a file picker for a document's `<input type="file">` (a capability query, the same on every host that has one): false on the web and natively today, where such an input is refused at build with a warning. Where true, a press sends the host a `filepick` event (its `accept` and `multiple`) and the host writes the chosen names to the input's bound value.

```ts
og.ui.filePicker(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_ui_file_picker` · publish builds: kept

```ogt
fn filePicker {
  doc "Whether this host opens a file picker for a document's `<input type=\"file\">` (a capability query, the same on every host that has one): false on the web and natively today, where such an input is refused at build with a warning. Where true, a press sends the host a `filepick` event (its `accept` and `multiple`) and the host writes the chosen names to the input's bound value."
  param world World "The world."
  returns bool
}
```

<sub>Module `ui` line 3439</sub>
