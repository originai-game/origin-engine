<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `atmosphere` module

Fog, a procedural sky, the day-night cycle, weather, the wind that sways vegetation and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before.

The day-night clock is a function of the tick (DayNight.start plus the simulated time times speed), so it is the same on every run and after a restore; rendering reads the world and never writes it.

- **TypeScript**: functions are the namespace `og.atmosphere`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-render`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module atmosphere {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 10244</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`WEATHER_NONE`](#const-WEATHER_NONE) | constant | Weather.kind: clear. |
| [`WEATHER_RAIN`](#const-WEATHER_RAIN) | constant | Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog. |
| [`WEATHER_SNOW`](#const-WEATHER_SNOW) | constant | Weather.kind: flakes drifting down on gusts, a pale fog. |
| [`WEATHER_DUST`](#const-WEATHER_DUST) | constant | Weather.kind: dust blowing along the wind near the ground, a warm thick fog. |
| [`WEATHER_MIST`](#const-WEATHER_MIST) | constant | Weather.kind: no particles, a dense low height fog. |
| [`og.Fog`](#record-Fog) | resource | Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ramp from start to full fog at end; the two apply together, and density defaults to 0.02, so setting only end still adds exponential fog (set density 0 for the ramp alone). Height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged. |
| [`og.Sky`](#record-Sky) | resource | A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it: the image-based light and the reflections' sky (where a surface reflects no probe and SSR misses) follow it as it is drawn, clouds and all, captured from the camera a slice a frame, eight frames to a whole capture, so a drifting cloud, a lightning flash or a fast day moves in the reflections and the ambient light within sixteen frames (about a quarter of a second at 60 fps). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself. |
| [`og.DayNight`](#record-DayNight) | resource | A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the sky's colours step every `step` minutes of game time while its sun moves smoothly, and the image-based light and the reflections' sky follow the sky as it is drawn (clouds, lightning and all), captured a slice a frame. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`. |
| [`og.PhysicalSky`](#record-PhysicalSky) | resource | A physical sky: sunlight scattered by the air (Rayleigh: the blue overhead, the red sunsets) and by haze (Mie: the white glow around the sun), with ozone keeping the twilight zenith blue, in a spherical atmosphere of Earth's proportions with multiple scattering (Hillaire 2020). It replaces the gradient of og.Sky and og.DayNight, and the image-based light follows it. With og.DayNight enabled the cycle also takes its light from it: the sun's colour is the light that crosses the air (white at noon, orange then red as it sets, gone below the horizon), the ambient light the scattered sky, and the fog's colour the sky's horizon; `get:og.DayNight` answers those. With og.Sky enabled, og.Sky's sun direction, disc, stars, moon and brightness are drawn through the air. Alone it draws the sky towards the first DirectionalLight. Visual only: drawn from the tick, never written to the world. |
| [`og.Clouds`](#record-Clouds) | resource | Clouds in the procedural sky: a volumetric layer of cumulus raymarched through a curved shell over the planet, lit by the first DirectionalLight (shadowed within themselves, with multiple scattering: bright tops, dark bases, silver edges against the sun) and the sky's light, faded into the sky with distance; and a thin, streaked high layer (cirrus) above. The wind carries them with the tick's clock, so they look the same on every run and after a restore. They are drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky; alone they bring the built-in procedural sky with them) and dim the sun and the sky's light by their cover (a broken sky hides the sun a share of the time; an overcast deck hides it from every surface, leaving the deck's flat grey light). Visual only: nothing in the simulation reads them. |
| [`og.Aurora`](#record-Aurora) | resource | The aurora: curtains of light in the upper atmosphere, raymarched in the procedural sky (after Lawlor and Genetti, Interactive Volume Rendering Aurora on the GPU, 2011). Each curtain is a sheet of falling electrons along the auroral oval, folded by slow travelling waves and curled by a Kelvin-Helmholtz instability, extruded upward by how the electrons deposit their energy in the air: green oxygen light peaking near 114 km, a pink-violet nitrogen border below it where the electrons are hard, red oxygen light above 200 km where they are soft (the heights observed by Whiter et al., 2023). Emission only, under the clouds and dimmed by the air near the horizon; invisible against a daylight sky. The display follows the tick's clock, so it looks the same on every run and after a restore. Drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky). Visual only: nothing in the simulation reads it. |
| [`og.WeatherState`](#record-WeatherState) | resource | The weather's effect on the world, simulated in the tick from og.Weather (its kind and intensity) so gameplay can read it: how wet the ground is, how much standing water and snow lie on it, and the lightning strikes, drawn from a random stream the engine forks from og.Rng (`fork("og.weather")`) and keeps here, so a replay, a restore and every host strike at the same ticks in the same places. Registered the first time a game names it: a world that never does keeps its type table and hash. The renderer draws the same values: lit surfaces darken and shine when wet, puddles spread on level ground and ripple in the rain, snow settles on what faces up (deeper snow on steeper faces), and a strike flashes the scene with a bolt at strikePosition. Rendering never writes it. |
| [`og.LightningStrike`](#record-LightningStrike) | event | Lightning struck, in the tick og.weather drew it (og.WeatherState.lightning): read in the systems after og.weather that tick, or the next tick's first. |
| [`og.Weather`](#record-Weather) | resource | A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation. |
| [`og.Wind`](#record-Wind) | resource | The wind the renderer sways vegetation with: og.TerrainScatter copies, og.Grass blades, and og.Model instances with a sway, lean downwind and sway in gusts that sweep across the field, each vertex moved by its height (the root stays put, the tip moves furthest). Off at strength 0. Visual only: drawn from the tick, so it sways the same on every run and after a restore, and nothing in the simulation reads it (og.Rope, og.Cloth and og.Weather keep their own wind). |
| [`og.Water`](#record-Water) | component | A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space, the sky's cubemap where that misses, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated). |
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

<sub>`api.ogt` line 10988</sub>

## Components, resources and events

<a id="record-Fog"></a>

### `og.Fog` (resource)

Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ramp from start to full fog at end; the two apply together, and density defaults to 0.02, so setting only end still adds exponential fog (set density 0 for the ramp alone). Height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged.

Token `og.Fog`, registered under the schema name `og.Fog`; its value type is `Fog`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the fog. |
| `color` | `Color` | `{ r: 0.62, g: 0.68, b: 0.76 }` | — | — | — | The colour the fog tends to (linear, before exposure). |
| `density` | `number` | `0.02` | 0 … 2 | — | tunable | Exponential distance fog per metre past start, with or without end's ramp (0: none). |
| `start` | `number` | `0` | ≥ 0 | `m` | tunable | The distance from the camera where distance fog starts. |
| `end` | `number` | `0` | ≥ 0 | `m` | tunable | The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp). |
| `heightDensity` | `number` | `0` | 0 … 10 | — | tunable | Height fog at and below height (0: none). |
| `height` | `number` | `0` | — | `m` | tunable, worldspace `world y` | The world height below which height fog is densest. |
| `falloff` | `number` | `0.5` | 0.0001 … 20 | — | tunable | How fast height fog thins above height, per metre. |
| `sunColor` | `Color` | `{ r: 1, g: 0.86, b: 0.62 }` | — | — | — | The fog's colour looking into the sun (forward scattering). |
| `sunPower` | `number` | `8` | 0 … 256 | — | tunable | How tight the sun's glow in the fog is (higher: smaller). |
| `maxOpacity` | `number` | `1` | 0 … 1 | `ratio` | tunable | The most the fog hides. |
| `sky` | `number` | `1` | 0 … 1 | `ratio` | tunable | How much of the fog covers the sky (0: the sky stays clear). |
| `volumetric` | `boolean` | `false` | — | — | — | Light the fog as a participating medium: within volumetricDistance of the camera the fog (density per metre, thickened below height by heightDensity and falloff, as above) scatters the sun through its shadow cascades, so light shafts fall through windows and between trees and move with the sun, and every point, spot and rect light it is lit by (up to RenderCapabilities.volumetricLights at each sample); it dims what lies behind it by the same amounts. RenderCapabilities.volumetricFog names the method (a froxel grid). Opaque surfaces and the sky are seen through the volume; blended surfaces, particles and sprites take the plain distance and height fog of the same medium. Past volumetricDistance nothing more is added. |
| `scattering` | `number` | `0.8` | 0 … 4 | — | tunable | How much of the light the fog takes from a ray it scatters (its albedo, tinted by color): 0 dims without glowing, 1 sends all of it on, above 1 brightens the shafts for effect. |
| `anisotropy` | `number` | `0.6` | -0.95 … 0.95 | — | tunable | Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light. |
| `volumetricDistance` | `number` | `48` | 4 … 400 | `m` | — | How far from the camera the volume reaches. |

```ogt
resource Fog {
  doc "Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ramp from start to full fog at end; the two apply together, and density defaults to 0.02, so setting only end still adds exponential fog (set density 0 for the ramp alone). Height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged."
  field enabled bool "Draw the fog."
  field color Color "The colour the fog tends to (linear, before exposure)." {
    default 0.62 0.68 0.76
  }
  field density f32 "Exponential distance fog per metre past start, with or without end's ramp (0: none)." {
    default 0.02
    min 0
    max 2
    tunable true
  }
  field start f32 "The distance from the camera where distance fog starts." {
    min 0
    unit m
    tunable true
  }
  field end f32 "The distance from the camera where fog is full: with end > start, a linear ramp from start to end applied together with density's exponential fog (0: no ramp)." {
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
    worldspace world y
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
  field volumetric bool "Light the fog as a participating medium: within volumetricDistance of the camera the fog (density per metre, thickened below height by heightDensity and falloff, as above) scatters the sun through its shadow cascades, so light shafts fall through windows and between trees and move with the sun, and every point, spot and rect light it is lit by (up to RenderCapabilities.volumetricLights at each sample); it dims what lies behind it by the same amounts. RenderCapabilities.volumetricFog names the method (a froxel grid). Opaque surfaces and the sky are seen through the volume; blended surfaces, particles and sprites take the plain distance and height fog of the same medium. Past volumetricDistance nothing more is added."
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

<sub>`api.ogt` line 10255</sub>

<a id="record-Sky"></a>

### `og.Sky` (resource)

A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it: the image-based light and the reflections' sky (where a surface reflects no probe and SSR misses) follow it as it is drawn, clouds and all, captured from the camera a slice a frame, eight frames to a whole capture, so a drifting cloud, a lightning flash or a fast day moves in the reflections and the ambient light within sixteen frames (about a quarter of a second at 60 fps). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself.

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
  doc "A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it: the image-based light and the reflections' sky (where a surface reflects no probe and SSR misses) follow it as it is drawn, clouds and all, captured from the camera a slice a frame, eight frames to a whole capture, so a drifting cloud, a lightning flash or a fast day moves in the reflections and the ambient light within sixteen frames (about a quarter of a second at 60 fps). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself."
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

<sub>`api.ogt` line 10337</sub>

<a id="record-DayNight"></a>

### `og.DayNight` (resource)

A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the sky's colours step every `step` minutes of game time while its sun moves smoothly, and the image-based light and the reflections' sky follow the sky as it is drawn (clouds, lightning and all), captured a slice a frame. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`.

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
| `step` | `number` | `10` | 1 … 120 | — | tunable | Game minutes between steps of the sky's colours (the image-based light follows the sky as drawn). |
| `moonSize` | `number` | `1.6` | 0 … 20 | `deg` | tunable | The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon). |
| `moonPhase` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter. |

```ogt
resource DayNight {
  doc "A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the sky's colours step every `step` minutes of game time while its sun moves smoothly, and the image-based light and the reflections' sky follow the sky as it is drawn (clouds, lightning and all), captured a slice a frame. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`."
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
  field step f32 "Game minutes between steps of the sky's colours (the image-based light follows the sky as drawn)." {
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

<sub>`api.ogt` line 10412</sub>

<a id="record-PhysicalSky"></a>

### `og.PhysicalSky` (resource)

A physical sky: sunlight scattered by the air (Rayleigh: the blue overhead, the red sunsets) and by haze (Mie: the white glow around the sun), with ozone keeping the twilight zenith blue, in a spherical atmosphere of Earth's proportions with multiple scattering (Hillaire 2020). It replaces the gradient of og.Sky and og.DayNight, and the image-based light follows it. With og.DayNight enabled the cycle also takes its light from it: the sun's colour is the light that crosses the air (white at noon, orange then red as it sets, gone below the horizon), the ambient light the scattered sky, and the fog's colour the sky's horizon; `get:og.DayNight` answers those. With og.Sky enabled, og.Sky's sun direction, disc, stars, moon and brightness are drawn through the air. Alone it draws the sky towards the first DirectionalLight. Visual only: drawn from the tick, never written to the world.

Token `og.PhysicalSky`, registered under the schema name `og.PhysicalSky`; its value type is `PhysicalSky`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the sky by scattering. |
| `haze` | `number` | `1` | 0 … 40 | — | tunable | How much haze the air holds, a multiplier on Earth's aerosols (1: a clear day; 0: pure air; 5 to 10: a hazy or smoggy day, a whiter sky and a wider glow round the sun). |
| `airDensity` | `number` | `1` | 0 … 10 | — | tunable | A multiplier on the air's scattering (1: Earth; above 1 a deeper blue and redder sunsets, below 1 a thinner, darker sky). |
| `ozone` | `number` | `1` | 0 … 10 | — | tunable | A multiplier on ozone's absorption, which keeps the sky blue overhead at twilight (0: none, the twilight sky turns grey and yellow). |
| `groundAlbedo` | `Color` | `{ r: 0.3, g: 0.3, b: 0.3 }` | — | — | — | The ground's colour as the air sees it from below: it lights the haze and the sky below the horizon. |
| `altitude` | `number` | `0` | 0 … 100000 | `m` | tunable | The height above sea level of the world's y 0 (the camera's height is added): from high up the sky darkens overhead and the sun stays up longer. |
| `night` | `Color` | `{ r: 0.0022, g: 0.0032, b: 0.0075 }` | — | — | — | The sky's light with the sun far below the horizon (airglow and starlight), so a night is dark blue rather than black and the land under it keeps a deep blue-grey ambient. |

```ogt
resource PhysicalSky {
  doc "A physical sky: sunlight scattered by the air (Rayleigh: the blue overhead, the red sunsets) and by haze (Mie: the white glow around the sun), with ozone keeping the twilight zenith blue, in a spherical atmosphere of Earth's proportions with multiple scattering (Hillaire 2020). It replaces the gradient of og.Sky and og.DayNight, and the image-based light follows it. With og.DayNight enabled the cycle also takes its light from it: the sun's colour is the light that crosses the air (white at noon, orange then red as it sets, gone below the horizon), the ambient light the scattered sky, and the fog's colour the sky's horizon; `get:og.DayNight` answers those. With og.Sky enabled, og.Sky's sun direction, disc, stars, moon and brightness are drawn through the air. Alone it draws the sky towards the first DirectionalLight. Visual only: drawn from the tick, never written to the world."
  field enabled bool "Draw the sky by scattering."
  field haze f32 "How much haze the air holds, a multiplier on Earth's aerosols (1: a clear day; 0: pure air; 5 to 10: a hazy or smoggy day, a whiter sky and a wider glow round the sun)." {
    default 1
    min 0
    max 40
    tunable true
  }
  field airDensity f32 "A multiplier on the air's scattering (1: Earth; above 1 a deeper blue and redder sunsets, below 1 a thinner, darker sky)." {
    default 1
    min 0
    max 10
    tunable true
  }
  field ozone f32 "A multiplier on ozone's absorption, which keeps the sky blue overhead at twilight (0: none, the twilight sky turns grey and yellow)." {
    default 1
    min 0
    max 10
    tunable true
  }
  field groundAlbedo Color "The ground's colour as the air sees it from below: it lights the haze and the sky below the horizon." {
    default 0.3 0.3 0.3
  }
  field altitude f32 "The height above sea level of the world's y 0 (the camera's height is added): from high up the sky darkens overhead and the sun stays up longer." {
    min 0
    max 100000
    unit m
    tunable true
  }
  field night Color "The sky's light with the sun far below the horizon (airglow and starlight), so a night is dark blue rather than black and the land under it keeps a deep blue-grey ambient." {
    default 0.0022 0.0032 0.0075
  }
}
```

<sub>`api.ogt` line 10469</sub>

<a id="record-Clouds"></a>

### `og.Clouds` (resource)

Clouds in the procedural sky: a volumetric layer of cumulus raymarched through a curved shell over the planet, lit by the first DirectionalLight (shadowed within themselves, with multiple scattering: bright tops, dark bases, silver edges against the sun) and the sky's light, faded into the sky with distance; and a thin, streaked high layer (cirrus) above. The wind carries them with the tick's clock, so they look the same on every run and after a restore. They are drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky; alone they bring the built-in procedural sky with them) and dim the sun and the sky's light by their cover (a broken sky hides the sun a share of the time; an overcast deck hides it from every surface, leaving the deck's flat grey light). Visual only: nothing in the simulation reads them.

Token `og.Clouds`, registered under the schema name `og.Clouds`; its value type is `Clouds`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the clouds. |
| `coverage` | `number` | `0.45` | 0 … 1 | `ratio` | tunable | How much of the sky the cumulus covers: 0 none, 0.3 fair weather, 0.6 broken, 1 overcast. |
| `density` | `number` | `0.04` | 0 … 0.5 | — | tunable | How thick they are per metre (0.04: cumulus; 0.01: thin and translucent; 0.08: dense storm cloud). |
| `base` | `number` | `1500` | ≥ 0 | `m` | tunable | The height of the cloud base above the world's y 0. |
| `thickness` | `number` | `1800` | ≥ 0 | `m` | tunable | How tall the layer is (0: no cumulus, only the high layer). |
| `scale` | `number` | `9000` | 500 … 200000 | `m` | tunable | The size of the largest cloud shapes. |
| `wind` | `Vector3` | `{ x: 12, y: 0.4, z: 4 }` | — | `mps` | tunable | How fast the wind carries them (x and z; y slowly turns their shapes over). |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Their albedo: white for fair-weather clouds, a grey for rain clouds. |
| `detail` | `number` | `0.35` | 0 … 1 | `ratio` | tunable | How much fine noise erodes their edges (0: smooth blobs, 1: wispy). |
| `layer` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | The high thin layer's cover (0: none). |
| `layerAltitude` | `number` | `8000` | ≥ 0 | `m` | tunable | The high layer's height (kept above the cumulus). |
| `steps` | `number` | `48` | 8 … 256 | — | — | Samples along each view ray through the cumulus: more is smoother and costs more. |

```ogt
resource Clouds {
  doc "Clouds in the procedural sky: a volumetric layer of cumulus raymarched through a curved shell over the planet, lit by the first DirectionalLight (shadowed within themselves, with multiple scattering: bright tops, dark bases, silver edges against the sun) and the sky's light, faded into the sky with distance; and a thin, streaked high layer (cirrus) above. The wind carries them with the tick's clock, so they look the same on every run and after a restore. They are drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky; alone they bring the built-in procedural sky with them) and dim the sun and the sky's light by their cover (a broken sky hides the sun a share of the time; an overcast deck hides it from every surface, leaving the deck's flat grey light). Visual only: nothing in the simulation reads them."
  field enabled bool "Draw the clouds."
  field coverage f32 "How much of the sky the cumulus covers: 0 none, 0.3 fair weather, 0.6 broken, 1 overcast." {
    default 0.45
    min 0
    max 1
    unit ratio
    tunable true
  }
  field density f32 "How thick they are per metre (0.04: cumulus; 0.01: thin and translucent; 0.08: dense storm cloud)." {
    default 0.04
    min 0
    max 0.5
    tunable true
  }
  field base f32 "The height of the cloud base above the world's y 0." {
    default 1500
    min 0
    unit m
    tunable true
  }
  field thickness f32 "How tall the layer is (0: no cumulus, only the high layer)." {
    default 1800
    min 0
    unit m
    tunable true
  }
  field scale f32 "The size of the largest cloud shapes." {
    default 9000
    min 500
    max 200000
    unit m
    tunable true
  }
  field wind Vector3 "How fast the wind carries them (x and z; y slowly turns their shapes over)." {
    default 12 0.4 4
    unit mps
    tunable true
  }
  field color Color "Their albedo: white for fair-weather clouds, a grey for rain clouds." {
    default 1 1 1
  }
  field detail f32 "How much fine noise erodes their edges (0: smooth blobs, 1: wispy)." {
    default 0.35
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layer f32 "The high thin layer's cover (0: none)." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field layerAltitude f32 "The high layer's height (kept above the cumulus)." {
    default 8000
    min 0
    unit m
    tunable true
  }
  field steps u32 "Samples along each view ray through the cumulus: more is smoother and costs more." {
    default 48
    min 8
    max 256
  }
}
```

<sub>`api.ogt` line 10504</sub>

<a id="record-Aurora"></a>

### `og.Aurora` (resource)

The aurora: curtains of light in the upper atmosphere, raymarched in the procedural sky (after Lawlor and Genetti, Interactive Volume Rendering Aurora on the GPU, 2011). Each curtain is a sheet of falling electrons along the auroral oval, folded by slow travelling waves and curled by a Kelvin-Helmholtz instability, extruded upward by how the electrons deposit their energy in the air: green oxygen light peaking near 114 km, a pink-violet nitrogen border below it where the electrons are hard, red oxygen light above 200 km where they are soft (the heights observed by Whiter et al., 2023). Emission only, under the clouds and dimmed by the air near the horizon; invisible against a daylight sky. The display follows the tick's clock, so it looks the same on every run and after a restore. Drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky). Visual only: nothing in the simulation reads it.

Token `og.Aurora`, registered under the schema name `og.Aurora`; its value type is `Aurora`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Draw the aurora. |
| `intensity` | `number` | `1` | 0 … 20 | — | tunable | How bright the display is: 1 a bright display against a night sky, 0.3 a faint one. |
| `activity` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | 0 quiet, smooth arcs; 0.5 folding, rayed curtains; 1 a substorm's breakup, curled and fragmented, moving fast. |
| `curtains` | `number` | `3` | 1 … 6 | — | — | How many arcs, one behind another. |
| `direction` | `number` | `90` | — | `deg` | tunable | Where the arcs lie, turned about +y from +x (the convention of og.DayNight's azimuth). |
| `distance` | `number` | `150000` | 0 … 1500000 | `m` | tunable | How far the first arc is from the world's origin over the ground (0: overhead). |
| `spread` | `number` | `70000` | ≥ 0 | `m` | tunable | The distance between successive arcs. |
| `energy` | `number` | `2` | 0.3 … 20 | — | tunable | The falling electrons' characteristic energy in keV: about 2 for a typical display; under 1 a high, red-topped aurora; over 5 a low one with a pink-violet lower border. |
| `speed` | `number` | `1` | 0 … 20 | — | tunable | How fast the folds travel and the rays drift (1: as observed). |
| `seed` | `number` | `0` | — | — | — | The display's seed: its curtains' shapes and their curls. |
| `greenColor` | `Color` | `{ r: 0.29, g: 1, b: 0 }` | — | — | — | The oxygen green line's colour (557.7 nm): by default its spectral colour, a yellow-green; a camera's photograph renders it bluer (0.1 1 0.55). |
| `redColor` | `Color` | `{ r: 1, g: 0, b: 0.06 }` | — | — | — | The oxygen red line's colour (630.0 nm), of the high tops. |
| `nitrogenColor` | `Color` | `{ r: 0.57, g: 0, b: 1 }` | — | — | — | The nitrogen bands' colour (427.8 nm and the red first positive band), of the lower border: violet to pink. |

```ogt
resource Aurora {
  doc "The aurora: curtains of light in the upper atmosphere, raymarched in the procedural sky (after Lawlor and Genetti, Interactive Volume Rendering Aurora on the GPU, 2011). Each curtain is a sheet of falling electrons along the auroral oval, folded by slow travelling waves and curled by a Kelvin-Helmholtz instability, extruded upward by how the electrons deposit their energy in the air: green oxygen light peaking near 114 km, a pink-violet nitrogen border below it where the electrons are hard, red oxygen light above 200 km where they are soft (the heights observed by Whiter et al., 2023). Emission only, under the clouds and dimmed by the air near the horizon; invisible against a daylight sky. The display follows the tick's clock, so it looks the same on every run and after a restore. Drawn over the procedural sky (og.Sky, og.DayNight or og.PhysicalSky). Visual only: nothing in the simulation reads it."
  field enabled bool "Draw the aurora."
  field intensity f32 "How bright the display is: 1 a bright display against a night sky, 0.3 a faint one." {
    default 1
    min 0
    max 20
    tunable true
  }
  field activity f32 "0 quiet, smooth arcs; 0.5 folding, rayed curtains; 1 a substorm's breakup, curled and fragmented, moving fast." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field curtains u32 "How many arcs, one behind another." {
    default 3
    min 1
    max 6
  }
  field direction f32 "Where the arcs lie, turned about +y from +x (the convention of og.DayNight's azimuth)." {
    default 90
    unit deg
    tunable true
  }
  field distance f32 "How far the first arc is from the world's origin over the ground (0: overhead)." {
    default 150000
    min 0
    max 1500000
    unit m
    tunable true
  }
  field spread f32 "The distance between successive arcs." {
    default 70000
    min 0
    unit m
    tunable true
  }
  field energy f32 "The falling electrons' characteristic energy in keV: about 2 for a typical display; under 1 a high, red-topped aurora; over 5 a low one with a pink-violet lower border." {
    default 2
    min 0.3
    max 20
    tunable true
  }
  field speed f32 "How fast the folds travel and the rays drift (1: as observed)." {
    default 1
    min 0
    max 20
    tunable true
  }
  field seed u32 "The display's seed: its curtains' shapes and their curls."
  field greenColor Color "The oxygen green line's colour (557.7 nm): by default its spectral colour, a yellow-green; a camera's photograph renders it bluer (0.1 1 0.55)." {
    default 0.29 1 0
  }
  field redColor Color "The oxygen red line's colour (630.0 nm), of the high tops." {
    default 1 0 0.06
  }
  field nitrogenColor Color "The nitrogen bands' colour (427.8 nm and the red first positive band), of the lower border: violet to pink." {
    default 0.57 0 1
  }
}
```

<sub>`api.ogt` line 10574</sub>

<a id="record-WeatherState"></a>

### `og.WeatherState` (resource)

The weather's effect on the world, simulated in the tick from og.Weather (its kind and intensity) so gameplay can read it: how wet the ground is, how much standing water and snow lie on it, and the lightning strikes, drawn from a random stream the engine forks from og.Rng (`fork("og.weather")`) and keeps here, so a replay, a restore and every host strike at the same ticks in the same places. Registered the first time a game names it: a world that never does keeps its type table and hash. The renderer draws the same values: lit surfaces darken and shine when wet, puddles spread on level ground and ripple in the rain, snow settles on what faces up (deeper snow on steeper faces), and a strike flashes the scene with a bolt at strikePosition. Rendering never writes it.

Token `og.WeatherState`, registered under the schema name `og.WeatherState`; its value type is `WeatherState`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `soakTime` | `number` | `60` | ≥ 0.1 | `s` | tunable | Seconds of rain at intensity 1 to soak dry ground (wetness 0 to 1). |
| `dryTime` | `number` | `300` | ≥ 0.1 | `s` | tunable | Seconds for soaked ground to dry once the rain stops. |
| `puddleTime` | `number` | `120` | ≥ 0.1 | `s` | tunable | Seconds of rain at intensity 1 on soaked ground to fill the puddles (puddles 0 to 1). |
| `drainTime` | `number` | `600` | ≥ 0.1 | `s` | tunable | Seconds for full puddles to drain once the rain stops. |
| `snowfall` | `number` | `0.01` | ≥ 0 | `m` | tunable | How much snow settles per minute of snowfall at intensity 1. |
| `melt` | `number` | `0.004` | ≥ 0 | `m` | tunable | How much snow melts per minute when none falls (three times as fast in rain). |
| `snowCover` | `number` | `0.08` | ≥ 0.001 | `m` | tunable | The snow depth at which it covers everything that can hold it (the renderer's full cover). |
| `lightning` | `number` | `0` | 0 … 600 | — | tunable | Lightning strikes per minute on average, at random ticks (0: none), whatever the kind of weather. |
| `lightningCenter` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | The centre of the area strikes land in (its y is the ground's height there). |
| `lightningRadius` | `number` | `300` | ≥ 0 | `m` | tunable | The radius of that area: strikes land evenly over the disc. |
| `shelterSize` | `number` | `64` | 0 … 1024 | `m` | tunable | The side of the square around the camera in which roofs, overhangs and canopies keep what is under them dry and free of snow, with a soft edge at the drip line (the renderer's weather occlusion map, looking along the rain or snow as og.Weather's wind slants it); past it every surface is open to the sky. 0: nothing shelters. Drawn only: the tick's wetness, puddles and snowDepth are the open ground's, so a game that needs to know whether a place is sheltered casts a ray up against the colliders (og.physics). |
| `wetness` | `number` | `0` | — | `ratio` | read only | How wet the ground is, 0 dry to 1 soaked (written by og.weather). |
| `puddles` | `number` | `0` | — | `ratio` | read only | How far standing water has spread, 0 none to 1 (written by og.weather). |
| `snowDepth` | `number` | `0` | — | `m` | read only | The depth of lying snow (written by og.weather). |
| `strikes` | `number` | `0` | — | — | read only | Lightning strikes so far (written by og.weather). |
| `lastStrike` | `number` | `0` | — | — | read only | The tick of the latest strike (0 before the first; written by og.weather). |
| `strikePosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | Where the latest strike landed (written by og.weather). |
| `stream` | `number` | `0` | — | — | read only | The lightning's random state, forked from og.Rng the first tick (written by og.weather). |

```ogt
resource WeatherState {
  doc "The weather's effect on the world, simulated in the tick from og.Weather (its kind and intensity) so gameplay can read it: how wet the ground is, how much standing water and snow lie on it, and the lightning strikes, drawn from a random stream the engine forks from og.Rng (`fork(\"og.weather\")`) and keeps here, so a replay, a restore and every host strike at the same ticks in the same places. Registered the first time a game names it: a world that never does keeps its type table and hash. The renderer draws the same values: lit surfaces darken and shine when wet, puddles spread on level ground and ripple in the rain, snow settles on what faces up (deeper snow on steeper faces), and a strike flashes the scene with a bolt at strikePosition. Rendering never writes it."
  field soakTime f32 "Seconds of rain at intensity 1 to soak dry ground (wetness 0 to 1)." {
    default 60
    min 0.1
    unit s
    tunable true
  }
  field dryTime f32 "Seconds for soaked ground to dry once the rain stops." {
    default 300
    min 0.1
    unit s
    tunable true
  }
  field puddleTime f32 "Seconds of rain at intensity 1 on soaked ground to fill the puddles (puddles 0 to 1)." {
    default 120
    min 0.1
    unit s
    tunable true
  }
  field drainTime f32 "Seconds for full puddles to drain once the rain stops." {
    default 600
    min 0.1
    unit s
    tunable true
  }
  field snowfall f32 "How much snow settles per minute of snowfall at intensity 1." {
    default 0.01
    min 0
    unit m
    tunable true
  }
  field melt f32 "How much snow melts per minute when none falls (three times as fast in rain)." {
    default 0.004
    min 0
    unit m
    tunable true
  }
  field snowCover f32 "The snow depth at which it covers everything that can hold it (the renderer's full cover)." {
    default 0.08
    min 0.001
    unit m
    tunable true
  }
  field lightning f32 "Lightning strikes per minute on average, at random ticks (0: none), whatever the kind of weather." {
    min 0
    max 600
    tunable true
  }
  field lightningCenter Vector3 "The centre of the area strikes land in (its y is the ground's height there)." {
    worldspace world
    unit m
  }
  field lightningRadius f32 "The radius of that area: strikes land evenly over the disc." {
    default 300
    min 0
    unit m
    tunable true
  }
  field shelterSize f32 "The side of the square around the camera in which roofs, overhangs and canopies keep what is under them dry and free of snow, with a soft edge at the drip line (the renderer's weather occlusion map, looking along the rain or snow as og.Weather's wind slants it); past it every surface is open to the sky. 0: nothing shelters. Drawn only: the tick's wetness, puddles and snowDepth are the open ground's, so a game that needs to know whether a place is sheltered casts a ray up against the colliders (og.physics)." {
    default 64
    min 0
    max 1024
    unit m
    tunable true
  }
  field wetness f32 "How wet the ground is, 0 dry to 1 soaked (written by og.weather)." {
    readonly true
    unit ratio
  }
  field puddles f32 "How far standing water has spread, 0 none to 1 (written by og.weather)." {
    readonly true
    unit ratio
  }
  field snowDepth f32 "The depth of lying snow (written by og.weather)." {
    readonly true
    unit m
  }
  field strikes u32 "Lightning strikes so far (written by og.weather)." {
    readonly true
  }
  field lastStrike u32 "The tick of the latest strike (0 before the first; written by og.weather)." {
    readonly true
  }
  field strikePosition Vector3 "Where the latest strike landed (written by og.weather)." {
    worldspace world
    readonly true
    unit m
  }
  field stream u32 "The lightning's random state, forked from og.Rng the first tick (written by og.weather)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 10637</sub>

<a id="record-LightningStrike"></a>

### `og.LightningStrike` (event)

Lightning struck, in the tick og.weather drew it (og.WeatherState.lightning): read in the systems after og.weather that tick, or the next tick's first.

Token `og.LightningStrike`, registered under the schema name `og.LightningStrike`; its value type is `LightningStrike`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where it landed: in the disc of og.WeatherState.lightningRadius around lightningCenter, at its height. |
| `strike` | `number` | `0` | — | — | — | Which strike it is, counting from 1 (og.WeatherState.strikes after it). |

```ogt
event LightningStrike {
  doc "Lightning struck, in the tick og.weather drew it (og.WeatherState.lightning): read in the systems after og.weather that tick, or the next tick's first."
  field position Vector3 "Where it landed: in the disc of og.WeatherState.lightningRadius around lightningCenter, at its height." {
    worldspace world
    unit m
  }
  field strike u32 "Which strike it is, counting from 1 (og.WeatherState.strikes after it)."
}
```

<sub>`api.ogt` line 10731</sub>

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

<sub>`api.ogt` line 10740</sub>

<a id="record-Wind"></a>

### `og.Wind` (resource)

The wind the renderer sways vegetation with: og.TerrainScatter copies, og.Grass blades, and og.Model instances with a sway, lean downwind and sway in gusts that sweep across the field, each vertex moved by its height (the root stays put, the tip moves furthest). Off at strength 0. Visual only: drawn from the tick, so it sways the same on every run and after a restore, and nothing in the simulation reads it (og.Rope, og.Cloth and og.Weather keep their own wind).

Token `og.Wind`, registered under the schema name `og.Wind`; its value type is `Wind`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `strength` | `number` | `0` | 0 … 2 | — | tunable | The mean lean: how far a copy's tip moves downwind per metre of its height (0.2: a 1 m stem's tip leans 20 cm); 0 is still. |
| `direction` | `Vector3` | `{ x: 1, y: 0, z: 0 }` | — | — | tunable | Where the wind blows: its horizontal part, normalised. |
| `speed` | `number` | `3` | 0 … 50 | `mps` | tunable | How fast the gusts travel downwind, over a 10 m wave. |
| `gust` | `number` | `0.6` | 0 … 1 | `ratio` | tunable | How much the lean varies in the gusts: 0 a steady lean, 1 from upright to twice the mean. |

```ogt
resource Wind {
  doc "The wind the renderer sways vegetation with: og.TerrainScatter copies, og.Grass blades, and og.Model instances with a sway, lean downwind and sway in gusts that sweep across the field, each vertex moved by its height (the root stays put, the tip moves furthest). Off at strength 0. Visual only: drawn from the tick, so it sways the same on every run and after a restore, and nothing in the simulation reads it (og.Rope, og.Cloth and og.Weather keep their own wind)."
  field strength f32 "The mean lean: how far a copy's tip moves downwind per metre of its height (0.2: a 1 m stem's tip leans 20 cm); 0 is still." {
    min 0
    max 2
    tunable true
  }
  field direction Vector3 "Where the wind blows: its horizontal part, normalised." {
    default 1 0 0
    tunable true
  }
  field speed f32 "How fast the gusts travel downwind, over a 10 m wave." {
    default 3
    min 0
    max 50
    unit mps
    tunable true
  }
  field gust f32 "How much the lean varies in the gusts: 0 a steady lean, 1 from upright to twice the mean." {
    default 0.6
    min 0
    max 1
    unit ratio
    tunable true
  }
}
```

<sub>`api.ogt` line 10780</sub>

<a id="record-Water"></a>

### `og.Water` (component)

A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space, the sky's cubemap where that misses, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated).

Token `og.Water`, registered under the schema name `og.Water`; its value type is `Water`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector2` | `{ x: 16, y: 16 }` | — | `m` | tunable | The extent along the entity's x and z. |
| `geometry` | `Geometry` | `0` | — | — | — | Draw the water over this mesh (its vertices in the entity's frame) instead of the size × size grid: a stream, channel or pool of any outline, with the same colour by depth, refraction, foam and reflections. 0: the grid. A mesh surface has no wave heights. |
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
| `planarReflection` | `number` | `0` | 0 … 1 | `ratio` | tunable | Reflect the scene by drawing it again from the camera mirrored in the surface, at this fraction of the screen's resolution (0: off, the screen-space and sky reflections). One surface a frame, the first that asks (a grid surface before one over a mesh), or two with RenderOptions.planarReflections 2 (the second while it is in view); each costs about a second scene pass at that resolution. |
| `shadows` | `number` | `0` | 0 … 1 | `ratio` | tunable | How much the sun's shadows darken the surface: its highlight, foam and colour (0: none). |
| `f0` | `number` | `0.02` | 0 … 1 | `ratio` | tunable | The reflectance looking straight down (Schlick's F0): 0.02 for clean water; three.js Water.js reflects at least 0.3. |
| `reflectionBias` | `number` | `0` | 0 … 1 | `ratio` | tunable | A floor under the reflected colour: bias + (1 - bias) times the reflection (three.js Water.js: 0.1). |
| `underwaterColor` | `Color` | `{ r: 0.03, g: 0.16, b: 0.2 }` | — | — | — | With the camera below the surface: the water's fog colour. |
| `underwaterDensity` | `number` | `0.09` | 0 … 2 | — | tunable | Its fog per metre (0: no underwater view). Below the surface the frame's fog becomes the water's, and the surface seen from below shows the sky inside Snell's window. |
| `caustics` | `number` | `0` | 0 … 4 | — | tunable | Caustics of the sunlight on everything below the surface (0: none). One surface a frame, the first that has them. |
| `causticsScale` | `number` | `0.6` | 0.01 … 10 | — | tunable | The caustics' cells per metre. |
| `normalMap` | `Texture` | `0` | — | — | — | Ripples from a tangent-space normal map (linear; red along +x, green along +z, blue up, as three.js' water normals) instead of the travelling waves: with flow 0, four layers drifting at three.js Water's rates over the world's xz times normalScale (its getNoise); with a flow, two layers sliding along it over the mesh's uv (or the grid's metres), cross-faded each 0.15 of a cycle (three.js Water2: then refraction 1 shifts the scene behind by N.xz · 0.05 of the screen, the reflection is tinted by shallow too, and there is no sun highlight). waveStrength scales the slopes (1: as the map), waveSpeed the lake's clock. 0: the travelling waves. |
| `normalScale` | `number` | `1` | ≥ 0 | — | tunable | The normal map's repeats per metre (the lake) or per uv unit (a flow). |
| `glint` | `number` | `0` | ≥ 0 | — | tunable | three.js Water's sun glint: pow(eye . reflect(-sun, normal), 100) times the sun's light times this, on the reflected image, through the Fresnel term (three.js: spec × sunColor over the light's intensity). With glint the surface is three.js Water.js throughout: deep is its waterColor, the body (sunColor · max(L·N, 0) · sunColor · 0.15 + waterColor · max(N·V, 0)) shaded by the ripples and the shadows, sunColor = the light × glint / 2. 0: the default half-vector highlight and depth colour. |
| `flow` | `Vector2` | `{ x: 0, y: 0 }` | — | — | tunable | Water2's flow: its direction in the mesh's uv (or the grid's x, z) and, as its length, the uv it slides per second. 0 0: the lake's four drifting layers. Without a normalMap a flow carries the travelling waves, in metres a second: a river. |
| `flowMap` | `Texture` | `0` | — | — | — | three.js Water2's flowMap: the flow's direction per point as rg * 2 - 1 (linear; the image's top at v 1), over the mesh's uv or the grid from its -x -z corner (u along x, v along z). With it flow's length is the speed (three.js' flowSpeed; 0 0: its 0.03) and the map the direction, for the normal map's two layers or, without one, the travelling waves: a stream that bends round rocks. 0: the flow alone. |
| `wind` | `Vector2` | `{ x: 0, y: 0 }` | — | `mps` | tunable | The wind over an open sea, its direction in the world's xz and its speed in m/s as its length: the grid rises in a sum of Gerstner waves of a fully developed sea for that wind (Pierson–Moskowitz; at 10 m/s waves about 80 m long and a significant height of 2.1 m), over the world's xz (a level surface), with og.physics.waterHeight reading the tick's waves in the tick. A mesh surface has none. The sea is drawn as a clipmap about the camera, finest under it, from a seeded JONSWAP spectrum transformed on the GPU each frame at the tick's time (up to three tiles of 256² by the graphics quality) over the tick's longest waves, which it draws as they are: the waves a floating body rides are the ones drawn, the shorter ones the GPU's alone (rendering only; never read back). Within size (centred on the entity) unless endless. The detail waves (waveStrength) fade where a pixel cannot hold them. Whitecaps foam where the crests fold. 0 0: no sea. |
| `swell` | `number` | `1` | 0 … 4 | `ratio` | tunable | The sea's height, as a fraction of a fully developed sea's for the wind. |
| `choppiness` | `number` | `0.6` | 0 … 1 | `ratio` | tunable | How far the sea's crests lean and sharpen (Gerstner steepness): 0 round swell, 1 the most that never folds over. Whitecaps foam where the crests bunch, by as much. |
| `endless` | `boolean` | `false` | — | — | — | A sea (wind) to the horizon all round the camera, wherever it goes, not only within size: its waves stay where they are in the world. false: the sea ends at the size rectangle (a bay, a lake in a gale). |

```ogt
component Water {
  doc "A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space, the sky's cubemap where that misses, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated)."
  field size Vector2 "The extent along the entity's x and z." {
    default 16 16
    unit m
    tunable true
  }
  field geometry Geometry "Draw the water over this mesh (its vertices in the entity's frame) instead of the size × size grid: a stream, channel or pool of any outline, with the same colour by depth, refraction, foam and reflections. 0: the grid. A mesh surface has no wave heights."
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
  field planarReflection f32 "Reflect the scene by drawing it again from the camera mirrored in the surface, at this fraction of the screen's resolution (0: off, the screen-space and sky reflections). One surface a frame, the first that asks (a grid surface before one over a mesh), or two with RenderOptions.planarReflections 2 (the second while it is in view); each costs about a second scene pass at that resolution." {
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
  field f0 f32 "The reflectance looking straight down (Schlick's F0): 0.02 for clean water; three.js Water.js reflects at least 0.3." {
    default 0.02
    min 0
    max 1
    unit ratio
    tunable true
  }
  field reflectionBias f32 "A floor under the reflected colour: bias + (1 - bias) times the reflection (three.js Water.js: 0.1)." {
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
  field normalMap Texture "Ripples from a tangent-space normal map (linear; red along +x, green along +z, blue up, as three.js' water normals) instead of the travelling waves: with flow 0, four layers drifting at three.js Water's rates over the world's xz times normalScale (its getNoise); with a flow, two layers sliding along it over the mesh's uv (or the grid's metres), cross-faded each 0.15 of a cycle (three.js Water2: then refraction 1 shifts the scene behind by N.xz · 0.05 of the screen, the reflection is tinted by shallow too, and there is no sun highlight). waveStrength scales the slopes (1: as the map), waveSpeed the lake's clock. 0: the travelling waves."
  field normalScale f32 "The normal map's repeats per metre (the lake) or per uv unit (a flow)." {
    default 1
    min 0
    tunable true
  }
  field glint f32 "three.js Water's sun glint: pow(eye . reflect(-sun, normal), 100) times the sun's light times this, on the reflected image, through the Fresnel term (three.js: spec × sunColor over the light's intensity). With glint the surface is three.js Water.js throughout: deep is its waterColor, the body (sunColor · max(L·N, 0) · sunColor · 0.15 + waterColor · max(N·V, 0)) shaded by the ripples and the shadows, sunColor = the light × glint / 2. 0: the default half-vector highlight and depth colour." {
    min 0
    tunable true
  }
  field flow Vector2 "Water2's flow: its direction in the mesh's uv (or the grid's x, z) and, as its length, the uv it slides per second. 0 0: the lake's four drifting layers. Without a normalMap a flow carries the travelling waves, in metres a second: a river." {
    default 0 0
    tunable true
  }
  field flowMap Texture "three.js Water2's flowMap: the flow's direction per point as rg * 2 - 1 (linear; the image's top at v 1), over the mesh's uv or the grid from its -x -z corner (u along x, v along z). With it flow's length is the speed (three.js' flowSpeed; 0 0: its 0.03) and the map the direction, for the normal map's two layers or, without one, the travelling waves: a stream that bends round rocks. 0: the flow alone."
  field wind Vector2 "The wind over an open sea, its direction in the world's xz and its speed in m/s as its length: the grid rises in a sum of Gerstner waves of a fully developed sea for that wind (Pierson–Moskowitz; at 10 m/s waves about 80 m long and a significant height of 2.1 m), over the world's xz (a level surface), with og.physics.waterHeight reading the tick's waves in the tick. A mesh surface has none. The sea is drawn as a clipmap about the camera, finest under it, from a seeded JONSWAP spectrum transformed on the GPU each frame at the tick's time (up to three tiles of 256² by the graphics quality) over the tick's longest waves, which it draws as they are: the waves a floating body rides are the ones drawn, the shorter ones the GPU's alone (rendering only; never read back). Within size (centred on the entity) unless endless. The detail waves (waveStrength) fade where a pixel cannot hold them. Whitecaps foam where the crests fold. 0 0: no sea." {
    default 0 0
    unit mps
    tunable true
  }
  field swell f32 "The sea's height, as a fraction of a fully developed sea's for the wind." {
    default 1
    min 0
    max 4
    unit ratio
    tunable true
  }
  field choppiness f32 "How far the sea's crests lean and sharpen (Gerstner steepness): 0 round swell, 1 the most that never folds over. Whitecaps foam where the crests bunch, by as much." {
    default 0.6
    min 0
    max 1
    unit ratio
    tunable true
  }
  field endless bool "A sea (wind) to the horizon all round the camera, wherever it goes, not only within size: its waves stay where they are in the world. false: the sea ends at the size rectangle (a bay, a lake in a gale)."
}
```

<sub>`api.ogt` line 10807</sub>

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

<sub>`api.ogt` line 10956</sub>

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

<sub>`api.ogt` line 10249</sub>

```ogt
const WEATHER_RAIN u32 1 "Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog."
```

<sub>`api.ogt` line 10250</sub>

```ogt
const WEATHER_SNOW u32 2 "Weather.kind: flakes drifting down on gusts, a pale fog."
```

<sub>`api.ogt` line 10251</sub>

```ogt
const WEATHER_DUST u32 3 "Weather.kind: dust blowing along the wind near the ground, a warm thick fog."
```

<sub>`api.ogt` line 10252</sub>

```ogt
const WEATHER_MIST u32 4 "Weather.kind: no particles, a dense low height fog."
```

<sub>`api.ogt` line 10253</sub>
