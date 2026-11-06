<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TextAlign"></a>

### `TextAlign` (enum)

How the lines of a label align.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | To the start (left). |
| `"center"` | Centred. |
| `"end"` | To the end (right). |

```ogt
enum TextAlign {
  doc "How the lines of a label align."
  variant start "To the start (left)."
  variant center "Centred."
  variant end "To the end (right)."
}
```

<sub>`api.ogt` line 15668</sub>
