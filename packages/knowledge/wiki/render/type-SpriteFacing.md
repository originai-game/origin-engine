<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SpriteFacing"></a>

### `SpriteFacing` (enum)

How a world sprite turns towards the camera that draws it: a billboard (three.js Sprite). Decided as a mode of og.Sprite and SpriteDesc rather than a component of its own: a billboard is the same quad, texture, region, flash, glow, blend, layer, drop shadow and texture array layer as any sprite, drawn by the same sprite batch and shader, which only stands its corners differently. The turn is computed on the GPU from each camera's own axes (the main camera, every og.CameraView and og.Mirror, every face of a reflection probe), so it never feeds back into the simulation and a snapshot holds only the mode. The sprite turns about its anchor (a nine-slice turns as one panel), `rotation` still turns it within its own plane, and its drop shadow is cast by the quad stood upright facing the sun, so it shows the whole shape from every camera. A faced sprite without sortY (and tone mapped) draws among the transparent meshes, back to front with them by the view depth of its anchor from the camera that draws it (each face of a reflection probe from the probe), its drop shadow before them, so a billboard behind a glass pane shows through it and one in front draws over it; its layer no longer orders it against other sprites. With sortY, or toneMapped false, it keeps the sprite order (layer, then key) after the transparent meshes and particles. A faced sprite is culled by its bounds (a sphere round the anchor through the farthest corner) against the camera's view. Sprites are depth tested and do not write depth. World text takes the same modes (TextDesc.facing, FloatingTextDesc.facing): it turns as one panel about its position and sorts as a billboard.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | In the world's xy plane facing +z, as every sprite before billboards. |
| `"camera"` | In the camera's view plane: its x along the camera's right, its y along its up, as particle billboards (smoke, glints, impostors seen from any side). |
| `"cameraY"` | Upright, turned about the world's y axis towards the camera: its y stays +y, its x the camera's right laid level. Trees, characters and props that must stand while the camera looks down on them. |

```ogt
enum SpriteFacing {
  doc "How a world sprite turns towards the camera that draws it: a billboard (three.js Sprite). Decided as a mode of og.Sprite and SpriteDesc rather than a component of its own: a billboard is the same quad, texture, region, flash, glow, blend, layer, drop shadow and texture array layer as any sprite, drawn by the same sprite batch and shader, which only stands its corners differently. The turn is computed on the GPU from each camera's own axes (the main camera, every og.CameraView and og.Mirror, every face of a reflection probe), so it never feeds back into the simulation and a snapshot holds only the mode. The sprite turns about its anchor (a nine-slice turns as one panel), `rotation` still turns it within its own plane, and its drop shadow is cast by the quad stood upright facing the sun, so it shows the whole shape from every camera. A faced sprite without sortY (and tone mapped) draws among the transparent meshes, back to front with them by the view depth of its anchor from the camera that draws it (each face of a reflection probe from the probe), its drop shadow before them, so a billboard behind a glass pane shows through it and one in front draws over it; its layer no longer orders it against other sprites. With sortY, or toneMapped false, it keeps the sprite order (layer, then key) after the transparent meshes and particles. A faced sprite is culled by its bounds (a sphere round the anchor through the farthest corner) against the camera's view. Sprites are depth tested and do not write depth. World text takes the same modes (TextDesc.facing, FloatingTextDesc.facing): it turns as one panel about its position and sorts as a billboard."
  variant none "In the world's xy plane facing +z, as every sprite before billboards."
  variant camera "In the camera's view plane: its x along the camera's right, its y along its up, as particle billboards (smoke, glints, impostors seen from any side)."
  variant cameraY "Upright, turned about the world's y axis towards the camera: its y stays +y, its x the camera's right laid level. Trees, characters and props that must stand while the camera looks down on them."
}
```

<sub>Module `render` line 879</sub>
