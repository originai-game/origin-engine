<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Side"></a>

### `Side` (enum)

Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide.

A string: one of the variants below. The zero value is `"front"`.

| Variant | Doc |
|---|---|
| `"front"` | Front faces only. |
| `"back"` | Back faces only. |
| `"double"` | Both. |

```ogt
enum Side {
  doc "Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide."
  variant front "Front faces only."
  variant back "Back faces only."
  variant double "Both."
}
```

<sub>`api.ogt` line 3797</sub>
