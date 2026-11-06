<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-list"></a>

### `og.ui.list`

A virtualised list built by `build(index)` for the rows in view.

```ts
og.ui.list(id: string, desc: ListDesc, build: UiItemBuild): ListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ListDesc` | no | Row count, size and the selected row. |
| `build` | `UiItemBuild` | no | Makes one row's content. |

**Returns** `ListResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn list {
  doc "A virtualised list built by `build(index)` for the rows in view."
  host false
  param id String "Its id."
  param desc ListDesc "Row count, size and the selected row."
  param build UiItemBuild "Makes one row's content."
  returns ListResult
}
```

<sub>`api.ogt` line 17128</sub>
