<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-A11yTree"></a>

### `A11yTree` (struct)

The last frame's accessibility tree, built from the same nodes the UI draws and takes input on.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nodes` | `readonly A11yNode[]` | `[]` | — | — | — | The nodes, in tree order: the page, then each layer over it (a dialog after the page). |
| `focusOrder` | `readonly string[]` | `[]` | — | — | — | The paths keyboard Tab and the pads visit, in order: the nodes marked focusable, within the topmost dialog while one is open. |
| `focused` | `string` | `""` | — | — | — | The path keyboard and pad 0's focus is on; empty for none. |

```ogt
struct A11yTree {
  doc "The last frame's accessibility tree, built from the same nodes the UI draws and takes input on."
  field nodes List.A11yNode "The nodes, in tree order: the page, then each layer over it (a dialog after the page)."
  field focusOrder List.String "The paths keyboard Tab and the pads visit, in order: the nodes marked focusable, within the topmost dialog while one is open."
  field focused String "The path keyboard and pad 0's focus is on; empty for none."
}
```

<sub>Module `ui` line 1250</sub>
