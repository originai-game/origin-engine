<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `render` module

What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.

- **TypeScript**: functions are the namespace `og.render`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module render {
  doc "What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1064</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Geometry`](#type-Geometry) | handle | Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF. |
| [`Material`](#type-Material) | handle | A material instance. |
| [`Texture`](#type-Texture) | handle | A cooked texture. |
| [`Geometry()`](#column-Geometry) | column kind | A Geometry handle field. |
| [`Material()`](#column-Material) | column kind | A Material handle field. |
| [`Side`](#type-Side) | enum | Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide. |
| [`RenderTier`](#type-RenderTier) | enum | The backend the renderer runs on; the probe's hello reports it. |
| [`Antialias`](#type-Antialias) | enum | Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved. |
| [`RenderOptions`](#type-RenderOptions) | desc struct | How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer. |
| [`StandardMaterialDesc`](#type-StandardMaterialDesc) | desc struct | A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names. |
| [`og.Mesh`](#record-Mesh) | component | Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh. |
| [`og.DirectionalLight`](#record-DirectionalLight) | component | The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with three shadow cascades. |
| [`og.PointLight`](#record-PointLight) | component | A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance. |
| [`og.SpotLight`](#record-SpotLight) | component | A cone of light along the entity's local -Z, as three.js SpotLight. |
| [`og.RenderSettings`](#record-RenderSettings) | resource | The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure. |
| [`TextDesc`](#type-TextDesc) | desc struct | A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show). |
| [`TextMeasureDesc`](#type-TextMeasureDesc) | desc struct | Text to measure with og.render.measureText, laid out as og.render.text lays it out. |
| [`TextMetrics`](#type-TextMetrics) | struct | The extent of a laid-out text, in world units. |
| [`SpriteDesc`](#type-SpriteDesc) | desc struct | A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene. |
| [`og.render.sprite`](#fn-sprite) | function | Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show. |
| [`og.render.measureText`](#fn-measureText) | function | The size text takes when og.render.text draws it, for centring, wrapping and layout. |
| [`BoxGeometryDesc`](#type-BoxGeometryDesc) | desc struct | A box centred at the origin, as three.js BoxGeometry(width, height, depth). |
| [`PlaneGeometryDesc`](#type-PlaneGeometryDesc) | desc struct | A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor. |
| [`SphereGeometryDesc`](#type-SphereGeometryDesc) | desc struct | A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments). |
| [`og.render.boxGeometry`](#fn-boxGeometry) | function | Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too. |
| [`og.render.planeGeometry`](#fn-planeGeometry) | function | Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too. |
| [`og.render.sphereGeometry`](#fn-sphereGeometry) | function | Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh). |
| [`MAX_LIGHTS`](#const-MAX_LIGHTS) | constant | The most point and spot lights one frame draws. |
| [`RenderCapabilities`](#type-RenderCapabilities) | struct | What the renderer can do on this device. Backend limits are reported here, never by missing members. |
| [`og.render.standardMaterial`](#fn-standardMaterial) | function | Create a standard material. |
| [`og.render.updateMaterial`](#fn-updateMaterial) | function | Change some parameters of a standard material. |
| [`og.render.capabilities`](#fn-capabilities) | function | The renderer's tier and limits on this device. |
| [`og.render.text`](#fn-text) | function | Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout. |

## Functions

<a id="fn-sprite"></a>

### `og.render.sprite`

Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show.

```ts
og.render.sprite(world: World, desc?: SpriteDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `SpriteDesc` | yes | The texture, placement, region and colour. |

**Returns** `Result<void>`: Refused for a dead texture.

Binding symbol `og_render_sprite` · publish builds: kept

```ogt
fn sprite {
  doc "Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show."
  param world World "The world whose renderer draws it."
  param desc SpriteDesc "The texture, placement, region and colour."
  returns Result.Void "Refused for a dead texture."
}
```

<sub>`api.ogt` line 1359</sub>

<a id="fn-measureText"></a>

### `og.render.measureText`

The size text takes when og.render.text draws it, for centring, wrapping and layout.

```ts
og.render.measureText(world: World, desc?: TextMeasureDesc): Result<TextMetrics>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the font. |
| `desc` | `TextMeasureDesc` | yes | The font, text and size. |

**Returns** `Result<TextMetrics>`: Refused for a dead font.

Binding symbol `og_render_measure_text` · publish builds: kept

```ogt
fn measureText {
  doc "The size text takes when og.render.text draws it, for centring, wrapping and layout."
  param world World "The world whose renderer owns the font."
  param desc TextMeasureDesc "The font, text and size."
  returns Result.TextMetrics "Refused for a dead font."
}
```

<sub>`api.ogt` line 1366</sub>

<a id="fn-boxGeometry"></a>

### `og.render.boxGeometry`

Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too.

```ts
og.render.boxGeometry(world: World, desc?: BoxGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `BoxGeometryDesc` | yes | Its extents. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_box_geometry` · publish builds: kept

```ogt
fn boxGeometry {
  doc "Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too."
  param world World "The world whose renderer owns the geometry."
  param desc BoxGeometryDesc "Its extents."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 1426</sub>

<a id="fn-planeGeometry"></a>

### `og.render.planeGeometry`

Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too.

```ts
og.render.planeGeometry(world: World, desc?: PlaneGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `PlaneGeometryDesc` | yes | Its extents. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_plane_geometry` · publish builds: kept

```ogt
fn planeGeometry {
  doc "Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too."
  param world World "The world whose renderer owns the geometry."
  param desc PlaneGeometryDesc "Its extents."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 1433</sub>

<a id="fn-sphereGeometry"></a>

### `og.render.sphereGeometry`

Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh).

```ts
og.render.sphereGeometry(world: World, desc?: SphereGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `SphereGeometryDesc` | yes | Its radius and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_sphere_geometry` · publish builds: kept

```ogt
fn sphereGeometry {
  doc "Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh)."
  param world World "The world whose renderer owns the geometry."
  param desc SphereGeometryDesc "Its radius and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 1440</sub>

<a id="fn-standardMaterial"></a>

### `og.render.standardMaterial`

Create a standard material.

```ts
og.render.standardMaterial(world: World, desc?: StandardMaterialDesc): Result<Material>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `desc` | `StandardMaterialDesc` | yes | The parameters; omitted ones take three.js defaults. |

**Returns** `Result<Material>`.

Binding symbol `og_render_standard_material` · publish builds: kept

```ogt
fn standardMaterial {
  doc "Create a standard material."
  param world World "The world whose renderer owns the material."
  param desc StandardMaterialDesc "The parameters; omitted ones take three.js defaults."
  returns Result.Material
}
```

<sub>`api.ogt` line 1460</sub>

<a id="fn-updateMaterial"></a>

### `og.render.updateMaterial`

Change some parameters of a standard material.

```ts
og.render.updateMaterial(world: World, material: Material, desc?: StandardMaterialDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `material` | `Material` | no | The material. |
| `desc` | `StandardMaterialDesc` | yes | The parameters to change; omitted ones keep their values. |

**Returns** `Result<void>`: Refused for a dead material.

Binding symbol `og_render_update_material` · publish builds: kept

```ogt
fn updateMaterial {
  doc "Change some parameters of a standard material."
  param world World "The world whose renderer owns the material."
  param material Material "The material."
  param desc StandardMaterialDesc "The parameters to change; omitted ones keep their values."
  returns Result.Void "Refused for a dead material."
}
```

<sub>`api.ogt` line 1467</sub>

<a id="fn-capabilities"></a>

### `og.render.capabilities`

The renderer's tier and limits on this device.

```ts
og.render.capabilities(world: World): RenderCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer is asked. |

**Returns** `RenderCapabilities`.

Binding symbol `og_render_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "The renderer's tier and limits on this device."
  param world World "The world whose renderer is asked."
  returns RenderCapabilities
}
```

<sub>`api.ogt` line 1475</sub>

<a id="fn-text"></a>

### `og.render.text`

Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout.

```ts
og.render.text(world: World, desc?: TextDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `TextDesc` | yes | The font, text, placement and colour. |

**Returns** `Result<void>`: Refused for a dead font.

Binding symbol `og_render_text` · publish builds: kept

```ogt
fn text {
  doc "Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout."
  param world World "The world whose renderer draws it."
  param desc TextDesc "The font, text, placement and colour."
  returns Result.Void "Refused for a dead font."
}
```

<sub>`api.ogt` line 1481</sub>

## Components, resources and events

<a id="record-Mesh"></a>

### `og.Mesh` (component)

Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh.

The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say.

Token `og.Mesh`, registered under the schema name `og.Mesh`; its value type is `Mesh`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | The geometry. |
| `material` | `Material` | `0` | — | — | — | The material. |
| `castShadow` | `boolean` | `false` | — | — | — | Cast shadows, as three.js Object3D.castShadow. |
| `receiveShadow` | `boolean` | `false` | — | — | — | Receive shadows, as three.js Object3D.receiveShadow. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Mesh {
  doc "Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh."
  doc "The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say."
  field geometry Geometry "The geometry."
  field material Material "The material."
  field castShadow bool "Cast shadows, as three.js Object3D.castShadow."
  field receiveShadow bool "Receive shadows, as three.js Object3D.receiveShadow."
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>`api.ogt` line 1166</sub>

<a id="record-DirectionalLight"></a>

### `og.DirectionalLight` (component)

The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with three shadow cascades.

Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene.

Token `og.DirectionalLight`, registered under the schema name `og.DirectionalLight`; its value type is `DirectionalLight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `3` | ≥ 0 | — | tunable | The radiance multiplier, as three.js DirectionalLight.intensity. |
| `castShadow` | `boolean` | `true` | — | — | — | Cast shadows through three cascades, as three.js Light.castShadow. |
| `shadowDistance` | `number` | `200` | ≥ 1 | `m` | — | How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper. |

```ogt
component DirectionalLight {
  doc "The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with three shadow cascades."
  doc "Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene."
  field color Color "The light's linear colour." {
    default 1 1 1
  }
  field intensity f32 "The radiance multiplier, as three.js DirectionalLight.intensity." {
    default 3
    min 0
    tunable true
  }
  field castShadow bool "Cast shadows through three cascades, as three.js Light.castShadow." {
    default true
  }
  field shadowDistance f32 "How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper." {
    default 200
    min 1
    unit m
  }
}
```

<sub>`api.ogt` line 1178</sub>

<a id="record-PointLight"></a>

### `og.PointLight` (component)

A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.

Up to MAX_LIGHTS point and spot lights are drawn per frame (the rest are counted, not drawn); the first two with castShadow get cube shadows.

Token `og.PointLight`, registered under the schema name `og.PointLight`; its value type is `PointLight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `10` | ≥ 0 | — | tunable | The radiance at one metre. |
| `distance` | `number` | `10` | ≥ 0.01 | `m` | tunable | Where the light reaches zero, as three.js PointLight.distance (never infinite here). |
| `castShadow` | `boolean` | `false` | — | — | — | Cast cube shadows. |

```ogt
component PointLight {
  doc "A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance."
  doc "Up to MAX_LIGHTS point and spot lights are drawn per frame (the rest are counted, not drawn); the first two with castShadow get cube shadows."
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
  field castShadow bool "Cast cube shadows."
}
```

<sub>`api.ogt` line 1199</sub>

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
}
```

<sub>`api.ogt` line 1219</sub>

<a id="record-RenderSettings"></a>

### `og.RenderSettings` (resource)

The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.

Token `og.RenderSettings`, registered under the schema name `og.RenderSettings`; its value type is `RenderSettings`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `environmentIntensity` | `number` | `0.55` | ≥ 0 | — | tunable | The strength of the image-based ambient light from the procedural sky. |
| `sky` | `boolean` | `false` | — | — | — | Draw the procedural sky behind the scene instead of Camera.background. |
| `bloom` | `boolean` | `false` | — | — | — | Light above the threshold bleeds into its surroundings. |
| `bloomThreshold` | `number` | `1` | ≥ 0 | — | tunable | Brightness after exposure where bloom starts; 1 is display white. |
| `bloomKnee` | `number` | `0.5` | ≥ 0 | — | — | Softens the threshold over threshold plus or minus knee; 0 is a hard cut. |
| `bloomIntensity` | `number` | `0.6` | ≥ 0 | — | tunable | How much of the blurred light is added back. |
| `autoExposure` | `boolean` | `false` | — | — | — | Meter every frame to middle grey (deterministic: the current frame only), then apply Camera.exposure as compensation. |
| `toneMapped` | `boolean` | `true` | — | — | — | ACES tone mapping; false clamps, so 2D colours come out exactly as authored at exposure 1. |

```ogt
resource RenderSettings {
  doc "The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure."
  field environmentIntensity f32 "The strength of the image-based ambient light from the procedural sky." {
    default 0.55
    min 0
    tunable true
  }
  field sky bool "Draw the procedural sky behind the scene instead of Camera.background."
  field bloom bool "Light above the threshold bleeds into its surroundings."
  field bloomThreshold f32 "Brightness after exposure where bloom starts; 1 is display white." {
    default 1
    min 0
    tunable true
  }
  field bloomKnee f32 "Softens the threshold over threshold plus or minus knee; 0 is a hard cut." {
    default 0.5
    min 0
  }
  field bloomIntensity f32 "How much of the blurred light is added back." {
    default 0.6
    min 0
    tunable true
  }
  field autoExposure bool "Meter every frame to middle grey (deterministic: the current frame only), then apply Camera.exposure as compensation."
  field toneMapped bool "ACES tone mapping; false clamps, so 2D colours come out exactly as authored at exposure 1." {
    default true
  }
}
```

<sub>`api.ogt` line 1250</sub>

## Types

<a id="type-Geometry"></a>

### `Geometry` (handle)

Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Geometry "Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF."
```

<sub>`api.ogt` line 1067</sub>

<a id="type-Material"></a>

### `Material` (handle)

A material instance.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Material "A material instance."
```

<sub>`api.ogt` line 1068</sub>

<a id="type-Texture"></a>

### `Texture` (handle)

A cooked texture.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Texture "A cooked texture."
```

<sub>`api.ogt` line 1069</sub>

<a id="type-Side"></a>

### `Side` (enum)

Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide.

A string: one of the variants below. The zero value is `"front"`.

| Variant | Doc |
|---|---|
| `"front"` | Front faces only. |
| `"back"` | Back faces only. |
| `"double"` | Both. |

```ogt
enum Side {
  doc "Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide."
  variant front "Front faces only."
  variant back "Back faces only."
  variant double "Both."
}
```

<sub>`api.ogt` line 1085</sub>

<a id="type-RenderTier"></a>

### `RenderTier` (enum)

The backend the renderer runs on; the probe's hello reports it.

A string: one of the variants below. The zero value is `"webgpu"`.

| Variant | Doc |
|---|---|
| `"webgpu"` | WebGPU in a browser: the full feature set. |
| `"webgl2"` | The WebGL2 fallback: no compute, CPU culling and particles, tiled lights. |
| `"vulkan"` | Native Vulkan, on lavapipe or a GPU. |
| `"metal"` | Native Metal. |
| `"none"` | No renderer: frames are not produced and capture refuses. |

```ogt
enum RenderTier {
  doc "The backend the renderer runs on; the probe's hello reports it."
  variant webgpu "WebGPU in a browser: the full feature set."
  variant webgl2 "The WebGL2 fallback: no compute, CPU culling and particles, tiled lights."
  variant vulkan "Native Vulkan, on lavapipe or a GPU."
  variant metal "Native Metal."
  variant none "No renderer: frames are not produced and capture refuses."
}
```

<sub>`api.ogt` line 1092</sub>

<a id="type-Antialias"></a>

### `Antialias` (enum)

Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved.

A string: one of the variants below. The zero value is `"tier"`.

| Variant | Doc |
|---|---|
| `"tier"` | The tier's method: TAA on the full tier (WebGPU, Metal, Vulkan), 4x MSAA on WebGL2. |
| `"off"` | One sample per pixel. |
| `"msaa4"` | 4x MSAA on any tier: sharp sprites and text, no temporal blur; off where the HDR format cannot be multisampled. |
| `"taa"` | Temporal antialiasing on any tier: a sub-pixel jitter per frame and a reprojected history; converges over about eight frames. |

```ogt
enum Antialias {
  doc "Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved."
  variant tier "The tier's method: TAA on the full tier (WebGPU, Metal, Vulkan), 4x MSAA on WebGL2."
  variant off "One sample per pixel."
  variant msaa4 "4x MSAA on any tier: sharp sprites and text, no temporal blur; off where the HDR format cannot be multisampled."
  variant taa "Temporal antialiasing on any tier: a sub-pixel jitter per frame and a reprojected history; converges over about eight frames."
}
```

<sub>`api.ogt` line 1101</sub>

<a id="type-RenderOptions"></a>

### `RenderOptions` (desc struct)

How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing method. |
| `shadowMapSize` | `number` | `2048` | 256 … 4096 | — | — | The side of each of the directional light's three cascade maps, in texels; smaller saves memory on phones (three 2048 maps are 64 MB). |

```ogt
struct RenderOptions {
  doc "How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer."
  desc true
  field antialias Antialias "The antialiasing method."
  field shadowMapSize u32 "The side of each of the directional light's three cascade maps, in texels; smaller saves memory on phones (three 2048 maps are 64 MB)." {
    default 2048
    min 256
    max 4096
  }
}
```

<sub>`api.ogt` line 1109</sub>

<a id="type-StandardMaterialDesc"></a>

### `StandardMaterialDesc` (desc struct)

A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names.

Mapping to the renderer (og-render Material): color and opacity are the base colour and its alpha; map the base colour texture; metalness and roughness the factors; metalnessMap and roughnessMap are one glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; normalMap the normal texture; emissive times emissiveIntensity the emitted colour, emissiveMap its texture; alphaTest above 0 is alpha masking with that cutoff.

Accepted and kept but not drawn yet: aoMap, transparent (blending), side (meshes draw front faces), flatShading and envMapIntensity; the renderer's own phase-2 list owns them.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The base colour. |
| `map` | `Texture \| null` | `null` | — | — | — | The base colour texture. |
| `metalness` | `number` | `0` | 0 … 1 | — | — | How metallic the surface is. |
| `metalnessMap` | `Texture \| null` | `null` | — | — | — | Metalness from the texture's blue channel. |
| `roughness` | `number` | `1` | 0 … 1 | — | — | How rough the surface is. |
| `roughnessMap` | `Texture \| null` | `null` | — | — | — | Roughness from the texture's green channel. |
| `normalMap` | `Texture \| null` | `null` | — | — | — | A tangent-space normal map. |
| `aoMap` | `Texture \| null` | `null` | — | — | — | Ambient occlusion from the texture's red channel. |
| `emissive` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The emitted colour. |
| `emissiveIntensity` | `number` | `1` | ≥ 0 | — | — | A multiplier on the emitted colour. |
| `emissiveMap` | `Texture \| null` | `null` | — | — | — | The emitted colour texture. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The opacity; below 1 only with transparent. |
| `transparent` | `boolean` | `false` | — | — | — | Blend instead of writing opaque pixels. |
| `alphaTest` | `number` | `0` | 0 … 1 | — | — | Discard pixels whose alpha is below this. |
| `side` | `Side` | `"front"` | — | — | — | Which faces are drawn. |
| `flatShading` | `boolean` | `false` | — | — | — | Shade with face normals. |
| `envMapIntensity` | `number` | `1` | ≥ 0 | — | — | A multiplier on image-based lighting. |

```ogt
struct StandardMaterialDesc {
  doc "A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names."
  doc "Mapping to the renderer (og-render Material): color and opacity are the base colour and its alpha; map the base colour texture; metalness and roughness the factors; metalnessMap and roughnessMap are one glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; normalMap the normal texture; emissive times emissiveIntensity the emitted colour, emissiveMap its texture; alphaTest above 0 is alpha masking with that cutoff."
  doc "Accepted and kept but not drawn yet: aoMap, transparent (blending), side (meshes draw front faces), flatShading and envMapIntensity; the renderer's own phase-2 list owns them."
  desc true
  field color Color "The base colour." {
    default 1 1 1
  }
  field map Option.Texture "The base colour texture."
  field metalness f32 "How metallic the surface is." {
    min 0
    max 1
  }
  field metalnessMap Option.Texture "Metalness from the texture's blue channel."
  field roughness f32 "How rough the surface is." {
    default 1
    min 0
    max 1
  }
  field roughnessMap Option.Texture "Roughness from the texture's green channel."
  field normalMap Option.Texture "A tangent-space normal map."
  field aoMap Option.Texture "Ambient occlusion from the texture's red channel."
  field emissive Color "The emitted colour."
  field emissiveIntensity f32 "A multiplier on the emitted colour." {
    default 1
    min 0
  }
  field emissiveMap Option.Texture "The emitted colour texture."
  field opacity f32 "The opacity; below 1 only with transparent." {
    default 1
    min 0
    max 1
  }
  field transparent bool "Blend instead of writing opaque pixels."
  field alphaTest f32 "Discard pixels whose alpha is below this." {
    min 0
    max 1
  }
  field side Side "Which faces are drawn."
  field flatShading bool "Shade with face normals."
  field envMapIntensity f32 "A multiplier on image-based lighting." {
    default 1
    min 0
  }
}
```

<sub>`api.ogt` line 1120</sub>

<a id="type-TextDesc"></a>

### `TextDesc` (desc struct)

A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `font` | `Font \| null` | `null` | — | — | — | The font from og.load.font; null draws nothing. |
| `text` | `string` | `""` | — | — | — | The text; `\n` starts a new line. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The baseline origin of the first line. |
| `size` | `number` | `1` | ≥ 0 | — | — | The em size in world units. |
| `rotation` | `number` | `0` | — | `rad` | — | Rotation about +z. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `layer` | `number` | `0` | — | — | — | Draw order among sprites and text; higher draws on top. |
| `align` | `TextAlign` | `"start"` | — | — | — | How each line sits on `position`: start puts its left end there, center its middle, end its right end. |

```ogt
struct TextDesc {
  doc "A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show)."
  desc true
  field font Option.Font "The font from og.load.font; null draws nothing."
  field text String "The text; `\\n` starts a new line."
  field position Vector3 "The baseline origin of the first line." {
    unit m
  }
  field size f32 "The em size in world units." {
    default 1
    min 0
  }
  field rotation f32 "Rotation about +z." {
    unit rad
  }
  field color Color "The linear colour." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field layer i32 "Draw order among sprites and text; higher draws on top."
  field align TextAlign "How each line sits on `position`: start puts its left end there, center its middle, end its right end."
}
```

<sub>`api.ogt` line 1279</sub>

<a id="type-TextMeasureDesc"></a>

### `TextMeasureDesc` (desc struct)

Text to measure with og.render.measureText, laid out as og.render.text lays it out.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `font` | `Font \| null` | `null` | — | — | — | The font from og.load.font. |
| `text` | `string` | `""` | — | — | — | The text; `\n` starts a new line. |
| `size` | `number` | `1` | ≥ 0 | — | — | The em size in world units. |

```ogt
struct TextMeasureDesc {
  doc "Text to measure with og.render.measureText, laid out as og.render.text lays it out."
  desc true
  field font Option.Font "The font from og.load.font."
  field text String "The text; `\\n` starts a new line."
  field size f32 "The em size in world units." {
    default 1
    min 0
  }
}
```

<sub>`api.ogt` line 1306</sub>

<a id="type-TextMetrics"></a>

### `TextMetrics` (struct)

The extent of a laid-out text, in world units.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | — | — | The widest line's advance. |
| `height` | `number` | `0` | — | — | — | Lines times the font's line height. |
| `lines` | `number` | `0` | — | — | — | How many lines. |

```ogt
struct TextMetrics {
  doc "The extent of a laid-out text, in world units."
  field width f32 "The widest line's advance."
  field height f32 "Lines times the font's line height."
  field lines u32 "How many lines."
}
```

<sub>`api.ogt` line 1317</sub>

<a id="type-SpriteDesc"></a>

### `SpriteDesc` (desc struct)

A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The texture from og.load.texture; null draws a solid quad of the colour. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the anchor point is. |
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its width in world units. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its height in world units. |
| `anchor` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | — | The point of the quad at `position`, as a fraction of its size from the bottom-left corner: (0.5, 0.5) is the centre. |
| `rotation` | `number` | `0` | — | `rad` | — | Counter-clockwise rotation about the anchor. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the texture. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation). |
| `layer` | `number` | `0` | — | — | — | Draw order among sprites and text; higher draws on top. |

```ogt
struct SpriteDesc {
  doc "A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene."
  desc true
  field texture Option.Texture "The texture from og.load.texture; null draws a solid quad of the colour."
  field position Vector3 "Where the anchor point is." {
    unit m
  }
  field width f32 "Its width in world units." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its height in world units." {
    default 1
    min 0
    unit m
  }
  field anchor Vector2 "The point of the quad at `position`, as a fraction of its size from the bottom-left corner: (0.5, 0.5) is the centre." {
    default 0.5 0.5
  }
  field rotation f32 "Counter-clockwise rotation about the anchor." {
    unit rad
  }
  field color Color "A linear colour multiplied with the texture." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field uv Option.Array.f32.4 "The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation)."
  field layer i32 "Draw order among sprites and text; higher draws on top."
}
```

<sub>`api.ogt` line 1324</sub>

<a id="type-BoxGeometryDesc"></a>

### `BoxGeometryDesc` (desc struct)

A box centred at the origin, as three.js BoxGeometry(width, height, depth).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its extent along x. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `depth` | `number` | `1` | ≥ 0 | `m` | — | Its extent along z. |

```ogt
struct BoxGeometryDesc {
  doc "A box centred at the origin, as three.js BoxGeometry(width, height, depth)."
  desc true
  field width f32 "Its extent along x." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its extent along y." {
    default 1
    min 0
    unit m
  }
  field depth f32 "Its extent along z." {
    default 1
    min 0
    unit m
  }
}
```

<sub>`api.ogt` line 1373</sub>

<a id="type-PlaneGeometryDesc"></a>

### `PlaneGeometryDesc` (desc struct)

A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its extent along x. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |

```ogt
struct PlaneGeometryDesc {
  doc "A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor."
  desc true
  field width f32 "Its extent along x." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its extent along y." {
    default 1
    min 0
    unit m
  }
}
```

<sub>`api.ogt` line 1393</sub>

<a id="type-SphereGeometryDesc"></a>

### `SphereGeometryDesc` (desc struct)

A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | Its radius. |
| `widthSegments` | `number` | `32` | ≥ 3 | — | — | Segments around the equator. |
| `heightSegments` | `number` | `16` | ≥ 2 | — | — | Rings from pole to pole. |

```ogt
struct SphereGeometryDesc {
  doc "A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments)."
  desc true
  field radius f32 "Its radius." {
    default 1
    min 0
    unit m
  }
  field widthSegments u32 "Segments around the equator." {
    default 32
    min 3
  }
  field heightSegments u32 "Rings from pole to pole." {
    default 16
    min 2
  }
}
```

<sub>`api.ogt` line 1408</sub>

<a id="type-RenderCapabilities"></a>

### `RenderCapabilities` (struct)

What the renderer can do on this device. Backend limits are reported here, never by missing members.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tier` | `RenderTier` | `"webgpu"` | — | — | — | The backend. |
| `compute` | `boolean` | `false` | — | — | — | Whether compute passes run (GPU culling, GPU particles). |
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing the device resolved RenderOptions.antialias to. |
| `maxLights` | `number` | `0` | — | — | — | How many point and spot lights one frame draws. |
| `maxShadowedLights` | `number` | `0` | — | — | — | How many point and spot lights may cast shadows. |
| `width` | `number` | `0` | — | — | — | The render target width in device pixels. |
| `height` | `number` | `0` | — | — | — | The render target height in device pixels. |

```ogt
struct RenderCapabilities {
  doc "What the renderer can do on this device. Backend limits are reported here, never by missing members."
  field tier RenderTier "The backend."
  field compute bool "Whether compute passes run (GPU culling, GPU particles)."
  field antialias Antialias "The antialiasing the device resolved RenderOptions.antialias to."
  field maxLights u32 "How many point and spot lights one frame draws."
  field maxShadowedLights u32 "How many point and spot lights may cast shadows."
  field width u32 "The render target width in device pixels."
  field height u32 "The render target height in device pixels."
}
```

<sub>`api.ogt` line 1449</sub>

## Column kinds

Each is a factory, `Geometry(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Geometry"></a>`Geometry` | `Geometry` | `Uint32Array` | 2 | A Geometry handle field. |
| <a id="column-Material"></a>`Material` | `Material` | `Uint32Array` | 2 | A Material handle field. |

```ogt
column Geometry {
  doc "A Geometry handle field."
  array Uint32Array
  stride 2
  value Geometry
}
```

<sub>`api.ogt` line 1071</sub>

```ogt
column Material {
  doc "A Material handle field."
  array Uint32Array
  stride 2
  value Material
}
```

<sub>`api.ogt` line 1078</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_LIGHTS"></a>`MAX_LIGHTS` | `number` | `8` | The most point and spot lights one frame draws. |

```ogt
const MAX_LIGHTS u32 8 "The most point and spot lights one frame draws."
```

<sub>`api.ogt` line 1447</sub>
