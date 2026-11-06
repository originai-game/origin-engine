<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Direction"></a>

### `Direction` (enum)

Which way a container lays out its children.

A string: one of the variants below. The zero value is `"row"`.

| Variant | Doc |
|---|---|
| `"row"` | Left to right. |
| `"column"` | Top to bottom. |

```ogt
enum Direction {
  doc "Which way a container lays out its children."
  variant row "Left to right."
  variant column "Top to bottom."
}
```

<sub>`api.ogt` line 15645</sub>
