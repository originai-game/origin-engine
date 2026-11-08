<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-gridItem"></a>

### `og.ui.gridItem`

Open cell `index` of the open grid (`Cell#index`): a slot on control, focusable; its content follows until close(). Drops answer `dropped` as listItem's.

```ts
og.ui.gridItem(index: number, selected: boolean): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |
| `selected` | `boolean` | no | Drawn selected (controlStrong with a primary bar). |

**Returns** `Response`.

Binding symbol `og_ui_grid_item` · publish builds: kept

```ogt
fn gridItem {
  doc "Open cell `index` of the open grid (`Cell#index`): a slot on control, focusable; its content follows until close(). Drops answer `dropped` as listItem's."
  param index u32 "Its index."
  param selected bool "Drawn selected (controlStrong with a primary bar)."
  returns Response
}
```

<sub>Module `ui` line 1143</sub>
