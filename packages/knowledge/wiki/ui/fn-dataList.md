<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dataList"></a>

### `og.ui.dataList`

A virtualised list with its empty, loading and error states.

```ts
og.ui.dataList(id: string, desc: DataListDesc, build: UiItemBuild): DataListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DataListDesc` | no | Count, size, selection and state. |
| `build` | `UiItemBuild` | no | Builds a row's content by its index. |

**Returns** `DataListResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn dataList {
  doc "A virtualised list with its empty, loading and error states."
  host false
  param id String "Its id."
  param desc DataListDesc "Count, size, selection and state."
  param build UiItemBuild "Builds a row's content by its index."
  returns DataListResult
}
```

<sub>`api.ogt` line 18825</sub>
