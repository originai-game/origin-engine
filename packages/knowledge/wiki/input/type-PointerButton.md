<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-PointerButton"></a>

### `PointerButton` (enum)

A physical pointer button; `none` is a hover move.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No button. |
| `"left"` | The primary button, or a touch. |
| `"middle"` | The middle button. |
| `"right"` | The secondary button. |

```ogt
enum PointerButton {
  doc "A physical pointer button; `none` is a hover move."
  variant none "No button."
  variant left "The primary button, or a touch."
  variant middle "The middle button."
  variant right "The secondary button."
}
```

<sub>Module `input` line 625</sub>
