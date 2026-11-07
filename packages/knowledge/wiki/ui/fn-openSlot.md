<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openSlot"></a>

### `og.ui.openSlot`

Open a document's slot (a `slot` element built this tick) so code fills it; its children follow until og.ui.close. False when no such slot was built this tick.

```ts
og.ui.openSlot(path: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | Its full path, or `<screen>/<slot id>`. |

**Returns** `boolean`.

Binding symbol `og_ui_open_slot` · publish builds: kept

```ogt
fn openSlot {
  doc "Open a document's slot (a `slot` element built this tick) so code fills it; its children follow until og.ui.close. False when no such slot was built this tick."
  param path String "Its full path, or `<screen>/<slot id>`."
  returns bool
}
```

<sub>Module `ui` line 8019</sub>
