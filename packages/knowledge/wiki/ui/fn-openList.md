<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openList"></a>

### `og.ui.openList`

Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList().

```ts
og.ui.openList(id: string, desc?: ListDesc): ListRange
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ListDesc` | yes | Row count and size. |

**Returns** `ListRange`.

Binding symbol `og_ui_open_list` · publish builds: kept

```ogt
fn openList {
  doc "Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList()."
  param id String "Its id."
  param desc ListDesc "Row count and size."
  returns ListRange
}
```

<sub>Module `ui` line 1449</sub>
