<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SpriteFacing"></a>

### `SpriteFacing` (enum)

How a world sprite turns towards the camera that draws it: a billboard (three.js Sprite), a mode of og.Sprite and SpriteDesc. The turn is computed on the GPU from each drawing camera's own axes, so it never feeds back into the simulation. It turns about its anchor (a nine-slice as one panel), `rotation` still turns it in its plane, and its drop shadow is cast by the quad stood upright facing the sun. Without sortY, and tone mapped, it sorts back to front among the transparent meshes by its anchor's view depth; otherwise it keeps the sprite order. World text takes the same modes.

See `render §11`.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | In the world's xy plane facing +z, as every sprite before billboards. |
| `"camera"` | In the camera's view plane: its x along the camera's right, its y along its up, as particle billboards (smoke, glints, impostors seen from any side). |
| `"cameraY"` | Upright, turned about the world's y axis towards the camera: its y stays +y, its x the camera's right laid level. Trees, characters and props that must stand while the camera looks down on them. |

```ogt
enum SpriteFacing {
  doc "How a world sprite turns towards the camera that draws it: a billboard (three.js Sprite), a mode of og.Sprite and SpriteDesc. The turn is computed on the GPU from each drawing camera's own axes, so it never feeds back into the simulation. It turns about its anchor (a nine-slice as one panel), `rotation` still turns it in its plane, and its drop shadow is cast by the quad stood upright facing the sun. Without sortY, and tone mapped, it sorts back to front among the transparent meshes by its anchor's view depth; otherwise it keeps the sprite order. World text takes the same modes."
  guide "render §11"
  variant none "In the world's xy plane facing +z, as every sprite before billboards."
  variant camera "In the camera's view plane: its x along the camera's right, its y along its up, as particle billboards (smoke, glints, impostors seen from any side)."
  variant cameraY "Upright, turned about the world's y axis towards the camera: its y stays +y, its x the camera's right laid level. Trees, characters and props that must stand while the camera looks down on them."
}
```

<sub>Module `render` line 1007</sub>
