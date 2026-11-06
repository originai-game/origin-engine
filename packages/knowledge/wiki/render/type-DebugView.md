<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-DebugView"></a>

### `DebugView` (enum)

What the debug overlay shows besides og.debug's draws (og.debug.setView, RenderOptions.debugView, `debug:<view>`), drawn over the finished frame; the lit image's views (cascades, ao, albedo, iblDiffuse, iblSpecular, direct, aoDepth, aoRaw) draw in place of the lit image instead, for looking at one term of the lighting. Streams add views as they add what to show.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Only og.debug's draws. |
| `"physics"` | The colliders' shapes, joints and contacts as the last physics step left them. |
| `"nav"` | The navigation grids (blocked cells) and navmeshes (polygon edges), and every og.NavAgent's path. |
| `"bounds"` | Every drawn mesh's bounds (its geometry's box at its transform). |
| `"lod"` | Every og.Model tinted by the level of detail it draws (green the full mesh, yellow level 1, orange level 2, red level 3 and beyond), so the levels' distances read in one frame. |
| `"cascades"` | In place of the lit image: each lit pixel coloured by the directional light's shadow cascade that covers it (red, green, blue, yellow for cascades 0 to 3; black beyond the last). |
| `"ao"` | In place of the lit image: the screen-space ambient occlusion term (white unoccluded), with PostEffects.ao set. |
| `"albedo"` | In place of the lit image: the lit materials' base colour after their maps and hooks, unlit. |
| `"iblDiffuse"` | In place of the lit image: the diffuse image-based light (Environment's map or sky) each lit material receives. |
| `"iblSpecular"` | In place of the lit image: the specular image-based light each lit material reflects. |
| `"direct"` | In place of the lit image: the direct light (directional, point and spot lights, shadows applied) each lit material receives. |
| `"aoDepth"` | The ao view with the depth the ambient occlusion pass reads in place of its term, as its fraction of a metre (one ramp per metre), to check the depth the AO sees. |
| `"aoRaw"` | The ao view before the term's depth-aware blur, to tell the search's noise from the blur's. |
| `"terrain"` | Terrain projection weights: XYZ as RGB (green is planar XZ). Dark lines show clipmap cells and white marks the outer morph band. dump_entity reports the clipmap regions and their triangle spacing. |
| `"terrainLayers"` | Terrain's strongest raw layer (control times slope and altitude), coloured by index; magenta means the first-layer fallback. Brightness shows its share before height blending. |
| `"terrainMip"` | Terrain normal-map mip footprint: one colour per estimated isotropic mip, darkening between levels. This is a derivative-based estimate, not the hardware anisotropic mip selection. |
| `"terrainProjection"` | Dominant raw terrain layer's actual projection weights, XYZ as RGB, without cell or morph overlays. Planar layers are green (top-down Y projection). |
| `"terrainDensity"` | Dominant raw terrain layer's vertical albedo texel footprints for X/Y/Z projections as RGB. Each linear channel is d/(1+d), where d is base-level texels per vertical screen pixel; 0.5 means one texel per pixel. Density is measured before projection weighting; a missing albedo shows the unsupported checker. |

```ogt
enum DebugView {
  doc "What the debug overlay shows besides og.debug's draws (og.debug.setView, RenderOptions.debugView, `debug:<view>`), drawn over the finished frame; the lit image's views (cascades, ao, albedo, iblDiffuse, iblSpecular, direct, aoDepth, aoRaw) draw in place of the lit image instead, for looking at one term of the lighting. Streams add views as they add what to show."
  variant none "Only og.debug's draws."
  variant physics "The colliders' shapes, joints and contacts as the last physics step left them."
  variant nav "The navigation grids (blocked cells) and navmeshes (polygon edges), and every og.NavAgent's path."
  variant bounds "Every drawn mesh's bounds (its geometry's box at its transform)."
  variant lod "Every og.Model tinted by the level of detail it draws (green the full mesh, yellow level 1, orange level 2, red level 3 and beyond), so the levels' distances read in one frame."
  variant cascades "In place of the lit image: each lit pixel coloured by the directional light's shadow cascade that covers it (red, green, blue, yellow for cascades 0 to 3; black beyond the last)."
  variant ao "In place of the lit image: the screen-space ambient occlusion term (white unoccluded), with PostEffects.ao set."
  variant albedo "In place of the lit image: the lit materials' base colour after their maps and hooks, unlit."
  variant iblDiffuse "In place of the lit image: the diffuse image-based light (Environment's map or sky) each lit material receives."
  variant iblSpecular "In place of the lit image: the specular image-based light each lit material reflects."
  variant direct "In place of the lit image: the direct light (directional, point and spot lights, shadows applied) each lit material receives."
  variant aoDepth "The ao view with the depth the ambient occlusion pass reads in place of its term, as its fraction of a metre (one ramp per metre), to check the depth the AO sees."
  variant aoRaw "The ao view before the term's depth-aware blur, to tell the search's noise from the blur's."
  variant terrain "Terrain projection weights: XYZ as RGB (green is planar XZ). Dark lines show clipmap cells and white marks the outer morph band. dump_entity reports the clipmap regions and their triangle spacing."
  variant terrainLayers "Terrain's strongest raw layer (control times slope and altitude), coloured by index; magenta means the first-layer fallback. Brightness shows its share before height blending."
  variant terrainMip "Terrain normal-map mip footprint: one colour per estimated isotropic mip, darkening between levels. This is a derivative-based estimate, not the hardware anisotropic mip selection."
  variant terrainProjection "Dominant raw terrain layer's actual projection weights, XYZ as RGB, without cell or morph overlays. Planar layers are green (top-down Y projection)."
  variant terrainDensity "Dominant raw terrain layer's vertical albedo texel footprints for X/Y/Z projections as RGB. Each linear channel is d/(1+d), where d is base-level texels per vertical screen pixel; 0.5 means one texel per pixel. Density is measured before projection weighting; a missing albedo shows the unsupported checker."
}
```

<sub>`api.ogt` line 3850</sub>
