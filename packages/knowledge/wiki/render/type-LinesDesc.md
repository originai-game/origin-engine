<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-LinesDesc"></a>

### `LinesDesc` (desc struct)

Lines through points, drawn this frame only (immediate mode, as og.render.sprite): submit them every tick they should show. three.js Line, LineSegments and LineLoop with a LineBasicMaterial or LineDashedMaterial, but any width (each segment is a quad in screen space with a feathered edge; WebGPU has no line width).

World lines are drawn in the scene after the transparent meshes and particles, exposed, bloomed and tone mapped with it; screen lines over the finished frame, after every post effect and under the UI.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | `m` | — | The points, in order: world positions, or with screen CSS pixels from the top left of the play surface with y down (z ignored). |
| `colors` | `readonly Color[]` | `[]` | — | — | — | Per point linear colours, blended along each segment (three.js vertexColors); empty is `color` for all. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `mode` | `LineMode` | `"strip"` | — | — | — | How the points join. |
| `width` | `number` | `1` | ≥ 0 | — | — | The width: CSS pixels, or world units with worldUnits. |
| `worldUnits` | `boolean` | `false` | — | — | — | Measure width in world units (a line that thins with distance) instead of CSS pixels (as wide at any distance). |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry, as three.js Material.depthTest; false draws it over the scene. Screen lines are never depth tested. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame: points in CSS pixels, width in CSS pixels. |
| `dashed` | `boolean` | `false` | — | — | — | Dashes, as three.js LineDashedMaterial: dashSize on, gapSize off, along the line from its first point. |
| `dashSize` | `number` | `3` | ≥ 0 | — | — | A dash's length: world units (CSS pixels for screen lines). |
| `gapSize` | `number` | `1` | ≥ 0 | — | — | A gap's length. |

```ogt
struct LinesDesc {
  doc "Lines through points, drawn this frame only (immediate mode, as og.render.sprite): submit them every tick they should show. three.js Line, LineSegments and LineLoop with a LineBasicMaterial or LineDashedMaterial, but any width (each segment is a quad in screen space with a feathered edge; WebGPU has no line width)."
  doc "World lines are drawn in the scene after the transparent meshes and particles, exposed, bloomed and tone mapped with it; screen lines over the finished frame, after every post effect and under the UI."
  desc true
  field points List.Vector3 "The points, in order: world positions, or with screen CSS pixels from the top left of the play surface with y down (z ignored)." {
    unit m
  }
  field colors List.Color "Per point linear colours, blended along each segment (three.js vertexColors); empty is `color` for all."
  field color Color "The linear colour." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field mode LineMode "How the points join."
  field width f32 "The width: CSS pixels, or world units with worldUnits." {
    default 1
    min 0
  }
  field worldUnits bool "Measure width in world units (a line that thins with distance) instead of CSS pixels (as wide at any distance)."
  field depthTest bool "Hidden behind nearer geometry, as three.js Material.depthTest; false draws it over the scene. Screen lines are never depth tested." {
    default true
  }
  field screen bool "Draw in screen space over the finished frame: points in CSS pixels, width in CSS pixels."
  field dashed bool "Dashes, as three.js LineDashedMaterial: dashSize on, gapSize off, along the line from its first point."
  field dashSize f32 "A dash's length: world units (CSS pixels for screen lines)." {
    default 3
    min 0
  }
  field gapSize f32 "A gap's length." {
    default 1
    min 0
  }
}
```

<sub>`api.ogt` line 7264</sub>
