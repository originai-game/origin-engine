<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-RectLight"></a>

### `og.RectLight` (component)

A rectangle of light, as three.js RectAreaLight: width by height metres centred on the entity in its local xy plane, facing along its local -Z, emitting color * intensity from its front (both faces with twoSided). Shaded with linearly transformed cosines (Heitz et al. 2016). og bake lightmaps traces its exact soft shadows into the bounce light, castShadow or not.

Rect lights are clustered with the point and spot lights and share their MAX_LIGHTS budget; each reaches where it would light a white surface facing it at one hundredth of one, fading smoothly to zero there.

See `render §3`.

Token `og.RectLight`, registered under the schema name `og.RectLight`; its value type is `RectLight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | The emitted radiance (luminance), as three.js RectAreaLight.intensity. |
| `width` | `number` | `10` | ≥ 0 | `m` | tunable | Its size along the entity's local x, as RectAreaLight.width. |
| `height` | `number` | `10` | ≥ 0 | `m` | tunable | Its size along the entity's local y, as RectAreaLight.height. |
| `twoSided` | `boolean` | `false` | — | — | — | Emit from the back face as well (a hanging panel lit both ways). |
| `castShadow` | `boolean` | `false` | — | — | — | Cast a soft shadow: a shadow map from the rectangle's centre (two when twoSided, counted in the shadowed lights' budget) read with percentage-closer soft shadows, sharp at contact and widening with the rectangle's size away from it. See `render §3`. |

```ogt
component RectLight {
  doc "A rectangle of light, as three.js RectAreaLight: width by height metres centred on the entity in its local xy plane, facing along its local -Z, emitting color * intensity from its front (both faces with twoSided). Shaded with linearly transformed cosines (Heitz et al. 2016). og bake lightmaps traces its exact soft shadows into the bounce light, castShadow or not."
  doc "Rect lights are clustered with the point and spot lights and share their MAX_LIGHTS budget; each reaches where it would light a white surface facing it at one hundredth of one, fading smoothly to zero there."
  guide "render §3"
  field color Color "The light's linear colour." {
    default 1 1 1
  }
  field intensity f32 "The emitted radiance (luminance), as three.js RectAreaLight.intensity." {
    default 1
    min 0
    tunable true
  }
  field width f32 "Its size along the entity's local x, as RectAreaLight.width." {
    default 10
    min 0
    unit m
    tunable true
  }
  field height f32 "Its size along the entity's local y, as RectAreaLight.height." {
    default 10
    min 0
    unit m
    tunable true
  }
  field twoSided bool "Emit from the back face as well (a hanging panel lit both ways)."
  field castShadow bool "Cast a soft shadow: a shadow map from the rectangle's centre (two when twoSided, counted in the shadowed lights' budget) read with percentage-closer soft shadows, sharp at contact and widening with the rectangle's size away from it." {
    guide "render §3"
  }
}
```

<sub>Module `render` line 4129</sub>
