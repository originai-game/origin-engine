<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dragging"></a>

### `og.ui.dragging`

The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer.

```ts
og.ui.dragging(id: string): number | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The list's or grid's id. |

**Returns** `number | null`.

Binding symbol `og_ui_dragging` · publish builds: kept

```ogt
fn dragging {
  doc "The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer."
  param id String "The list's or grid's id."
  returns Option.u32
}
```

<sub>Module `ui` line 1388</sub>
