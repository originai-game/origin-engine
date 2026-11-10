<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-StripBlend"></a>

### `StripBlend` (enum)

How a trail, ribbon or beam is blended.

A string: one of the variants below. The zero value is `"additive"`.

| Variant | Doc |
|---|---|
| `"additive"` | Added: glowing streaks, lasers, magic. |
| `"alpha"` | Alpha blended: smoke trails, tyre marks. |

```ogt
enum StripBlend {
  doc "How a trail, ribbon or beam is blended."
  variant additive "Added: glowing streaks, lasers, magic."
  variant alpha "Alpha blended: smoke trails, tyre marks."
}
```

<sub>Module `render` line 902</sub>
