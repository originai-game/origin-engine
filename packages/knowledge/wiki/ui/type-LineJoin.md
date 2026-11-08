<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LineJoin"></a>

### `LineJoin` (enum)

Where a line's segments meet.

A string: one of the variants below. The zero value is `"miter"`.

| Variant | Doc |
|---|---|
| `"miter"` | Sharp, cut to a bevel past DrawShape.miterLimit half widths. |
| `"round"` | Rounded. |
| `"bevel"` | Cut across. |

```ogt
enum LineJoin {
  doc "Where a line's segments meet."
  variant miter "Sharp, cut to a bevel past DrawShape.miterLimit half widths."
  variant round "Rounded."
  variant bevel "Cut across."
}
```

<sub>Module `ui` line 6836</sub>
