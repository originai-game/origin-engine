<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openKeyedList"></a>

### `og.ui.openKeyedList`

Open a variable-height list. Ordered unique keys determine row identity and count; rowHeight estimates unbuilt rows. Build listItem(i), its content and close() for first..last, then closeList(). Measurements follow keys through edits; scroll anchors to the item in view. Keyboard/gamepad navigation builds offscreen rows. Measurements and scroll restore with the UI snapshot.

```ts
og.ui.openKeyedList(id: string, keys: readonly string[], desc?: ListDesc): ListRange
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `keys` | `readonly string[]` | no | Ordered persistent keys, never recycled indexes. |
| `desc` | `ListDesc` | yes | The viewport, gap and estimated height; count is ignored. |

**Returns** `ListRange`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn openKeyedList {
  doc "Open a variable-height list. Ordered unique keys determine row identity and count; rowHeight estimates unbuilt rows. Build listItem(i), its content and close() for first..last, then closeList(). Measurements follow keys through edits; scroll anchors to the item in view. Keyboard/gamepad navigation builds offscreen rows. Measurements and scroll restore with the UI snapshot."
  host false
  param id String "Its id."
  param keys List.String "Ordered persistent keys, never recycled indexes."
  param desc ListDesc "The viewport, gap and estimated height; count is ignored."
  returns ListRange
}
```

<sub>Module `ui` line 1382</sub>
