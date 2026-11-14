<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-AoResolution"></a>

### `AoResolution` (enum)

The resolution ambient occlusion is searched at.

A string: one of the variants below. The zero value is `"full"`.

| Variant | Doc |
|---|---|
| `"full"` | Every pixel searched, then a 4x4 depth-aware blur. |
| `"half"` | Every second pixel in x and y searched (a quarter of the work) and blurred at that resolution, then brought to full resolution by a depth-aware upsample: softer contacts and thin creases, a fraction of the GPU time. |

```ogt
enum AoResolution {
  doc "The resolution ambient occlusion is searched at."
  variant full "Every pixel searched, then a 4x4 depth-aware blur."
  variant half "Every second pixel in x and y searched (a quarter of the work) and blurred at that resolution, then brought to full resolution by a depth-aware upsample: softer contacts and thin creases, a fraction of the GPU time."
}
```

<sub>Module `render` line 955</sub>
