<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `atmosphere` module

Fog, a procedural sky, the day-night cycle, weather, the wind that sways vegetation and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before.

The day-night clock is a function of the tick (DayNight.start plus the simulated time times speed), so it is the same on every run and after a restore; rendering reads the world and never writes it.

- **TypeScript**: functions are the namespace `og.atmosphere`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module atmosphere {
  doc "Fog, a procedural sky, the day-night cycle, weather, the wind that sways vegetation and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before."
  doc "The day-night clock is a function of the tick (DayNight.start plus the simulated time times speed), so it is the same on every run and after a restore; rendering reads the world and never writes it."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 10545</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`WEATHER_NONE`](#const-WEATHER_NONE) | constant |  | Weather.kind: clear. |
| [`WEATHER_RAIN`](#const-WEATHER_RAIN) | constant |  | Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog. |
| [`WEATHER_SNOW`](#const-WEATHER_SNOW) | constant |  | Weather.kind: flakes drifting down on gusts, a pale fog. |
| [`WEATHER_DUST`](#const-WEATHER_DUST) | constant |  | Weather.kind: dust blowing along the wind near the ground, a warm thick fog. |
| [`WEATHER_MIST`](#const-WEATHER_MIST) | constant |  | Weather.kind: no particles, a dense low height fog. |
| [`og.Fog`](atmosphere/record-Fog.md) | resource |  | Distance and height fog over the 3D scene and world sprites: distance fog, measured from the camera, thickens exponentially by density per metre past start and, when end > start, also by a linear ram… |
| [`og.Sky`](atmosphere/record-Sky.md) | resource |  | A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it: the image-based light and the reflections' sky (where a surface reflects no probe and SSR misses) follow it as it… |
| [`og.DayNight`](atmosphere/record-DayNight.md) | resource |  | A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient li… |
| [`og.PhysicalSky`](atmosphere/record-PhysicalSky.md) | resource |  | A physical sky: sunlight scattered by the air (Rayleigh: the blue overhead, the red sunsets) and by haze (Mie: the white glow around the sun), with ozone keeping the twilight zenith blue, in a spheri… |
| [`og.Clouds`](atmosphere/record-Clouds.md) | resource |  | Clouds in the procedural sky: a volumetric layer of cumulus raymarched through a curved shell over the planet, lit by the first DirectionalLight (shadowed within themselves, with multiple scattering:… |
| [`og.Aurora`](atmosphere/record-Aurora.md) | resource |  | The aurora: curtains of light in the upper atmosphere, raymarched in the procedural sky (after Lawlor and Genetti, Interactive Volume Rendering Aurora on the GPU, 2011). |
| [`og.WeatherState`](atmosphere/record-WeatherState.md) | resource |  | The weather's effect on the world, simulated in the tick from og.Weather (its kind and intensity) so gameplay can read it: how wet the ground is, how much standing water and snow lie on it, and the l… |
| [`og.LightningStrike`](atmosphere/record-LightningStrike.md) | event |  | Lightning struck, in the tick og.weather drew it (og.WeatherState.lightning): read in the systems after og.weather that tick, or the next tick's first. |
| [`og.Weather`](atmosphere/record-Weather.md) | resource |  | A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. |
| [`og.Wind`](atmosphere/record-Wind.md) | resource |  | The wind the renderer sways vegetation with: og.TerrainScatter copies and their impostor cards, og.Grass blades, swaying trees, and og.Model instances with a sway, lean downwind and sway in gusts tha… |
| [`WindMotorKind`](atmosphere/type-WindMotorKind.md) | enum |  | How an og.WindMotor pushes the wind. |
| [`WindMotorKind()`](#column-WindMotorKind) | column kind |  | A WindMotorKind field: the variant's index. |
| [`og.WindMotor`](atmosphere/record-WindMotor.md) | component |  | A local push on the wind the renderer sways vegetation with, at the entity's position: grass blades, og.TerrainScatter copies and their impostor cards, and swaying trees within `radius` lean away fro… |
| [`og.Water`](atmosphere/record-Water.md) | component |  | A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflectio… |
| [`og.Water2D`](atmosphere/record-Water2D.md) | component |  | Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface. |
| [`og.atmosphere.timeOfDay`](atmosphere/fn-timeOfDay.md) | function | `(world: World): number` | The DayNight clock at the current tick, in hours from 0 to 24 (its start when the cycle is off): for lamps that light at dusk and shops that close at night. |

## Column kinds

Each is a factory, `WindMotorKind(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-WindMotorKind"></a>`WindMotorKind` | `WindMotorKind` | `Uint32Array` | 1 | A WindMotorKind field: the variant's index. |

```ogt
column WindMotorKind {
  doc "A WindMotorKind field: the variant's index."
  array Uint32Array
  stride 1
  value WindMotorKind
}
```

<sub>`api.ogt` line 11147</sub>

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

<sub>`api.ogt` line 10549</sub>

```ogt
const WEATHER_RAIN u32 1 "Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog."
```

<sub>`api.ogt` line 10550</sub>

```ogt
const WEATHER_SNOW u32 2 "Weather.kind: flakes drifting down on gusts, a pale fog."
```

<sub>`api.ogt` line 10551</sub>

```ogt
const WEATHER_DUST u32 3 "Weather.kind: dust blowing along the wind near the ground, a warm thick fog."
```

<sub>`api.ogt` line 10552</sub>

```ogt
const WEATHER_MIST u32 4 "Weather.kind: no particles, a dense low height fog."
```

<sub>`api.ogt` line 10553</sub>
