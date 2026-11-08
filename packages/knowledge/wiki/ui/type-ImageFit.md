<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ImageFit"></a>

### `ImageFit` (enum)

How an image fills the element's box.

A string: one of the variants below. The zero value is `"fill"`.

| Variant | Doc |
|---|---|
| `"fill"` | Stretched to the box. |
| `"cover"` | Scaled to cover the box, cropped around the focal point. |
| `"contain"` | Scaled to fit inside the box, placed by the focal point. |
| `"none"` | At its own size (one image pixel per CSS px), placed by the focal point. |
| `"nineSlice"` | Nine-slice: corners kept at slice, edges and centre filled by edges. |
| `"pattern"` | Tiled at tile CSS px, turned, offset and scrolling. |

```ogt
enum ImageFit {
  doc "How an image fills the element's box."
  variant fill "Stretched to the box."
  variant cover "Scaled to cover the box, cropped around the focal point."
  variant contain "Scaled to fit inside the box, placed by the focal point."
  variant none "At its own size (one image pixel per CSS px), placed by the focal point."
  variant nineSlice "Nine-slice: corners kept at slice, edges and centre filled by edges."
  variant pattern "Tiled at tile CSS px, turned, offset and scrolling."
}
```

<sub>Module `ui` line 2560</sub>
