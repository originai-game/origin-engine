<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-StyliseSpace"></a>

### `StyliseSpace` (enum)

Where Stylise lays a pattern (the paint canvas, the hatching, the halftone).

See `looks §8`.

A string: one of the variants below. The zero value is `"screen"`.

| Variant | Doc |
|---|---|
| `"screen"` | Over the frame in logical pixels, as a print or a sheet of paper over the screen: the pattern stays put while the camera moves and the scene slides under it. |
| `"surface"` | On the surfaces, through the depth-normal prepass's position and normal (three world planes blended by the normal): the pattern sticks to the scene as the camera moves and wraps round forms. The frame runs the prepass. |
| `"object"` | As surface, but in each mesh instance's own space (three planes blended by the instance's normal, in metres along its axes): a moving, turning, scaled or skinned object carries its pattern. Terrain, scattered copies and shader-moved vertices keep the world pattern. It adds an object pass. |

```ogt
enum StyliseSpace {
  doc "Where Stylise lays a pattern (the paint canvas, the hatching, the halftone)."
  variant screen "Over the frame in logical pixels, as a print or a sheet of paper over the screen: the pattern stays put while the camera moves and the scene slides under it."
  variant surface "On the surfaces, through the depth-normal prepass's position and normal (three world planes blended by the normal): the pattern sticks to the scene as the camera moves and wraps round forms. The frame runs the prepass."
  variant object "As surface, but in each mesh instance's own space (three planes blended by the instance's normal, in metres along its axes): a moving, turning, scaled or skinned object carries its pattern. Terrain, scattered copies and shader-moved vertices keep the world pattern. It adds an object pass."
  guide "looks §8"
}
```

<sub>Module `render` line 1538</sub>
