<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LineCap"></a>

### `LineCap` (enum)

The end of an open line, an arc or a dash.

A string: one of the variants below. The zero value is `"butt"`.

| Variant | Doc |
|---|---|
| `"butt"` | Cut square at the end. |
| `"round"` | A half disc past the end. |
| `"square"` | Cut square half the width past the end. |

```ogt
enum LineCap {
  doc "The end of an open line, an arc or a dash."
  variant butt "Cut square at the end."
  variant round "A half disc past the end."
  variant square "Cut square half the width past the end."
}
```

<sub>Module `ui` line 2759</sub>
