<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ScrollAxes"></a>

### `ScrollAxes` (enum)

Which ways a scroll container scrolls.

A string: one of the variants below. The zero value is `"vertical"`.

| Variant | Doc |
|---|---|
| `"vertical"` | Up and down (the default). |
| `"horizontal"` | Left and right. |
| `"both"` | Both ways. |

```ogt
enum ScrollAxes {
  doc "Which ways a scroll container scrolls."
  variant vertical "Up and down (the default)."
  variant horizontal "Left and right."
  variant both "Both ways."
}
```

<sub>`api.ogt` line 15928</sub>
