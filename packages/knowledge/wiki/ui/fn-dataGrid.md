<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dataGrid"></a>

### `og.ui.dataGrid`

A virtualised grid of cells with its empty, loading and error states.

```ts
og.ui.dataGrid(id: string, desc: DataGridDesc, build: UiItemBuild): DataListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DataGridDesc` | no | Count, cell size, selection and state. |
| `build` | `UiItemBuild` | no | Builds a cell's content by its index. |

**Returns** `DataListResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn dataGrid {
  doc "A virtualised grid of cells with its empty, loading and error states."
  host false
  param id String "Its id."
  param desc DataGridDesc "Count, cell size, selection and state."
  param build UiItemBuild "Builds a cell's content by its index."
  returns DataListResult
}
```

<sub>Module `ui` line 2977</sub>
