<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StrokeAlign"></a>

### `StrokeAlign` (enum)

Which side of the outline a stroke lies on.

A string: one of the variants below. The zero value is `"center"`.

| Variant | Doc |
|---|---|
| `"center"` | Across the outline. |
| `"inside"` | Inside it. |
| `"outside"` | Outside it. |

```ogt
enum StrokeAlign {
  doc "Which side of the outline a stroke lies on."
  variant center "Across the outline."
  variant inside "Inside it."
  variant outside "Outside it."
}
```

<sub>Module `ui` line 7235</sub>
