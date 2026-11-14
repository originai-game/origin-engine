<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-DofQuality"></a>

### `DofQuality` (enum)

How many taps depth of field gathers per pixel.

A string: one of the variants below. The zero value is `"low"`.

| Variant | Doc |
|---|---|
| `"low"` | 16 taps at half resolution. |
| `"high"` | 48 taps at half resolution: rounder, smoother bokeh. |

```ogt
enum DofQuality {
  doc "How many taps depth of field gathers per pixel."
  variant low "16 taps at half resolution."
  variant high "48 taps at half resolution: rounder, smoother bokeh."
}
```

<sub>Module `render` line 948</sub>
