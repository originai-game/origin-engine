<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `render` module

What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.

- **TypeScript**: functions are the namespace `og.render`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module render {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1582</sub>

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
| [`ToneMapping`](#type-ToneMapping) | enum | The curve from the lit HDR image to the display, as three.js toneMapping. RenderSettings.toneMapped false clamps instead, whichever is chosen. |
| [`ToneMapping()`](#column-ToneMapping) | column kind | A ToneMapping field: the variant's index. |
| [`og.RenderSettings`](#record-RenderSettings) | resource | The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure. |
| [`Lut`](#type-Lut) | handle | A colour lookup table for PostEffects.lut, from og.load.lut (a `.cube` file). |
| [`EnvironmentMap`](#type-EnvironmentMap) | handle | A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`). |
| [`ParticleEmitter`](#type-ParticleEmitter) | handle | A particle emitter asset for ParticleSystem.emitter, from og.load.emitter (an `.ogt` of `type particles`). |
| [`Texture()`](#column-Texture) | column kind | A Texture handle field (0 is none). |
| [`Lut()`](#column-Lut) | column kind | A Lut handle field (0 is none). |
| [`EnvironmentMap()`](#column-EnvironmentMap) | column kind | An EnvironmentMap handle field (0 is none). |
| [`og.Environment`](#record-Environment) | resource | Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. Load the map with og.load.environment: the cook has already prefiltered it (GGX levels for reflections from mirror to rough, harmonics for the diffuse light), so setting it costs a few copy passes on any tier, WebGL2 included. Without a map the sky lights the scene as before (og.Sky, a DayNight, or the built-in one). Read after each tick; it never feeds back into the simulation. |
| [`ParticleEmitter()`](#column-ParticleEmitter) | column kind | A ParticleEmitter handle field (0 is none). |
| [`AoMethod`](#type-AoMethod) | enum | Which ambient occlusion algorithm runs. |
| [`AoMethod()`](#column-AoMethod) | column kind | An AoMethod field: the variant's index. |
| [`DofQuality`](#type-DofQuality) | enum | How many taps depth of field gathers per pixel. |
| [`DofQuality()`](#column-DofQuality) | column kind | A DofQuality field: the variant's index. |
| [`SpriteBlend`](#type-SpriteBlend) | enum | How a sprite is blended over what is below it. |
| [`SpriteBlend()`](#column-SpriteBlend) | column kind | A SpriteBlend field: the variant's index. |
| [`StripBlend`](#type-StripBlend) | enum | How a trail, ribbon or beam is blended. |
| [`StripBlend()`](#column-StripBlend) | column kind | A StripBlend field: the variant's index. |
| [`og.PostEffects`](#record-PostEffects) | resource | The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. Every effect is off by default (its switch false or its amount 0) and then costs nothing: no pass is recorded. Read after each tick; it never feeds back into the simulation. |
| [`og.Lighting2D`](#record-Lighting2D) | resource | 2D lighting of the sprite layers minLayer to maxLayer: their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default. |
| [`og.Light2D`](#record-Light2D) | component | A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff, shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn. |
| [`og.Occluder2D`](#record-Occluder2D) | component | A rectangle that blocks 2D light (walls, crates, characters), centred on the entity's position and turned with it about z. Pixels inside it are not shadowed by it. At most 128 are drawn. |
| [`og.Decal`](#record-Decal) | component | A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one). At most 32 are drawn. |
| [`og.ParticleSystem`](#record-ParticleSystem) | component | Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay, and never feed back into the simulation: compute on the full tier, the CPU on WebGL2, identical either way. |
| [`og.Trail`](#record-Trail) | component | A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail. |
| [`og.Beam`](#record-Beam) | component | A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter). |
| [`RibbonPoint`](#type-RibbonPoint) | desc struct | One point of a ribbon. |
| [`RibbonDesc`](#type-RibbonDesc) | desc struct | An explicit camera-facing strip through points, drawn this frame only (immediate mode, as og.render.sprite): submit it every tick it should show. |
| [`og.render.ribbon`](#fn-ribbon) | function | Draw a ribbon this frame. Systems and commands may call it; the list is cleared at every tick. |
| [`TextDesc`](#type-TextDesc) | desc struct | A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show). |
| [`TextMeasureDesc`](#type-TextMeasureDesc) | desc struct | Text to measure with og.render.measureText, laid out as og.render.text lays it out. |
| [`TextMetrics`](#type-TextMetrics) | struct | The extent of a laid-out text, in world units. |
| [`SpriteDesc`](#type-SpriteDesc) | desc struct | A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene. |
| [`og.render.sprite`](#fn-sprite) | function | Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show. |
| [`og.Sprite`](#record-Sprite) | component | A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture, region and anchor come from the flipbook's current frame. |
| [`og.Afterimage`](#record-Afterimage) | component | Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by color. A dash, a speed-up, a teleport. The history is world state (the og.sprite state block). |
| [`FloatingTextDesc`](#type-FloatingTextDesc) | desc struct | A damage number or pickup text that pops at a point, rises, spreads and fades by itself (no per-tick submission). Deterministic: its drift comes from the tick and its order, and it is world state (the og.sprite state block). |
| [`og.render.floatingText`](#fn-floatingText) | function | Spawn a floating text (damage numbers, +coins, `blocked`). It lives `lifetime` seconds of scaled time on its own; systems and commands may call it. |
| [`og.render.measureText`](#fn-measureText) | function | The size text takes when og.render.text draws it, for centring, wrapping and layout. |
| [`BoxGeometryDesc`](#type-BoxGeometryDesc) | desc struct | A box centred at the origin, as three.js BoxGeometry(width, height, depth). |
| [`PlaneGeometryDesc`](#type-PlaneGeometryDesc) | desc struct | A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor. |
| [`SphereGeometryDesc`](#type-SphereGeometryDesc) | desc struct | A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments). |
| [`og.render.boxGeometry`](#fn-boxGeometry) | function | Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too. |
| [`og.render.planeGeometry`](#fn-planeGeometry) | function | Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too. |
| [`og.render.sphereGeometry`](#fn-sphereGeometry) | function | Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh). |
| [`MAX_LIGHTS`](#const-MAX_LIGHTS) | constant | The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept. |
| [`MAX_CLUSTER_LIGHTS`](#const-MAX_CLUSTER_LIGHTS) | constant | The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there. |
| [`RenderCapabilities`](#type-RenderCapabilities) | struct | What the renderer can do on this device. Backend limits are reported here, never by missing members. |
| [`og.render.standardMaterial`](#fn-standardMaterial) | function | Create a standard material. |
| [`og.render.updateMaterial`](#fn-updateMaterial) | function | Change some parameters of a standard material. |
| [`Shader`](#type-Shader) | handle | A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader and the parameters they read. |
| [`og.render.shaderMaterial`](#fn-shaderMaterial) | function | Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive; the og-game skill's Material shaders section has their WGSL contract), its parameters at their defaults. The hooks apply to skinned meshes too, and shadows and the depth prepass (ambient occlusion, outlines, depth of field) see the mesh where vertex_offset moved it and cut where surface_color's alpha falls below alphaTest. |
| [`og.render.setMaterialShader`](#fn-setMaterialShader) | function | Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's parameters at their defaults. Every mesh drawn with that material changes. |
| [`og.render.setShaderParam`](#fn-setShaderParam) | function | Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color. |
| [`og.render.capabilities`](#fn-capabilities) | function | The renderer's tier and limits on this device. |
| [`og.render.text`](#fn-text) | function | Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout. |

## Functions

<a id="fn-ribbon"></a>

### `og.render.ribbon`

Draw a ribbon this frame. Systems and commands may call it; the list is cleared at every tick.

```ts
og.render.ribbon(world: World, desc?: RibbonDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `RibbonDesc` | yes | The points and the look. |

**Returns** `Result<void>`: Refused for a dead texture or fewer than two points.

Binding symbol `og_render_ribbon` · publish builds: kept

```ogt
fn ribbon {
  doc "Draw a ribbon this frame. Systems and commands may call it; the list is cleared at every tick."
  param world World "The world whose renderer draws it."
  param desc RibbonDesc "The points and the look."
  returns Result.Void "Refused for a dead texture or fewer than two points."
}
```

<sub>`api.ogt` line 2404</sub>

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

<sub>`api.ogt` line 2520</sub>

<a id="fn-floatingText"></a>

### `og.render.floatingText`

Spawn a floating text (damage numbers, +coins, `blocked`). It lives `lifetime` seconds of scaled time on its own; systems and commands may call it.

```ts
og.render.floatingText(world: World, desc?: FloatingTextDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `FloatingTextDesc` | yes | The text, the point and the style. |

**Returns** `Result<void>`: Refused for a dead font, or no font when the UI has none.

Binding symbol `og_render_floating_text` · publish builds: kept

```ogt
fn floatingText {
  doc "Spawn a floating text (damage numbers, +coins, `blocked`). It lives `lifetime` seconds of scaled time on its own; systems and commands may call it."
  param world World "The world whose renderer draws it."
  param desc FloatingTextDesc "The text, the point and the style."
  returns Result.Void "Refused for a dead font, or no font when the UI has none."
}
```

<sub>`api.ogt` line 2684</sub>

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

<sub>`api.ogt` line 2691</sub>

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

<sub>`api.ogt` line 2751</sub>

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

<sub>`api.ogt` line 2758</sub>

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

<sub>`api.ogt` line 2765</sub>

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

<sub>`api.ogt` line 2786</sub>

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

<sub>`api.ogt` line 2793</sub>

<a id="fn-shaderMaterial"></a>

### `og.render.shaderMaterial`

Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive; the og-game skill's Material shaders section has their WGSL contract), its parameters at their defaults. The hooks apply to skinned meshes too, and shadows and the depth prepass (ambient occlusion, outlines, depth of field) see the mesh where vertex_offset moved it and cut where surface_color's alpha falls below alphaTest.

```ts
og.render.shaderMaterial(world: World, shader: Shader, desc?: StandardMaterialDesc): Result<Material>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `shader` | `Shader` | no | The shader. |
| `desc` | `StandardMaterialDesc` | yes | The base material; omitted parameters take three.js defaults. A dissolve sets alphaTest so that alpha below it is cut. |

**Returns** `Result<Material>`: Refused for a shader not loaded in this world.

Binding symbol `og_render_shader_material` · publish builds: kept

```ogt
fn shaderMaterial {
  doc "Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive; the og-game skill's Material shaders section has their WGSL contract), its parameters at their defaults. The hooks apply to skinned meshes too, and shadows and the depth prepass (ambient occlusion, outlines, depth of field) see the mesh where vertex_offset moved it and cut where surface_color's alpha falls below alphaTest."
  param world World "The world whose renderer owns the material."
  param shader Shader "The shader."
  param desc StandardMaterialDesc "The base material; omitted parameters take three.js defaults. A dissolve sets alphaTest so that alpha below it is cut."
  returns Result.Material "Refused for a shader not loaded in this world."
}
```

<sub>`api.ogt` line 2803</sub>

<a id="fn-setMaterialShader"></a>

### `og.render.setMaterialShader`

Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's parameters at their defaults. Every mesh drawn with that material changes.

```ts
og.render.setMaterialShader(world: World, material: Material, shader: Shader | null): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `material` | `Material` | no | The material, from og.render.standardMaterial, og.render.shaderMaterial or a loaded glTF. |
| `shader` | `Shader \| null` | no | The shader, or null to draw the material plain again. |

**Returns** `Result<void>`: Refused for a dead material or a shader not loaded in this world.

Binding symbol `og_render_set_material_shader` · publish builds: kept

```ogt
fn setMaterialShader {
  doc "Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's parameters at their defaults. Every mesh drawn with that material changes."
  param world World "The world whose renderer owns the material."
  param material Material "The material, from og.render.standardMaterial, og.render.shaderMaterial or a loaded glTF."
  param shader Option.Shader "The shader, or null to draw the material plain again."
  returns Result.Void "Refused for a dead material or a shader not loaded in this world."
}
```

<sub>`api.ogt` line 2811</sub>

<a id="fn-setShaderParam"></a>

### `og.render.setShaderParam`

Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color.

```ts
og.render.setShaderParam(world: World, material: Material, name: string, value: readonly number[]): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the material. |
| `material` | `Material` | no | A material made by og.render.shaderMaterial or given a shader by og.render.setMaterialShader. |
| `name` | `string` | no | The parameter's name. |
| `value` | `readonly number[]` | no | Its value; missing components are 0. |

**Returns** `Result<void>`: Refused for a material without a shader or a parameter its shader does not declare.

Binding symbol `og_render_set_shader_param` · publish builds: kept

```ogt
fn setShaderParam {
  doc "Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color."
  param world World "The world whose renderer owns the material."
  param material Material "A material made by og.render.shaderMaterial or given a shader by og.render.setMaterialShader."
  param name String "The parameter's name."
  param value List.f32 "Its value; missing components are 0."
  returns Result.Void "Refused for a material without a shader or a parameter its shader does not declare."
}
```

<sub>`api.ogt` line 2819</sub>

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

<sub>`api.ogt` line 2828</sub>

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

<sub>`api.ogt` line 2834</sub>

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
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A hit flash: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). Drawn through a copy of the material per flash level (16 levels). |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |

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
  field flash f32 "A hit flash: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). Drawn through a copy of the material per flash level (16 levels)." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear colour." {
    default 1 1 1
  }
}
```

<sub>`api.ogt` line 1746</sub>

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

<sub>`api.ogt` line 1766</sub>

<a id="record-PointLight"></a>

### `og.PointLight` (component)

A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.

Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it, so hundreds are cheap. Lights out of view are skipped; up to MAX_LIGHTS in view are drawn, and past that (or past what one screen cluster holds, MAX_CLUSTER_LIGHTS) the ones brightest at the camera are kept. The first two in view with castShadow get cube shadows.

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
  doc "Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it, so hundreds are cheap. Lights out of view are skipped; up to MAX_LIGHTS in view are drawn, and past that (or past what one screen cluster holds, MAX_CLUSTER_LIGHTS) the ones brightest at the camera are kept. The first two in view with castShadow get cube shadows."
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

<sub>`api.ogt` line 1787</sub>

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

<sub>`api.ogt` line 1807</sub>

<a id="record-RenderSettings"></a>

### `og.RenderSettings` (resource)

The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.

Token `og.RenderSettings`, registered under the schema name `og.RenderSettings`; its value type is `RenderSettings`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `environmentIntensity` | `number` | `0.55` | ≥ 0 | — | tunable | The strength of the image-based ambient light from the sky; og.Environment.intensity takes its place while an environment map is set. |
| `sky` | `boolean` | `false` | — | — | — | Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map). |
| `bloom` | `boolean` | `false` | — | — | — | Light above the threshold bleeds into its surroundings. |
| `bloomThreshold` | `number` | `1` | ≥ 0 | — | tunable | Brightness after exposure where bloom starts; 1 is display white. |
| `bloomKnee` | `number` | `0.5` | ≥ 0 | — | — | Softens the threshold over threshold plus or minus knee; 0 is a hard cut. |
| `bloomIntensity` | `number` | `0.6` | ≥ 0 | — | tunable | How much of the blurred light is added back. |
| `autoExposure` | `boolean` | `false` | — | — | — | Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the current frame's alone; otherwise it adapts over time like an eye. |
| `adaptSpeedUp` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure rises when the view gets darker, in EV per second of game time (the ticks, never the wall clock, so a replay adapts the same); 0 is instant. Snapshots and probe sweeps carry the adapted exposure. |
| `adaptSpeedDown` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. Eyes adapt to brightness faster than to the dark: 3 down and 1 up is a natural pair. |
| `toneMapped` | `boolean` | `true` | — | — | — | Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. |
| `toneMapping` | `ToneMapping` | `"aces"` | — | — | — | Which curve tone maps the frame while toneMapped is true: aces (the default), agx or none. |
| `decalAtlas` | `Texture` | `0` | — | — | — | The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour. |

```ogt
resource RenderSettings {
  doc "The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure."
  field environmentIntensity f32 "The strength of the image-based ambient light from the sky; og.Environment.intensity takes its place while an environment map is set." {
    default 0.55
    min 0
    tunable true
  }
  field sky bool "Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map)."
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
  field autoExposure bool "Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the current frame's alone; otherwise it adapts over time like an eye."
  field adaptSpeedUp f32 "How fast auto exposure rises when the view gets darker, in EV per second of game time (the ticks, never the wall clock, so a replay adapts the same); 0 is instant. Snapshots and probe sweeps carry the adapted exposure." {
    min 0
    max 100
    unit evPerSecond
    tunable true
  }
  field adaptSpeedDown f32 "How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. Eyes adapt to brightness faster than to the dark: 3 down and 1 up is a natural pair." {
    min 0
    max 100
    unit evPerSecond
    tunable true
  }
  field toneMapped bool "Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1." {
    default true
  }
  field toneMapping ToneMapping "Which curve tone maps the frame while toneMapped is true: aces (the default), agx or none."
  field decalAtlas Texture "The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour."
}
```

<sub>`api.ogt` line 1852</sub>

<a id="record-Environment"></a>

### `og.Environment` (resource)

Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. Load the map with og.load.environment: the cook has already prefiltered it (GGX levels for reflections from mirror to rough, harmonics for the diffuse light), so setting it costs a few copy passes on any tier, WebGL2 included. Without a map the sky lights the scene as before (og.Sky, a DayNight, or the built-in one). Read after each tick; it never feeds back into the simulation.

Token `og.Environment`, registered under the schema name `og.Environment`; its value type is `Environment`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `EnvironmentMap` | `0` | — | — | — | The map from og.load.environment; 0 lights by the sky. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | The strength of the map's light on every surface, as three.js scene.environmentIntensity; while a map is set it replaces RenderSettings.environmentIntensity. |
| `rotation` | `number` | `0` | — | `deg` | tunable | Turns the map about +Y, as three.js scene.environmentRotation.y: positive turns what was at +X towards -Z. The light and the background turn together. |
| `background` | `boolean` | `false` | — | — | — | Draw the map behind the scene, as three.js scene.background = the environment (Camera.background, og.Sky and RenderSettings.sky give way); false leaves the background as they set it. |
| `backgroundIntensity` | `number` | `1` | ≥ 0 | — | tunable | Multiplies the map where it is drawn behind the scene, as three.js scene.backgroundIntensity; the light is not changed. |

```ogt
resource Environment {
  doc "Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. Load the map with og.load.environment: the cook has already prefiltered it (GGX levels for reflections from mirror to rough, harmonics for the diffuse light), so setting it costs a few copy passes on any tier, WebGL2 included. Without a map the sky lights the scene as before (og.Sky, a DayNight, or the built-in one). Read after each tick; it never feeds back into the simulation."
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
}
```

<sub>`api.ogt` line 1922</sub>

<a id="record-PostEffects"></a>

### `og.PostEffects` (resource)

The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. Every effect is off by default (its switch false or its amount 0) and then costs nothing: no pass is recorded. Read after each tick; it never feeds back into the simulation.

Screen-space ambient occlusion, outlines, depth of field, motion blur and god rays run a depth-normal prepass, turned on with them. Grading, vignette, aberration, grain and the flash apply to the display colour after tone mapping; the UI and screen-space sprites (the overlay) come after all of it.

Token `og.PostEffects`, registered under the schema name `og.PostEffects`; its value type is `PostEffects`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ao` | `boolean` | `false` | — | — | — | Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. |
| `aoRadius` | `number` | `0.6` | ≥ 0 | `m` | tunable | The world-space radius searched for occluders. |
| `aoIntensity` | `number` | `1` | ≥ 0 | — | tunable | 0 none, 1 full; above 1 exaggerates. |
| `aoMethod` | `AoMethod` | `"auto"` | — | — | — | The algorithm. |
| `outline` | `boolean` | `false` | — | — | — | Outlines every opaque surface from depth and normal edges (materials with outlineWidth get theirs anyway). |
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The line colour, linear. |
| `outlineWidth` | `number` | `2` | 0 … 31 | — | tunable | Line width in pixels, drawn inside the silhouette. |
| `outlineDepthThreshold` | `number` | `0.04` | ≥ 0 | — | — | The relative depth step that counts as a silhouette (0.04 is 4 %). |
| `outlineNormalThreshold` | `number` | `0.4` | ≥ 0 | — | — | The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees). |
| `depthOfField` | `boolean` | `false` | — | — | — | Bokeh depth of field: pixels away from the focus distance blur into discs. |
| `dofFocusDistance` | `number` | `10` | ≥ 0 | `m` | tunable | The distance from the camera that is sharp. |
| `dofFocusRange` | `number` | `2` | ≥ 0 | `m` | tunable | The half-width of the sharp band around it. |
| `dofMaxBlur` | `number` | `0.012` | 0 … 0.1 | — | — | The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p). |
| `dofFalloff` | `number` | `4` | ≥ 0 | — | — | How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away. |
| `dofQuality` | `DofQuality` | `"low"` | — | — | — | Taps per pixel. |
| `motionBlur` | `boolean` | `false` | — | — | — | Per-object and camera motion blur along the screen-space velocity. |
| `motionBlurShutter` | `number` | `0.5` | 0 … 2 | — | tunable | The shutter as a fraction of the frame (0.5 is a 180 degree shutter). |
| `motionBlurSamples` | `number` | `12` | 4 … 32 | — | — | Taps along the velocity. |
| `motionBlurMaxPixels` | `number` | `40` | ≥ 1 | — | — | The longest blur in pixels. |
| `godRays` | `boolean` | `false` | — | — | — | Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera). |
| `godRaysIntensity` | `number` | `0.5` | ≥ 0 | — | tunable | How bright the shafts are. |
| `godRaysDecay` | `number` | `0.965` | 0.5 … 1 | — | — | How fast the rays fade along their length. |
| `godRaysDensity` | `number` | `1` | ≥ 0 | — | — | How far towards the sun each ray samples (a fraction of the distance). |
| `godRaysTint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear tint on the light the rays carry. |
| `godRaysSamples` | `number` | `48` | 8 … 128 | — | — | Taps per ray. |
| `godRaysThreshold` | `number` | `2` | ≥ 0 | — | — | Sky brightness (linear, before exposure) below which the sky carries no rays. |
| `glow` | `boolean` | `false` | — | — | — | Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom. |
| `glowIntensity` | `number` | `1` | ≥ 0 | — | tunable | How much of the blurred glow is added back. |
| `lut` | `Lut` | `0` | — | — | — | Colour grading by a lookup table from og.load.lut; 0 is none. |
| `lutIntensity` | `number` | `1` | 0 … 1 | — | tunable | 0 leaves the image as tone mapped, 1 applies the table fully. |
| `vignette` | `boolean` | `false` | — | — | — | Darkened (or tinted) edges of the frame. |
| `vignetteIntensity` | `number` | `0.35` | 0 … 1 | — | tunable | 0 none, 1 black corners. |
| `vignetteRadius` | `number` | `0.55` | 0 … 1 | — | — | Where darkening starts, as a fraction of the half-diagonal. |
| `vignetteSmoothness` | `number` | `0.45` | 0 … 1 | — | — | The width of the fade. |
| `vignetteColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The linear colour the edges move towards. |
| `chromaticAberration` | `number` | `0` | 0 … 20 | — | tunable | Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off. |
| `filmGrain` | `number` | `0` | 0 … 1 | — | tunable | Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off. |
| `filmGrainSize` | `number` | `1.5` | ≥ 0.5 | — | — | The grain size in pixels. |
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A full-screen flash: the display colour mixed towards flashColor by this much (animate it per tick); 0 is off. |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear display colour. |
| `shakeTrauma` | `number` | `0` | 0 … 1 | — | tunable | Camera shake held at a fixed level (decay it yourself): 0 none, 1 the most (the amplitude is trauma squared). Prefer og.camera.shake, which decays by itself; the frame shakes by the larger of the two. Smooth value noise by the tick, so a replay shakes the same. Visual only. |
| `shakeMaxOffset` | `number` | `0.05` | ≥ 0 | — | — | The largest offset at trauma 1, as a fraction of the view height. |
| `shakeMaxRoll` | `number` | `3` | ≥ 0 | `deg` | — | The largest roll at trauma 1. |
| `shakeFrequency` | `number` | `12` | ≥ 0 | `hz` | — | How fast the shake moves. |
| `shakeSeed` | `number` | `0` | — | — | — | The shake's noise seed. |

```ogt
resource PostEffects {
  doc "The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. Every effect is off by default (its switch false or its amount 0) and then costs nothing: no pass is recorded. Read after each tick; it never feeds back into the simulation."
  doc "Screen-space ambient occlusion, outlines, depth of field, motion blur and god rays run a depth-normal prepass, turned on with them. Grading, vignette, aberration, grain and the flash apply to the display colour after tone mapping; the UI and screen-space sprites (the overlay) come after all of it."
  field ao bool "Screen-space ambient occlusion: darkens the ambient light in creases and contact areas."
  field aoRadius f32 "The world-space radius searched for occluders." {
    default 0.6
    min 0
    unit m
    tunable true
  }
  field aoIntensity f32 "0 none, 1 full; above 1 exaggerates." {
    default 1
    min 0
    tunable true
  }
  field aoMethod AoMethod "The algorithm."
  field outline bool "Outlines every opaque surface from depth and normal edges (materials with outlineWidth get theirs anyway)."
  field outlineColor Color "The line colour, linear." {
    default 0.02 0.02 0.03
  }
  field outlineWidth f32 "Line width in pixels, drawn inside the silhouette." {
    default 2
    min 0
    max 31
    tunable true
  }
  field outlineDepthThreshold f32 "The relative depth step that counts as a silhouette (0.04 is 4 %)." {
    default 0.04
    min 0
  }
  field outlineNormalThreshold f32 "The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees)." {
    default 0.4
    min 0
  }
  field depthOfField bool "Bokeh depth of field: pixels away from the focus distance blur into discs."
  field dofFocusDistance f32 "The distance from the camera that is sharp." {
    default 10
    min 0
    unit m
    tunable true
  }
  field dofFocusRange f32 "The half-width of the sharp band around it." {
    default 2
    min 0
    unit m
    tunable true
  }
  field dofMaxBlur f32 "The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p)." {
    default 0.012
    min 0
    max 0.1
  }
  field dofFalloff f32 "How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away." {
    default 4
    min 0
  }
  field dofQuality DofQuality "Taps per pixel."
  field motionBlur bool "Per-object and camera motion blur along the screen-space velocity."
  field motionBlurShutter f32 "The shutter as a fraction of the frame (0.5 is a 180 degree shutter)." {
    default 0.5
    min 0
    max 2
    tunable true
  }
  field motionBlurSamples u32 "Taps along the velocity." {
    default 12
    min 4
    max 32
  }
  field motionBlurMaxPixels f32 "The longest blur in pixels." {
    default 40
    min 1
  }
  field godRays bool "Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera)."
  field godRaysIntensity f32 "How bright the shafts are." {
    default 0.5
    min 0
    tunable true
  }
  field godRaysDecay f32 "How fast the rays fade along their length." {
    default 0.965
    min 0.5
    max 1
  }
  field godRaysDensity f32 "How far towards the sun each ray samples (a fraction of the distance)." {
    default 1
    min 0
  }
  field godRaysTint Color "A linear tint on the light the rays carry." {
    default 1 1 1
  }
  field godRaysSamples u32 "Taps per ray." {
    default 48
    min 8
    max 128
  }
  field godRaysThreshold f32 "Sky brightness (linear, before exposure) below which the sky carries no rays." {
    default 2
    min 0
  }
  field glow bool "Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom."
  field glowIntensity f32 "How much of the blurred glow is added back." {
    default 1
    min 0
    tunable true
  }
  field lut Lut "Colour grading by a lookup table from og.load.lut; 0 is none."
  field lutIntensity f32 "0 leaves the image as tone mapped, 1 applies the table fully." {
    default 1
    min 0
    max 1
    tunable true
  }
  field vignette bool "Darkened (or tinted) edges of the frame."
  field vignetteIntensity f32 "0 none, 1 black corners." {
    default 0.35
    min 0
    max 1
    tunable true
  }
  field vignetteRadius f32 "Where darkening starts, as a fraction of the half-diagonal." {
    default 0.55
    min 0
    max 1
  }
  field vignetteSmoothness f32 "The width of the fade." {
    default 0.45
    min 0
    max 1
  }
  field vignetteColor Color "The linear colour the edges move towards."
  field chromaticAberration f32 "Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off." {
    min 0
    max 20
    tunable true
  }
  field filmGrain f32 "Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off." {
    min 0
    max 1
    tunable true
  }
  field filmGrainSize f32 "The grain size in pixels." {
    default 1.5
    min 0.5
  }
  field flash f32 "A full-screen flash: the display colour mixed towards flashColor by this much (animate it per tick); 0 is off." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear display colour." {
    default 1 1 1
  }
  field shakeTrauma f32 "Camera shake held at a fixed level (decay it yourself): 0 none, 1 the most (the amplitude is trauma squared). Prefer og.camera.shake, which decays by itself; the frame shakes by the larger of the two. Smooth value noise by the tick, so a replay shakes the same. Visual only." {
    min 0
    max 1
    tunable true
  }
  field shakeMaxOffset f32 "The largest offset at trauma 1, as a fraction of the view height." {
    default 0.05
    min 0
  }
  field shakeMaxRoll f32 "The largest roll at trauma 1." {
    default 3
    min 0
    unit deg
  }
  field shakeFrequency f32 "How fast the shake moves." {
    default 12
    min 0
    unit hz
  }
  field shakeSeed u32 "The shake's noise seed."
}
```

<sub>`api.ogt` line 2002</sub>

<a id="record-Lighting2D"></a>

### `og.Lighting2D` (resource)

2D lighting of the sprite layers minLayer to maxLayer: their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default.

Token `og.Lighting2D`, registered under the schema name `og.Lighting2D`; its value type is `Lighting2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Light the layers. |
| `ambient` | `Color` | `{ r: 0.15, g: 0.15, b: 0.15 }` | — | — | — | Light everywhere (0 is darkness outside the lights). |
| `minLayer` | `number` | `-1000000` | — | — | — | The lowest lit sprite layer. |
| `maxLayer` | `number` | `99` | — | — | — | The highest lit sprite layer (above it: UI and HUD layers stay unlit). |
| `resolution` | `number` | `0.5` | 0.25 … 1 | — | — | The light map's resolution as a fraction of the target. |

```ogt
resource Lighting2D {
  doc "2D lighting of the sprite layers minLayer to maxLayer: their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default."
  field enabled bool "Light the layers."
  field ambient Color "Light everywhere (0 is darkness outside the lights)." {
    default 0.15 0.15 0.15
  }
  field minLayer i32 "The lowest lit sprite layer." {
    default -1000000
  }
  field maxLayer i32 "The highest lit sprite layer (above it: UI and HUD layers stay unlit)." {
    default 99
  }
  field resolution f32 "The light map's resolution as a fraction of the target." {
    default 0.5
    min 0.25
    max 1
  }
}
```

<sub>`api.ogt` line 2177</sub>

<a id="record-Light2D"></a>

### `og.Light2D` (component)

A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff, shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn.

Token `og.Light2D`, registered under the schema name `og.Light2D`; its value type is `Light2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | The brightness at the centre. |
| `radius` | `number` | `5` | ≥ 0 | `m` | tunable | Where the light reaches zero, in world units. |
| `falloff` | `number` | `2` | ≥ 0.1 | — | — | The exponent of the fade. |
| `sourceRadius` | `number` | `0.25` | ≥ 0 | `m` | — | The light's own size: larger gives softer shadows. |
| `castShadows` | `boolean` | `true` | — | — | — | Occluders between the light and a pixel shade it. |

```ogt
component Light2D {
  doc "A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff, shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn."
  field color Color "The light's linear colour." {
    default 1 1 1
  }
  field intensity f32 "The brightness at the centre." {
    default 1
    min 0
    tunable true
  }
  field radius f32 "Where the light reaches zero, in world units." {
    default 5
    min 0
    unit m
    tunable true
  }
  field falloff f32 "The exponent of the fade." {
    default 2
    min 0.1
  }
  field sourceRadius f32 "The light's own size: larger gives softer shadows." {
    default 0.25
    min 0
    unit m
  }
  field castShadows bool "Occluders between the light and a pixel shade it." {
    default true
  }
}
```

<sub>`api.ogt` line 2196</sub>

<a id="record-Occluder2D"></a>

### `og.Occluder2D` (component)

A rectangle that blocks 2D light (walls, crates, characters), centred on the entity's position and turned with it about z. Pixels inside it are not shadowed by it. At most 128 are drawn.

Token `og.Occluder2D`, registered under the schema name `og.Occluder2D`; its value type is `Occluder2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its width in world units. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its height in world units. |

```ogt
component Occluder2D {
  doc "A rectangle that blocks 2D light (walls, crates, characters), centred on the entity's position and turned with it about z. Pixels inside it are not shadowed by it. At most 128 are drawn."
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
}
```

<sub>`api.ogt` line 2226</sub>

<a id="record-Decal"></a>

### `og.Decal` (component)

A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one). At most 32 are drawn.

Token `og.Decal`, registered under the schema name `og.Decal`; its value type is `Decal`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the atlas. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The decal's opacity. |
| `uvMin` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The atlas region's top-left corner, v down. |
| `uvMax` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | The atlas region's bottom-right corner, v down. |
| `emissive` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | Linear light added where the decal covers (glowing runes, lava). |
| `roughness` | `number` | `-1` | -1 … 1 | — | — | The surface roughness under the decal; -1 keeps the surface's. |
| `angleFade` | `number` | `75` | 0 … 90 | `deg` | — | Surfaces turned further than this from the projection direction are not covered (the last 15 degrees fade). |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Decal {
  doc "A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one). At most 32 are drawn."
  field color Color "A linear colour multiplied with the atlas." {
    default 1 1 1
  }
  field opacity f32 "The decal's opacity." {
    default 1
    min 0
    max 1
    tunable true
  }
  field uvMin Vector2 "The atlas region's top-left corner, v down."
  field uvMax Vector2 "The atlas region's bottom-right corner, v down." {
    default 1 1
  }
  field emissive Color "Linear light added where the decal covers (glowing runes, lava)."
  field roughness f32 "The surface roughness under the decal; -1 keeps the surface's." {
    default -1
    min -1
    max 1
  }
  field angleFade f32 "Surfaces turned further than this from the projection direction are not covered (the last 15 degrees fade)." {
    default 75
    min 0
    max 90
    unit deg
  }
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>`api.ogt` line 2240</sub>

<a id="record-ParticleSystem"></a>

### `og.ParticleSystem` (component)

Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay, and never feed back into the simulation: compute on the full tier, the CPU on WebGL2, identical either way.

Token `og.ParticleSystem`, registered under the schema name `og.ParticleSystem`; its value type is `ParticleSystem`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `emitter` | `ParticleEmitter` | `0` | — | — | — | The emitter asset; 0 draws nothing. |
| `seed` | `number` | `0` | — | — | — | The random seed: systems with different seeds differ. |
| `startTick` | `number` | `0` | — | — | — | The tick emission started: write og.world tick when spawning a one-shot burst. |
| `stopTick` | `number` | `0` | — | — | — | The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn (a hidden system loses its particles). |

```ogt
component ParticleSystem {
  doc "Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay, and never feed back into the simulation: compute on the full tier, the CPU on WebGL2, identical either way."
  field emitter ParticleEmitter "The emitter asset; 0 draws nothing."
  field seed u32 "The random seed: systems with different seeds differ."
  field startTick u32 "The tick emission started: write og.world tick when spawning a one-shot burst."
  field stopTick u32 "The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops."
  field visible bool "Whether it is drawn (a hidden system loses its particles)." {
    default true
  }
}
```

<sub>`api.ogt` line 2272</sub>

<a id="record-Trail"></a>

### `og.Trail` (component)

A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail.

Token `og.Trail`, registered under the schema name `og.Trail`; its value type is `Trail`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lifetime` | `number` | `0.4` | ≥ 0 | `s` | tunable | How long a position stays in the trail. |
| `minDistance` | `number` | `0.05` | ≥ 0 | `m` | — | A new point is kept when the head has moved this far. |
| `widthStart` | `number` | `0.15` | ≥ 0 | `m` | tunable | The width at the head. |
| `widthEnd` | `number` | `0` | ≥ 0 | `m` | — | The width at the tail. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour at the head; above 1 blooms. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha at the head. |
| `colorEnd` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour at the tail. |
| `opacityEnd` | `number` | `0` | 0 … 1 | — | — | The alpha at the tail. |
| `texture` | `Texture` | `0` | — | — | — | A texture along the strip; 0 is solid. |
| `uvTile` | `number` | `0` | ≥ 0 | `m` | — | The texture repeats every this many world units; 0 stretches it once over the length. |
| `blend` | `StripBlend` | `"additive"` | — | — | — | Additive or alpha blended. |
| `layered` | `boolean` | `false` | — | — | — | Draw it in the 2D sprite order at `layer` (a sprite's trail over a tile background) instead of with the 3D scene. |
| `layer` | `number` | `0` | — | — | — | With layered: the sprite layer it is drawn in, before that layer's sprites. |

```ogt
component Trail {
  doc "A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail."
  field lifetime f32 "How long a position stays in the trail." {
    default 0.4
    min 0
    unit s
    tunable true
  }
  field minDistance f32 "A new point is kept when the head has moved this far." {
    default 0.05
    min 0
    unit m
  }
  field widthStart f32 "The width at the head." {
    default 0.15
    min 0
    unit m
    tunable true
  }
  field widthEnd f32 "The width at the tail." {
    min 0
    unit m
  }
  field color Color "The linear colour at the head; above 1 blooms." {
    default 1 1 1
  }
  field opacity f32 "The alpha at the head." {
    default 1
    min 0
    max 1
  }
  field colorEnd Color "The linear colour at the tail." {
    default 1 1 1
  }
  field opacityEnd f32 "The alpha at the tail." {
    min 0
    max 1
  }
  field texture Texture "A texture along the strip; 0 is solid."
  field uvTile f32 "The texture repeats every this many world units; 0 stretches it once over the length." {
    min 0
    unit m
  }
  field blend StripBlend "Additive or alpha blended."
  field layered bool "Draw it in the 2D sprite order at `layer` (a sprite's trail over a tile background) instead of with the 3D scene."
  field layer i32 "With layered: the sprite layer it is drawn in, before that layer's sprites."
}
```

<sub>`api.ogt` line 2283</sub>

<a id="record-Beam"></a>

### `og.Beam` (component)

A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter).

Token `og.Beam`, registered under the schema name `og.Beam`; its value type is `Beam`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `end` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The far end, in world space. |
| `width` | `number` | `0.1` | ≥ 0 | `m` | tunable | The beam's width. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour; above 1 blooms. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `texture` | `Texture` | `0` | — | — | — | A texture along the beam; 0 is solid. |
| `tile` | `number` | `1` | ≥ 0 | — | — | Texture repeats per world unit along the beam. |
| `scroll` | `number` | `0` | — | — | tunable | Texture scroll in repeats per second. |
| `segments` | `number` | `1` | 1 … 64 | — | — | Segments: 1 is straight, more let jitter bend it. |
| `jitter` | `number` | `0` | ≥ 0 | `m` | — | Sideways jitter in world units, new every tick (lightning). |
| `seed` | `number` | `0` | — | — | — | The jitter's seed. |
| `blend` | `StripBlend` | `"additive"` | — | — | — | Additive or alpha blended. |

```ogt
component Beam {
  doc "A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter)."
  field end Vector3 "The far end, in world space." {
    unit m
  }
  field width f32 "The beam's width." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field color Color "The linear colour; above 1 blooms." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field texture Texture "A texture along the beam; 0 is solid."
  field tile f32 "Texture repeats per world unit along the beam." {
    default 1
    min 0
  }
  field scroll f32 "Texture scroll in repeats per second." {
    tunable true
  }
  field segments u32 "Segments: 1 is straight, more let jitter bend it." {
    default 1
    min 1
    max 64
  }
  field jitter f32 "Sideways jitter in world units, new every tick (lightning)." {
    min 0
    unit m
  }
  field seed u32 "The jitter's seed."
  field blend StripBlend "Additive or alpha blended."
}
```

<sub>`api.ogt` line 2331</sub>

<a id="record-Sprite"></a>

### `og.Sprite` (component)

A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture, region and anchor come from the flipbook's current frame.

Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer.

Token `og.Sprite`, registered under the schema name `og.Sprite`; its value type is `Sprite`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The texture (og.load.texture); 0 draws a solid quad of the colour. |
| `width` | `number` | `1` | ≥ 0 | `m` | tunable | Its width in world units (times Transform.scale.x). |
| `height` | `number` | `1` | ≥ 0 | `m` | tunable | Its height in world units (times Transform.scale.y). |
| `anchor` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | — | — | The point of the quad at the entity's position, as a fraction from the bottom-left corner: (0.5, 0) stands it on its feet. A flipbook frame's pivot replaces it. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear colour multiplied with the texture. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The alpha. |
| `uvMin` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The texture region's top-left corner, v down. |
| `uvMax` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | The texture region's bottom-right corner, v down. |
| `layer` | `number` | `0` | — | — | — | Draw order among sprites, tiles and text; higher draws on top. |
| `sortY` | `boolean` | `false` | — | — | — | Within its layer, lower y draws in front: top-down characters, props and trees overlap by their feet. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror left to right (a character facing left). |
| `flipY` | `boolean` | `false` | — | — | — | Mirror top to bottom. |
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A hit flash: the colour mixed towards flashColor by this much (og.tween.flash fades it out). |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |
| `glow` | `number` | `0` | ≥ 0 | — | — | Strength in the 2D glow buffer (PostEffects.glow); 0 none. |
| `blend` | `SpriteBlend` | `"alpha"` | — | — | — | Alpha or additive. |
| `castShadow` | `boolean` | `false` | — | — | — | Block 2D light (Lighting2D) with the sprite's rectangle. |
| `sliceX` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Nine-slice: the left and right insets in texture pixels of the region (0 0 with sliceY 0 0 draws it whole). World panels, health bars, speech bubbles. |
| `sliceY` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Nine-slice: the top and bottom insets in texture pixels. |
| `sliceScale` | `number` | `0.0625` | ≥ 0 | — | — | Nine-slice: world units per texture pixel of the insets. |
| `shadow` | `number` | `0` | 0 … 1 | — | tunable | A soft blob shadow on the ground under the sprite's anchor, this dark (0 none, 0.4 typical), drawn under every sprite of its layer. |
| `shadowSize` | `Vector2` | `{ x: 0, y: 0 }` | ≥ 0 | `m` | — | The blob's width and height; 0 0 is 0.8 by 0.3 of the sprite's width. |
| `shadowOffset` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The blob's centre relative to the anchor. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Sprite {
  doc "A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture, region and anchor come from the flipbook's current frame."
  doc "Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer."
  field texture Texture "The texture (og.load.texture); 0 draws a solid quad of the colour."
  field width f32 "Its width in world units (times Transform.scale.x)." {
    default 1
    min 0
    unit m
    tunable true
  }
  field height f32 "Its height in world units (times Transform.scale.y)." {
    default 1
    min 0
    unit m
    tunable true
  }
  field anchor Vector2 "The point of the quad at the entity's position, as a fraction from the bottom-left corner: (0.5, 0) stands it on its feet. A flipbook frame's pivot replaces it." {
    default 0.5 0.5
  }
  field color Color "A linear colour multiplied with the texture." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
    tunable true
  }
  field uvMin Vector2 "The texture region's top-left corner, v down."
  field uvMax Vector2 "The texture region's bottom-right corner, v down." {
    default 1 1
  }
  field layer i32 "Draw order among sprites, tiles and text; higher draws on top."
  field sortY bool "Within its layer, lower y draws in front: top-down characters, props and trees overlap by their feet."
  field flipX bool "Mirror left to right (a character facing left)."
  field flipY bool "Mirror top to bottom."
  field flash f32 "A hit flash: the colour mixed towards flashColor by this much (og.tween.flash fades it out)." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear colour." {
    default 1 1 1
  }
  field glow f32 "Strength in the 2D glow buffer (PostEffects.glow); 0 none." {
    min 0
  }
  field blend SpriteBlend "Alpha or additive."
  field castShadow bool "Block 2D light (Lighting2D) with the sprite's rectangle."
  field sliceX Vector2 "Nine-slice: the left and right insets in texture pixels of the region (0 0 with sliceY 0 0 draws it whole). World panels, health bars, speech bubbles."
  field sliceY Vector2 "Nine-slice: the top and bottom insets in texture pixels."
  field sliceScale f32 "Nine-slice: world units per texture pixel of the insets." {
    default 0.0625
    min 0
  }
  field shadow f32 "A soft blob shadow on the ground under the sprite's anchor, this dark (0 none, 0.4 typical), drawn under every sprite of its layer." {
    min 0
    max 1
    tunable true
  }
  field shadowSize Vector2 "The blob's width and height; 0 0 is 0.8 by 0.3 of the sprite's width." {
    min 0
    unit m
  }
  field shadowOffset Vector2 "The blob's centre relative to the anchor." {
    unit m
  }
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>`api.ogt` line 2527</sub>

<a id="record-Afterimage"></a>

### `og.Afterimage` (component)

Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by color. A dash, a speed-up, a teleport. The history is world state (the og.sprite state block).

Token `og.Afterimage`, registered under the schema name `og.Afterimage`; its value type is `Afterimage`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `4` | 0 … 16 | — | — | How many ghosts. |
| `interval` | `number` | `3` | ≥ 1 | `ticks` | — | Ticks between ghosts. |
| `opacity` | `number` | `0.5` | 0 … 1 | — | tunable | The newest ghost's alpha. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear tint (above 1 with additive, a glow). |
| `additive` | `boolean` | `false` | — | — | — | Add the ghosts instead of blending them. |
| `active` | `boolean` | `true` | — | — | — | Record new ghosts; false lets the existing ones fade out. |

```ogt
component Afterimage {
  doc "Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by color. A dash, a speed-up, a teleport. The history is world state (the og.sprite state block)."
  field count u32 "How many ghosts." {
    default 4
    min 0
    max 16
  }
  field interval u32 "Ticks between ghosts." {
    default 3
    min 1
    unit ticks
  }
  field opacity f32 "The newest ghost's alpha." {
    default 0.5
    min 0
    max 1
    tunable true
  }
  field color Color "A linear tint (above 1 with additive, a glow)." {
    default 1 1 1
  }
  field additive bool "Add the ghosts instead of blending them."
  field active bool "Record new ghosts; false lets the existing ones fade out." {
    default true
  }
}
```

<sub>`api.ogt` line 2599</sub>

## Types

<a id="type-Geometry"></a>

### `Geometry` (handle)

Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Geometry "Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF."
```

<sub>`api.ogt` line 1586</sub>

<a id="type-Material"></a>

### `Material` (handle)

A material instance.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Material "A material instance."
```

<sub>`api.ogt` line 1587</sub>

<a id="type-Texture"></a>

### `Texture` (handle)

A cooked texture.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Texture "A cooked texture."
```

<sub>`api.ogt` line 1588</sub>

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

<sub>`api.ogt` line 1604</sub>

<a id="type-RenderTier"></a>

### `RenderTier` (enum)

The backend the renderer runs on; the probe's hello reports it.

A string: one of the variants below. The zero value is `"webgpu"`.

| Variant | Doc |
|---|---|
| `"webgpu"` | WebGPU in a browser: the full feature set. |
| `"webgl2"` | The WebGL2 fallback: no compute, CPU culling and particles, tiled lights. |
| `"vulkan"` | Native Vulkan on a GPU. |
| `"vulkan-cpu"` | Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU. |
| `"metal"` | Native Metal. |
| `"none"` | No renderer: frames are not produced and capture refuses. |

```ogt
enum RenderTier {
  doc "The backend the renderer runs on; the probe's hello reports it."
  variant webgpu "WebGPU in a browser: the full feature set."
  variant webgl2 "The WebGL2 fallback: no compute, CPU culling and particles, tiled lights."
  variant vulkan "Native Vulkan on a GPU."
  variant vulkan-cpu "Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU."
  variant metal "Native Metal."
  variant none "No renderer: frames are not produced and capture refuses."
}
```

<sub>`api.ogt` line 1611</sub>

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

<sub>`api.ogt` line 1621</sub>

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

<sub>`api.ogt` line 1629</sub>

<a id="type-StandardMaterialDesc"></a>

### `StandardMaterialDesc` (desc struct)

A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names.

It is og-render's reflected StandardMaterial field for field: metalnessMap and roughnessMap read a glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; alphaTest above 0 discards below that alpha; transparent blends, sorted back to front.

The stylised fields: unlit (three.js MeshBasicMaterial), toon with its gradientMap ramp, a rim (fresnel) light, a hit flash, dissolve with a glowing edge, and a per-material outline. All are off at their defaults. clearcoat and clearcoatRoughness add MeshPhysicalMaterial's clear coat (no clearcoat maps).

normalMap is read in glTF's tangent frame: red tilts towards +u, green up the image. The tangents come from the model (the cook generates MikkTSpace ones where a glTF has none) or are generated when the geometry is made; normals follow a non-uniform scale correctly.

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
| `normalScale` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How strongly the normal map bends the normal, per tangent axis. |
| `aoMapIntensity` | `number` | `1` | ≥ 0 | — | — | How strongly aoMap darkens the ambient light. |
| `unlit` | `boolean` | `false` | — | — | — | No lighting at all, as three.js MeshBasicMaterial: the colour (times map) plus emissive. |
| `toon` | `boolean` | `false` | — | — | — | Cel shading, as three.js MeshToonMaterial: light is quantized by gradientMap (three built-in bands without one), highlights are hard. |
| `gradientMap` | `Texture \| null` | `null` | — | — | — | The toon ramp: a texture whose first row maps the light's angle (left dark, right lit) to the light received, read without filtering. |
| `rimColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Rim (fresnel) light added at grazing angles, unshadowed (linear). |
| `rimIntensity` | `number` | `0` | ≥ 0 | — | — | The rim light's strength; 0 is none. |
| `rimPower` | `number` | `3` | ≥ 0.1 | — | — | How tight the rim is (higher: thinner). |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The colour a hit flash mixes towards (linear, may exceed 1). |
| `flash` | `number` | `0` | 0 … 1 | — | — | How far the final colour is mixed towards flashColor (0 none, 1 flat): animate it per tick for a hit flash. |
| `dissolve` | `number` | `0` | 0 … 1 | — | — | The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below it are discarded (shadows too); the band just above glows with dissolveEdgeColor. |
| `dissolveMap` | `Texture \| null` | `null` | — | — | — | The dissolve noise from the texture's red channel; without one, value noise over the object-space position. |
| `dissolveEdgeColor` | `Color` | `{ r: 4, g: 1.5, b: 0.375 }` | — | — | — | The glowing edge (linear, usually above 1 so it blooms). |
| `dissolveEdgeWidth` | `number` | `0.0625` | 0 … 1 | — | — | The width of the glowing band in noise units. |
| `dissolveScale` | `number` | `4` | ≥ 0 | — | — | The frequency of the built-in noise (cells per object-space unit). |
| `outlineColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | This material's outline colour (linear). |
| `outlineWidth` | `number` | `0` | 0 … 31 | — | — | This material's outline width in pixels; 0 is none (PostEffects.outline outlines everything). |
| `clearcoat` | `number` | `0` | 0 … 1 | `ratio` | — | A clear lacquer over the surface, as three.js MeshPhysicalMaterial.clearcoat: a second, 4 % reflective specular layer on the surface's normal before normalMap (car paint, varnished wood, wet plastic); 0 is none. |
| `clearcoatRoughness` | `number` | `0` | 0 … 1 | `ratio` | — | How rough the clear coat is, as three.js MeshPhysicalMaterial.clearcoatRoughness. |

```ogt
struct StandardMaterialDesc {
  doc "A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names."
  doc "It is og-render's reflected StandardMaterial field for field: metalnessMap and roughnessMap read a glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; alphaTest above 0 discards below that alpha; transparent blends, sorted back to front."
  doc "The stylised fields: unlit (three.js MeshBasicMaterial), toon with its gradientMap ramp, a rim (fresnel) light, a hit flash, dissolve with a glowing edge, and a per-material outline. All are off at their defaults. clearcoat and clearcoatRoughness add MeshPhysicalMaterial's clear coat (no clearcoat maps)."
  doc "normalMap is read in glTF's tangent frame: red tilts towards +u, green up the image. The tangents come from the model (the cook generates MikkTSpace ones where a glTF has none) or are generated when the geometry is made; normals follow a non-uniform scale correctly."
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
  field normalScale Vector2 "How strongly the normal map bends the normal, per tangent axis." {
    default 1 1
  }
  field aoMapIntensity f32 "How strongly aoMap darkens the ambient light." {
    default 1
    min 0
  }
  field unlit bool "No lighting at all, as three.js MeshBasicMaterial: the colour (times map) plus emissive."
  field toon bool "Cel shading, as three.js MeshToonMaterial: light is quantized by gradientMap (three built-in bands without one), highlights are hard."
  field gradientMap Option.Texture "The toon ramp: a texture whose first row maps the light's angle (left dark, right lit) to the light received, read without filtering."
  field rimColor Color "Rim (fresnel) light added at grazing angles, unshadowed (linear)." {
    default 1 1 1
  }
  field rimIntensity f32 "The rim light's strength; 0 is none." {
    min 0
  }
  field rimPower f32 "How tight the rim is (higher: thinner)." {
    default 3
    min 0.1
  }
  field flashColor Color "The colour a hit flash mixes towards (linear, may exceed 1)." {
    default 1 1 1
  }
  field flash f32 "How far the final colour is mixed towards flashColor (0 none, 1 flat): animate it per tick for a hit flash." {
    min 0
    max 1
  }
  field dissolve f32 "The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below it are discarded (shadows too); the band just above glows with dissolveEdgeColor." {
    min 0
    max 1
  }
  field dissolveMap Option.Texture "The dissolve noise from the texture's red channel; without one, value noise over the object-space position."
  field dissolveEdgeColor Color "The glowing edge (linear, usually above 1 so it blooms)." {
    default 4 1.5 0.375
  }
  field dissolveEdgeWidth f32 "The width of the glowing band in noise units." {
    default 0.0625
    min 0
    max 1
  }
  field dissolveScale f32 "The frequency of the built-in noise (cells per object-space unit)." {
    default 4
    min 0
  }
  field outlineColor Color "This material's outline colour (linear)."
  field outlineWidth f32 "This material's outline width in pixels; 0 is none (PostEffects.outline outlines everything)." {
    min 0
    max 31
  }
  field clearcoat f32 "A clear lacquer over the surface, as three.js MeshPhysicalMaterial.clearcoat: a second, 4 % reflective specular layer on the surface's normal before normalMap (car paint, varnished wood, wet plastic); 0 is none." {
    min 0
    max 1
    unit ratio
  }
  field clearcoatRoughness f32 "How rough the clear coat is, as three.js MeshPhysicalMaterial.clearcoatRoughness." {
    min 0
    max 1
    unit ratio
  }
}
```

<sub>`api.ogt` line 1640</sub>

<a id="type-ToneMapping"></a>

### `ToneMapping` (enum)

The curve from the lit HDR image to the display, as three.js toneMapping. RenderSettings.toneMapped false clamps instead, whichever is chosen.

A string: one of the variants below. The zero value is `"aces"`.

| Variant | Doc |
|---|---|
| `"aces"` | ACES (the fitted filmic curve), as three.js ACESFilmicToneMapping: the default. |
| `"agx"` | AgX (Blender 4's and three.js AgXToneMapping's fit, base look): bright saturated lights go to white instead of skewing hue, and keep their hue further up than under ACES. |
| `"none"` | No curve: the exposed linear colour is clamped to 0..1, so 2D colours come out exactly as authored at exposure 1 (the same as toneMapped false). |

```ogt
enum ToneMapping {
  doc "The curve from the lit HDR image to the display, as three.js toneMapping. RenderSettings.toneMapped false clamps instead, whichever is chosen."
  variant aces "ACES (the fitted filmic curve), as three.js ACESFilmicToneMapping: the default."
  variant agx "AgX (Blender 4's and three.js AgXToneMapping's fit, base look): bright saturated lights go to white instead of skewing hue, and keep their hue further up than under ACES."
  variant none "No curve: the exposed linear colour is clamped to 0..1, so 2D colours come out exactly as authored at exposure 1 (the same as toneMapped false)."
}
```

<sub>`api.ogt` line 1838</sub>

<a id="type-Lut"></a>

### `Lut` (handle)

A colour lookup table for PostEffects.lut, from og.load.lut (a `.cube` file).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Lut "A colour lookup table for PostEffects.lut, from og.load.lut (a `.cube` file)."
```

<sub>`api.ogt` line 1897</sub>

<a id="type-EnvironmentMap"></a>

### `EnvironmentMap` (handle)

A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle EnvironmentMap "A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`)."
```

<sub>`api.ogt` line 1898</sub>

<a id="type-ParticleEmitter"></a>

### `ParticleEmitter` (handle)

A particle emitter asset for ParticleSystem.emitter, from og.load.emitter (an `.ogt` of `type particles`).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle ParticleEmitter "A particle emitter asset for ParticleSystem.emitter, from og.load.emitter (an `.ogt` of `type particles`)."
```

<sub>`api.ogt` line 1899</sub>

<a id="type-AoMethod"></a>

### `AoMethod` (enum)

Which ambient occlusion algorithm runs.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | GTAO on the full tier, SSAO on WebGL2. |
| `"gtao"` | Ground-truth ambient occlusion (3 slices): the better looking one. |
| `"ssao"` | Hemisphere SSAO with 8 samples: the cheaper one. |

```ogt
enum AoMethod {
  doc "Which ambient occlusion algorithm runs."
  variant auto "GTAO on the full tier, SSAO on WebGL2."
  variant gtao "Ground-truth ambient occlusion (3 slices): the better looking one."
  variant ssao "Hemisphere SSAO with 8 samples: the cheaper one."
}
```

<sub>`api.ogt` line 1949</sub>

<a id="type-DofQuality"></a>

### `DofQuality` (enum)

How many taps depth of field gathers per pixel.

A string: one of the variants below. The zero value is `"low"`.

| Variant | Doc |
|---|---|
| `"low"` | 16 taps at half resolution. |
| `"high"` | 48 taps at half resolution: rounder, smoother bokeh. |

```ogt
enum DofQuality {
  doc "How many taps depth of field gathers per pixel."
  variant low "16 taps at half resolution."
  variant high "48 taps at half resolution: rounder, smoother bokeh."
}
```

<sub>`api.ogt` line 1963</sub>

<a id="type-SpriteBlend"></a>

### `SpriteBlend` (enum)

How a sprite is blended over what is below it.

A string: one of the variants below. The zero value is `"alpha"`.

| Variant | Doc |
|---|---|
| `"alpha"` | Ordinary alpha blending. |
| `"additive"` | Added (light, fire, magic): order-independent within a layer. |

```ogt
enum SpriteBlend {
  doc "How a sprite is blended over what is below it."
  variant alpha "Ordinary alpha blending."
  variant additive "Added (light, fire, magic): order-independent within a layer."
}
```

<sub>`api.ogt` line 1976</sub>

<a id="type-StripBlend"></a>

### `StripBlend` (enum)

How a trail, ribbon or beam is blended.

A string: one of the variants below. The zero value is `"additive"`.

| Variant | Doc |
|---|---|
| `"additive"` | Added: glowing streaks, lasers, magic. |
| `"alpha"` | Alpha blended: smoke trails, tyre marks. |

```ogt
enum StripBlend {
  doc "How a trail, ribbon or beam is blended."
  variant additive "Added: glowing streaks, lasers, magic."
  variant alpha "Alpha blended: smoke trails, tyre marks."
}
```

<sub>`api.ogt` line 1989</sub>

<a id="type-RibbonPoint"></a>

### `RibbonPoint` (desc struct)

One point of a ribbon.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `width` | `number` | `0.1` | ≥ 0 | `m` | — | The width there. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour there. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha there. |

```ogt
struct RibbonPoint {
  doc "One point of a ribbon."
  desc true
  field position Vector3 "Where it is." {
    unit m
  }
  field width f32 "The width there." {
    default 0.1
    min 0
    unit m
  }
  field color Color "The linear colour there." {
    default 1 1 1
  }
  field opacity f32 "The alpha there." {
    default 1
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 2371</sub>

<a id="type-RibbonDesc"></a>

### `RibbonDesc` (desc struct)

An explicit camera-facing strip through points, drawn this frame only (immediate mode, as og.render.sprite): submit it every tick it should show.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly RibbonPoint[]` | `[]` | — | — | — | The points, in order (at least two). |
| `texture` | `Texture \| null` | `null` | — | — | — | A texture along the strip; null is solid. |
| `uvTile` | `number` | `0` | ≥ 0 | `m` | — | The texture repeats every this many world units; 0 stretches it once. |
| `blend` | `StripBlend` | `"additive"` | — | — | — | Additive or alpha blended. |

```ogt
struct RibbonDesc {
  doc "An explicit camera-facing strip through points, drawn this frame only (immediate mode, as og.render.sprite): submit it every tick it should show."
  desc true
  field points List.RibbonPoint "The points, in order (at least two)."
  field texture Option.Texture "A texture along the strip; null is solid."
  field uvTile f32 "The texture repeats every this many world units; 0 stretches it once." {
    min 0
    unit m
  }
  field blend StripBlend "Additive or alpha blended."
}
```

<sub>`api.ogt` line 2392</sub>

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
| `outline` | `number` | `0` | 0 … 1 | — | — | World text: an outline around the glyphs, as a fraction (0 to 1) of the widest the font's distance range allows (a font cooked with a larger px_range takes thicker outlines). |
| `outlineColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The outline's linear colour. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame: position is the first baseline's origin in CSS pixels from the top left, y down, size in CSS pixels, rotation clockwise. |
| `clip` | `[number, number, number, number] \| null` | `null` | — | — | — | Screen text: cut to [x, y, width, height] in CSS pixels, per pixel; null draws it whole. |

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
  field outline f32 "World text: an outline around the glyphs, as a fraction (0 to 1) of the widest the font's distance range allows (a font cooked with a larger px_range takes thicker outlines)." {
    min 0
    max 1
  }
  field outlineColor Color "The outline's linear colour."
  field screen bool "Draw in screen space over the finished frame: position is the first baseline's origin in CSS pixels from the top left, y down, size in CSS pixels, rotation clockwise."
  field clip Option.Array.f32.4 "Screen text: cut to [x, y, width, height] in CSS pixels, per pixel; null draws it whole."
}
```

<sub>`api.ogt` line 2411</sub>

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

<sub>`api.ogt` line 2445</sub>

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

<sub>`api.ogt` line 2456</sub>

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
| `flash` | `number` | `0` | 0 … 1 | — | — | A hit flash: the colour mixed towards flashColor by this much, keeping the alpha. |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour. |
| `glow` | `number` | `0` | ≥ 0 | — | — | Strength in the 2D glow buffer (PostEffects.glow); 0 none. |
| `blend` | `SpriteBlend` | `"alpha"` | — | — | — | Alpha or additive. |
| `castShadow` | `boolean` | `false` | — | — | — | Block 2D light (Lighting2D) with the sprite's rectangle. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror the region left to right. |
| `flipY` | `boolean` | `false` | — | — | — | Mirror the region top to bottom. |
| `sortY` | `boolean` | `false` | — | — | — | Within its layer, draw by y: lower on the screen draws in front (top-down games). |
| `slice` | `[number, number, number, number] \| null` | `null` | — | — | — | Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole. |
| `sliceScale` | `number` | `0.0625` | ≥ 0 | — | — | Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame instead of the world: position x, y in CSS pixels from the top left with y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested (a HUD). flash, glow, blend and castShadow apply to world sprites only. |
| `clip` | `[number, number, number, number] \| null` | `null` | — | — | — | Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole. |

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
  field flash f32 "A hit flash: the colour mixed towards flashColor by this much, keeping the alpha." {
    min 0
    max 1
  }
  field flashColor Color "The flash's linear colour." {
    default 1 1 1
  }
  field glow f32 "Strength in the 2D glow buffer (PostEffects.glow); 0 none." {
    min 0
  }
  field blend SpriteBlend "Alpha or additive."
  field castShadow bool "Block 2D light (Lighting2D) with the sprite's rectangle."
  field flipX bool "Mirror the region left to right."
  field flipY bool "Mirror the region top to bottom."
  field sortY bool "Within its layer, draw by y: lower on the screen draws in front (top-down games)."
  field slice Option.Array.f32.4 "Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole."
  field sliceScale f32 "Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets." {
    default 0.0625
    min 0
  }
  field screen bool "Draw in screen space over the finished frame instead of the world: position x, y in CSS pixels from the top left with y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested (a HUD). flash, glow, blend and castShadow apply to world sprites only."
  field clip Option.Array.f32.4 "Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole."
}
```

<sub>`api.ogt` line 2463</sub>

<a id="type-FloatingTextDesc"></a>

### `FloatingTextDesc` (desc struct)

A damage number or pickup text that pops at a point, rises, spreads and fades by itself (no per-tick submission). Deterministic: its drift comes from the tick and its order, and it is world state (the og.sprite state block).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The text, such as `-12` or `+1 key`. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it appears (above a hit, usually). |
| `font` | `Font \| null` | `null` | — | — | — | The font; null uses the UI's body font. |
| `size` | `number` | `0.5` | ≥ 0 | `m` | — | The em size in world units. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear fill colour. |
| `outline` | `number` | `0.6` | 0 … 1 | — | — | The outline's width as a fraction of the font's distance range (0 none). |
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The outline's linear colour. |
| `rise` | `number` | `1` | — | `m` | — | How far it climbs over its life. |
| `spread` | `number` | `0.3` | ≥ 0 | `m` | — | The largest sideways drift, left or right. |
| `lifetime` | `number` | `0.8` | ≥ 0.05 | `s` | — | How long it shows. |
| `fade` | `number` | `0.4` | 0 … 1 | `ratio` | — | The last part of its life it fades over. |
| `pop` | `number` | `0.4` | ≥ 0 | — | — | How much bigger it starts, settling to its size in the first fifth of its life. |
| `crit` | `boolean` | `false` | — | — | — | A critical hit: critScale times bigger, a stronger pop. |
| `critScale` | `number` | `1.6` | ≥ 1 | — | — | The size of a crit. |
| `layer` | `number` | `100` | — | — | — | The sprite layer (above the action). |

```ogt
struct FloatingTextDesc {
  doc "A damage number or pickup text that pops at a point, rises, spreads and fades by itself (no per-tick submission). Deterministic: its drift comes from the tick and its order, and it is world state (the og.sprite state block)."
  desc true
  field text String "The text, such as `-12` or `+1 key`."
  field position Vector3 "Where it appears (above a hit, usually)." {
    unit m
  }
  field font Option.Font "The font; null uses the UI's body font."
  field size f32 "The em size in world units." {
    default 0.5
    min 0
    unit m
  }
  field color Color "The linear fill colour." {
    default 1 1 1
  }
  field outline f32 "The outline's width as a fraction of the font's distance range (0 none)." {
    default 0.6
    min 0
    max 1
  }
  field outlineColor Color "The outline's linear colour." {
    default 0.02 0.02 0.03
  }
  field rise f32 "How far it climbs over its life." {
    default 1
    unit m
  }
  field spread f32 "The largest sideways drift, left or right." {
    default 0.3
    min 0
    unit m
  }
  field lifetime f32 "How long it shows." {
    default 0.8
    min 0.05
    unit s
  }
  field fade f32 "The last part of its life it fades over." {
    default 0.4
    min 0
    max 1
    unit ratio
  }
  field pop f32 "How much bigger it starts, settling to its size in the first fifth of its life." {
    default 0.4
    min 0
  }
  field crit bool "A critical hit: critScale times bigger, a stronger pop."
  field critScale f32 "The size of a crit." {
    default 1.6
    min 1
  }
  field layer i32 "The sprite layer (above the action)." {
    default 100
  }
}
```

<sub>`api.ogt` line 2626</sub>

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

<sub>`api.ogt` line 2698</sub>

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

<sub>`api.ogt` line 2718</sub>

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

<sub>`api.ogt` line 2733</sub>

<a id="type-RenderCapabilities"></a>

### `RenderCapabilities` (struct)

What the renderer can do on this device. Backend limits are reported here, never by missing members.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tier` | `RenderTier` | `"webgpu"` | — | — | — | The backend. |
| `compute` | `boolean` | `false` | — | — | — | Whether compute passes run (GPU culling, GPU particles). |
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing the device resolved RenderOptions.antialias to. |
| `maxLights` | `number` | `0` | — | — | — | How many point and spot lights one frame draws (MAX_LIGHTS on every tier). |
| `maxShadowedLights` | `number` | `0` | — | — | — | How many point and spot lights may cast shadows. |
| `width` | `number` | `0` | — | — | — | The render target width in device pixels. |
| `height` | `number` | `0` | — | — | — | The render target height in device pixels. |

```ogt
struct RenderCapabilities {
  doc "What the renderer can do on this device. Backend limits are reported here, never by missing members."
  field tier RenderTier "The backend."
  field compute bool "Whether compute passes run (GPU culling, GPU particles)."
  field antialias Antialias "The antialiasing the device resolved RenderOptions.antialias to."
  field maxLights u32 "How many point and spot lights one frame draws (MAX_LIGHTS on every tier)."
  field maxShadowedLights u32 "How many point and spot lights may cast shadows."
  field width u32 "The render target width in device pixels."
  field height u32 "The render target height in device pixels."
}
```

<sub>`api.ogt` line 2775</sub>

<a id="type-Shader"></a>

### `Shader` (handle)

A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader and the parameters they read.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Shader "A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader and the parameters they read."
```

<sub>`api.ogt` line 2801</sub>

## Column kinds

Each is a factory, `Geometry(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Geometry"></a>`Geometry` | `Geometry` | `Uint32Array` | 2 | A Geometry handle field. |
| <a id="column-Material"></a>`Material` | `Material` | `Uint32Array` | 2 | A Material handle field. |
| <a id="column-ToneMapping"></a>`ToneMapping` | `ToneMapping` | `Uint32Array` | 1 | A ToneMapping field: the variant's index. |
| <a id="column-Texture"></a>`Texture` | `Texture` | `Uint32Array` | 2 | A Texture handle field (0 is none). |
| <a id="column-Lut"></a>`Lut` | `Lut` | `Uint32Array` | 2 | A Lut handle field (0 is none). |
| <a id="column-EnvironmentMap"></a>`EnvironmentMap` | `EnvironmentMap` | `Uint32Array` | 2 | An EnvironmentMap handle field (0 is none). |
| <a id="column-ParticleEmitter"></a>`ParticleEmitter` | `ParticleEmitter` | `Uint32Array` | 2 | A ParticleEmitter handle field (0 is none). |
| <a id="column-AoMethod"></a>`AoMethod` | `AoMethod` | `Uint32Array` | 1 | An AoMethod field: the variant's index. |
| <a id="column-DofQuality"></a>`DofQuality` | `DofQuality` | `Uint32Array` | 1 | A DofQuality field: the variant's index. |
| <a id="column-SpriteBlend"></a>`SpriteBlend` | `SpriteBlend` | `Uint32Array` | 1 | A SpriteBlend field: the variant's index. |
| <a id="column-StripBlend"></a>`StripBlend` | `StripBlend` | `Uint32Array` | 1 | A StripBlend field: the variant's index. |

```ogt
column Geometry {
  doc "A Geometry handle field."
  array Uint32Array
  stride 2
  value Geometry
}
```

<sub>`api.ogt` line 1590</sub>

```ogt
column Material {
  doc "A Material handle field."
  array Uint32Array
  stride 2
  value Material
}
```

<sub>`api.ogt` line 1597</sub>

```ogt
column ToneMapping {
  doc "A ToneMapping field: the variant's index."
  array Uint32Array
  stride 1
  value ToneMapping
}
```

<sub>`api.ogt` line 1845</sub>

```ogt
column Texture {
  doc "A Texture handle field (0 is none)."
  array Uint32Array
  stride 2
  value Texture
}
```

<sub>`api.ogt` line 1901</sub>

```ogt
column Lut {
  doc "A Lut handle field (0 is none)."
  array Uint32Array
  stride 2
  value Lut
}
```

<sub>`api.ogt` line 1908</sub>

```ogt
column EnvironmentMap {
  doc "An EnvironmentMap handle field (0 is none)."
  array Uint32Array
  stride 2
  value EnvironmentMap
}
```

<sub>`api.ogt` line 1915</sub>

```ogt
column ParticleEmitter {
  doc "A ParticleEmitter handle field (0 is none)."
  array Uint32Array
  stride 2
  value ParticleEmitter
}
```

<sub>`api.ogt` line 1942</sub>

```ogt
column AoMethod {
  doc "An AoMethod field: the variant's index."
  array Uint32Array
  stride 1
  value AoMethod
}
```

<sub>`api.ogt` line 1956</sub>

```ogt
column DofQuality {
  doc "A DofQuality field: the variant's index."
  array Uint32Array
  stride 1
  value DofQuality
}
```

<sub>`api.ogt` line 1969</sub>

```ogt
column SpriteBlend {
  doc "A SpriteBlend field: the variant's index."
  array Uint32Array
  stride 1
  value SpriteBlend
}
```

<sub>`api.ogt` line 1982</sub>

```ogt
column StripBlend {
  doc "A StripBlend field: the variant's index."
  array Uint32Array
  stride 1
  value StripBlend
}
```

<sub>`api.ogt` line 1995</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_LIGHTS"></a>`MAX_LIGHTS` | `number` | `256` | The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept. |
| <a id="const-MAX_CLUSTER_LIGHTS"></a>`MAX_CLUSTER_LIGHTS` | `number` | `64` | The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there. |

```ogt
const MAX_LIGHTS u32 256 "The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept."
```

<sub>`api.ogt` line 2772</sub>

```ogt
const MAX_CLUSTER_LIGHTS u32 64 "The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there."
```

<sub>`api.ogt` line 2773</sub>
