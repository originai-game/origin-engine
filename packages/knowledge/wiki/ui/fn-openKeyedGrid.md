<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openKeyedGrid"></a>

### `og.ui.openKeyedGrid`

Open a variable-height grid with persistent keys. Each row occupies its tallest cell; cellHeight estimates unbuilt cells. Build gridItem(i), content and close() for first..last, then closeList(). Keys, anchoring, focus and snapshots follow openKeyedList.

```ts
og.ui.openKeyedGrid(id: string, keys: readonly string[], desc?: GridDesc): ListRange
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `keys` | `readonly string[]` | no | Ordered persistent unique keys. |
| `desc` | `GridDesc` | yes | Columns, viewport and estimated cell height; count is ignored. |

**Returns** `ListRange`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn openKeyedGrid {
  doc "Open a variable-height grid with persistent keys. Each row occupies its tallest cell; cellHeight estimates unbuilt cells. Build gridItem(i), content and close() for first..last, then closeList(). Keys, anchoring, focus and snapshots follow openKeyedList."
  host false
  param id String "Its id."
  param keys List.String "Ordered persistent unique keys."
  param desc GridDesc "Columns, viewport and estimated cell height; count is ignored."
  returns ListRange
}
```

<sub>Module `ui` line 1388</sub>
