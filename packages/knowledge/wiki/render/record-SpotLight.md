<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-SpotLight"></a>

### `og.SpotLight` (component)

A cone of light along the entity's local -Z, as three.js SpotLight.

Token `og.SpotLight`, registered under the schema name `og.SpotLight`; its value type is `SpotLight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `10` | ≥ 0 | — | tunable | The radiance at one metre on the axis. |
| `distance` | `number` | `10` | ≥ 0.01 | `m` | tunable | Where the light reaches zero. |
| `angle` | `number` | `0.5236` | 0 … 1.5708 | `rad` | tunable | The half-angle of the outer cone, as three.js SpotLight.angle. |
| `penumbra` | `number` | `0` | 0 … 1 | `ratio` | — | The fraction of the cone over which the light fades out, as three.js SpotLight.penumbra. |
| `castShadow` | `boolean` | `false` | — | — | — | Cast a shadow map. |
| `cookie` | `Texture` | `0` | — | — | — | A light cookie, as three.js SpotLight.map: the texture projected along the cone (its centre on the axis, its edges at the outer angle, its top towards world up), multiplying the light's colour. 0 is none. A shadowed spot keeps its shadow over the pattern. See `render §3`. |

```ogt
component SpotLight {
  doc "A cone of light along the entity's local -Z, as three.js SpotLight."
  field color Color "The light's linear colour." {
    default 1 1 1
  }
  field intensity f32 "The radiance at one metre on the axis." {
    default 10
    min 0
    tunable true
  }
  field distance f32 "Where the light reaches zero." {
    default 10
    min 0.01
    unit m
    tunable true
  }
  field angle f32 "The half-angle of the outer cone, as three.js SpotLight.angle." {
    default 0.5236
    min 0
    max 1.5708
    unit rad
    tunable true
  }
  field penumbra f32 "The fraction of the cone over which the light fades out, as three.js SpotLight.penumbra." {
    min 0
    max 1
    unit ratio
  }
  field castShadow bool "Cast a shadow map."
  field cookie Texture "A light cookie, as three.js SpotLight.map: the texture projected along the cone (its centre on the axis, its edges at the outer angle, its top towards world up), multiplying the light's colour. 0 is none. A shadowed spot keeps its shadow over the pattern." {
    guide "render §3"
  }
}
```

<sub>Module `render` line 593</sub>
