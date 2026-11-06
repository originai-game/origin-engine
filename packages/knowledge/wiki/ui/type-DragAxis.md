<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DragAxis"></a>

### `DragAxis` (enum)

Which way a drag moves its element.

A string: one of the variants below. The zero value is `"both"`.

| Variant | Doc |
|---|---|
| `"both"` | Both ways. |
| `"x"` | Across only. |
| `"y"` | Down only. |
| `"lock"` | Either, locked to the first one the drag moves along. |

```ogt
enum DragAxis {
  doc "Which way a drag moves its element."
  variant both "Both ways."
  variant x "Across only."
  variant y "Down only."
  variant lock "Either, locked to the first one the drag moves along."
}
```

<sub>`api.ogt` line 20136</sub>
