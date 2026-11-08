<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ArtFit"></a>

### `ArtFit` (enum)

How art fills its slot.

A string: one of the variants below. The zero value is `"cover"`.

| Variant | Doc |
|---|---|
| `"cover"` | Fills it, cropped around the focal point (the default). |
| `"contain"` | Fits inside it whole, the rest the slot's ground. |
| `"fill"` | Stretched to it. |

```ogt
enum ArtFit {
  doc "How art fills its slot."
  variant cover "Fills it, cropped around the focal point (the default)."
  variant contain "Fits inside it whole, the rest the slot's ground."
  variant fill "Stretched to it."
}
```

<sub>Module `ui` line 3125</sub>
