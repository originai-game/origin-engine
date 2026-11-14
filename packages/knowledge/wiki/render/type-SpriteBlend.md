<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SpriteBlend"></a>

### `SpriteBlend` (enum)

How a sprite is blended over what is below it.

A string: one of the variants below. The zero value is `"alpha"`.

| Variant | Doc |
|---|---|
| `"alpha"` | Ordinary alpha blending. |
| `"additive"` | Added (light, fire, magic): order-independent within a layer. |

```ogt
enum SpriteBlend {
  doc "How a sprite is blended over what is below it."
  variant alpha "Ordinary alpha blending."
  variant additive "Added (light, fire, magic): order-independent within a layer."
}
```

<sub>Module `render` line 994</sub>
