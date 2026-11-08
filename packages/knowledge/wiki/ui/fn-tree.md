<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-tree"></a>

### `og.ui.tree`

A tree of expandable items, virtualised.

```ts
og.ui.tree(id: string, desc?: TreeDesc): TreeResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `TreeDesc` | yes | Items and the open ones. |

**Returns** `TreeResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn tree {
  doc "A tree of expandable items, virtualised."
  host false
  param id String "Its id."
  param desc TreeDesc "Items and the open ones."
  returns TreeResult
}
```

<sub>Module `ui` line 2522</sub>
