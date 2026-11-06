<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-listItem"></a>

### `og.ui.listItem`

Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`.

```ts
og.ui.listItem(index: number, selected: boolean): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |
| `selected` | `boolean` | no | Drawn selected (controlStrong). |

**Returns** `Response`.

Binding symbol `og_ui_list_item` · publish builds: kept

```ogt
fn listItem {
  doc "Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`."
  param index u32 "Its index."
  param selected bool "Drawn selected (controlStrong)."
  returns Response
}
```

<sub>`api.ogt` line 17110</sub>
