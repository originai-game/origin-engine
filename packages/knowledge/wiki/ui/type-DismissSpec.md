<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DismissSpec"></a>

### `DismissSpec` (desc struct)

Swipe to dismiss: a release past distance (or faster than velocity) along the drag's axis throws the element off.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `distance` | `number` | `0` | — | `cssPx` | — | 0: 40 % of its size along the axis. |
| `velocity` | `number` | `0` | — | — | — | CSS px per second (0: 800). |

```ogt
struct DismissSpec {
  doc "Swipe to dismiss: a release past distance (or faster than velocity) along the drag's axis throws the element off."
  desc true
  field distance f32 "0: 40 % of its size along the axis." {
    unit cssPx
  }
  field velocity f32 "CSS px per second (0: 800)."
}
```

<sub>Module `ui` line 2093</sub>
