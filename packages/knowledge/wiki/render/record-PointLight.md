<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-PointLight"></a>

### `og.PointLight` (component)

A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.

Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it. Up to MAX_LIGHTS in view are drawn (MAX_CLUSTER_LIGHTS a screen cluster), the brightest at the camera kept; RenderSettings.shadowedLights chooses how many with castShadow get shadows, the brightest first.

See `render §3`.

Also known as: omni light, point light, lamp, bulb, Godot OmniLight3D

Token `og.PointLight`, registered under the schema name `og.PointLight`; its value type is `PointLight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `10` | ≥ 0 | — | tunable | The radiance at one metre. |
| `distance` | `number` | `10` | ≥ 0.01 | `m` | tunable | Where the light reaches zero, as three.js PointLight.distance (never infinite here). |
| `castShadow` | `boolean` | `false` | — | — | — | Cast shadows: six cube faces or two paraboloid hemispheres (RenderOptions.pointShadows). |
| `cookie` | `Texture` | `0` | — | — | — | A light cookie projected all around the light: six square faces stacked top to bottom as +X, -X, +Y, -Y, +Z, -Z, six times as tall as wide (a KTX2 cube's face order, a cube map's orientation, world axes). The light's colour is multiplied by the texel its direction meets. 0 is none. See `render §3`. |

```ogt
component PointLight {
  doc "A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance."
  doc "Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it. Up to MAX_LIGHTS in view are drawn (MAX_CLUSTER_LIGHTS a screen cluster), the brightest at the camera kept; RenderSettings.shadowedLights chooses how many with castShadow get shadows, the brightest first."
  aliases "omni light, point light, lamp, bulb, Godot OmniLight3D"
  guide "render §3"
  field color Color "The light's linear colour." {
    default 1 1 1
  }
  field intensity f32 "The radiance at one metre." {
    default 10
    min 0
    tunable true
  }
  field distance f32 "Where the light reaches zero, as three.js PointLight.distance (never infinite here)." {
    default 10
    min 0.01
    unit m
    tunable true
  }
  field castShadow bool "Cast shadows: six cube faces or two paraboloid hemispheres (RenderOptions.pointShadows)."
  field cookie Texture "A light cookie projected all around the light: six square faces stacked top to bottom as +X, -X, +Y, -Y, +Z, -Z, six times as tall as wide (a KTX2 cube's face order, a cube map's orientation, world axes). The light's colour is multiplied by the texel its direction meets. 0 is none." {
    guide "render §3"
  }
}
```

<sub>Module `render` line 515</sub>
