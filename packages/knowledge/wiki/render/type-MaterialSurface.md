<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-MaterialSurface"></a>

### `MaterialSurface` (enum)

A made surface MaterialStyle.surface lays on every lit material: the hand-made materials of the shape styles, drawn by the material's shading itself (its normal, its colour and how it takes the light) in each mesh's own space, so a moving object carries it.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No surface: each material as declared. |
| `"clay"` | Modelling clay: soft lumps and thumb-pressed dimples bend the normal, the light wraps past the terminator as through a soft body and warms there, and the surface stays matte (roughness at least 0.75). |
| `"paper"` | Paper and card: a fibre grain darkens and lifts the colour in fine streaks and bends the normal so lit faces catch it, the light wraps a little as through a sheet, and the surface is fully rough. |
| `"plastic"` | Toy plastic or painted wood: a satin sheen (roughness held between 0.3 and 0.45, no metal), a soft wrap of the light and a faint moulding ripple, so pieces read as small and handled. |

```ogt
enum MaterialSurface {
  doc "A made surface MaterialStyle.surface lays on every lit material: the hand-made materials of the shape styles, drawn by the material's shading itself (its normal, its colour and how it takes the light) in each mesh's own space, so a moving object carries it."
  variant none "No surface: each material as declared."
  variant clay "Modelling clay: soft lumps and thumb-pressed dimples bend the normal, the light wraps past the terminator as through a soft body and warms there, and the surface stays matte (roughness at least 0.75)."
  variant paper "Paper and card: a fibre grain darkens and lifts the colour in fine streaks and bends the normal so lit faces catch it, the light wraps a little as through a sheet, and the surface is fully rough."
  variant plastic "Toy plastic or painted wood: a satin sheen (roughness held between 0.3 and 0.45, no metal), a soft wrap of the light and a faint moulding ripple, so pieces read as small and handled."
}
```

<sub>`api.ogt` line 5173</sub>
