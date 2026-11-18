<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Blend"></a>

### `Blend` (enum)

How a sprite, trail, ribbon or beam is blended over what is below it. Sprites default to alpha; trails, ribbons and beams to additive.

A string: one of the variants below. The zero value is `"alpha"`.

| Variant | Doc |
|---|---|
| `"alpha"` | Ordinary alpha blending: smoke, dust, tyre marks, solid shapes. |
| `"additive"` | Added (light, fire, magic, glowing streaks, lasers): order-independent within a layer. |

```ogt
enum Blend {
  doc "How a sprite, trail, ribbon or beam is blended over what is below it. Sprites default to alpha; trails, ribbons and beams to additive."
  variant alpha "Ordinary alpha blending: smoke, dust, tyre marks, solid shapes."
  variant additive "Added (light, fire, magic, glowing streaks, lasers): order-independent within a layer."
}
```

<sub>Module `render` line 1039</sub>
