<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Points"></a>

### `og.Points` (component)

Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. Vertex colours times color.

Token `og.Points`, registered under the schema name `og.Points`; its value type is `Points`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | The vertices. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour, times the vertex colours. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The alpha. |
| `size` | `number` | `1` | ≥ 0 | — | tunable | World units with sizeAttenuation, else CSS pixels. |
| `sizeAttenuation` | `boolean` | `true` | — | — | — | Size in world units, smaller with distance. |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Mesh.layers: drawn by every camera whose mask shares a bit with it (og.Camera, og.CameraView, og.Mirror). |

```ogt
component Points {
  doc "Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. Vertex colours times color."
  field geometry Geometry "The vertices."
  field color Color "The linear colour, times the vertex colours." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
    tunable true
  }
  field size f32 "World units with sizeAttenuation, else CSS pixels." {
    default 1
    min 0
    tunable true
  }
  field sizeAttenuation bool "Size in world units, smaller with distance." {
    default true
  }
  field depthTest bool "Hidden behind nearer geometry." {
    default true
  }
  field visible bool "Whether it is drawn." {
    default true
  }
  field layers u32 "Three.js layers, as Mesh.layers: drawn by every camera whose mask shares a bit with it (og.Camera, og.CameraView, og.Mirror)." {
    default 1
  }
}
```

<sub>Module `render` line 3854</sub>
