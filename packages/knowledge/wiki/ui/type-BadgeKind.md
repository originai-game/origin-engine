<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BadgeKind"></a>

### `BadgeKind` (enum)

What a count badge shows.

A string: one of the variants below. The zero value is `"count"`.

| Variant | Doc |
|---|---|
| `"count"` | A number (99+ past max). |
| `"dot"` | A dot: something new, unnumbered. |
| `"label"` | A short word (NEW, HOT, -50%). |

```ogt
enum BadgeKind {
  doc "What a count badge shows."
  variant count "A number (99+ past max)."
  variant dot "A dot: something new, unnumbered."
  variant label "A short word (NEW, HOT, -50%)."
}
```

<sub>Module `ui` line 5380</sub>
