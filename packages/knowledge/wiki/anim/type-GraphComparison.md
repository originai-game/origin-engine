<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphComparison"></a>

### `GraphComparison` (enum)

Compare a graph parameter to a number.

A string: one of the variants below. The zero value is `"gt"`.

| Variant | Doc |
|---|---|
| `"gt"` | Greater than. |
| `"ge"` | Greater than or equal. |
| `"lt"` | Less than. |
| `"le"` | Less than or equal. |
| `"eq"` | Equal. |
| `"ne"` | Not equal. |

```ogt
enum GraphComparison {
  doc "Compare a graph parameter to a number."
  variant gt "Greater than."
  variant ge "Greater than or equal."
  variant lt "Less than."
  variant le "Less than or equal."
  variant eq "Equal."
  variant ne "Not equal."
}
```

<sub>Module `anim` line 694</sub>
