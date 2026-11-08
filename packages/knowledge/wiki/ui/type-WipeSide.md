<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-WipeSide"></a>

### `WipeSide` (enum)

Where a wipe or a slide comes from.

A string: one of the variants below. The zero value is `"left"`.

| Variant | Doc |
|---|---|
| `"left"` | From the left. |
| `"right"` | From the right. |
| `"top"` | From the top. |
| `"bottom"` | From the bottom. |

```ogt
enum WipeSide {
  doc "Where a wipe or a slide comes from."
  variant left "From the left."
  variant right "From the right."
  variant top "From the top."
  variant bottom "From the bottom."
}
```

<sub>Module `ui` line 1894</sub>
