<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Environment"></a>

### `og.Environment` (resource)

Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. Load the map with og.load.environment: the cook has already prefiltered it (GGX levels for reflections from mirror to rough, harmonics for the diffuse light), so setting it costs a few copy passes. Without a map the sky lights the scene as before (og.Sky, a DayNight, or the built-in one). Read after each tick; it never feeds back into the simulation.

Token `og.Environment`, registered under the schema name `og.Environment`; its value type is `Environment`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `EnvironmentMap` | `0` | — | — | — | The map from og.load.environment; 0 lights by the sky. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | The strength of the map's light on every surface, as three.js scene.environmentIntensity; while a map is set it replaces RenderSettings.environmentIntensity. |
| `rotation` | `number` | `0` | — | `deg` | tunable | Turns the map about +Y, as three.js scene.environmentRotation.y: positive turns what was at +X towards -Z. The light and the background turn together. |
| `background` | `boolean` | `false` | — | — | — | Draw the map behind the scene, as three.js scene.background = the environment (Camera.background, og.Sky and RenderSettings.sky give way); false leaves the background as they set it. |
| `backgroundIntensity` | `number` | `1` | ≥ 0 | — | tunable | Multiplies the map where it is drawn behind the scene, as three.js scene.backgroundIntensity; the light is not changed. |
| `backgroundBlurriness` | `number` | `0` | 0 … 1 | — | tunable | Blurs the sky where it is drawn behind the scene, as three.js scene.backgroundBlurriness: 0 sharp to 1 the roughest of the prefiltered (GGX) levels the light already reads, so it costs nothing extra; the light is not changed. Applies to the sky background with or without a map. |
| `groundHeight` | `number` | `0` | ≥ 0 | `m` | tunable | Draws the sky on a dome with a flat ground under it, as three.js GroundedSkybox: how far above the ground the map was photographed (metres). Objects then stand on the map's ground instead of floating over its horizon, and reflections of the environment (mirrors, water, clear coats) show the same projected ground. 0 (the default) draws the sky at infinity. |
| `groundRadius` | `number` | `100` | ≥ 0 | `m` | tunable | The grounded dome's radius (metres, larger than groundHeight), as GroundedSkybox's radius: the ground looks flat out to about this distance. |
| `groundCenter` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | The ground point under where the map was photographed: the dome's centre is groundHeight above it. |
| `blendTo` | `EnvironmentMap` | `0` | — | — | — | A second map from og.load.environment that the light and the background blend towards (indoor to outdoor); 0 is none. Both maps turn by rotation and the background is scaled by backgroundIntensity. |
| `blendWeight` | `number` | `0` | 0 … 1 | — | tunable | How far the blend has gone: 0 the map, 1 blendTo, in between a mix of both maps' light (every GGX level and the harmonics) and background. Drive it from the tick (a tween or a system), never from a clock, so a replay blends alike; it is drawn in 128 steps, each regenerating the light once. |
| `hemiSky` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A hemisphere light's sky colour (linear), as three.js HemisphereLight.color: diffuse-only ambient light mix(hemiGround, hemiSky, 0.5 n.y + 0.5) * hemiIntensity, darkened by ambient occlusion like the map's light. Unlike light baked into the map it adds no reflection. |
| `hemiGround` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The hemisphere light's ground colour (linear), as three.js HemisphereLight.groundColor. |
| `hemiIntensity` | `number` | `0` | ≥ 0 | — | tunable | The hemisphere light's intensity, as three.js HemisphereLight.intensity; 0 (the default) is off. |

```ogt
resource Environment {
  doc "Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. Load the map with og.load.environment: the cook has already prefiltered it (GGX levels for reflections from mirror to rough, harmonics for the diffuse light), so setting it costs a few copy passes. Without a map the sky lights the scene as before (og.Sky, a DayNight, or the built-in one). Read after each tick; it never feeds back into the simulation."
  field map EnvironmentMap "The map from og.load.environment; 0 lights by the sky."
  field intensity f32 "The strength of the map's light on every surface, as three.js scene.environmentIntensity; while a map is set it replaces RenderSettings.environmentIntensity." {
    default 1
    min 0
    tunable true
  }
  field rotation f32 "Turns the map about +Y, as three.js scene.environmentRotation.y: positive turns what was at +X towards -Z. The light and the background turn together." {
    unit deg
    tunable true
  }
  field background bool "Draw the map behind the scene, as three.js scene.background = the environment (Camera.background, og.Sky and RenderSettings.sky give way); false leaves the background as they set it."
  field backgroundIntensity f32 "Multiplies the map where it is drawn behind the scene, as three.js scene.backgroundIntensity; the light is not changed." {
    default 1
    min 0
    tunable true
  }
  field backgroundBlurriness f32 "Blurs the sky where it is drawn behind the scene, as three.js scene.backgroundBlurriness: 0 sharp to 1 the roughest of the prefiltered (GGX) levels the light already reads, so it costs nothing extra; the light is not changed. Applies to the sky background with or without a map." {
    default 0
    min 0
    max 1
    tunable true
  }
  field groundHeight f32 "Draws the sky on a dome with a flat ground under it, as three.js GroundedSkybox: how far above the ground the map was photographed (metres). Objects then stand on the map's ground instead of floating over its horizon, and reflections of the environment (mirrors, water, clear coats) show the same projected ground. 0 (the default) draws the sky at infinity." {
    default 0
    min 0
    unit m
    tunable true
  }
  field groundRadius f32 "The grounded dome's radius (metres, larger than groundHeight), as GroundedSkybox's radius: the ground looks flat out to about this distance." {
    default 100
    min 0
    unit m
    tunable true
  }
  field groundCenter Vector3 "The ground point under where the map was photographed: the dome's centre is groundHeight above it." {
    worldspace world
    unit m
  }
  field blendTo EnvironmentMap "A second map from og.load.environment that the light and the background blend towards (indoor to outdoor); 0 is none. Both maps turn by rotation and the background is scaled by backgroundIntensity."
  field blendWeight f32 "How far the blend has gone: 0 the map, 1 blendTo, in between a mix of both maps' light (every GGX level and the harmonics) and background. Drive it from the tick (a tween or a system), never from a clock, so a replay blends alike; it is drawn in 128 steps, each regenerating the light once." {
    default 0
    min 0
    max 1
    tunable true
  }
  field hemiSky Color "A hemisphere light's sky colour (linear), as three.js HemisphereLight.color: diffuse-only ambient light mix(hemiGround, hemiSky, 0.5 n.y + 0.5) * hemiIntensity, darkened by ambient occlusion like the map's light. Unlike light baked into the map it adds no reflection." {
    default 1 1 1
  }
  field hemiGround Color "The hemisphere light's ground colour (linear), as three.js HemisphereLight.groundColor." {
    default 0 0 0
  }
  field hemiIntensity f32 "The hemisphere light's intensity, as three.js HemisphereLight.intensity; 0 (the default) is off." {
    default 0
    min 0
    tunable true
  }
}
```

<sub>Module `render` line 743</sub>
