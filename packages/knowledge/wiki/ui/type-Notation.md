<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Notation"></a>

### `Notation` (enum)

How a number is written.

A string: one of the variants below. The zero value is `"grouped"`.

| Variant | Doc |
|---|---|
| `"grouped"` | Every digit, grouped by thousands (12,840,000). |
| `"compact"` | Shortened past 9,999 to three significant digits with K, M, B, T, then scientific (12.8M). |
| `"letters"` | Shortened as idle games do: K, M, B, T, then aa, ab, … az, ba … (1.25ab), never running out. |
| `"scientific"` | Mantissa and exponent past 9,999 (1.28e7). |
| `"engineering"` | As scientific with the exponent a multiple of three (12.8e6). |

```ogt
enum Notation {
  doc "How a number is written."
  variant grouped "Every digit, grouped by thousands (12,840,000)."
  variant compact "Shortened past 9,999 to three significant digits with K, M, B, T, then scientific (12.8M)."
  variant letters "Shortened as idle games do: K, M, B, T, then aa, ab, … az, ba … (1.25ab), never running out."
  variant scientific "Mantissa and exponent past 9,999 (1.28e7)."
  variant engineering "As scientific with the exponent a multiple of three (12.8e6)."
}
```

<sub>`api.ogt` line 21349</sub>
