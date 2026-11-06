<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-column"></a>

### `og.ui.column`

A column built by `build`: open(id, {...desc, direction: column}), build(), close().

```ts
og.ui.column(id: string, desc: ElementDesc, build: UiBuild): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ElementDesc` | no | Its layout and look; direction is column. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `Response`: Its response, for an interactive column.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn column {
  doc "A column built by `build`: open(id, {...desc, direction: column}), build(), close()."
  host false
  param id String "Its id."
  param desc ElementDesc "Its layout and look; direction is column."
  param build UiBuild "Makes its children."
  returns Response "Its response, for an interactive column."
}
```

<sub>`api.ogt` line 16277</sub>
