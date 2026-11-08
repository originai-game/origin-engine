<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-pagination"></a>

### `og.ui.pagination`

Previous, next and the pages around the current one.

```ts
og.ui.pagination(id: string, desc?: PaginationDesc): PageResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PaginationDesc` | yes | Pages and the current one. |

**Returns** `PageResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn pagination {
  doc "Previous, next and the pages around the current one."
  host false
  param id String "Its id."
  param desc PaginationDesc "Pages and the current one."
  returns PageResult
}
```

<sub>Module `ui` line 2914</sub>
