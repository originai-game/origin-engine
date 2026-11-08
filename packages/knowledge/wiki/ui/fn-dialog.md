<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dialog"></a>

### `og.ui.dialog`

A dialog on the modal layer: tone mark, message, body and actions.

```ts
og.ui.dialog(id: string, open: boolean, desc: DialogDesc, build: UiBuild): DialogResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it is open. |
| `desc` | `DialogDesc` | no | Title, message, tone and actions. |
| `build` | `UiBuild` | no | Builds its body under the message. |

**Returns** `DialogResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn dialog {
  doc "A dialog on the modal layer: tone mark, message, body and actions."
  host false
  param id String "Its id."
  param open bool "Whether it is open."
  param desc DialogDesc "Title, message, tone and actions."
  param build UiBuild "Builds its body under the message."
  returns DialogResult
}
```

<sub>Module `ui` line 2857</sub>
