<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-CsgOp"></a>

### `CsgOp` (enum)

A boolean operation on two solids.

A string: one of the variants below. The zero value is `"union"`.

| Variant | Doc |
|---|---|
| `"union"` | Everything in either: a + b. |
| `"subtract"` | What of a is not in b: a - b (holes, notches, a dice's pips). |
| `"intersect"` | What is in both. |

```ogt
enum CsgOp {
  doc "A boolean operation on two solids."
  variant union "Everything in either: a + b."
  variant subtract "What of a is not in b: a - b (holes, notches, a dice's pips)."
  variant intersect "What is in both."
}
```

<sub>Module `render` line 3613</sub>
