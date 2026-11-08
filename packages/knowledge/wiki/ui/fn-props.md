<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-props"></a>

### `og.ui.props`

Give the element built just before (any widget, or a container just closed) the element model's props: transform, opacity, z, placement, grid, variants, overlaps, motion, states, presence, gestures (docs/ui.md §2, §3.1). Call it right after the element.

```ts
og.ui.props(desc?: ElementProps): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `ElementProps` | yes | Its props. |

**Returns** `boolean`: False when no element was built before it.

Binding symbol `og_ui_props` · publish builds: kept

```ogt
fn props {
  doc "Give the element built just before (any widget, or a container just closed) the element model's props: transform, opacity, z, placement, grid, variants, overlaps, motion, states, presence, gestures (docs/ui.md §2, §3.1). Call it right after the element."
  param desc ElementProps "Its props."
  returns bool "False when no element was built before it."
}
```

<sub>Module `ui` line 5094</sub>
