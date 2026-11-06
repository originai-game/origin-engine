<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Water"></a>

### `og.Water` (component)

A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space, the sky's cubemap where that misses, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated).

Token `og.Water`, registered under the schema name `og.Water`; its value type is `Water`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector2` | `{ x: 16, y: 16 }` | — | `m` | tunable | The extent along the entity's x and z. |
| `geometry` | `Geometry` | `0` | — | — | — | Draw the water over this mesh (its vertices in the entity's frame) instead of the size × size grid: a stream, channel or pool of any outline, with the same colour by depth, refraction, foam and reflections. 0: the grid. A mesh surface has no wave heights. |
| `river` | `Path` | `0` | — | — | — | A river along this path (og.path.create or og.path.find; its points where the water runs, their height the water's level): the water is a ribbon riverWidth across down the course, drawn in place of the size grid, its current riverSpeed mid-stream along the path, turning with it and slowing to nothing at the banks; an og.Buoyancy body in it drifts with that current and floats at its level (computed in the tick, the same on every run). Where the course drops steeper than 45 degrees the water falls, a sheet streaked white, and the pool below it churns white for some three widths. 0: none. |
| `riverWidth` | `number` | `6` | ≥ 0 | `m` | tunable | The river's width across. |
| `riverSpeed` | `number` | `1.5` | ≥ 0 | `mps` | tunable | The river's current mid-stream; it slows to nothing at the banks. |
| `segments` | `number` | `64` | 1 … 256 | — | — | Grid cells per side (the height field's detail). |
| `shallow` | `Color` | `{ r: 0.55, g: 0.85, b: 0.82 }` | — | — | — | Shallow water's colour (tints what is seen through it). |
| `deep` | `Color` | `{ r: 0.02, g: 0.09, b: 0.14 }` | — | — | — | Deep water's colour. |
| `depthFade` | `number` | `2.5` | ≥ 0.01 | `m` | tunable | The depth over which water turns from shallow to deep. |
| `absorption` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | How much of each colour the water absorbs per metre of light's path through it (1/m; pure sea water about 0.45 0.07 0.03, the red gone in a few metres). With absorption or scattering set, the water's body is physical: what lies below is dimmed by the path the light takes down to it and back up to the eye, and the water scatters the sun and sky it lets in back out, so shallows over sand turn turquoise and deep water dark blue-green by themselves; shallow, deep, depthFade and opacity are then unused. 0 0 0 with scattering 0 0 0: shallow and deep. |
| `scattering` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | How much of each colour the water scatters per metre (1/m; clear ocean about 0.002 0.004 0.008, a silty coast ten times that). See absorption. |
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
| `seabed` | `Entity` | `0` | — | — | — | The og.Terrain entity whose heights are the sea's floor (0: none, the open sea everywhere): the sea feels the depth over it, each wave fading where the water is shallower than half its length and the whole held under its breaking height (a significant height of 0.78 times the depth), so it never runs through a beach; trains of waves roll in parallel to the shore and break into surf (shore, shoreFoam). og.physics.waterHeight and og.Buoyancy read the same shoaled sea: a boat in the shallows rides what is drawn there. Read when the water, the terrain's placement or its edits change. |
| `shore` | `number` | `1` | 0 … 4 | `ratio` | tunable | How high the trains rolling in on the seabed's shores are, times a quarter of the sea's significant height (0: none; the waves still shoal). |
| `shoreWaveLength` | `number` | `0` | 0 … 400 | `m` | tunable | The trains' length from crest to crest (0: 40 m). |
| `shoreFoam` | `number` | `1` | 0 … 2 | `ratio` | tunable | How much surf foams where the trains break and the water runs shallow over the seabed (0: none). |
| `foamStreaks` | `number` | `1` | 0 … 4 | `ratio` | tunable | How strongly a sea's ageing foam is drawn into streaks along the wind (0: none): a whitecap's dense foam bursts within seconds and leaves a thin lace that lasts some ten seconds, which the surface's convergence along the wind (Langmuir circulation, about 1 % of the wind's speed) gathers into windrows. Rendering only. |
| `meniscus` | `number` | `2` | -1 … 16 | — | tunable | Where the camera's near plane crosses the surface (a camera bobbing at the waterline), the width in pixels of the meniscus drawn along the line between the air above and the water's fog below it, each pixel under the surface or not as the waves are there (as og.physics.waterHeight puts them); 0: the line with no meniscus; below 0: no waterline, the whole frame above or under the water as the eye is. Needs underwaterDensity. |
| `dynamicWaves` | `number` | `1` | 0 … 4 | `ratio` | tunable | How high the waves an og.Buoyancy hull raises on a sea are drawn (0: none): a hull presses the water down by its draft, and as it moves it throws a bow wave and leaves a Kelvin wake (the 19.5 degree wedge of transverse and diverging waves deep water makes behind any moving body), with foam where those crests break and along its sides, all running at deep water's own speeds within some 64 m of the camera. Rendering only: the tick floats every hull on the sea's own waves and never sees these. |
| `spray` | `number` | `1` | 0 … 4 | `ratio` | tunable | How much spray a sea's breaking crests throw (0: none): droplets torn off where the foam is freshest and the crests steepest, carried by the wind and falling back, more of them as the wind rises past a moderate breeze. Rendering only; the tick never sees them. |

```ogt
component Water {
  doc "A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space, the sky's cubemap where that misses, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated)."
  field size Vector2 "The extent along the entity's x and z." {
    default 16 16
    unit m
    tunable true
  }
  field geometry Geometry "Draw the water over this mesh (its vertices in the entity's frame) instead of the size × size grid: a stream, channel or pool of any outline, with the same colour by depth, refraction, foam and reflections. 0: the grid. A mesh surface has no wave heights."
  field river Path "A river along this path (og.path.create or og.path.find; its points where the water runs, their height the water's level): the water is a ribbon riverWidth across down the course, drawn in place of the size grid, its current riverSpeed mid-stream along the path, turning with it and slowing to nothing at the banks; an og.Buoyancy body in it drifts with that current and floats at its level (computed in the tick, the same on every run). Where the course drops steeper than 45 degrees the water falls, a sheet streaked white, and the pool below it churns white for some three widths. 0: none."
  field riverWidth f32 "The river's width across." {
    default 6
    min 0
    unit m
    tunable true
  }
  field riverSpeed f32 "The river's current mid-stream; it slows to nothing at the banks." {
    default 1.5
    min 0
    unit mps
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
  field absorption Color "How much of each colour the water absorbs per metre of light's path through it (1/m; pure sea water about 0.45 0.07 0.03, the red gone in a few metres). With absorption or scattering set, the water's body is physical: what lies below is dimmed by the path the light takes down to it and back up to the eye, and the water scatters the sun and sky it lets in back out, so shallows over sand turn turquoise and deep water dark blue-green by themselves; shallow, deep, depthFade and opacity are then unused. 0 0 0 with scattering 0 0 0: shallow and deep." {
    default 0 0 0
  }
  field scattering Color "How much of each colour the water scatters per metre (1/m; clear ocean about 0.002 0.004 0.008, a silty coast ten times that). See absorption." {
    default 0 0 0
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
  field seabed Entity "The og.Terrain entity whose heights are the sea's floor (0: none, the open sea everywhere): the sea feels the depth over it, each wave fading where the water is shallower than half its length and the whole held under its breaking height (a significant height of 0.78 times the depth), so it never runs through a beach; trains of waves roll in parallel to the shore and break into surf (shore, shoreFoam). og.physics.waterHeight and og.Buoyancy read the same shoaled sea: a boat in the shallows rides what is drawn there. Read when the water, the terrain's placement or its edits change."
  field shore f32 "How high the trains rolling in on the seabed's shores are, times a quarter of the sea's significant height (0: none; the waves still shoal)." {
    default 1
    min 0
    max 4
    unit ratio
    tunable true
  }
  field shoreWaveLength f32 "The trains' length from crest to crest (0: 40 m)." {
    min 0
    max 400
    unit m
    tunable true
  }
  field shoreFoam f32 "How much surf foams where the trains break and the water runs shallow over the seabed (0: none)." {
    default 1
    min 0
    max 2
    unit ratio
    tunable true
  }
  field foamStreaks f32 "How strongly a sea's ageing foam is drawn into streaks along the wind (0: none): a whitecap's dense foam bursts within seconds and leaves a thin lace that lasts some ten seconds, which the surface's convergence along the wind (Langmuir circulation, about 1 % of the wind's speed) gathers into windrows. Rendering only." {
    default 1
    min 0
    max 4
    unit ratio
    tunable true
  }
  field meniscus f32 "Where the camera's near plane crosses the surface (a camera bobbing at the waterline), the width in pixels of the meniscus drawn along the line between the air above and the water's fog below it, each pixel under the surface or not as the waves are there (as og.physics.waterHeight puts them); 0: the line with no meniscus; below 0: no waterline, the whole frame above or under the water as the eye is. Needs underwaterDensity." {
    default 2
    min -1
    max 16
    tunable true
  }
  field dynamicWaves f32 "How high the waves an og.Buoyancy hull raises on a sea are drawn (0: none): a hull presses the water down by its draft, and as it moves it throws a bow wave and leaves a Kelvin wake (the 19.5 degree wedge of transverse and diverging waves deep water makes behind any moving body), with foam where those crests break and along its sides, all running at deep water's own speeds within some 64 m of the camera. Rendering only: the tick floats every hull on the sea's own waves and never sees these." {
    default 1
    min 0
    max 4
    unit ratio
    tunable true
  }
  field spray f32 "How much spray a sea's breaking crests throw (0: none): droplets torn off where the foam is freshest and the crests steepest, carried by the wind and falling back, more of them as the wind rises past a moderate breeze. Rendering only; the tick never sees them." {
    default 1
    min 0
    max 4
    unit ratio
    tunable true
  }
}
```

<sub>`api.ogt` line 11218</sub>
