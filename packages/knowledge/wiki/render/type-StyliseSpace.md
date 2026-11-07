<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-StyliseSpace"></a>

### `StyliseSpace` (enum)

Where Stylise lays a pattern (the paint canvas, the hatching, the halftone).

A string: one of the variants below. The zero value is `"screen"`.

| Variant | Doc |
|---|---|
| `"screen"` | Over the frame in logical pixels, as a print or a sheet of paper over the screen: the pattern stays put while the camera moves and the scene slides under it. |
| `"surface"` | On the surfaces, through the depth-normal prepass's position and normal (three world planes blended by the normal): the pattern sticks to the scene as the camera moves and wraps round forms, a tonal art map's strokes on the objects. Hatching and halftone keep about their pixel spacing by fading between whole octaves of world spacing; the sky keeps the screen pattern. The frame runs the prepass PostEffects.outline uses. |
| `"object"` | On the surfaces as surface, but in each mesh instance's own space (three planes of the instance blended by its normal, in metres along its axes): a moving, turning or skinned object carries its strokes, dots and grain instead of sliding through them, and keeps them however it is scaled. Terrain, scattered copies and the vertices a material shader moves keep the world pattern, the sky the screen pattern. The frame runs the prepass and an object pass that draws the opaque instances again. |

```ogt
enum StyliseSpace {
  doc "Where Stylise lays a pattern (the paint canvas, the hatching, the halftone)."
  variant screen "Over the frame in logical pixels, as a print or a sheet of paper over the screen: the pattern stays put while the camera moves and the scene slides under it."
  variant surface "On the surfaces, through the depth-normal prepass's position and normal (three world planes blended by the normal): the pattern sticks to the scene as the camera moves and wraps round forms, a tonal art map's strokes on the objects. Hatching and halftone keep about their pixel spacing by fading between whole octaves of world spacing; the sky keeps the screen pattern. The frame runs the prepass PostEffects.outline uses."
  variant object "On the surfaces as surface, but in each mesh instance's own space (three planes of the instance blended by its normal, in metres along its axes): a moving, turning or skinned object carries its strokes, dots and grain instead of sliding through them, and keeps them however it is scaled. Terrain, scattered copies and the vertices a material shader moves keep the world pattern, the sky the screen pattern. The frame runs the prepass and an object pass that draws the opaque instances again."
}
```

<sub>Module `render` line 1443</sub>
