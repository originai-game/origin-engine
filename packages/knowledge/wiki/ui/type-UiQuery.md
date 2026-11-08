<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiQuery"></a>

### `UiQuery` (desc struct)

When a variant applies: every condition given must hold.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `breakpoints` | `readonly Breakpoint[]` | `[]` | — | — | — | The size classes it applies on (empty: any). |
| `orientation` | `Orientation \| null` | `null` | — | — | — | The orientation it applies in; null is either. |
| `minWidth` | `number` | `0` | — | `cssPx` | — | The narrowest width it applies at (0: unbounded): the viewport's, or with container the element's own laid-out width. |
| `maxWidth` | `number` | `0` | — | `cssPx` | — | The widest (0: unbounded). |
| `container` | `boolean` | `false` | — | — | — | A container query: the widths are the element's own (the same card in a four-column shop and a one-column phone list). |

```ogt
struct UiQuery {
  doc "When a variant applies: every condition given must hold."
  desc true
  field breakpoints List.Breakpoint "The size classes it applies on (empty: any)."
  field orientation Option.Orientation "The orientation it applies in; null is either."
  field minWidth f32 "The narrowest width it applies at (0: unbounded): the viewport's, or with container the element's own laid-out width." {
    unit cssPx
  }
  field maxWidth f32 "The widest (0: unbounded)." {
    unit cssPx
  }
  field container bool "A container query: the widths are the element's own (the same card in a four-column shop and a one-column phone list)."
}
```

<sub>Module `ui` line 4529</sub>
