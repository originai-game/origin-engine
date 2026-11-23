<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="type-CloudKind"></a>

### `CloudKind` (enum)

What kind of cloud og.Clouds' volumetric layer holds: the shape its clouds take, whatever their cover.

A string: one of the variants below. The zero value is `"cumulus"`.

| Variant | Doc |
|---|---|
| `"cumulus"` | Heaped clouds on a flat base, their tops domes of billows: fair weather at a low cover, towering at a high one. |
| `"stratus"` | A flat, even sheet low in the layer: a grey overcast, or a sea of cloud seen from above. |
| `"stratocumulus"` | A low sheet of rolls and patches with gaps between them. |
| `"cumulonimbus"` | Towers through the whole layer, the strongest spreading into an anvil under its top: give the layer 6 to 10 km of thickness. |

```ogt
enum CloudKind {
  doc "What kind of cloud og.Clouds' volumetric layer holds: the shape its clouds take, whatever their cover."
  variant cumulus "Heaped clouds on a flat base, their tops domes of billows: fair weather at a low cover, towering at a high one."
  variant stratus "A flat, even sheet low in the layer: a grey overcast, or a sea of cloud seen from above."
  variant stratocumulus "A low sheet of rolls and patches with gaps between them."
  variant cumulonimbus "Towers through the whole layer, the strongest spreading into an anvil under its top: give the layer 6 to 10 km of thickness."
}
```

<sub>Module `atmosphere` line 350</sub>
