<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Mirror"></a>

### `og.Mirror` (component)

A planar mirror: the main camera reflected in the entity's xy plane (its local +Z the mirror's facing side), drawn into target fitted to a width by height rect centred on the entity, as a mirrored og.CameraView. Give the mirror's mesh (an og.render.planeGeometry of the same size) a material that reads the target: an hdr target as emissiveMap with color black, or an ldr target as an unlit map. Nothing behind the glass is drawn; seen from behind, it draws nothing.

Token `og.Mirror`, registered under the schema name `og.Mirror`; its value type is `Mirror`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `RenderTarget` | `0` | — | — | — | The target it draws into. |
| `width` | `number` | `1` | ≥ 0.001 | `m` | — | The mirror's width along the entity's local +X. |
| `height` | `number` | `1` | ≥ 0.001 | `m` | — | Its height along local +Y. |
| `layers` | `number` | `1` | — | — | — | Three.js layers it reflects, as og.Camera.layers. |
| `every` | `number` | `1` | — | — | — | Draw once every this many ticks, as CameraView.every. |
| `active` | `boolean` | `true` | — | — | — | Whether it draws. |

```ogt
component Mirror {
  doc "A planar mirror: the main camera reflected in the entity's xy plane (its local +Z the mirror's facing side), drawn into target fitted to a width by height rect centred on the entity, as a mirrored og.CameraView. Give the mirror's mesh (an og.render.planeGeometry of the same size) a material that reads the target: an hdr target as emissiveMap with color black, or an ldr target as an unlit map. Nothing behind the glass is drawn; seen from behind, it draws nothing."
  field target RenderTarget "The target it draws into."
  field width f32 "The mirror's width along the entity's local +X." {
    default 1
    min 0.001
    unit m
  }
  field height f32 "Its height along local +Y." {
    default 1
    min 0.001
    unit m
  }
  field layers u32 "Three.js layers it reflects, as og.Camera.layers." {
    default 1
  }
  field every u32 "Draw once every this many ticks, as CameraView.every." {
    default 1
  }
  field active bool "Whether it draws." {
    default true
  }
}
```

<sub>Module `render` line 4066</sub>
