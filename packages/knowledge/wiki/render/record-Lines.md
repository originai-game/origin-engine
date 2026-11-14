<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Lines"></a>

### `og.Lines` (component)

Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a track, a constellation. The vertices are taken in order (the geometry's indices are not read), with its vertex colours times color.

Token `og.Lines`, registered under the schema name `og.Lines`; its value type is `Lines`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | The vertices (og.render.bufferGeometry: an unindexed one of any count). |
| `mode` | `LineMode` | `"strip"` | — | — | — | How the vertices join. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour, times the vertex colours. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The alpha. |
| `width` | `number` | `1` | ≥ 0 | — | tunable | CSS pixels, or world units with worldUnits. |
| `worldUnits` | `boolean` | `false` | — | — | — | Width in world units. |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry. |
| `dashSize` | `number` | `0` | ≥ 0 | — | — | Dashes' length in world units along the line; 0 is solid (three.js LineDashedMaterial.dashSize). |
| `gapSize` | `number` | `1` | ≥ 0 | — | — | Gaps' length between dashes. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Mesh.layers: drawn by every camera whose mask shares a bit with it (og.Camera, og.CameraView, og.Mirror). |

```ogt
component Lines {
  doc "Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a track, a constellation. The vertices are taken in order (the geometry's indices are not read), with its vertex colours times color."
  field geometry Geometry "The vertices (og.render.bufferGeometry: an unindexed one of any count)."
  field mode LineMode "How the vertices join."
  field color Color "The linear colour, times the vertex colours." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
    tunable true
  }
  field width f32 "CSS pixels, or world units with worldUnits." {
    default 1
    min 0
    tunable true
  }
  field worldUnits bool "Width in world units."
  field depthTest bool "Hidden behind nearer geometry." {
    default true
  }
  field dashSize f32 "Dashes' length in world units along the line; 0 is solid (three.js LineDashedMaterial.dashSize)." {
    min 0
  }
  field gapSize f32 "Gaps' length between dashes." {
    default 1
    min 0
  }
  field visible bool "Whether it is drawn." {
    default true
  }
  field layers u32 "Three.js layers, as Mesh.layers: drawn by every camera whose mask shares a bit with it (og.Camera, og.CameraView, og.Mirror)." {
    default 1
  }
}
```

<sub>Module `render` line 3848</sub>
