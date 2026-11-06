<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-grid"></a>

### `og.ui.grid`

A virtualised grid built by `build(index)` for the cells in view.

```ts
og.ui.grid(id: string, desc: GridDesc, build: UiItemBuild): ListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `GridDesc` | no | Cell count, size, columns and the selected cell. |
| `build` | `UiItemBuild` | no | Makes one cell's content. |

**Returns** `ListResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn grid {
  doc "A virtualised grid built by `build(index)` for the cells in view."
  host false
  param id String "Its id."
  param desc GridDesc "Cell count, size, columns and the selected cell."
  param build UiItemBuild "Makes one cell's content."
  returns ListResult
}
```

<sub>`api.ogt` line 17137</sub>
