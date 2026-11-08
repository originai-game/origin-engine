<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BlendMode"></a>

### `BlendMode` (enum)

How a paint mixes with what is under it.

A string: one of the variants below. The zero value is `"normal"`.

| Variant | Doc |
|---|---|
| `"normal"` | Over it. |
| `"multiply"` | Multiplied: darkens. |
| `"screen"` | Screened: lightens. |
| `"add"` | Added: glows. |
| `"overlay"` | Multiply under half grey, screen over it. |
| `"softLight"` | A gentle overlay. |
| `"colorDodge"` | Brightens what is under it by the paint. |

```ogt
enum BlendMode {
  doc "How a paint mixes with what is under it."
  variant normal "Over it."
  variant multiply "Multiplied: darkens."
  variant screen "Screened: lightens."
  variant add "Added: glows."
  variant overlay "Multiply under half grey, screen over it."
  variant softLight "A gentle overlay."
  variant colorDodge "Brightens what is under it by the paint."
}
```

<sub>Module `ui` line 3317</sub>
