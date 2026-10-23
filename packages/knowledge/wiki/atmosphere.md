<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `atmosphere` module

Fog, a procedural sky, the day-night cycle, weather and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before.

The day-night clock is a function of the tick (DayNight.start plus the simulated time times speed), so it is the same on every run and after a restore; rendering reads the world and never writes it.

- **TypeScript**: functions are the namespace `og.atmosphere`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module atmosphere {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 6735</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`WEATHER_NONE`](#const-WEATHER_NONE) | constant | Weather.kind: clear. |
| [`WEATHER_RAIN`](#const-WEATHER_RAIN) | constant | Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog. |
| [`WEATHER_SNOW`](#const-WEATHER_SNOW) | constant | Weather.kind: flakes drifting down on gusts, a pale fog. |
| [`WEATHER_DUST`](#const-WEATHER_DUST) | constant | Weather.kind: dust blowing along the wind near the ground, a warm thick fog. |
| [`WEATHER_MIST`](#const-WEATHER_MIST) | constant | Weather.kind: no particles, a dense low height fog. |
| [`og.Fog`](#record-Fog) | resource | Distance and height fog over the 3D scene and world sprites: distance fog thickens exponentially by density from start (or linearly from start to end when end > start), height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged. |
| [`og.Sky`](#record-Sky) | resource | A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it (the image-based light follows it). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself. |
| [`og.DayNight`](#record-DayNight) | resource | A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the image-based light is regenerated every `step` minutes of game time while the sky drawn per pixel moves smoothly. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`. |
| [`og.Weather`](#record-Weather) | resource | A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation. |
| [`og.Water`](#record-Water) | component | A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space where the tier allows, the sky's cubemap on WebGL2, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated). |
| [`og.Water2D`](#record-Water2D) | component | Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface. |
| [`og.atmosphere.timeOfDay`](#fn-timeOfDay) | function | The DayNight clock at the current tick, in hours from 0 to 24 (its start when the cycle is off): for lamps that light at dusk and shops that close at night. Inside a system, declare og.DayNight among its reads. |

## Functions

<a id="fn-timeOfDay"></a>

### `og.atmosphere.timeOfDay`

The DayNight clock at the current tick, in hours from 0 to 24 (its start when the cycle is off): for lamps that light at dusk and shops that close at night. Inside a system, declare og.DayNight among its reads.

```ts
og.atmosphere.timeOfDay(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`.

Binding symbol `og_atmosphere_time_of_day` · publish builds: kept

```ogt
fn timeOfDay {
  doc "The DayNight clock at the current tick, in hours from 0 to 24 (its start when the cycle is off): for lamps that light at dusk and shops that close at night. Inside a system, declare og.DayNight among its reads."
  param world World "The world."
  returns f32
}
```

<sub>`api.ogt` line 7131</sub>

## Components, resources and events

<a id="record-Fog"></a>

### `og.Fog` (resource)

Distance and height fog over the 3D scene and world sprites: distance fog thickens exponentially by density from start (or linearly from start to end when end > start), height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged.

Token `og.Fog`, registered under the schema name `og.Fog`; its value type is `Fog`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the fog. |
| `color` | `Color` | `{ r: 0.62, g: 0.68, b: 0.76 }` | — | — | — | The colour the fog tends to (linear, before exposure). |
| `density` | `number` | `0.02` | 0 … 2 | — | tunable | Distance fog per metre (0: none). |
| `start` | `number` | `0` | ≥ 0 | `m` | tunable | Where distance fog starts. |
| `end` | `number` | `0` | ≥ 0 | `m` | tunable | With end > start, a linear ramp to full fog at end as well. |
| `heightDensity` | `number` | `0` | 0 … 10 | — | tunable | Height fog at and below height (0: none). |
| `height` | `number` | `0` | — | `m` | tunable | The world height below which height fog is densest. |
| `falloff` | `number` | `0.5` | 0.0001 … 20 | — | tunable | How fast height fog thins above height, per metre. |
| `sunColor` | `Color` | `{ r: 1, g: 0.86, b: 0.62 }` | — | — | — | The fog's colour looking into the sun (forward scattering). |
| `sunPower` | `number` | `8` | 0 … 256 | — | tunable | How tight the sun's glow in the fog is (higher: smaller). |
| `maxOpacity` | `number` | `1` | 0 … 1 | `ratio` | tunable | The most the fog hides. |
| `sky` | `number` | `1` | 0 … 1 | `ratio` | tunable | How much of the fog covers the sky (0: the sky stays clear). |
| `volumetric` | `boolean` | `false` | — | — | — | Light the fog as a participating medium: within volumetricDistance of the camera the fog (density per metre, thickened below height by heightDensity and falloff, as above) scatters the sun through its shadow cascades, so light shafts fall through windows and between trees and move with the sun, and every point, spot and rect light it is lit by (up to RenderCapabilities.volumetricLights at each sample); it dims what lies behind it by the same amounts. RenderCapabilities.volumetricFog names the method (a froxel grid on the full tier, a raymarch on WebGL2). Opaque surfaces and the sky are seen through the volume; blended surfaces, particles and sprites take the plain distance and height fog of the same medium. Past volumetricDistance nothing more is added. |
| `scattering` | `number` | `0.8` | 0 … 4 | — | tunable | How much of the light the fog takes from a ray it scatters (its albedo, tinted by color): 0 dims without glowing, 1 sends all of it on, above 1 brightens the shafts for effect. |
| `anisotropy` | `number` | `0.6` | -0.95 … 0.95 | — | tunable | Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light. |
| `volumetricDistance` | `number` | `48` | 4 … 400 | `m` | — | How far from the camera the volume reaches. |

```ogt
resource Fog {
  doc "Distance and height fog over the 3D scene and world sprites: distance fog thickens exponentially by density from start (or linearly from start to end when end > start), height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged."
  field enabled bool "Draw the fog."
  field color Color "The colour the fog tends to (linear, before exposure)." {
    default 0.62 0.68 0.76
  }
  field density f32 "Distance fog per metre (0: none)." {
    default 0.02
    min 0
    max 2
    tunable true
  }
  field start f32 "Where distance fog starts." {
    min 0
    unit m
    tunable true
  }
  field end f32 "With end > start, a linear ramp to full fog at end as well." {
    min 0
    unit m
    tunable true
  }
  field heightDensity f32 "Height fog at and below height (0: none)." {
    min 0
    max 10
    tunable true
  }
  field height f32 "The world height below which height fog is densest." {
    unit m
    tunable true
  }
  field falloff f32 "How fast height fog thins above height, per metre." {
    default 0.5
    min 0.0001
    max 20
    tunable true
  }
  field sunColor Color "The fog's colour looking into the sun (forward scattering)." {
    default 1 0.86 0.62
  }
  field sunPower f32 "How tight the sun's glow in the fog is (higher: smaller)." {
    default 8
    min 0
    max 256
    tunable true
  }
  field maxOpacity f32 "The most the fog hides." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field sky f32 "How much of the fog covers the sky (0: the sky stays clear)." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field volumetric bool "Light the fog as a participating medium: within volumetricDistance of the camera the fog (density per metre, thickened below height by heightDensity and falloff, as above) scatters the sun through its shadow cascades, so light shafts fall through windows and between trees and move with the sun, and every point, spot and rect light it is lit by (up to RenderCapabilities.volumetricLights at each sample); it dims what lies behind it by the same amounts. RenderCapabilities.volumetricFog names the method (a froxel grid on the full tier, a raymarch on WebGL2). Opaque surfaces and the sky are seen through the volume; blended surfaces, particles and sprites take the plain distance and height fog of the same medium. Past volumetricDistance nothing more is added."
  field scattering f32 "How much of the light the fog takes from a ray it scatters (its albedo, tinted by color): 0 dims without glowing, 1 sends all of it on, above 1 brightens the shafts for effect." {
    default 0.8
    min 0
    max 4
    tunable true
  }
  field anisotropy f32 "Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light." {
    default 0.6
    min -0.95
    max 0.95
    tunable true
  }
  field volumetricDistance f32 "How far from the camera the volume reaches." {
    default 48
    min 4
    max 400
    unit m
  }
}
```

<sub>`api.ogt` line 6746</sub>

<a id="record-Sky"></a>

### `og.Sky` (resource)

A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it (the image-based light follows it). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself.

Token `og.Sky`, registered under the schema name `og.Sky`; its value type is `Sky`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw and light by this sky. |
| `zenith` | `Color` | `{ r: 0.18, g: 0.36, b: 0.78 }` | — | — | — | Overhead. |
| `horizon` | `Color` | `{ r: 0.78, g: 0.84, b: 0.92 }` | — | — | — | At the horizon. |
| `ground` | `Color` | `{ r: 0.16, g: 0.14, b: 0.12 }` | — | — | — | Below the horizon. |
| `sunDirection` | `Vector3` | `{ x: 0.45, y: 0.8, z: 0.35 }` | — | — | — | Towards the sun (match your DirectionalLight). |
| `sunColor` | `Color` | `{ r: 1, g: 0.95, b: 0.88 }` | — | — | — | The sun's colour. |
| `sunSize` | `number` | `1.8` | 0 … 20 | `deg` | tunable | The sun disc's angular radius (0: no disc). |
| `sunIntensity` | `number` | `40` | ≥ 0 | — | tunable | The disc's brightness. |
| `glow` | `number` | `0.6` | ≥ 0 | — | tunable | The glow around the sun. |
| `glowPower` | `number` | `24` | ≥ 1 | — | tunable | How tight the glow is. |
| `stars` | `number` | `0` | 0 … 1 | `ratio` | tunable | Stars where the sky is dark: 0 none, 1 a full night sky. |
| `brightness` | `number` | `1.1` | ≥ 0 | — | tunable | A multiplier on the whole sky. |
| `moonSize` | `number` | `0` | 0 … 20 | `deg` | tunable | The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it. |
| `moonPhase` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half). |
| `moonIntensity` | `number` | `1.4` | ≥ 0 | — | tunable | The lit moon's brightness. |
| `moonColor` | `Color` | `{ r: 0.9, g: 0.92, b: 1 }` | — | — | — | The moon's colour. |
| `moonDirection` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Towards the moon; zero puts it opposite sunDirection. |

```ogt
resource Sky {
  doc "A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it (the image-based light follows it). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself."
  field enabled bool "Draw and light by this sky."
  field zenith Color "Overhead." {
    default 0.18 0.36 0.78
  }
  field horizon Color "At the horizon." {
    default 0.78 0.84 0.92
  }
  field ground Color "Below the horizon." {
    default 0.16 0.14 0.12
  }
  field sunDirection Vector3 "Towards the sun (match your DirectionalLight)." {
    default 0.45 0.8 0.35
  }
  field sunColor Color "The sun's colour." {
    default 1 0.95 0.88
  }
  field sunSize f32 "The sun disc's angular radius (0: no disc)." {
    default 1.8
    min 0
    max 20
    unit deg
    tunable true
  }
  field sunIntensity f32 "The disc's brightness." {
    default 40
    min 0
    tunable true
  }
  field glow f32 "The glow around the sun." {
    default 0.6
    min 0
    tunable true
  }
  field glowPower f32 "How tight the glow is." {
    default 24
    min 1
    tunable true
  }
  field stars f32 "Stars where the sky is dark: 0 none, 1 a full night sky." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field brightness f32 "A multiplier on the whole sky." {
    default 1.1
    min 0
    tunable true
  }
  field moonSize f32 "The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it." {
    min 0
    max 20
    unit deg
    tunable true
  }
  field moonPhase f32 "How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half)." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field moonIntensity f32 "The lit moon's brightness." {
    default 1.4
    min 0
    tunable true
  }
  field moonColor Color "The moon's colour." {
    default 0.9 0.92 1
  }
  field moonDirection Vector3 "Towards the moon; zero puts it opposite sunDirection."
}
```

<sub>`api.ogt` line 6827</sub>

<a id="record-DayNight"></a>

### `og.DayNight` (resource)

A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the image-based light is regenerated every `step` minutes of game time while the sky drawn per pixel moves smoothly. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`.

Token `og.DayNight`, registered under the schema name `og.DayNight`; its value type is `DayNight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Run the cycle. |
| `start` | `number` | `9` | 0 … 24 | — | tunable | The time of day at tick 0, in hours (6 sunrise, 12 noon, 18 sunset). |
| `speed` | `number` | `0.05` | 0 … 24 | — | tunable | Game hours per simulated second (0.05: a day in 8 minutes; 0 holds the time). |
| `tilt` | `number` | `25` | 0 … 89 | `deg` | tunable | How far the sun's path leans from overhead. |
| `azimuth` | `number` | `0` | — | `deg` | tunable | Where the sun rises, turned about +y from +x. |
| `lightIntensity` | `number` | `1` | 0 … 10 | — | tunable | A multiplier on the sun and moon light. |
| `tintFog` | `boolean` | `true` | — | — | — | Turn the og.Fog colour towards the horizon's. |
| `step` | `number` | `10` | 1 … 120 | — | tunable | Game minutes between regenerations of the image-based light. |
| `moonSize` | `number` | `1.6` | 0 … 20 | `deg` | tunable | The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon). |
| `moonPhase` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter. |

```ogt
resource DayNight {
  doc "A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the image-based light is regenerated every `step` minutes of game time while the sky drawn per pixel moves smoothly. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`."
  field enabled bool "Run the cycle."
  field start f32 "The time of day at tick 0, in hours (6 sunrise, 12 noon, 18 sunset)." {
    default 9
    min 0
    max 24
    tunable true
  }
  field speed f32 "Game hours per simulated second (0.05: a day in 8 minutes; 0 holds the time)." {
    default 0.05
    min 0
    max 24
    tunable true
  }
  field tilt f32 "How far the sun's path leans from overhead." {
    default 25
    min 0
    max 89
    unit deg
    tunable true
  }
  field azimuth f32 "Where the sun rises, turned about +y from +x." {
    unit deg
    tunable true
  }
  field lightIntensity f32 "A multiplier on the sun and moon light." {
    default 1
    min 0
    max 10
    tunable true
  }
  field tintFog bool "Turn the og.Fog colour towards the horizon's." {
    default true
  }
  field step f32 "Game minutes between regenerations of the image-based light." {
    default 10
    min 1
    max 120
    tunable true
  }
  field moonSize f32 "The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon)." {
    default 1.6
    min 0
    max 20
    unit deg
    tunable true
  }
  field moonPhase f32 "How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
}
```

<sub>`api.ogt` line 6902</sub>

<a id="record-Weather"></a>

### `og.Weather` (resource)

A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation.

Token `og.Weather`, registered under the schema name `og.Weather`; its value type is `Weather`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | 0 … 4 | — | label “Kind” | WEATHER_NONE 0, WEATHER_RAIN 1, WEATHER_SNOW 2, WEATHER_DUST 3, WEATHER_MIST 4. |
| `intensity` | `number` | `1` | 0 … 2 | — | tunable | How heavy: scales the particle rate, the fog and the dimming. |
| `wind` | `Vector3` | `{ x: 1.5, y: 0, z: 0.5 }` | — | `mps` | tunable | Where the wind blows the particles. |
| `radius` | `number` | `18` | 1 … 200 | `m` | tunable | The size of the region around the camera the particles fill. |
| `tint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Multiplies the particles' colour. |
| `dim` | `number` | `1` | 0 … 1 | `ratio` | tunable | How much of the preset's dimming of the sunlight applies. |
| `layer` | `number` | `50` | — | — | — | With an orthographic camera (2D), the sprite layer the particles draw in. |

```ogt
resource Weather {
  doc "A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation."
  field kind u32 "WEATHER_NONE 0, WEATHER_RAIN 1, WEATHER_SNOW 2, WEATHER_DUST 3, WEATHER_MIST 4." {
    min 0
    max 4
    label "Kind"
  }
  field intensity f32 "How heavy: scales the particle rate, the fog and the dimming." {
    default 1
    min 0
    max 2
    tunable true
  }
  field wind Vector3 "Where the wind blows the particles." {
    default 1.5 0 0.5
    unit mps
    tunable true
  }
  field radius f32 "The size of the region around the camera the particles fill." {
    default 18
    min 1
    max 200
    unit m
    tunable true
  }
  field tint Color "Multiplies the particles' colour." {
    default 1 1 1
  }
  field dim f32 "How much of the preset's dimming of the sunlight applies." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layer i32 "With an orthographic camera (2D), the sprite layer the particles draw in." {
    default 50
  }
}
```

<sub>`api.ogt` line 6959</sub>

<a id="record-Water"></a>

### `og.Water` (component)

A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space where the tier allows, the sky's cubemap on WebGL2, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated).

Token `og.Water`, registered under the schema name `og.Water`; its value type is `Water`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector2` | `{ x: 16, y: 16 }` | — | `m` | tunable | The extent along the entity's x and z. |
| `segments` | `number` | `64` | 1 … 256 | — | — | Grid cells per side (the height field's detail). |
| `shallow` | `Color` | `{ r: 0.55, g: 0.85, b: 0.82 }` | — | — | — | Shallow water's colour (tints what is seen through it). |
| `deep` | `Color` | `{ r: 0.02, g: 0.09, b: 0.14 }` | — | — | — | Deep water's colour. |
| `depthFade` | `number` | `2.5` | ≥ 0.01 | `m` | tunable | The depth over which water turns from shallow to deep. |
| `opacity` | `number` | `0.92` | 0 … 1 | `ratio` | tunable | How opaque deep water gets. |
| `foam` | `Color` | `{ r: 0.95, g: 0.97, b: 1 }` | — | — | — | The foam's colour. |
| `foamWidth` | `number` | `0.35` | ≥ 0 | `m` | tunable | The water depth over which foam rings geometry (0: none). |
| `waveScale` | `number` | `0.6` | ≥ 0 | — | tunable | Detail waves per metre. |
| `waveSpeed` | `number` | `1` | — | — | tunable | How fast the detail waves travel. |
| `waveStrength` | `number` | `0.35` | 0 … 4 | — | tunable | How strongly they bend the surface (0: a mirror). |
| `refraction` | `number` | `1` | ≥ 0 | — | tunable | How far refraction shifts what is seen through the water. |
| `reflectivity` | `number` | `1` | 0 … 2 | — | tunable | How much it reflects. |
| `planarReflection` | `number` | `0` | 0 … 1 | `ratio` | tunable | Reflect the scene by drawing it again from the camera mirrored in the surface, at this fraction of the screen's resolution (0: off, the screen-space and sky reflections). One surface a frame, the first that asks; it costs about a second scene pass at that resolution. |
| `shadows` | `number` | `0` | 0 … 1 | `ratio` | tunable | How much the sun's shadows darken the surface: its highlight, foam and colour (0: none). |
| `underwaterColor` | `Color` | `{ r: 0.03, g: 0.16, b: 0.2 }` | — | — | — | With the camera below the surface: the water's fog colour. |
| `underwaterDensity` | `number` | `0.09` | 0 … 2 | — | tunable | Its fog per metre (0: no underwater view). Below the surface the frame's fog becomes the water's, and the surface seen from below shows the sky inside Snell's window. |
| `caustics` | `number` | `0` | 0 … 4 | — | tunable | Caustics of the sunlight on everything below the surface (0: none). One surface a frame, the first that has them. |
| `causticsScale` | `number` | `0.6` | 0.01 … 10 | — | tunable | The caustics' cells per metre. |

```ogt
component Water {
  doc "A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space where the tier allows, the sky's cubemap on WebGL2, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated)."
  field size Vector2 "The extent along the entity's x and z." {
    default 16 16
    unit m
    tunable true
  }
  field segments u32 "Grid cells per side (the height field's detail)." {
    default 64
    min 1
    max 256
  }
  field shallow Color "Shallow water's colour (tints what is seen through it)." {
    default 0.55 0.85 0.82
  }
  field deep Color "Deep water's colour." {
    default 0.02 0.09 0.14
  }
  field depthFade f32 "The depth over which water turns from shallow to deep." {
    default 2.5
    min 0.01
    unit m
    tunable true
  }
  field opacity f32 "How opaque deep water gets." {
    default 0.92
    min 0
    max 1
    unit ratio
    tunable true
  }
  field foam Color "The foam's colour." {
    default 0.95 0.97 1
  }
  field foamWidth f32 "The water depth over which foam rings geometry (0: none)." {
    default 0.35
    min 0
    unit m
    tunable true
  }
  field waveScale f32 "Detail waves per metre." {
    default 0.6
    min 0
    tunable true
  }
  field waveSpeed f32 "How fast the detail waves travel." {
    default 1
    tunable true
  }
  field waveStrength f32 "How strongly they bend the surface (0: a mirror)." {
    default 0.35
    min 0
    max 4
    tunable true
  }
  field refraction f32 "How far refraction shifts what is seen through the water." {
    default 1
    min 0
    tunable true
  }
  field reflectivity f32 "How much it reflects." {
    default 1
    min 0
    max 2
    tunable true
  }
  field planarReflection f32 "Reflect the scene by drawing it again from the camera mirrored in the surface, at this fraction of the screen's resolution (0: off, the screen-space and sky reflections). One surface a frame, the first that asks; it costs about a second scene pass at that resolution." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field shadows f32 "How much the sun's shadows darken the surface: its highlight, foam and colour (0: none)." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field underwaterColor Color "With the camera below the surface: the water's fog colour." {
    default 0.03 0.16 0.2
  }
  field underwaterDensity f32 "Its fog per metre (0: no underwater view). Below the surface the frame's fog becomes the water's, and the surface seen from below shows the sky inside Snell's window." {
    default 0.09
    min 0
    max 2
    tunable true
  }
  field caustics f32 "Caustics of the sunlight on everything below the surface (0: none). One surface a frame, the first that has them." {
    min 0
    max 4
    tunable true
  }
  field causticsScale f32 "The caustics' cells per metre." {
    default 0.6
    min 0.01
    max 10
    tunable true
  }
}
```

<sub>`api.ogt` line 6999</sub>

<a id="record-Water2D"></a>

### `og.Water2D` (component)

Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface.

Token `og.Water2D`, registered under the schema name `og.Water2D`; its value type is `Water2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `top` | `Color` | `{ r: 0.16, g: 0.5, b: 0.72 }` | — | — | — | The body's colour at the surface. |
| `topOpacity` | `number` | `0.72` | 0 … 1 | `ratio` | — | Its opacity there. |
| `bottom` | `Color` | `{ r: 0.02, g: 0.1, b: 0.25 }` | — | — | — | The colour at the bottom. |
| `bottomOpacity` | `number` | `0.92` | 0 … 1 | `ratio` | — | Its opacity there. |
| `lip` | `Color` | `{ r: 0.8, g: 0.95, b: 1 }` | — | — | — | The lip along the surface. |
| `lipWidth` | `number` | `0.06` | ≥ 0 | `m` | tunable | How thick the lip is (0: none). |
| `layer` | `number` | `0` | — | — | — | The sprite layer it is drawn in. |

```ogt
component Water2D {
  doc "Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface."
  field top Color "The body's colour at the surface." {
    default 0.16 0.5 0.72
  }
  field topOpacity f32 "Its opacity there." {
    default 0.72
    min 0
    max 1
    unit ratio
  }
  field bottom Color "The colour at the bottom." {
    default 0.02 0.1 0.25
  }
  field bottomOpacity f32 "Its opacity there." {
    default 0.92
    min 0
    max 1
    unit ratio
  }
  field lip Color "The lip along the surface." {
    default 0.8 0.95 1
  }
  field lipWidth f32 "How thick the lip is (0: none)." {
    default 0.06
    min 0
    unit m
    tunable true
  }
  field layer i32 "The sprite layer it is drawn in."
}
```

<sub>`api.ogt` line 7099</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-WEATHER_NONE"></a>`WEATHER_NONE` | `number` | `0` | Weather.kind: clear. |
| <a id="const-WEATHER_RAIN"></a>`WEATHER_RAIN` | `number` | `1` | Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog. |
| <a id="const-WEATHER_SNOW"></a>`WEATHER_SNOW` | `number` | `2` | Weather.kind: flakes drifting down on gusts, a pale fog. |
| <a id="const-WEATHER_DUST"></a>`WEATHER_DUST` | `number` | `3` | Weather.kind: dust blowing along the wind near the ground, a warm thick fog. |
| <a id="const-WEATHER_MIST"></a>`WEATHER_MIST` | `number` | `4` | Weather.kind: no particles, a dense low height fog. |

```ogt
const WEATHER_NONE u32 0 "Weather.kind: clear."
```

<sub>`api.ogt` line 6740</sub>

```ogt
const WEATHER_RAIN u32 1 "Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog."
```

<sub>`api.ogt` line 6741</sub>

```ogt
const WEATHER_SNOW u32 2 "Weather.kind: flakes drifting down on gusts, a pale fog."
```

<sub>`api.ogt` line 6742</sub>

```ogt
const WEATHER_DUST u32 3 "Weather.kind: dust blowing along the wind near the ground, a warm thick fog."
```

<sub>`api.ogt` line 6743</sub>

```ogt
const WEATHER_MIST u32 4 "Weather.kind: no particles, a dense low height fog."
```

<sub>`api.ogt` line 6744</sub>
