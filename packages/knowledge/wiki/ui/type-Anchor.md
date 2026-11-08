<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Anchor"></a>

### `Anchor` (enum)

A point of the safe area an anchored group sits at.

A string: one of the variants below. The zero value is `"topLeft"`.

| Variant | Doc |
|---|---|
| `"topLeft"` | The top left corner. |
| `"top"` | The middle of the top edge. |
| `"topRight"` | The top right corner. |
| `"left"` | The middle of the left edge. |
| `"center"` | The centre. |
| `"right"` | The middle of the right edge. |
| `"bottomLeft"` | The bottom left corner. |
| `"bottom"` | The middle of the bottom edge. |
| `"bottomRight"` | The bottom right corner. |

```ogt
enum Anchor {
  doc "A point of the safe area an anchored group sits at."
  variant topLeft "The top left corner."
  variant top "The middle of the top edge."
  variant topRight "The top right corner."
  variant left "The middle of the left edge."
  variant center "The centre."
  variant right "The middle of the right edge."
  variant bottomLeft "The bottom left corner."
  variant bottom "The middle of the bottom edge."
  variant bottomRight "The bottom right corner."
}
```

<sub>Module `ui` line 631</sub>
