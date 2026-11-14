<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-DirectionalLight"></a>

### `og.DirectionalLight` (component)

The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with RenderSettings.shadowCascades shadow cascades (3 by default).

Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene.

See `render §3`.

Also known as: sun, directional light, Godot DirectionalLight3D

Token `og.DirectionalLight`, registered under the schema name `og.DirectionalLight`; its value type is `DirectionalLight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `3` | ≥ 0 | — | tunable | The radiance multiplier, as three.js DirectionalLight.intensity. |
| `castShadow` | `boolean` | `true` | — | — | — | Cast shadows through the cascades (RenderSettings.shadowCascades), as three.js Light.castShadow. |
| `shadowDistance` | `number` | `200` | ≥ 1 | `m` | — | How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper. |
| `shadowSoftness` | `number` | `0` | 0 … 0.2 | — | tunable | Soft shadows (PCSS): the penumbra's width per metre between caster and receiver, the tangent of the sun's apparent radius (the real sun's is 0.005). 0 keeps the hard one-texel edge; 0.01 to 0.05 is a soft sun, sharp at contact and blurring with distance. |
| `shadowBox` | `number` | `0` | ≥ 0 | `m` | tunable | One shadow map over a fixed box instead of the cascades: its half size across the light. 0 keeps the cascades. Everything outside the box is lit; the casters are drawn once, not once per cascade (three.js games that fit the light's orthographic shadow camera around the player draw this way). |
| `shadowFocus` | `number` | `0` | — | `m` | tunable | With shadowBox: how far ahead of the camera (along its level forward) the box is centred. |
| `shadowFocusDrop` | `number` | `0` | — | `m` | tunable | With shadowBox: how far below the camera the box's centre sits (the ground under a first-person eye). |
| `shadowBias` | `number` | `0` | ≥ 0 | `m` | tunable | With shadowBox: the depth bias toward the light (three.js shadow.bias times its camera's depth range). 0 is one texel. Leaf cards need a few centimetres or they shadow each other dark. |
| `shadowNormalBias` | `number` | `0` | ≥ 0 | `m` | tunable | With shadowBox: how far a receiver is pushed along its normal before it reads the map (three.js shadow.normalBias). 0 is one texel. |
| `cascadeBlend` | `number` | `0` | 0 … 0.5 | `ratio` | tunable | The fraction of each shadow cascade's depth over which it fades into the next, hiding the seam where the shadow's resolution changes. 0 is none; 0.1 to 0.2 is typical (those pixels read two cascades). |

```ogt
component DirectionalLight {
  doc "The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with RenderSettings.shadowCascades shadow cascades (3 by default)."
  doc "Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene."
  aliases "sun, directional light, Godot DirectionalLight3D"
  guide "render §3"
  field color Color "The light's linear colour." {
    default 1 1 1
  }
  field intensity f32 "The radiance multiplier, as three.js DirectionalLight.intensity." {
    default 3
    min 0
    tunable true
  }
  field castShadow bool "Cast shadows through the cascades (RenderSettings.shadowCascades), as three.js Light.castShadow." {
    default true
  }
  field shadowDistance f32 "How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper." {
    default 200
    min 1
    unit m
  }
  field shadowSoftness f32 "Soft shadows (PCSS): the penumbra's width per metre between caster and receiver, the tangent of the sun's apparent radius (the real sun's is 0.005). 0 keeps the hard one-texel edge; 0.01 to 0.05 is a soft sun, sharp at contact and blurring with distance." {
    min 0
    max 0.2
    tunable true
  }
  field shadowBox f32 "One shadow map over a fixed box instead of the cascades: its half size across the light. 0 keeps the cascades. Everything outside the box is lit; the casters are drawn once, not once per cascade (three.js games that fit the light's orthographic shadow camera around the player draw this way)." {
    min 0
    unit m
    tunable true
  }
  field shadowFocus f32 "With shadowBox: how far ahead of the camera (along its level forward) the box is centred." {
    unit m
    tunable true
  }
  field shadowFocusDrop f32 "With shadowBox: how far below the camera the box's centre sits (the ground under a first-person eye)." {
    unit m
    tunable true
  }
  field shadowBias f32 "With shadowBox: the depth bias toward the light (three.js shadow.bias times its camera's depth range). 0 is one texel. Leaf cards need a few centimetres or they shadow each other dark." {
    min 0
    unit m
    tunable true
  }
  field shadowNormalBias f32 "With shadowBox: how far a receiver is pushed along its normal before it reads the map (three.js shadow.normalBias). 0 is one texel." {
    min 0
    unit m
    tunable true
  }
  field cascadeBlend f32 "The fraction of each shadow cascade's depth over which it fades into the next, hiding the seam where the shadow's resolution changes. 0 is none; 0.1 to 0.2 is typical (those pixels read two cascades)." {
    min 0
    max 0.5
    unit ratio
    tunable true
  }
}
```

<sub>Module `render` line 511</sub>
