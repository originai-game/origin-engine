<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openGrid"></a>

### `og.ui.openGrid`

Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList().

```ts
og.ui.openGrid(id: string, desc?: GridDesc): ListRange
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `GridDesc` | yes | Cell count, size and columns. |

**Returns** `ListRange`.

Binding symbol `og_ui_open_grid` · publish builds: kept

```ogt
fn openGrid {
  doc "Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList()."
  param id String "Its id."
  param desc GridDesc "Cell count, size and columns."
  returns ListRange
}
```

<sub>`api.ogt` line 17081</sub>
