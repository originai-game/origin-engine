<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BezierKey"></a>

### `BezierKey` (struct)

A cubic Bézier into the key.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bezier` | `readonly number[]` | `[]` | — | — | — | x1, y1, x2, y2. |

```ogt
struct BezierKey {
  doc "A cubic Bézier into the key."
  field bezier List.f32 "x1, y1, x2, y2."
}
```

<sub>Module `ui` line 2367</sub>
