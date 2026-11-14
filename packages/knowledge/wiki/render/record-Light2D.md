<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Light2D"></a>

### `og.Light2D` (component)

A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff (faded over its last tenth, so even a falloff below 1 ends without a rim), shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn.

Token `og.Light2D`, registered under the schema name `og.Light2D`; its value type is `Light2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | The brightness at the centre. |
| `radius` | `number` | `5` | ≥ 0 | `m` | tunable | Where the light reaches zero, in world units. |
| `falloff` | `number` | `2` | ≥ 0.1 | — | — | The exponent of the fade. |
| `sourceRadius` | `number` | `0.25` | ≥ 0 | `m` | — | The light's own size: larger gives softer shadows. |
| `castShadows` | `boolean` | `true` | — | — | — | Occluders between the light and a pixel shade it (point and spot lights). |
| `kind` | `Light2DKind` | `"point"` | — | — | — | Point, spot or global. |
| `innerAngle` | `number` | `0.35` | 0 … 3.14159 | — | — | A spot's half-angle at full intensity, in radians. |
| `outerAngle` | `number` | `0.6` | 0 … 3.14159 | — | — | A spot's half-angle where it reaches zero, in radians (at least innerAngle). |
| `height` | `number` | `1` | ≥ 0 | `m` | — | How far above the 2D plane the light is, in world units: the slant normal-mapped sprites see it at (0 grazes the surface). |
| `layers` | `number` | `4294967295` | — | — | — | The light layers it lights: a bit per layer, matched against each sprite's, rig's or tilemap's og.LightMask2D (layer 0, bit 1, without one). |

```ogt
component Light2D {
  doc "A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff (faded over its last tenth, so even a falloff below 1 ends without a rim), shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn."
  field color Color "The light's linear colour." {
    default 1 1 1
  }
  field intensity f32 "The brightness at the centre." {
    default 1
    min 0
    tunable true
  }
  field radius f32 "Where the light reaches zero, in world units." {
    default 5
    min 0
    unit m
    tunable true
  }
  field falloff f32 "The exponent of the fade." {
    default 2
    min 0.1
  }
  field sourceRadius f32 "The light's own size: larger gives softer shadows." {
    default 0.25
    min 0
    unit m
  }
  field castShadows bool "Occluders between the light and a pixel shade it (point and spot lights)." {
    default true
  }
  field kind Light2DKind "Point, spot or global."
  field innerAngle f32 "A spot's half-angle at full intensity, in radians." {
    default 0.35
    min 0
    max 3.14159
  }
  field outerAngle f32 "A spot's half-angle where it reaches zero, in radians (at least innerAngle)." {
    default 0.6
    min 0
    max 3.14159
  }
  field height f32 "How far above the 2D plane the light is, in world units: the slant normal-mapped sprites see it at (0 grazes the surface)." {
    default 1
    min 0
    unit m
  }
  field layers u32 "The light layers it lights: a bit per layer, matched against each sprite's, rig's or tilemap's og.LightMask2D (layer 0, bit 1, without one)." {
    default 4294967295
  }
}
```

<sub>Module `render` line 2058</sub>
