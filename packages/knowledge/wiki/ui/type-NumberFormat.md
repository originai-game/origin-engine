<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NumberFormat"></a>

### `NumberFormat` (enum)

How a number reads in a data view.

A string: one of the variants below. The zero value is `"grouped"`.

| Variant | Doc |
|---|---|
| `"grouped"` | Grouped digits: 12,840 (the default). |
| `"percent"` | A fraction as a percentage: 0.42 is 42%. |
| `"compact"` | Compact units: 12.4K, 3.40M. |
| `"plain"` | As it is. |

```ogt
enum NumberFormat {
  doc "How a number reads in a data view."
  variant grouped "Grouped digits: 12,840 (the default)."
  variant percent "A fraction as a percentage: 0.42 is 42%."
  variant compact "Compact units: 12.4K, 3.40M."
  variant plain "As it is."
}
```

<sub>Module `ui` line 2543</sub>
