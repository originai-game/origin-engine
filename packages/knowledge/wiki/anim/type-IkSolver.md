<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-IkSolver"></a>

### `IkSolver` (enum)

How an IK chain is solved.

A string: one of the variants below. The zero value is `"twoBone"`.

| Variant | Doc |
|---|---|
| `"twoBone"` | Analytic two-bone (thigh, shin, foot), bending towards the pole. |
| `"fabrik"` | FABRIK over `bones` bones, iteratively. |

```ogt
enum IkSolver {
  doc "How an IK chain is solved."
  variant twoBone "Analytic two-bone (thigh, shin, foot), bending towards the pole."
  variant fabrik "FABRIK over `bones` bones, iteratively."
}
```

<sub>`api.ogt` line 11524</sub>
