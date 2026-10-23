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

<sub>`api.ogt` line 2234</sub>

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
| [`PointShadows`](#type-PointShadows) | enum | How a point light's shadow map is stored, as og-render's PointShadows. RenderCapabilities.pointShadows reports what the device resolved. |
| [`TaaHistory`](#type-TaaHistory) | enum | How TAA resamples its history each frame, as og-render's TaaHistory. |
| [`DebugView`](#type-DebugView) | enum | What the debug overlay shows besides og.debug's draws (og.debug.setView, RenderOptions.debugView, `debug:<view>`), drawn over the finished frame. Streams add views as they add what to show. |
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
| [`EnvironmentMap`](#type-EnvironmentMap) | handle | A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`, a KTX2 cube map, or six face images). |
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
| [`Palette`](#type-Palette) | handle | A colour palette for Stylise.palette, from og.load.palette (a `.hex` or GIMP `.gpl` file of up to 256 colours). |
| [`Look`](#type-Look) | handle | A look from og.load.look (an `.ogt` of `type look`): the frame's whole art direction as data, applied with og.render.applyLook. |
| [`Palette()`](#column-Palette) | column kind | A Palette handle field (0 is none). |
| [`DitherPattern`](#type-DitherPattern) | enum | The threshold pattern Stylise.dither spreads the steps of posterize and palette with. |
| [`DitherPattern()`](#column-DitherPattern) | column kind | A DitherPattern field: the variant's index. |
| [`StyliseSpace`](#type-StyliseSpace) | enum | Where Stylise lays a pattern (the paint canvas, the hatching, the halftone). |
| [`StyliseSpace()`](#column-StyliseSpace) | column kind | A StyliseSpace field: the variant's index. |
| [`og.Stylise`](#record-Stylise) | resource | Stylisation primitives that move the frame away from the photographic: a low-resolution pixel grid scaled up by whole numbers, fewer tones or a fixed palette through an ordered or blue-noise dither, halftone dots, cross-hatching, painterly smoothing over a canvas, ink lines on tonal edges, flat colour bands by depth, and the vertex snapping, affine textures and coarse depth of early 3D hardware. They compose: a look (og.load.look) sets this with PostEffects and RenderSettings, and the game's art direction is in that data, not in the engine. Every primitive is off at its zero value (0, false, none) and then costs nothing: no pass is recorded and no pipeline is built. RenderCapabilities.stylise names the primitives this renderer draws; a field it does not name is carried and read back but not drawn. Read after each tick; it never feeds back into the simulation. |
| [`og.MaterialStyle`](#record-MaterialStyle) | resource | Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls off with distance, and flat facets. Each applies to a material (og.render.standardMaterial or a loaded glTF's) that leaves that field at its own default; unlit and shader materials, materials with StandardMaterialDesc.styled false, sprites and the UI are untouched, and toon passes by a material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which keeps its physical shading. All off (false, 0) is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds back into the simulation. |
| [`LookInfo`](#type-LookInfo) | struct | A loaded look: where it came from and what applying it sets. |
| [`og.Lighting2D`](#record-Lighting2D) | resource | 2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default. |
| [`og.Light2D`](#record-Light2D) | component | A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff, shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn. |
| [`og.Occluder2D`](#record-Occluder2D) | component | A rectangle that blocks 2D light (walls, crates, characters), centred on the entity's position and turned with it about z. Pixels inside it are not shadowed by it. At most 128 are drawn. |
| [`og.Decal`](#record-Decal) | component | A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one). At most 32 are drawn. |
| [`PARTICLES_ARMED`](#const-PARTICLES_ARMED) | constant | ParticleSystem.startTick's default: never written. A looping emitter emits as if started at tick 0; a one-shot (`looping false`) emits nothing until its startTick is written, so a burst spawned in `start` waits for its first use. Write it back to re-arm a one-shot without firing it. |
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
| [`og.Sprite`](#record-Sprite) | component | A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture (or an `array true` sheet's texture array layer), region and anchor come from the flipbook's current frame. |
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
| [`UI_LAYER`](#const-UI_LAYER) | constant | The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the screen transition included (a HUD element made of sprites sitting on a UI panel: draw it only while its screen shows), in their layers' order either way. |
| [`RenderCapabilities`](#type-RenderCapabilities) | struct | What the renderer can do on this device. Backend limits are reported here, never by missing members. |
| [`og.render.standardMaterial`](#fn-standardMaterial) | function | Create a standard material. |
| [`og.render.updateMaterial`](#fn-updateMaterial) | function | Change some parameters of a standard material. |
| [`Shader`](#type-Shader) | handle | A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader and the parameters they read. |
| [`og.render.shaderMaterial`](#fn-shaderMaterial) | function | Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive; the og-game skill's Material shaders section has their WGSL contract), its parameters at their defaults. The hooks apply to skinned meshes too, and shadows and the depth prepass (ambient occlusion, outlines, depth of field) see the mesh where vertex_offset moved it and cut where surface_color's alpha falls below alphaTest. |
| [`og.render.setMaterialShader`](#fn-setMaterialShader) | function | Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's parameters at their defaults. Every mesh drawn with that material changes. |
| [`og.render.setShaderParam`](#fn-setShaderParam) | function | Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color. |
| [`og.render.applyLook`](#fn-applyLook) | function | Apply a look, in GameDesc.start (or name it in GameDesc.look) or at any tick for a level's mood, a flashback or a photo mode: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma, which game code animates, keep theirs), and the UI is configured with its skin, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes. A look that switches the stylisation (a Stylise pass on or off, the pixel grid, retro 3D, the material style) starts TAA's history over, as a camera cut does, so its first frame holds no ghost of the old look. |
| [`og.render.look`](#fn-look) | function | The look last applied (by og.render.applyLook or GameDesc.look), as og.load.lookInfo describes it; null before any. |
| [`og.render.capabilities`](#fn-capabilities) | function | The renderer's tier and limits on this device. |
| [`og.render.text`](#fn-text) | function | Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout. |
| [`BufferGeometryDesc`](#type-BufferGeometryDesc) | desc struct | Geometry from attribute arrays, as three.js BufferGeometry with its attributes set: one item per vertex in each array, triangles three indices each (or, with no indices, every three vertices). |
| [`GeometryInfo`](#type-GeometryInfo) | struct | What a geometry holds (og.render.geometryInfo). |
| [`og.render.bufferGeometry`](#fn-bufferGeometry) | function | Make a geometry from attribute arrays (three.js BufferGeometry): procedural levels, 2D polygons, meshes a tool made. It draws as any geometry (og.Mesh), its BVH is built for og.render.raycast, and (desc.physics) it is a Collider.mesh at once. |
| [`og.render.updateGeometry`](#fn-updateGeometry) | function | Rewrite a dynamic geometry's attributes in place (desc as og.render.bufferGeometry's, every attribute given again; `dynamic` and `physics` keep their values from when it was made). The same vertex and index counts rewrite its GPU buffers in place; other counts reallocate them, which costs more and is fine now and then. Its BVH is rebuilt, and its physics mesh re-registered (colliders made from it are rebuilt at the next step). Every og.Mesh, og.Lines and og.Points showing it changes. |
| [`og.render.geometryInfo`](#fn-geometryInfo) | function | A geometry's counts and bounds. |
| [`CylinderGeometryDesc`](#type-CylinderGeometryDesc) | desc struct | A cylinder (or a truncated cone) along +Y centred at the origin, as three.js CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength). |
| [`ConeGeometryDesc`](#type-ConeGeometryDesc) | desc struct | A cone along +Y centred at the origin, its tip up, as three.js ConeGeometry(radius, height, radialSegments, heightSegments, openEnded). |
| [`CapsuleGeometryDesc`](#type-CapsuleGeometryDesc) | desc struct | A capsule along +Y centred at the origin, as three.js CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments): a cylinder of `height` between two hemispheres, `height + 2 radius` long in all (an og.Collider capsule of halfHeight height / 2 and that radius matches it). |
| [`TorusGeometryDesc`](#type-TorusGeometryDesc) | desc struct | A torus in the xy plane around +Z, as three.js TorusGeometry(radius, tube, radialSegments, tubularSegments, arc). |
| [`RingGeometryDesc`](#type-RingGeometryDesc) | desc struct | A flat ring in the xy plane facing +Z, as three.js RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength). |
| [`CircleGeometryDesc`](#type-CircleGeometryDesc) | desc struct | A disc (or a sector) in the xy plane facing +Z, as three.js CircleGeometry(radius, segments, thetaStart, thetaLength): a 2D polygon's simplest shape. |
| [`TubeGeometryDesc`](#type-TubeGeometryDesc) | desc struct | A tube along a curve through points, as three.js TubeGeometry(new CatmullRomCurve3(points, closed, 'centripetal'), tubularSegments, radius, radialSegments, closed): u runs along it, v around. The points may come from anywhere: a path's samples, a rope's points, a spline a tool made. |
| [`og.render.cylinderGeometry`](#fn-cylinderGeometry) | function | Make a cylinder geometry for og.Mesh (give its body a cylinder-like Collider: a capsule, a convex Collider.mesh of it, or the geometry itself). |
| [`og.render.coneGeometry`](#fn-coneGeometry) | function | Make a cone geometry for og.Mesh. Physics can use it as a convex Collider.mesh. |
| [`og.render.capsuleGeometry`](#fn-capsuleGeometry) | function | Make a capsule geometry for og.Mesh (its body's Collider is a capsule of the same radius and half the height). |
| [`og.render.torusGeometry`](#fn-torusGeometry) | function | Make a torus geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh. |
| [`og.render.ringGeometry`](#fn-ringGeometry) | function | Make a ring geometry for og.Mesh (one-sided: give it a double-sided material to see it from behind). |
| [`og.render.circleGeometry`](#fn-circleGeometry) | function | Make a circle geometry for og.Mesh (one-sided, as ringGeometry). |
| [`og.render.tubeGeometry`](#fn-tubeGeometry) | function | Make a tube along a curve through points for og.Mesh: pipes, cables, rails, a path shown as a ribbon of geometry. Physics can use it as a triangle-mesh Collider.mesh. |
| [`LineMode`](#type-LineMode) | enum | How a line set's points join, as three.js's three line objects. |
| [`LineMode()`](#column-LineMode) | column kind | A LineMode field: the variant's index. |
| [`LinesDesc`](#type-LinesDesc) | desc struct | Lines through points, drawn this frame only (immediate mode, as og.render.sprite): submit them every tick they should show. three.js Line, LineSegments and LineLoop with a LineBasicMaterial or LineDashedMaterial, but any width on every tier (each segment is a quad in screen space with a feathered edge, so WebGL2's one-pixel lines do not apply). |
| [`og.render.lines`](#fn-lines) | function | Draw lines this frame. Systems and commands may call it; the list is cleared at every tick, so submit them every tick they should show (og.Lines keeps a static set). |
| [`PointsDesc`](#type-PointsDesc) | desc struct | A point cloud drawn this frame only (immediate mode), as three.js Points with a PointsMaterial: a square per point facing the camera. Submit it every tick it should show (og.Points keeps a static set). |
| [`og.render.points`](#fn-points) | function | Draw a point cloud this frame. Systems and commands may call it; the list is cleared at every tick. |
| [`og.Lines`](#record-Lines) | component | Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a track, a constellation. The vertices are taken in order (the geometry's indices are not read), with its vertex colours times color. |
| [`og.Points`](#record-Points) | component | Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. Vertex colours times color. |
| [`RaycastGeometryDesc`](#type-RaycastGeometryDesc) | desc struct | Which meshes og.render.raycast tests, as three.js Raycaster's near, far and layers. |
| [`GeometryHit`](#type-GeometryHit) | struct | Where a ray meets a drawn triangle, as three.js Raycaster's intersection. |
| [`og.render.raycast`](#fn-raycast) | function | Every drawn triangle a ray crosses, nearest first, as three.js Raycaster.intersectObjects: what is drawn, not the colliders (og.physics.raycast answers those). It tests og.Mesh and og.Model entities at their world transforms through a BVH per geometry (built when the geometry is made, loaded or updated), both faces of each triangle, skinned models in their bind pose. Read the world as of the call; inside a system, its transforms as earlier systems left them. |

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

<sub>`api.ogt` line 3473</sub>

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

<sub>`api.ogt` line 3601</sub>

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

<sub>`api.ogt` line 3778</sub>

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

<sub>`api.ogt` line 3785</sub>

<a id="fn-boxGeometry"></a>

### `og.render.boxGeometry`

Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too.

```ts
og.render.boxGeometry(world: World, desc?: BoxGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `BoxGeometryDesc` | yes | Its extents and texture repeat. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_box_geometry` · publish builds: kept

```ogt
fn boxGeometry {
  doc "Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too."
  param world World "The world whose renderer owns the geometry."
  param desc BoxGeometryDesc "Its extents and texture repeat."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 3854</sub>

<a id="fn-planeGeometry"></a>

### `og.render.planeGeometry`

Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too.

```ts
og.render.planeGeometry(world: World, desc?: PlaneGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `PlaneGeometryDesc` | yes | Its extents and texture repeat. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_plane_geometry` · publish builds: kept

```ogt
fn planeGeometry {
  doc "Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too."
  param world World "The world whose renderer owns the geometry."
  param desc PlaneGeometryDesc "Its extents and texture repeat."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 3861</sub>

<a id="fn-sphereGeometry"></a>

### `og.render.sphereGeometry`

Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh).

```ts
og.render.sphereGeometry(world: World, desc?: SphereGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `SphereGeometryDesc` | yes | Its radius, tessellation and texture repeat. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_sphere_geometry` · publish builds: kept

```ogt
fn sphereGeometry {
  doc "Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh)."
  param world World "The world whose renderer owns the geometry."
  param desc SphereGeometryDesc "Its radius, tessellation and texture repeat."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 3868</sub>

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

<sub>`api.ogt` line 3892</sub>

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

<sub>`api.ogt` line 3899</sub>

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

<sub>`api.ogt` line 3909</sub>

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

<sub>`api.ogt` line 3917</sub>

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

<sub>`api.ogt` line 3925</sub>

<a id="fn-applyLook"></a>

### `og.render.applyLook`

Apply a look, in GameDesc.start (or name it in GameDesc.look) or at any tick for a level's mood, a flashback or a photo mode: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma, which game code animates, keep theirs), and the UI is configured with its skin, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes. A look that switches the stylisation (a Stylise pass on or off, the pixel grid, retro 3D, the material style) starts TAA's history over, as a camera cut does, so its first frame holds no ghost of the old look.

```ts
og.render.applyLook(world: World, look: Look): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose look changes. |
| `look` | `Look` | no | The look, from og.load.look. |

**Returns** `Result<void>`: Refused for a look not loaded in this world, one in a streaming group that is not resident (the reason names it), or a resource the system did not declare.

Binding symbol `og_render_apply_look` · publish builds: kept

```ogt
fn applyLook {
  doc "Apply a look, in GameDesc.start (or name it in GameDesc.look) or at any tick for a level's mood, a flashback or a photo mode: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma, which game code animates, keep theirs), and the UI is configured with its skin, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes. A look that switches the stylisation (a Stylise pass on or off, the pixel grid, retro 3D, the material style) starts TAA's history over, as a camera cut does, so its first frame holds no ghost of the old look."
  param world World "The world whose look changes."
  param look Look "The look, from og.load.look."
  returns Result.Void "Refused for a look not loaded in this world, one in a streaming group that is not resident (the reason names it), or a resource the system did not declare."
}
```

<sub>`api.ogt` line 3934</sub>

<a id="fn-look"></a>

### `og.render.look`

The look last applied (by og.render.applyLook or GameDesc.look), as og.load.lookInfo describes it; null before any.

```ts
og.render.look(world: World): LookInfo | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `LookInfo | null`.

Binding symbol `og_render_look` · publish builds: kept

```ogt
fn look {
  doc "The look last applied (by og.render.applyLook or GameDesc.look), as og.load.lookInfo describes it; null before any."
  param world World "The world asked."
  returns Option.LookInfo
}
```

<sub>`api.ogt` line 3941</sub>

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

<sub>`api.ogt` line 3947</sub>

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

<sub>`api.ogt` line 3953</sub>

<a id="fn-bufferGeometry"></a>

### `og.render.bufferGeometry`

Make a geometry from attribute arrays (three.js BufferGeometry): procedural levels, 2D polygons, meshes a tool made. It draws as any geometry (og.Mesh), its BVH is built for og.render.raycast, and (desc.physics) it is a Collider.mesh at once.

```ts
og.render.bufferGeometry(world: World, desc?: BufferGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `BufferGeometryDesc` | yes | The attributes. |

**Returns** `Result<Geometry>`: Refused for an attribute whose length does not fit the vertex count, an index past the vertices, or a number that is not finite (the reason names the attribute).

Binding symbol `og_render_buffer_geometry` · publish builds: kept

```ogt
fn bufferGeometry {
  doc "Make a geometry from attribute arrays (three.js BufferGeometry): procedural levels, 2D polygons, meshes a tool made. It draws as any geometry (og.Mesh), its BVH is built for og.render.raycast, and (desc.physics) it is a Collider.mesh at once."
  param world World "The world whose renderer owns the geometry."
  param desc BufferGeometryDesc "The attributes."
  returns Result.Geometry "Refused for an attribute whose length does not fit the vertex count, an index past the vertices, or a number that is not finite (the reason names the attribute)."
}
```

<sub>`api.ogt` line 3991</sub>

<a id="fn-updateGeometry"></a>

### `og.render.updateGeometry`

Rewrite a dynamic geometry's attributes in place (desc as og.render.bufferGeometry's, every attribute given again; `dynamic` and `physics` keep their values from when it was made). The same vertex and index counts rewrite its GPU buffers in place; other counts reallocate them, which costs more and is fine now and then. Its BVH is rebuilt, and its physics mesh re-registered (colliders made from it are rebuilt at the next step). Every og.Mesh, og.Lines and og.Points showing it changes.

```ts
og.render.updateGeometry(world: World, geometry: Geometry, desc?: BufferGeometryDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `geometry` | `Geometry` | no | A geometry og.render.bufferGeometry made with dynamic true. |
| `desc` | `BufferGeometryDesc` | yes | The new attributes. |

**Returns** `Result<void>`: Refused for a dead or static geometry, or attributes og.render.bufferGeometry refuses.

Binding symbol `og_render_update_geometry` · publish builds: kept

```ogt
fn updateGeometry {
  doc "Rewrite a dynamic geometry's attributes in place (desc as og.render.bufferGeometry's, every attribute given again; `dynamic` and `physics` keep their values from when it was made). The same vertex and index counts rewrite its GPU buffers in place; other counts reallocate them, which costs more and is fine now and then. Its BVH is rebuilt, and its physics mesh re-registered (colliders made from it are rebuilt at the next step). Every og.Mesh, og.Lines and og.Points showing it changes."
  param world World "The world whose renderer owns the geometry."
  param geometry Geometry "A geometry og.render.bufferGeometry made with dynamic true."
  param desc BufferGeometryDesc "The new attributes."
  returns Result.Void "Refused for a dead or static geometry, or attributes og.render.bufferGeometry refuses."
}
```

<sub>`api.ogt` line 3998</sub>

<a id="fn-geometryInfo"></a>

### `og.render.geometryInfo`

A geometry's counts and bounds.

```ts
og.render.geometryInfo(world: World, geometry: Geometry): Result<GeometryInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `geometry` | `Geometry` | no | Any geometry: made, a primitive or a loaded glTF's. |

**Returns** `Result<GeometryInfo>`: Refused for a dead geometry.

Binding symbol `og_render_geometry_info` · publish builds: kept

```ogt
fn geometryInfo {
  doc "A geometry's counts and bounds."
  param world World "The world whose renderer owns the geometry."
  param geometry Geometry "Any geometry: made, a primitive or a loaded glTF's."
  returns Result.GeometryInfo "Refused for a dead geometry."
}
```

<sub>`api.ogt` line 4006</sub>

<a id="fn-cylinderGeometry"></a>

### `og.render.cylinderGeometry`

Make a cylinder geometry for og.Mesh (give its body a cylinder-like Collider: a capsule, a convex Collider.mesh of it, or the geometry itself).

```ts
og.render.cylinderGeometry(world: World, desc?: CylinderGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `CylinderGeometryDesc` | yes | Its radii, height and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_cylinder_geometry` · publish builds: kept

```ogt
fn cylinderGeometry {
  doc "Make a cylinder geometry for og.Mesh (give its body a cylinder-like Collider: a capsule, a convex Collider.mesh of it, or the geometry itself)."
  param world World "The world whose renderer owns the geometry."
  param desc CylinderGeometryDesc "Its radii, height and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 4200</sub>

<a id="fn-coneGeometry"></a>

### `og.render.coneGeometry`

Make a cone geometry for og.Mesh. Physics can use it as a convex Collider.mesh.

```ts
og.render.coneGeometry(world: World, desc?: ConeGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `ConeGeometryDesc` | yes | Its radius, height and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_cone_geometry` · publish builds: kept

```ogt
fn coneGeometry {
  doc "Make a cone geometry for og.Mesh. Physics can use it as a convex Collider.mesh."
  param world World "The world whose renderer owns the geometry."
  param desc ConeGeometryDesc "Its radius, height and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 4207</sub>

<a id="fn-capsuleGeometry"></a>

### `og.render.capsuleGeometry`

Make a capsule geometry for og.Mesh (its body's Collider is a capsule of the same radius and half the height).

```ts
og.render.capsuleGeometry(world: World, desc?: CapsuleGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `CapsuleGeometryDesc` | yes | Its radius, length and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_capsule_geometry` · publish builds: kept

```ogt
fn capsuleGeometry {
  doc "Make a capsule geometry for og.Mesh (its body's Collider is a capsule of the same radius and half the height)."
  param world World "The world whose renderer owns the geometry."
  param desc CapsuleGeometryDesc "Its radius, length and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 4214</sub>

<a id="fn-torusGeometry"></a>

### `og.render.torusGeometry`

Make a torus geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh.

```ts
og.render.torusGeometry(world: World, desc?: TorusGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `TorusGeometryDesc` | yes | Its radii and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_torus_geometry` · publish builds: kept

```ogt
fn torusGeometry {
  doc "Make a torus geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh."
  param world World "The world whose renderer owns the geometry."
  param desc TorusGeometryDesc "Its radii and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 4221</sub>

<a id="fn-ringGeometry"></a>

### `og.render.ringGeometry`

Make a ring geometry for og.Mesh (one-sided: give it a double-sided material to see it from behind).

```ts
og.render.ringGeometry(world: World, desc?: RingGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `RingGeometryDesc` | yes | Its radii and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_ring_geometry` · publish builds: kept

```ogt
fn ringGeometry {
  doc "Make a ring geometry for og.Mesh (one-sided: give it a double-sided material to see it from behind)."
  param world World "The world whose renderer owns the geometry."
  param desc RingGeometryDesc "Its radii and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 4228</sub>

<a id="fn-circleGeometry"></a>

### `og.render.circleGeometry`

Make a circle geometry for og.Mesh (one-sided, as ringGeometry).

```ts
og.render.circleGeometry(world: World, desc?: CircleGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `CircleGeometryDesc` | yes | Its radius and tessellation. |

**Returns** `Result<Geometry>`.

Binding symbol `og_render_circle_geometry` · publish builds: kept

```ogt
fn circleGeometry {
  doc "Make a circle geometry for og.Mesh (one-sided, as ringGeometry)."
  param world World "The world whose renderer owns the geometry."
  param desc CircleGeometryDesc "Its radius and tessellation."
  returns Result.Geometry
}
```

<sub>`api.ogt` line 4235</sub>

<a id="fn-tubeGeometry"></a>

### `og.render.tubeGeometry`

Make a tube along a curve through points for og.Mesh: pipes, cables, rails, a path shown as a ribbon of geometry. Physics can use it as a triangle-mesh Collider.mesh.

```ts
og.render.tubeGeometry(world: World, desc?: TubeGeometryDesc): Result<Geometry>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the geometry. |
| `desc` | `TubeGeometryDesc` | yes | The points, radius and tessellation. |

**Returns** `Result<Geometry>`: Refused for fewer than two points or a point that is not finite.

Binding symbol `og_render_tube_geometry` · publish builds: kept

```ogt
fn tubeGeometry {
  doc "Make a tube along a curve through points for og.Mesh: pipes, cables, rails, a path shown as a ribbon of geometry. Physics can use it as a triangle-mesh Collider.mesh."
  param world World "The world whose renderer owns the geometry."
  param desc TubeGeometryDesc "The points, radius and tessellation."
  returns Result.Geometry "Refused for fewer than two points or a point that is not finite."
}
```

<sub>`api.ogt` line 4242</sub>

<a id="fn-lines"></a>

### `og.render.lines`

Draw lines this frame. Systems and commands may call it; the list is cleared at every tick, so submit them every tick they should show (og.Lines keeps a static set).

```ts
og.render.lines(world: World, desc?: LinesDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws them. |
| `desc` | `LinesDesc` | yes | The points and the look. |

**Returns** `Result<void>`: Refused for fewer than two points or a colours list whose length is not the points'.

Binding symbol `og_render_lines` · publish builds: kept

```ogt
fn lines {
  doc "Draw lines this frame. Systems and commands may call it; the list is cleared at every tick, so submit them every tick they should show (og.Lines keeps a static set)."
  param world World "The world whose renderer draws them."
  param desc LinesDesc "The points and the look."
  returns Result.Void "Refused for fewer than two points or a colours list whose length is not the points'."
}
```

<sub>`api.ogt` line 4300</sub>

<a id="fn-points"></a>

### `og.render.points`

Draw a point cloud this frame. Systems and commands may call it; the list is cleared at every tick.

```ts
og.render.points(world: World, desc?: PointsDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `PointsDesc` | yes | The positions and the look. |

**Returns** `Result<void>`: Refused for a colours list whose length is not the points'.

Binding symbol `og_render_points` · publish builds: kept

```ogt
fn points {
  doc "Draw a point cloud this frame. Systems and commands may call it; the list is cleared at every tick."
  param world World "The world whose renderer draws it."
  param desc PointsDesc "The positions and the look."
  returns Result.Void "Refused for a colours list whose length is not the points'."
}
```

<sub>`api.ogt` line 4335</sub>

<a id="fn-raycast"></a>

### `og.render.raycast`

Every drawn triangle a ray crosses, nearest first, as three.js Raycaster.intersectObjects: what is drawn, not the colliders (og.physics.raycast answers those). It tests og.Mesh and og.Model entities at their world transforms through a BVH per geometry (built when the geometry is made, loaded or updated), both faces of each triangle, skinned models in their bind pose. Read the world as of the call; inside a system, its transforms as earlier systems left them.

```ts
og.render.raycast(world: World, ray: Ray, desc?: RaycastGeometryDesc): Result<readonly GeometryHit[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |
| `ray` | `Ray` | no | The ray; its direction need not be normalised and must not be zero. |
| `desc` | `RaycastGeometryDesc` | yes | Reach, nearest only and layers. |

**Returns** `Result<readonly GeometryHit[]>`: Refused for a zero or non-finite direction.

Binding symbol `og_render_raycast` · publish builds: kept

```ogt
fn raycast {
  doc "Every drawn triangle a ray crosses, nearest first, as three.js Raycaster.intersectObjects: what is drawn, not the colliders (og.physics.raycast answers those). It tests og.Mesh and og.Model entities at their world transforms through a BVH per geometry (built when the geometry is made, loaded or updated), both faces of each triangle, skinned models in their bind pose. Read the world as of the call; inside a system, its transforms as earlier systems left them."
  param world World "The world asked."
  param ray Ray "The ray; its direction need not be normalised and must not be zero."
  param desc RaycastGeometryDesc "Reach, nearest only and layers."
  returns Result.List.GeometryHit "Refused for a zero or non-finite direction."
}
```

<sub>`api.ogt` line 4433</sub>

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
| `castShadow` | `boolean` | `false` | — | — | — | Cast shadows, as three.js Object3D.castShadow: drawn into the sun's cascades and the shadowed lights' maps. Off by default: a lamp's own glass around its light, a glow card or a decal-like slab casts none. |
| `receiveShadow` | `boolean` | `true` | — | — | — | Receive shadows, as three.js Object3D.receiveShadow: darkened where the sun's cascades and the shadowed lights' maps say something stands between it and the light. On by default (three.js's is off): every mesh received shadows before the flag was applied. false lights it as if unshadowed by every light, while it still casts as castShadow says: a glowing sign, a hero kept readable in a dark alley. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |
| `flash` | `number` | `0` | 0 … 1 | — | tunable | A hit flash on this mesh only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). Carried with its transform to the lit shaders, as og.Model's, so a flashing mesh is one draw with the other meshes of its geometry and material, and the amount is continuous. |
| `flashColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash's linear colour, 0 to 1 a channel (8 bits a channel). |
| `materialLayer` | `number` | `-1` | ≥ -1 | — | tunable | The texture-array layer this mesh's material reads (StandardMaterialDesc.mapArray and normalMapArray), in place of the material's mapLayer and normalMapLayer: one material, one draw, a layer per mesh (a tile set, a crowd's skins). -1 keeps the material's layers; a material without arrays ignores it. |

```ogt
component Mesh {
  doc "Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh."
  doc "The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say."
  field geometry Geometry "The geometry."
  field material Material "The material."
  field castShadow bool "Cast shadows, as three.js Object3D.castShadow: drawn into the sun's cascades and the shadowed lights' maps. Off by default: a lamp's own glass around its light, a glow card or a decal-like slab casts none."
  field receiveShadow bool "Receive shadows, as three.js Object3D.receiveShadow: darkened where the sun's cascades and the shadowed lights' maps say something stands between it and the light. On by default (three.js's is off): every mesh received shadows before the flag was applied. false lights it as if unshadowed by every light, while it still casts as castShadow says: a glowing sign, a hero kept readable in a dark alley." {
    default true
  }
  field visible bool "Whether it is drawn." {
    default true
  }
  field flash f32 "A hit flash on this mesh only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). Carried with its transform to the lit shaders, as og.Model's, so a flashing mesh is one draw with the other meshes of its geometry and material, and the amount is continuous." {
    min 0
    max 1
    tunable true
  }
  field flashColor Color "The flash's linear colour, 0 to 1 a channel (8 bits a channel)." {
    default 1 1 1
  }
  field materialLayer i32 "The texture-array layer this mesh's material reads (StandardMaterialDesc.mapArray and normalMapArray), in place of the material's mapLayer and normalMapLayer: one material, one draw, a layer per mesh (a tile set, a crowd's skins). -1 keeps the material's layers; a material without arrays ignores it." {
    default -1
    min -1
    tunable true
  }
}
```

<sub>`api.ogt` line 2501</sub>

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
| `shadowSoftness` | `number` | `0` | 0 … 0.2 | — | tunable | Soft shadows (PCSS): the penumbra's width per metre between caster and receiver, the tangent of the sun's apparent radius (the real sun's is 0.005). 0 keeps the hard one-texel edge; 0.01 to 0.05 is a soft sun, sharp at contact and blurring with distance. Costs a blocker search and a 16-tap filter per shadowed pixel. |
| `cascadeBlend` | `number` | `0` | 0 … 0.5 | `ratio` | tunable | The fraction of each shadow cascade's depth over which it fades into the next, hiding the seam where the shadow's resolution changes. 0 is none; 0.1 to 0.2 is typical (those pixels read two cascades). |

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
  field shadowSoftness f32 "Soft shadows (PCSS): the penumbra's width per metre between caster and receiver, the tangent of the sun's apparent radius (the real sun's is 0.005). 0 keeps the hard one-texel edge; 0.01 to 0.05 is a soft sun, sharp at contact and blurring with distance. Costs a blocker search and a 16-tap filter per shadowed pixel." {
    min 0
    max 0.2
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

<sub>`api.ogt` line 2528</sub>

<a id="record-PointLight"></a>

### `og.PointLight` (component)

A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.

Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it, so hundreds are cheap. Lights out of view are skipped; up to MAX_LIGHTS in view are drawn, and past that (or past what one screen cluster holds, MAX_CLUSTER_LIGHTS) the ones brightest at the camera are kept. Of those in view with castShadow, RenderCapabilities.maxShadowedLights (8 on the full tier, 4 on WebGL2) get shadows, the brightest at the camera first.

Token `og.PointLight`, registered under the schema name `og.PointLight`; its value type is `PointLight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The light's linear colour. |
| `intensity` | `number` | `10` | ≥ 0 | — | tunable | The radiance at one metre. |
| `distance` | `number` | `10` | ≥ 0.01 | `m` | tunable | Where the light reaches zero, as three.js PointLight.distance (never infinite here). |
| `castShadow` | `boolean` | `false` | — | — | — | Cast shadows: six cube faces or two paraboloid hemispheres (RenderOptions.pointShadows). |

```ogt
component PointLight {
  doc "A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance."
  doc "Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it, so hundreds are cheap. Lights out of view are skipped; up to MAX_LIGHTS in view are drawn, and past that (or past what one screen cluster holds, MAX_CLUSTER_LIGHTS) the ones brightest at the camera are kept. Of those in view with castShadow, RenderCapabilities.maxShadowedLights (8 on the full tier, 4 on WebGL2) get shadows, the brightest at the camera first."
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
}
```

<sub>`api.ogt` line 2560</sub>

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

<sub>`api.ogt` line 2580</sub>

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
| `autoExposure` | `boolean` | `false` | — | — | — | Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the current frame's alone; otherwise it adapts over time like an eye, starting at the metered exposure of the first frame drawn after the first tick (a frame drawn before it, such as the web shell's first frame behind the play gate, meters at once and is never adapted from), so every host draws the same exposures for the same ticks. |
| `adaptSpeedUp` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame (never the wall clock, so a replay and every host adapt the same, however many frames each drew); 0 is instant. Snapshots and probe sweeps carry the adapted exposure. |
| `adaptSpeedDown` | `number` | `0` | 0 … 100 | `evPerSecond` | tunable | How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. Eyes adapt to brightness faster than to the dark: 3 down and 1 up is a natural pair. |
| `toneMapped` | `boolean` | `true` | — | — | — | Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1. |
| `toneMapping` | `ToneMapping` | `"aces"` | — | — | — | Which curve tone maps the frame while toneMapped is true: aces (the default), agx or none. |
| `exposure` | `number` | `1` | 0 … 16 | — | tunable | A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are. |
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
  field autoExposure bool "Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the current frame's alone; otherwise it adapts over time like an eye, starting at the metered exposure of the first frame drawn after the first tick (a frame drawn before it, such as the web shell's first frame behind the play gate, meters at once and is never adapted from), so every host draws the same exposures for the same ticks."
  field adaptSpeedUp f32 "How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame (never the wall clock, so a replay and every host adapt the same, however many frames each drew); 0 is instant. Snapshots and probe sweeps carry the adapted exposure." {
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
  field exposure f32 "A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are." {
    default 1
    min 0
    max 16
    tunable true
  }
  field decalAtlas Texture "The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour."
}
```

<sub>`api.ogt` line 2625</sub>

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

<sub>`api.ogt` line 2701</sub>

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
| `outline` | `boolean` | `false` | — | — | — | Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead). |
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
| `lut` | `Lut` | `0` | — | — | — | Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident). |
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
  field outline bool "Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead)."
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
  field lut Lut "Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident)."
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

<sub>`api.ogt` line 2781</sub>

<a id="record-Stylise"></a>

### `og.Stylise` (resource)

Stylisation primitives that move the frame away from the photographic: a low-resolution pixel grid scaled up by whole numbers, fewer tones or a fixed palette through an ordered or blue-noise dither, halftone dots, cross-hatching, painterly smoothing over a canvas, ink lines on tonal edges, flat colour bands by depth, and the vertex snapping, affine textures and coarse depth of early 3D hardware. They compose: a look (og.load.look) sets this with PostEffects and RenderSettings, and the game's art direction is in that data, not in the engine. Every primitive is off at its zero value (0, false, none) and then costs nothing: no pass is recorded and no pipeline is built. RenderCapabilities.stylise names the primitives this renderer draws; a field it does not name is carried and read back but not drawn. Read after each tick; it never feeds back into the simulation.

Order in the frame: depth bands colour the lit scene with the fog, before tone mapping; after tone mapping and PostEffects' grading come paint, hatching, halftone and ink, then posterize and palette quantise through the dither, all at the logical resolution, and the pixel grid is scaled up last. The UI and screen-space sprites are drawn over the scaled frame at the output's resolution, unstylised.

Token `og.Stylise`, registered under the schema name `og.Stylise`; its value type is `Stylise`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pixelHeight` | `number` | `0` | 0 … 2160 | — | — | The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number with nearest filtering, as a low-resolution three.js render target drawn with NearestFilter; 0 renders at the output's resolution. The height sets the scale (pixelUpscale); the width is the output's aspect times the height, rounded, but at most the output's width over the scale, so the last column gives way rather than the scale: 240 rows at 1280x720 are 426.67 columns, rounded to 427, and 427 x 3 = 1281 does not fit, so the grid is 426 x 240 at 3x (1278 x 720, a 1-pixel border each side). Heights that divide the output's evenly fill it: 180 (4x), 240 (3x) and 360 (2x) at 720p, 270 (4x) and 360 (3x) at 1080p. |
| `pixelUpscale` | `number` | `0` | 0 … 16 | — | — | The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height (a smaller scale than the largest leaves the border it implies: 240 lines at 2x in 1280x720 draw 854 x 480 in a 213 x 120 border), else the largest that fits; 0 takes the largest that fits (720 / 240 = 3, 1080 / 240 = 4, rounded down). A frame that does not fill the output is centred with Camera.background round it. |
| `pixelSnap` | `boolean` | `true` | — | — | — | Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets). |
| `posterize` | `number` | `0` | 0 … 64 | — | — | Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off. |
| `palette` | `Palette` | `0` | — | — | — | Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none. |
| `dither` | `DitherPattern` | `"none"` | — | — | — | How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion. |
| `ditherStrength` | `number` | `1` | 0 … 2 | — | tunable | The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands. |
| `ditherScale` | `number` | `1` | 1 … 8 | — | — | The size of one dither cell in logical pixels. |
| `halftone` | `number` | `0` | 0 … 1 | `ratio` | tunable | Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only. |
| `halftoneSize` | `number` | `6` | 2 … 64 | — | — | The dot grid's pitch in logical pixels. |
| `halftoneAngle` | `number` | `45` | — | `deg` | — | The dot grid's angle. |
| `halftoneColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The ink of the dots (linear). |
| `halftoneSpace` | `StyliseSpace` | `"screen"` | — | — | — | Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots. |
| `hatching` | `number` | `0` | 0 … 1 | `ratio` | tunable | Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground; 0 is off, 1 strokes only. |
| `hatchingScale` | `number` | `6` | 2 … 64 | — | — | The spacing of hatch lines in logical pixels. |
| `hatchingLevels` | `number` | `4` | 1 … 6 | — | — | How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6). |
| `hatchingAngle` | `number` | `45` | — | `deg` | — | The direction of the first stroke set. |
| `hatchingColor` | `Color` | `{ r: 0.04, g: 0.035, b: 0.03 }` | — | — | — | The stroke colour (linear): graphite, sepia or ink. |
| `hatchingSpace` | `StyliseSpace` | `"screen"` | — | — | — | Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes laid on the surfaces that follow the tone and the form, wrapping round curved objects and staying on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same strokes in each instance's own space, so a moving or turning object carries its strokes. |
| `hatchingDepth` | `number` | `0` | 0 … 1 | `ratio` | tunable | Hatching from luminance and depth: the depth-normal prepass's silhouettes and creases darken the tone the strokes follow, at this strength, so strokes gather along contours and folds as inkDepth inks them (the edges are inkDepthThreshold's and inkNormalThreshold's, found within one stroke spacing). 0 is the tone alone; above 0 the frame runs the prepass PostEffects.outline uses. |
| `paintRadius` | `number` | `0` | 0 … 8 | — | tunable | Painterly smoothing (a Kuwahara filter): flat regions become strokes of one colour this many logical pixels across while edges stay sharp; 0 is off. |
| `paintAnisotropy` | `number` | `0` | 0 … 1 | `ratio` | — | How far the strokes follow the image's edges: 0 square brush cells, 1 strokes stretched along every contour (an anisotropic Kuwahara). |
| `paintEdgeDarkening` | `number` | `0` | 0 … 1 | `ratio` | tunable | Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none. |
| `paintBleed` | `number` | `0` | 0 … 1 | `ratio` | tunable | Colour bleeding across edges into its neighbours, wet in wet; 0 is none. |
| `paintCanvas` | `Texture` | `0` | — | — | — | A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none. |
| `paintCanvasStrength` | `number` | `0` | 0 … 1 | `ratio` | tunable | How much the canvas texture modulates the frame: 0 none, 1 its full grain. |
| `paintCanvasScale` | `number` | `1` | 0.125 … 8 | — | — | Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky). |
| `paintCanvasSpace` | `StyliseSpace` | `"screen"` | — | — | — | Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain. |
| `paintCanvasWorldScale` | `number` | `128` | 1 … 4096 | — | — | Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres. |
| `ink` | `boolean` | `false` | — | — | — | Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases. |
| `inkThreshold` | `number` | `0.1` | 0 … 1 | — | tunable | The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines. |
| `inkWidth` | `number` | `1` | 0.5 … 8 | — | tunable | The line width in logical pixels. |
| `inkColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The line colour (linear). |
| `inkDepth` | `number` | `0` | 0 … 1 | `ratio` | tunable | With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses. |
| `inkDepthThreshold` | `number` | `0.04` | ≥ 0 | — | — | With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps. |
| `inkNormalThreshold` | `number` | `0.4` | 0 … 2 | — | — | With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds. |
| `depthBands` | `number` | `0` | 0 … 8 | — | — | Colour bands by depth (2 to 8): the scene from depthBandsNear to depthBandsFar is cut into this many equal layers, the first in depthBandsNearColor, the last in depthBandsFarColor, those between stepping evenly from one to the other in lightness, chroma and hue (never through grey), for layered-silhouette and stage looks; 0 is off. For side and three-quarter views, where the depth runs across the frame: seen from above the play area lies at one depth and falls in one band. |
| `depthBandsNear` | `number` | `5` | ≥ 0 | `m` | — | Where the first band starts. |
| `depthBandsFar` | `number` | `60` | ≥ 0 | `m` | — | Where the last band ends; the last band (the far colour) starts one band's depth before it, and everything beyond it is the far colour too. |
| `depthBandsNearColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The nearest band's colour (linear, before exposure). |
| `depthBandsFarColor` | `Color` | `{ r: 0.55, g: 0.6, b: 0.7 }` | — | — | — | The farthest band's colour (linear, before exposure). |
| `depthBandsFlatten` | `number` | `1` | 0 … 1 | `ratio` | tunable | How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two. |
| `vertexSnap` | `number` | `0` | 0 … 16 | — | — | Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off. |
| `affineTexturing` | `number` | `0` | 0 … 1 | `ratio` | — | How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off. |
| `depthBits` | `number` | `0` | 0 … 24 | — | — | Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision. |

```ogt
resource Stylise {
  doc "Stylisation primitives that move the frame away from the photographic: a low-resolution pixel grid scaled up by whole numbers, fewer tones or a fixed palette through an ordered or blue-noise dither, halftone dots, cross-hatching, painterly smoothing over a canvas, ink lines on tonal edges, flat colour bands by depth, and the vertex snapping, affine textures and coarse depth of early 3D hardware. They compose: a look (og.load.look) sets this with PostEffects and RenderSettings, and the game's art direction is in that data, not in the engine. Every primitive is off at its zero value (0, false, none) and then costs nothing: no pass is recorded and no pipeline is built. RenderCapabilities.stylise names the primitives this renderer draws; a field it does not name is carried and read back but not drawn. Read after each tick; it never feeds back into the simulation."
  doc "Order in the frame: depth bands colour the lit scene with the fog, before tone mapping; after tone mapping and PostEffects' grading come paint, hatching, halftone and ink, then posterize and palette quantise through the dither, all at the logical resolution, and the pixel grid is scaled up last. The UI and screen-space sprites are drawn over the scaled frame at the output's resolution, unstylised."
  field pixelHeight u32 "The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number with nearest filtering, as a low-resolution three.js render target drawn with NearestFilter; 0 renders at the output's resolution. The height sets the scale (pixelUpscale); the width is the output's aspect times the height, rounded, but at most the output's width over the scale, so the last column gives way rather than the scale: 240 rows at 1280x720 are 426.67 columns, rounded to 427, and 427 x 3 = 1281 does not fit, so the grid is 426 x 240 at 3x (1278 x 720, a 1-pixel border each side). Heights that divide the output's evenly fill it: 180 (4x), 240 (3x) and 360 (2x) at 720p, 270 (4x) and 360 (3x) at 1080p." {
    min 0
    max 2160
  }
  field pixelUpscale u32 "The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height (a smaller scale than the largest leaves the border it implies: 240 lines at 2x in 1280x720 draw 854 x 480 in a 213 x 120 border), else the largest that fits; 0 takes the largest that fits (720 / 240 = 3, 1080 / 240 = 4, rounded down). A frame that does not fill the output is centred with Camera.background round it." {
    min 0
    max 16
  }
  field pixelSnap bool "Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets)." {
    default true
  }
  field posterize u32 "Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off." {
    min 0
    max 64
  }
  field palette Palette "Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none."
  field dither DitherPattern "How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion."
  field ditherStrength f32 "The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands." {
    default 1
    min 0
    max 2
    tunable true
  }
  field ditherScale u32 "The size of one dither cell in logical pixels." {
    default 1
    min 1
    max 8
  }
  field halftone f32 "Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field halftoneSize f32 "The dot grid's pitch in logical pixels." {
    default 6
    min 2
    max 64
  }
  field halftoneAngle f32 "The dot grid's angle." {
    default 45
    unit deg
  }
  field halftoneColor Color "The ink of the dots (linear)." {
    default 0.02 0.02 0.03
  }
  field halftoneSpace StyliseSpace "Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots."
  field hatching f32 "Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground; 0 is off, 1 strokes only." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field hatchingScale f32 "The spacing of hatch lines in logical pixels." {
    default 6
    min 2
    max 64
  }
  field hatchingLevels u32 "How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6)." {
    default 4
    min 1
    max 6
  }
  field hatchingAngle f32 "The direction of the first stroke set." {
    default 45
    unit deg
  }
  field hatchingColor Color "The stroke colour (linear): graphite, sepia or ink." {
    default 0.04 0.035 0.03
  }
  field hatchingSpace StyliseSpace "Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes laid on the surfaces that follow the tone and the form, wrapping round curved objects and staying on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same strokes in each instance's own space, so a moving or turning object carries its strokes."
  field hatchingDepth f32 "Hatching from luminance and depth: the depth-normal prepass's silhouettes and creases darken the tone the strokes follow, at this strength, so strokes gather along contours and folds as inkDepth inks them (the edges are inkDepthThreshold's and inkNormalThreshold's, found within one stroke spacing). 0 is the tone alone; above 0 the frame runs the prepass PostEffects.outline uses." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintRadius f32 "Painterly smoothing (a Kuwahara filter): flat regions become strokes of one colour this many logical pixels across while edges stay sharp; 0 is off." {
    min 0
    max 8
    tunable true
  }
  field paintAnisotropy f32 "How far the strokes follow the image's edges: 0 square brush cells, 1 strokes stretched along every contour (an anisotropic Kuwahara)." {
    min 0
    max 1
    unit ratio
  }
  field paintEdgeDarkening f32 "Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintBleed f32 "Colour bleeding across edges into its neighbours, wet in wet; 0 is none." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintCanvas Texture "A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none."
  field paintCanvasStrength f32 "How much the canvas texture modulates the frame: 0 none, 1 its full grain." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field paintCanvasScale f32 "Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky)." {
    default 1
    min 0.125
    max 8
  }
  field paintCanvasSpace StyliseSpace "Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain."
  field paintCanvasWorldScale f32 "Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres." {
    default 128
    min 1
    max 4096
  }
  field ink bool "Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases."
  field inkThreshold f32 "The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines." {
    default 0.1
    min 0
    max 1
    tunable true
  }
  field inkWidth f32 "The line width in logical pixels." {
    default 1
    min 0.5
    max 8
    tunable true
  }
  field inkColor Color "The line colour (linear)." {
    default 0.02 0.02 0.03
  }
  field inkDepth f32 "With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field inkDepthThreshold f32 "With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps." {
    default 0.04
    min 0
  }
  field inkNormalThreshold f32 "With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds." {
    default 0.4
    min 0
    max 2
  }
  field depthBands u32 "Colour bands by depth (2 to 8): the scene from depthBandsNear to depthBandsFar is cut into this many equal layers, the first in depthBandsNearColor, the last in depthBandsFarColor, those between stepping evenly from one to the other in lightness, chroma and hue (never through grey), for layered-silhouette and stage looks; 0 is off. For side and three-quarter views, where the depth runs across the frame: seen from above the play area lies at one depth and falls in one band." {
    min 0
    max 8
  }
  field depthBandsNear f32 "Where the first band starts." {
    default 5
    min 0
    unit m
  }
  field depthBandsFar f32 "Where the last band ends; the last band (the far colour) starts one band's depth before it, and everything beyond it is the far colour too." {
    default 60
    min 0
    unit m
  }
  field depthBandsNearColor Color "The nearest band's colour (linear, before exposure)." {
    default 0.02 0.02 0.03
  }
  field depthBandsFarColor Color "The farthest band's colour (linear, before exposure)." {
    default 0.55 0.6 0.7
  }
  field depthBandsFlatten f32 "How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
  field vertexSnap f32 "Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off." {
    min 0
    max 16
  }
  field affineTexturing f32 "How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off." {
    min 0
    max 1
    unit ratio
  }
  field depthBits u32 "Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision." {
    min 0
    max 24
  }
}
```

<sub>`api.ogt` line 2999</sub>

<a id="record-MaterialStyle"></a>

### `og.MaterialStyle` (resource)

Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls off with distance, and flat facets. Each applies to a material (og.render.standardMaterial or a loaded glTF's) that leaves that field at its own default; unlit and shader materials, materials with StandardMaterialDesc.styled false, sprites and the UI are untouched, and toon passes by a material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which keeps its physical shading. All off (false, 0) is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds back into the simulation.

Token `og.MaterialStyle`, registered under the schema name `og.MaterialStyle`; its value type is `MaterialStyle`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toon` | `boolean` | `false` | — | — | — | Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway. |
| `gradientMap` | `Texture` | `0` | — | — | — | The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands. |
| `shadowTint` | `Color` | `{ r: 0.18, g: 0.16, b: 0.32 }` | — | — | — | The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration. |
| `shadowTintAmount` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far shadowed light moves towards shadowTint; 0 is off. |
| `rimColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Rim (fresnel) light for materials without their own (linear). |
| `rimIntensity` | `number` | `0` | ≥ 0 | — | tunable | The rim light's strength; 0 is none. |
| `rimPower` | `number` | `3` | ≥ 0.1 | — | — | How tight the rim is (higher: thinner). |
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The outline colour for materials without their own (linear). |
| `outlineWidth` | `number` | `0` | 0 … 31 | — | tunable | The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one, so voxel and flat-shaded models stay clean. A surface seen at a grazing angle is lined only where it ends: a 1,600 m ground under a chase camera keeps its far edge and nothing else (no band along the horizon). |
| `outlineDistance` | `number` | `0` | ≥ 0 | `m` | — | The distance at which outlines are outlineWidth wide: nearer they grow, up to four times outlineWidth, farther they thin in proportion (an inked look keeps line weight by depth); 0 keeps every outline outlineWidth wide. |
| `flatShading` | `boolean` | `false` | — | — | — | Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks. |

```ogt
resource MaterialStyle {
  doc "Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls off with distance, and flat facets. Each applies to a material (og.render.standardMaterial or a loaded glTF's) that leaves that field at its own default; unlit and shader materials, materials with StandardMaterialDesc.styled false, sprites and the UI are untouched, and toon passes by a material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which keeps its physical shading. All off (false, 0) is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds back into the simulation."
  field toon bool "Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway."
  field gradientMap Texture "The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands."
  field shadowTint Color "The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration." {
    default 0.18 0.16 0.32
  }
  field shadowTintAmount f32 "How far shadowed light moves towards shadowTint; 0 is off." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field rimColor Color "Rim (fresnel) light for materials without their own (linear)." {
    default 1 1 1
  }
  field rimIntensity f32 "The rim light's strength; 0 is none." {
    min 0
    tunable true
  }
  field rimPower f32 "How tight the rim is (higher: thinner)." {
    default 3
    min 0.1
  }
  field outlineColor Color "The outline colour for materials without their own (linear)." {
    default 0.02 0.02 0.03
  }
  field outlineWidth f32 "The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one, so voxel and flat-shaded models stay clean. A surface seen at a grazing angle is lined only where it ends: a 1,600 m ground under a chase camera keeps its far edge and nothing else (no band along the horizon)." {
    min 0
    max 31
    tunable true
  }
  field outlineDistance f32 "The distance at which outlines are outlineWidth wide: nearer they grow, up to four times outlineWidth, farther they thin in proportion (an inked look keeps line weight by depth); 0 keeps every outline outlineWidth wide." {
    min 0
    unit m
  }
  field flatShading bool "Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks."
}
```

<sub>`api.ogt` line 3192</sub>

<a id="record-Lighting2D"></a>

### `og.Lighting2D` (resource)

2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default.

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
  doc "2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default."
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

<sub>`api.ogt` line 3242</sub>

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

<sub>`api.ogt` line 3261</sub>

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

<sub>`api.ogt` line 3291</sub>

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

<sub>`api.ogt` line 3305</sub>

<a id="record-ParticleSystem"></a>

### `og.ParticleSystem` (component)

Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay, and never feed back into the simulation: compute on the full tier, the CPU on WebGL2, identical either way.

Token `og.ParticleSystem`, registered under the schema name `og.ParticleSystem`; its value type is `ParticleSystem`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `emitter` | `ParticleEmitter` | `0` | — | — | — | The emitter asset; 0 draws nothing. |
| `seed` | `number` | `0` | — | — | — | The random seed: systems with different seeds differ. |
| `startTick` | `number` | `4294967295` | — | — | — | The tick emission started: write the tick (`time.tick`, `world.tick`) to fire a one-shot burst, or to restart one. Its default, PARTICLES_ARMED, is never written: a looping emitter emits as if started at tick 0, while a one-shot (`looping false`) stays armed and emits nothing until a tick is written, so effects spawned ahead of their first use wait. |
| `stopTick` | `number` | `0` | — | — | — | The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn (a hidden system loses its particles). |

```ogt
component ParticleSystem {
  doc "Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay, and never feed back into the simulation: compute on the full tier, the CPU on WebGL2, identical either way."
  field emitter ParticleEmitter "The emitter asset; 0 draws nothing."
  field seed u32 "The random seed: systems with different seeds differ."
  field startTick u32 "The tick emission started: write the tick (`time.tick`, `world.tick`) to fire a one-shot burst, or to restart one. Its default, PARTICLES_ARMED, is never written: a looping emitter emits as if started at tick 0, while a one-shot (`looping false`) stays armed and emits nothing until a tick is written, so effects spawned ahead of their first use wait." {
    default 4294967295
  }
  field stopTick u32 "The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops."
  field visible bool "Whether it is drawn (a hidden system loses its particles)." {
    default true
  }
}
```

<sub>`api.ogt` line 3339</sub>

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

<sub>`api.ogt` line 3352</sub>

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

<sub>`api.ogt` line 3400</sub>

<a id="record-Sprite"></a>

### `og.Sprite` (component)

A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture (or an `array true` sheet's texture array layer), region and anchor come from the flipbook's current frame.

Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer.

Token `og.Sprite`, registered under the schema name `og.Sprite`; its value type is `Sprite`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The texture (og.load.texture); 0 draws a solid quad of the colour. |
| `array` | `TextureArray` | `0` | — | — | — | Draw from a layer of this texture array (og.load.textureArray) instead of `texture`; 0 uses `texture`. Every sprite of one array is one draw whichever layer each shows. While the array's streaming group is not resident the sprite draws nothing. |
| `arrayLayer` | `number` | `0` | — | — | — | With `array`: the layer drawn (past the last layer, the last). uvMin and uvMax are the region within the layer; a nine-slice measures the layer. |
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
| `dropShadow` | `number` | `0` | 0 … 1 | — | tunable | A drop shadow this dark (0 none, 0.5 typical): the sprite's shape flattened along the sun onto the ground under it (dropShadowHeight, or the og.Terrain there), faded where that ground already lies in the sun's shadow, drawn before every sprite of its layer (SpriteDesc.dropShadow): pixel sprites standing in a 3D set. |
| `dropShadowHeight` | `number` | `0` | — | `m` | — | The height of the level ground the drop shadow lies on; where an og.Terrain under the anchor is higher, the terrain (SpriteDesc.dropShadowHeight). |
| `toneMapped` | `boolean` | `true` | — | — | — | As three.js Material.toneMapped: false keeps the sprite's colours as given, outside exposure, bloom, fog and the tone curve, still placed and depth tested in the world (SpriteDesc.toneMapped): speech bubbles and markers that must not blow out at dusk. Alpha blending only. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Sprite {
  doc "A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture (or an `array true` sheet's texture array layer), region and anchor come from the flipbook's current frame."
  doc "Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer."
  field texture Texture "The texture (og.load.texture); 0 draws a solid quad of the colour."
  field array TextureArray "Draw from a layer of this texture array (og.load.textureArray) instead of `texture`; 0 uses `texture`. Every sprite of one array is one draw whichever layer each shows. While the array's streaming group is not resident the sprite draws nothing."
  field arrayLayer u32 "With `array`: the layer drawn (past the last layer, the last). uvMin and uvMax are the region within the layer; a nine-slice measures the layer."
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
  field dropShadow f32 "A drop shadow this dark (0 none, 0.5 typical): the sprite's shape flattened along the sun onto the ground under it (dropShadowHeight, or the og.Terrain there), faded where that ground already lies in the sun's shadow, drawn before every sprite of its layer (SpriteDesc.dropShadow): pixel sprites standing in a 3D set." {
    min 0
    max 1
    tunable true
  }
  field dropShadowHeight f32 "The height of the level ground the drop shadow lies on; where an og.Terrain under the anchor is higher, the terrain (SpriteDesc.dropShadowHeight)." {
    unit m
  }
  field toneMapped bool "As three.js Material.toneMapped: false keeps the sprite's colours as given, outside exposure, bloom, fog and the tone curve, still placed and depth tested in the world (SpriteDesc.toneMapped): speech bubbles and markers that must not blow out at dusk. Alpha blending only." {
    default true
  }
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>`api.ogt` line 3608</sub>

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

<sub>`api.ogt` line 3693</sub>

<a id="record-Lines"></a>

### `og.Lines` (component)

Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a track, a constellation. The vertices are taken in order (the geometry's indices are not read), with its vertex colours times color.

Token `og.Lines`, registered under the schema name `og.Lines`; its value type is `Lines`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | The vertices (og.render.bufferGeometry: an unindexed one of any count). |
| `mode` | `LineMode` | `"strip"` | — | — | — | How the vertices join. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour, times the vertex colours. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The alpha. |
| `width` | `number` | `1` | ≥ 0 | — | tunable | CSS pixels, or world units with worldUnits. |
| `worldUnits` | `boolean` | `false` | — | — | — | Width in world units. |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry. |
| `dashSize` | `number` | `0` | ≥ 0 | — | — | Dashes' length in world units along the line; 0 is solid (three.js LineDashedMaterial.dashSize). |
| `gapSize` | `number` | `1` | ≥ 0 | — | — | Gaps' length between dashes. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Lines {
  doc "Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a track, a constellation. The vertices are taken in order (the geometry's indices are not read), with its vertex colours times color."
  field geometry Geometry "The vertices (og.render.bufferGeometry: an unindexed one of any count)."
  field mode LineMode "How the vertices join."
  field color Color "The linear colour, times the vertex colours." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
    tunable true
  }
  field width f32 "CSS pixels, or world units with worldUnits." {
    default 1
    min 0
    tunable true
  }
  field worldUnits bool "Width in world units."
  field depthTest bool "Hidden behind nearer geometry." {
    default true
  }
  field dashSize f32 "Dashes' length in world units along the line; 0 is solid (three.js LineDashedMaterial.dashSize)." {
    min 0
  }
  field gapSize f32 "Gaps' length between dashes." {
    default 1
    min 0
  }
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>`api.ogt` line 4342</sub>

<a id="record-Points"></a>

### `og.Points` (component)

Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. Vertex colours times color.

Token `og.Points`, registered under the schema name `og.Points`; its value type is `Points`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `geometry` | `Geometry` | `0` | — | — | — | The vertices. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour, times the vertex colours. |
| `opacity` | `number` | `1` | 0 … 1 | — | tunable | The alpha. |
| `size` | `number` | `1` | ≥ 0 | — | tunable | World units with sizeAttenuation, else CSS pixels. |
| `sizeAttenuation` | `boolean` | `true` | — | — | — | Size in world units, smaller with distance. |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn. |

```ogt
component Points {
  doc "Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. Vertex colours times color."
  field geometry Geometry "The vertices."
  field color Color "The linear colour, times the vertex colours." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
    tunable true
  }
  field size f32 "World units with sizeAttenuation, else CSS pixels." {
    default 1
    min 0
    tunable true
  }
  field sizeAttenuation bool "Size in world units, smaller with distance." {
    default true
  }
  field depthTest bool "Hidden behind nearer geometry." {
    default true
  }
  field visible bool "Whether it is drawn." {
    default true
  }
}
```

<sub>`api.ogt` line 4376</sub>

## Types

<a id="type-Geometry"></a>

### `Geometry` (handle)

Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Geometry "Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF."
```

<sub>`api.ogt` line 2238</sub>

<a id="type-Material"></a>

### `Material` (handle)

A material instance.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Material "A material instance."
```

<sub>`api.ogt` line 2239</sub>

<a id="type-Texture"></a>

### `Texture` (handle)

A cooked texture.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Texture "A cooked texture."
```

<sub>`api.ogt` line 2240</sub>

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

<sub>`api.ogt` line 2256</sub>

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
| `"none"` | No renderer (a sandbox without a GPU; natively also OG_RENDERER=none): the simulation, the dumps and the world hash are the same as with one, a screenshot step writes no PNG and reports its frame unchecked, and capture answers null. |

```ogt
enum RenderTier {
  doc "The backend the renderer runs on; the probe's hello reports it."
  variant webgpu "WebGPU in a browser: the full feature set."
  variant webgl2 "The WebGL2 fallback: no compute, CPU culling and particles, tiled lights."
  variant vulkan "Native Vulkan on a GPU."
  variant vulkan-cpu "Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU."
  variant metal "Native Metal."
  variant none "No renderer (a sandbox without a GPU; natively also OG_RENDERER=none): the simulation, the dumps and the world hash are the same as with one, a screenshot step writes no PNG and reports its frame unchecked, and capture answers null."
}
```

<sub>`api.ogt` line 2263</sub>

<a id="type-Antialias"></a>

### `Antialias` (enum)

Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved.

A string: one of the variants below. The zero value is `"tier"`.

| Variant | Doc |
|---|---|
| `"tier"` | The tier's method: TAA on the full tier (WebGPU, Metal, Vulkan), 4x MSAA on WebGL2, FXAA where the HDR format cannot be multisampled. |
| `"off"` | One sample per pixel. |
| `"msaa4"` | 4x MSAA on any tier: sharp sprites and text, no temporal blur; FXAA where the HDR format cannot be multisampled. |
| `"taa"` | Temporal antialiasing on any tier: a sub-pixel jitter per frame and a reprojected history; converges over about eight frames. RenderOptions.taaHistory and taaSharpen refine it. |
| `"fxaa"` | FXAA on any tier: one full-screen pass over the tonemapped image (the overlay stays sharp), no extra samples and nothing to converge; softer than MSAA on fine texture. |

```ogt
enum Antialias {
  doc "Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved."
  variant tier "The tier's method: TAA on the full tier (WebGPU, Metal, Vulkan), 4x MSAA on WebGL2, FXAA where the HDR format cannot be multisampled."
  variant off "One sample per pixel."
  variant msaa4 "4x MSAA on any tier: sharp sprites and text, no temporal blur; FXAA where the HDR format cannot be multisampled."
  variant taa "Temporal antialiasing on any tier: a sub-pixel jitter per frame and a reprojected history; converges over about eight frames. RenderOptions.taaHistory and taaSharpen refine it."
  variant fxaa "FXAA on any tier: one full-screen pass over the tonemapped image (the overlay stays sharp), no extra samples and nothing to converge; softer than MSAA on fine texture."
}
```

<sub>`api.ogt` line 2273</sub>

<a id="type-PointShadows"></a>

### `PointShadows` (enum)

How a point light's shadow map is stored, as og-render's PointShadows. RenderCapabilities.pointShadows reports what the device resolved.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Cube faces on the full tier, paraboloids on WebGL2. |
| `"cube"` | Six 90 degree faces: six shadow passes per light. |
| `"paraboloid"` | Two paraboloid hemispheres (below and above the light): two passes per light instead of six, coarser, and large casters' silhouettes approximate (their edges stay straight in the curved projection). |

```ogt
enum PointShadows {
  doc "How a point light's shadow map is stored, as og-render's PointShadows. RenderCapabilities.pointShadows reports what the device resolved."
  variant auto "Cube faces on the full tier, paraboloids on WebGL2."
  variant cube "Six 90 degree faces: six shadow passes per light."
  variant paraboloid "Two paraboloid hemispheres (below and above the light): two passes per light instead of six, coarser, and large casters' silhouettes approximate (their edges stay straight in the curved projection)."
}
```

<sub>`api.ogt` line 2282</sub>

<a id="type-TaaHistory"></a>

### `TaaHistory` (enum)

How TAA resamples its history each frame, as og-render's TaaHistory.

A string: one of the variants below. The zero value is `"bilinear"`.

| Variant | Doc |
|---|---|
| `"bilinear"` | One bilinear tap: cheap, slightly soft while the camera moves. |
| `"catmullRom"` | A Catmull-Rom filter over 4x4 texels (9 taps): the history stays sharp under camera motion. |

```ogt
enum TaaHistory {
  doc "How TAA resamples its history each frame, as og-render's TaaHistory."
  variant bilinear "One bilinear tap: cheap, slightly soft while the camera moves."
  variant catmullRom "A Catmull-Rom filter over 4x4 texels (9 taps): the history stays sharp under camera motion."
}
```

<sub>`api.ogt` line 2289</sub>

<a id="type-DebugView"></a>

### `DebugView` (enum)

What the debug overlay shows besides og.debug's draws (og.debug.setView, RenderOptions.debugView, `debug:<view>`), drawn over the finished frame. Streams add views as they add what to show.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Only og.debug's draws. |
| `"physics"` | The colliders' shapes, joints and contacts as the last physics step left them. |
| `"nav"` | The navigation grids (blocked cells) and navmeshes (polygon edges), and every og.NavAgent's path. |
| `"bounds"` | Every drawn mesh's bounds (its geometry's box at its transform). |

```ogt
enum DebugView {
  doc "What the debug overlay shows besides og.debug's draws (og.debug.setView, RenderOptions.debugView, `debug:<view>`), drawn over the finished frame. Streams add views as they add what to show."
  variant none "Only og.debug's draws."
  variant physics "The colliders' shapes, joints and contacts as the last physics step left them."
  variant nav "The navigation grids (blocked cells) and navmeshes (polygon edges), and every og.NavAgent's path."
  variant bounds "Every drawn mesh's bounds (its geometry's box at its transform)."
}
```

<sub>`api.ogt` line 2295</sub>

<a id="type-RenderOptions"></a>

### `RenderOptions` (desc struct)

How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing method. |
| `shadowMapSize` | `number` | `2048` | 256 … 4096 | — | — | The side of each of the directional light's three cascade maps, in texels; smaller saves memory on phones (three 2048 maps are 64 MB). |
| `pointShadows` | `PointShadows` | `"auto"` | — | — | — | How point lights' shadows are stored: cube faces, or two paraboloid hemispheres (fewer passes, coarser). |
| `taaHistory` | `TaaHistory` | `"bilinear"` | — | — | — | With TAA: how the history is resampled. |
| `taaSharpen` | `number` | `0` | 0 … 1 | `ratio` | — | With TAA: a sharpening pass after it, 0 (none) to 1; it never pushes a pixel past its neighbours' range. |
| `debugView` | `DebugView` | `"none"` | — | — | — | A debug overlay from the first frame (og.debug.setView, the command channel's `debug:<view>`): none in a game, a view while building one. Not in publish builds. |

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
  field pointShadows PointShadows "How point lights' shadows are stored: cube faces, or two paraboloid hemispheres (fewer passes, coarser)."
  field taaHistory TaaHistory "With TAA: how the history is resampled."
  field taaSharpen f32 "With TAA: a sharpening pass after it, 0 (none) to 1; it never pushes a pixel past its neighbours' range." {
    min 0
    max 1
    unit ratio
  }
  field debugView DebugView "A debug overlay from the first frame (og.debug.setView, the command channel's `debug:<view>`): none in a game, a view while building one. Not in publish builds."
}
```

<sub>`api.ogt` line 2303</sub>

<a id="type-StandardMaterialDesc"></a>

### `StandardMaterialDesc` (desc struct)

A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names.

It is og-render's reflected StandardMaterial field for field: metalnessMap and roughnessMap read a glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; alphaTest above 0 discards below that alpha; transparent blends, sorted back to front.

The stylised fields: unlit (three.js MeshBasicMaterial), toon with its gradientMap ramp, a rim (fresnel) light, a hit flash, dissolve with a glowing edge, and a per-material outline. All are off at their defaults. og.MaterialStyle fills the ones a material leaves at their defaults unless the material sets styled false.

The MeshPhysicalMaterial fields, under three.js's names and glTF's KHR_materials_* meanings, all off at their defaults: clearcoat with clearcoatMap, clearcoatRoughnessMap and clearcoatNormalMap; sheen (velvet, cloth) with sheenColor, sheenRoughness and their maps; transmission (glass, water) refracting the opaque scene behind through thickness, attenuationColor and ior; iridescence (soap film, oil) with its film's iridescenceIOR and iridescenceThicknessRange; anisotropy (brushed metal) along anisotropyRotation or anisotropyMap. They draw on every tier, WebGL2 included; unlit and toon materials ignore them.

A transmissive material is drawn after the opaque scene, back to front with the transparent ones, and refracts what was drawn before it (not other transmissive or transparent objects); thickness is in the mesh's own units, scaled with it.

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
| `outlineWidth` | `number` | `0` | 0 … 31 | — | — | This material's outline width in pixels; 0 is none (PostEffects.outline outlines everything). A hull outline, as toon shading draws one: the object's silhouette, where one part of it passes in front of another, and its creases against other objects (where it meets the ground or a wall), never the edges between its own faces, so a .vox model or any flat-shaded, hard-edged mesh keeps clean faces inside its silhouette; a face seen at a grazing angle (a box's side nearly edge-on, a large ground plane) is not lined across, only where it ends. Creases inside an object are Stylise.inkDepth's. |
| `styled` | `boolean` | `true` | — | — | — | Whether og.MaterialStyle (a look's scene-wide toon, rim, outline, flat facets, shadow tint and outline distance) reaches this material. false keeps it exactly as declared under any look: a lantern's lacquer, a UI-like prop, a hero kept photographic in a toon scene. A material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, an ior other than 1.5) keeps its physical shading under the style's toon even when styled (toon would drop those lobes) and takes the rest of the style; its own toon true still makes it toon. |
| `clearcoat` | `number` | `0` | 0 … 1 | `ratio` | — | A clear lacquer over the surface, as three.js MeshPhysicalMaterial.clearcoat: a second, 4 % reflective specular layer on the surface's normal before normalMap (car paint, varnished wood, wet plastic); 0 is none. |
| `clearcoatRoughness` | `number` | `0` | 0 … 1 | `ratio` | — | How rough the clear coat is, as three.js MeshPhysicalMaterial.clearcoatRoughness. |
| `clearcoatMap` | `Texture \| null` | `null` | — | — | — | The coat's strength from the texture's red channel, times clearcoat (MeshPhysicalMaterial.clearcoatMap). |
| `clearcoatRoughnessMap` | `Texture \| null` | `null` | — | — | — | The coat's roughness from the texture's green channel, times clearcoatRoughness. |
| `clearcoatNormalMap` | `Texture \| null` | `null` | — | — | — | A tangent-space normal map of the coat alone; the base keeps normalMap (MeshPhysicalMaterial.clearcoatNormalMap). |
| `clearcoatNormalScale` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How strongly clearcoatNormalMap bends the coat's normal, per tangent axis. |
| `sheen` | `number` | `0` | 0 … 1 | `ratio` | — | A velvet-like layer that brightens grazing angles (cloth, fuzz, peach skin), as MeshPhysicalMaterial.sheen: its strength, times sheenColor; 0 is none. |
| `sheenColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The sheen's colour (linear), times sheenColorMap; black is none. |
| `sheenRoughness` | `number` | `1` | 0 … 1 | `ratio` | — | How rough the sheen is: low keeps it to a thin rim, high spreads it over the surface. |
| `sheenColorMap` | `Texture \| null` | `null` | — | — | — | The sheen's colour from the texture's RGB (sRGB). |
| `sheenRoughnessMap` | `Texture \| null` | `null` | — | — | — | The sheen's roughness from the texture's alpha channel. |
| `transmission` | `number` | `0` | 0 … 1 | `ratio` | — | How much light passes through, as MeshPhysicalMaterial.transmission (glass, water, thin plastic): the opaque scene behind is refracted and blurred by roughness; 0 is none. Unlike opacity, the surface keeps its reflections. |
| `transmissionMap` | `Texture \| null` | `null` | — | — | — | Transmission from the texture's red channel. |
| `thickness` | `number` | `0` | ≥ 0 | — | — | The volume's thickness under the surface in the mesh's own units (glTF KHR_materials_volume): how far refraction bends the view and how much attenuationColor absorbs; 0 is a thin wall. |
| `thicknessMap` | `Texture \| null` | `null` | — | — | — | Thickness from the texture's green channel. |
| `attenuationDistance` | `number` | `0` | ≥ 0 | — | — | The distance light travels in the volume before it is tinted to attenuationColor (world units); 0 is no absorption, three.js's Infinity. |
| `attenuationColor` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The colour white light becomes after attenuationDistance in the volume (linear). |
| `ior` | `number` | `1.5` | 1 … 2.333 | — | — | The index of refraction (glTF KHR_materials_ior): the reflectance of a dielectric, ((ior - 1) / (ior + 1))^2 (4 % at 1.5), and how far transmission refracts. |
| `iridescence` | `number` | `0` | 0 … 1 | `ratio` | — | A thin-film interference layer, as MeshPhysicalMaterial.iridescence (soap bubbles, oil, beetle shells); 0 is none. |
| `iridescenceIOR` | `number` | `1.3` | 1 … 2.333 | — | — | The film's index of refraction. |
| `iridescenceThicknessRange` | `Vector2` | `{ x: 100, y: 400 }` | — | — | — | The film's thickness in nanometres: x where iridescenceThicknessMap is black, y where it is white and everywhere without the map (three.js's [100, 400]). |
| `iridescenceMap` | `Texture \| null` | `null` | — | — | — | Iridescence from the texture's red channel. |
| `iridescenceThicknessMap` | `Texture \| null` | `null` | — | — | — | The film's thickness between iridescenceThicknessRange's ends from the texture's green channel. |
| `anisotropy` | `number` | `0` | 0 … 1 | `ratio` | — | Stretches the specular highlight along a direction of the surface, as MeshPhysicalMaterial.anisotropy (brushed metal, hair, satin); 0 is none, 1 fully stretched. Needs the geometry's tangents (generated when it has none). |
| `anisotropyRotation` | `number` | `0` | — | — | — | The stretch's direction in radians from the tangent (+u) towards the bitangent. |
| `anisotropyMap` | `Texture \| null` | `null` | — | — | — | The stretch's direction in red and green (-1 to 1 in the tangent frame, turned by anisotropyRotation) and its strength in blue, times anisotropy. |
| `mapArray` | `TextureArray \| null` | `null` | — | — | — | The base colour from layer mapLayer of a texture array (og.load.textureArray) instead of map: variants of one look (tiles, skins, damage states) are one material, each Mesh picking its layer with Mesh.materialLayer, and every mesh of the material is one draw whichever layer it shows. A material with mapArray or normalMapArray reads its base colour and normal map from arrays only (map and normalMap are not read); it draws on every tier, WebGL2 included. Physical materials (the MeshPhysicalMaterial fields) and shader materials do not take arrays. |
| `mapLayer` | `number` | `0` | — | — | — | The layer of mapArray drawn where the Mesh does not pick one (past the array's last layer: the last). |
| `normalMapArray` | `TextureArray \| null` | `null` | — | — | — | A tangent-space normal map from layer normalMapLayer of a texture array (linear: cook it with `kind normal`), the counterpart of mapArray. |
| `normalMapLayer` | `number` | `0` | — | — | — | The layer of normalMapArray read where the Mesh does not pick one. |
| `wireframe` | `boolean` | `false` | — | — | — | Draw the edges of the mesh's triangles instead of its faces, as three.js Material.wireframe: one CSS pixel wide, in color plus the emissive light, unlit, depth tested; on every tier (lines, not a polygon mode WebGL2 lacks). Its meshes cast no shadow. |

```ogt
struct StandardMaterialDesc {
  doc "A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names."
  doc "It is og-render's reflected StandardMaterial field for field: metalnessMap and roughnessMap read a glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; alphaTest above 0 discards below that alpha; transparent blends, sorted back to front."
  doc "The stylised fields: unlit (three.js MeshBasicMaterial), toon with its gradientMap ramp, a rim (fresnel) light, a hit flash, dissolve with a glowing edge, and a per-material outline. All are off at their defaults. og.MaterialStyle fills the ones a material leaves at their defaults unless the material sets styled false."
  doc "The MeshPhysicalMaterial fields, under three.js's names and glTF's KHR_materials_* meanings, all off at their defaults: clearcoat with clearcoatMap, clearcoatRoughnessMap and clearcoatNormalMap; sheen (velvet, cloth) with sheenColor, sheenRoughness and their maps; transmission (glass, water) refracting the opaque scene behind through thickness, attenuationColor and ior; iridescence (soap film, oil) with its film's iridescenceIOR and iridescenceThicknessRange; anisotropy (brushed metal) along anisotropyRotation or anisotropyMap. They draw on every tier, WebGL2 included; unlit and toon materials ignore them."
  doc "A transmissive material is drawn after the opaque scene, back to front with the transparent ones, and refracts what was drawn before it (not other transmissive or transparent objects); thickness is in the mesh's own units, scaled with it."
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
  field outlineWidth f32 "This material's outline width in pixels; 0 is none (PostEffects.outline outlines everything). A hull outline, as toon shading draws one: the object's silhouette, where one part of it passes in front of another, and its creases against other objects (where it meets the ground or a wall), never the edges between its own faces, so a .vox model or any flat-shaded, hard-edged mesh keeps clean faces inside its silhouette; a face seen at a grazing angle (a box's side nearly edge-on, a large ground plane) is not lined across, only where it ends. Creases inside an object are Stylise.inkDepth's." {
    min 0
    max 31
  }
  field styled bool "Whether og.MaterialStyle (a look's scene-wide toon, rim, outline, flat facets, shadow tint and outline distance) reaches this material. false keeps it exactly as declared under any look: a lantern's lacquer, a UI-like prop, a hero kept photographic in a toon scene. A material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, an ior other than 1.5) keeps its physical shading under the style's toon even when styled (toon would drop those lobes) and takes the rest of the style; its own toon true still makes it toon." {
    default true
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
  field clearcoatMap Option.Texture "The coat's strength from the texture's red channel, times clearcoat (MeshPhysicalMaterial.clearcoatMap)."
  field clearcoatRoughnessMap Option.Texture "The coat's roughness from the texture's green channel, times clearcoatRoughness."
  field clearcoatNormalMap Option.Texture "A tangent-space normal map of the coat alone; the base keeps normalMap (MeshPhysicalMaterial.clearcoatNormalMap)."
  field clearcoatNormalScale Vector2 "How strongly clearcoatNormalMap bends the coat's normal, per tangent axis." {
    default 1 1
  }
  field sheen f32 "A velvet-like layer that brightens grazing angles (cloth, fuzz, peach skin), as MeshPhysicalMaterial.sheen: its strength, times sheenColor; 0 is none." {
    min 0
    max 1
    unit ratio
  }
  field sheenColor Color "The sheen's colour (linear), times sheenColorMap; black is none."
  field sheenRoughness f32 "How rough the sheen is: low keeps it to a thin rim, high spreads it over the surface." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field sheenColorMap Option.Texture "The sheen's colour from the texture's RGB (sRGB)."
  field sheenRoughnessMap Option.Texture "The sheen's roughness from the texture's alpha channel."
  field transmission f32 "How much light passes through, as MeshPhysicalMaterial.transmission (glass, water, thin plastic): the opaque scene behind is refracted and blurred by roughness; 0 is none. Unlike opacity, the surface keeps its reflections." {
    min 0
    max 1
    unit ratio
  }
  field transmissionMap Option.Texture "Transmission from the texture's red channel."
  field thickness f32 "The volume's thickness under the surface in the mesh's own units (glTF KHR_materials_volume): how far refraction bends the view and how much attenuationColor absorbs; 0 is a thin wall." {
    min 0
  }
  field thicknessMap Option.Texture "Thickness from the texture's green channel."
  field attenuationDistance f32 "The distance light travels in the volume before it is tinted to attenuationColor (world units); 0 is no absorption, three.js's Infinity." {
    min 0
  }
  field attenuationColor Color "The colour white light becomes after attenuationDistance in the volume (linear)." {
    default 1 1 1
  }
  field ior f32 "The index of refraction (glTF KHR_materials_ior): the reflectance of a dielectric, ((ior - 1) / (ior + 1))^2 (4 % at 1.5), and how far transmission refracts." {
    default 1.5
    min 1
    max 2.333
  }
  field iridescence f32 "A thin-film interference layer, as MeshPhysicalMaterial.iridescence (soap bubbles, oil, beetle shells); 0 is none." {
    min 0
    max 1
    unit ratio
  }
  field iridescenceIOR f32 "The film's index of refraction." {
    default 1.3
    min 1
    max 2.333
  }
  field iridescenceThicknessRange Vector2 "The film's thickness in nanometres: x where iridescenceThicknessMap is black, y where it is white and everywhere without the map (three.js's [100, 400])." {
    default 100 400
  }
  field iridescenceMap Option.Texture "Iridescence from the texture's red channel."
  field iridescenceThicknessMap Option.Texture "The film's thickness between iridescenceThicknessRange's ends from the texture's green channel."
  field anisotropy f32 "Stretches the specular highlight along a direction of the surface, as MeshPhysicalMaterial.anisotropy (brushed metal, hair, satin); 0 is none, 1 fully stretched. Needs the geometry's tangents (generated when it has none)." {
    min 0
    max 1
    unit ratio
  }
  field anisotropyRotation f32 "The stretch's direction in radians from the tangent (+u) towards the bitangent."
  field anisotropyMap Option.Texture "The stretch's direction in red and green (-1 to 1 in the tangent frame, turned by anisotropyRotation) and its strength in blue, times anisotropy."
  field mapArray Option.TextureArray "The base colour from layer mapLayer of a texture array (og.load.textureArray) instead of map: variants of one look (tiles, skins, damage states) are one material, each Mesh picking its layer with Mesh.materialLayer, and every mesh of the material is one draw whichever layer it shows. A material with mapArray or normalMapArray reads its base colour and normal map from arrays only (map and normalMap are not read); it draws on every tier, WebGL2 included. Physical materials (the MeshPhysicalMaterial fields) and shader materials do not take arrays."
  field mapLayer u32 "The layer of mapArray drawn where the Mesh does not pick one (past the array's last layer: the last)."
  field normalMapArray Option.TextureArray "A tangent-space normal map from layer normalMapLayer of a texture array (linear: cook it with `kind normal`), the counterpart of mapArray."
  field normalMapLayer u32 "The layer of normalMapArray read where the Mesh does not pick one."
  field wireframe bool "Draw the edges of the mesh's triangles instead of its faces, as three.js Material.wireframe: one CSS pixel wide, in color plus the emissive light, unlit, depth tested; on every tier (lines, not a polygon mode WebGL2 lacks). Its meshes cast no shadow."
}
```

<sub>`api.ogt` line 2322</sub>

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

<sub>`api.ogt` line 2611</sub>

<a id="type-Lut"></a>

### `Lut` (handle)

A colour lookup table for PostEffects.lut, from og.load.lut (a `.cube` file).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Lut "A colour lookup table for PostEffects.lut, from og.load.lut (a `.cube` file)."
```

<sub>`api.ogt` line 2676</sub>

<a id="type-EnvironmentMap"></a>

### `EnvironmentMap` (handle)

A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`, a KTX2 cube map, or six face images).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle EnvironmentMap "A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`, a KTX2 cube map, or six face images)."
```

<sub>`api.ogt` line 2677</sub>

<a id="type-ParticleEmitter"></a>

### `ParticleEmitter` (handle)

A particle emitter asset for ParticleSystem.emitter, from og.load.emitter (an `.ogt` of `type particles`).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle ParticleEmitter "A particle emitter asset for ParticleSystem.emitter, from og.load.emitter (an `.ogt` of `type particles`)."
```

<sub>`api.ogt` line 2678</sub>

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

<sub>`api.ogt` line 2728</sub>

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

<sub>`api.ogt` line 2742</sub>

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

<sub>`api.ogt` line 2755</sub>

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

<sub>`api.ogt` line 2768</sub>

<a id="type-Palette"></a>

### `Palette` (handle)

A colour palette for Stylise.palette, from og.load.palette (a `.hex` or GIMP `.gpl` file of up to 256 colours).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Palette "A colour palette for Stylise.palette, from og.load.palette (a `.hex` or GIMP `.gpl` file of up to 256 colours)."
```

<sub>`api.ogt` line 2958</sub>

<a id="type-Look"></a>

### `Look` (handle)

A look from og.load.look (an `.ogt` of `type look`): the frame's whole art direction as data, applied with og.render.applyLook.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Look "A look from og.load.look (an `.ogt` of `type look`): the frame's whole art direction as data, applied with og.render.applyLook."
```

<sub>`api.ogt` line 2959</sub>

<a id="type-DitherPattern"></a>

### `DitherPattern` (enum)

The threshold pattern Stylise.dither spreads the steps of posterize and palette with.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No dither: every pixel rounds to its nearest step or colour, in flat bands. |
| `"bayer2"` | Ordered dither by a 2 by 2 Bayer matrix: the coarsest checker, as on early handhelds. |
| `"bayer4"` | Ordered dither by a 4 by 4 Bayer matrix: the cross-hatched ramps of 8- and 16-bit art. |
| `"bayer8"` | Ordered dither by an 8 by 8 Bayer matrix: finer ramps with 64 levels between two colours. |
| `"blueNoise"` | Blue noise: an even grain without a visible grid (a 64 by 64 void-and-cluster tile the engine ships, repeated), fixed per pixel so a still frame does not crawl. |
| `"interleavedGradient"` | Interleaved gradient noise: computed per pixel without a texture; a finer diagonal structure than blueNoise up close. |

```ogt
enum DitherPattern {
  doc "The threshold pattern Stylise.dither spreads the steps of posterize and palette with."
  variant none "No dither: every pixel rounds to its nearest step or colour, in flat bands."
  variant bayer2 "Ordered dither by a 2 by 2 Bayer matrix: the coarsest checker, as on early handhelds."
  variant bayer4 "Ordered dither by a 4 by 4 Bayer matrix: the cross-hatched ramps of 8- and 16-bit art."
  variant bayer8 "Ordered dither by an 8 by 8 Bayer matrix: finer ramps with 64 levels between two colours."
  variant blueNoise "Blue noise: an even grain without a visible grid (a 64 by 64 void-and-cluster tile the engine ships, repeated), fixed per pixel so a still frame does not crawl."
  variant interleavedGradient "Interleaved gradient noise: computed per pixel without a texture; a finer diagonal structure than blueNoise up close."
}
```

<sub>`api.ogt` line 2968</sub>

<a id="type-StyliseSpace"></a>

### `StyliseSpace` (enum)

Where Stylise lays a pattern (the paint canvas, the hatching, the halftone).

A string: one of the variants below. The zero value is `"screen"`.

| Variant | Doc |
|---|---|
| `"screen"` | Over the frame in logical pixels, as a print or a sheet of paper over the screen: the pattern stays put while the camera moves and the scene slides under it. |
| `"surface"` | On the surfaces, through the depth-normal prepass's position and normal (three world planes blended by the normal): the pattern sticks to the scene as the camera moves and wraps round forms, a tonal art map's strokes on the objects. Hatching and halftone keep about their pixel spacing by fading between whole octaves of world spacing; the sky keeps the screen pattern. The frame runs the prepass PostEffects.outline uses. |
| `"object"` | On the surfaces as surface, but in each mesh instance's own space (three planes of the instance blended by its normal, in metres along its axes): a moving, turning or skinned object carries its strokes, dots and grain instead of sliding through them, and keeps them however it is scaled. Terrain, scattered copies and the vertices a material shader moves keep the world pattern, the sky the screen pattern. The frame runs the prepass and an object pass that draws the opaque instances again. |

```ogt
enum StyliseSpace {
  doc "Where Stylise lays a pattern (the paint canvas, the hatching, the halftone)."
  variant screen "Over the frame in logical pixels, as a print or a sheet of paper over the screen: the pattern stays put while the camera moves and the scene slides under it."
  variant surface "On the surfaces, through the depth-normal prepass's position and normal (three world planes blended by the normal): the pattern sticks to the scene as the camera moves and wraps round forms, a tonal art map's strokes on the objects. Hatching and halftone keep about their pixel spacing by fading between whole octaves of world spacing; the sky keeps the screen pattern. The frame runs the prepass PostEffects.outline uses."
  variant object "On the surfaces as surface, but in each mesh instance's own space (three planes of the instance blended by its normal, in metres along its axes): a moving, turning or skinned object carries its strokes, dots and grain instead of sliding through them, and keeps them however it is scaled. Terrain, scattered copies and the vertices a material shader moves keep the world pattern, the sky the screen pattern. The frame runs the prepass and an object pass that draws the opaque instances again."
}
```

<sub>`api.ogt` line 2985</sub>

<a id="type-LookInfo"></a>

### `LookInfo` (struct)

A loaded look: where it came from and what applying it sets.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `look` | `Look` | `0` | — | — | — | Its handle. |
| `path` | `string` | `""` | — | — | — | Its asset id. |
| `name` | `string` | `""` | — | — | — | Its name (the file's `name`, else its file name without `.ogt`). |
| `extends` | `readonly string[]` | `[]` | — | — | — | The looks it builds on, nearest first: asset ids under assets/ and built-in preset slugs. |
| `resources` | `readonly string[]` | `[]` | — | — | — | The resources it sets, by IDL name (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment). Applying it replaces each of them whole: the fields the look names take its values, the rest their defaults. |
| `style` | `Style \| null` | `null` | — | — | — | The UI style its `ui` line names (a game's own or a built-in by slug), loaded with it; applying the look configures the UI with it. |
| `antialias` | `Antialias \| null` | `null` | — | — | — | The antialiasing it asks for. GameDesc.look's is the renderer's when GameDesc.render leaves antialias out; a running renderer keeps its method. |

```ogt
struct LookInfo {
  doc "A loaded look: where it came from and what applying it sets."
  field look Look "Its handle."
  field path String "Its asset id."
  field name String "Its name (the file's `name`, else its file name without `.ogt`)."
  field extends List.String "The looks it builds on, nearest first: asset ids under assets/ and built-in preset slugs."
  field resources List.String "The resources it sets, by IDL name (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment). Applying it replaces each of them whole: the fields the look names take its values, the rest their defaults."
  field style Option.Style "The UI style its `ui` line names (a game's own or a built-in by slug), loaded with it; applying the look configures the UI with it."
  field antialias Option.Antialias "The antialiasing it asks for. GameDesc.look's is the renderer's when GameDesc.render leaves antialias out; a running renderer keeps its method."
}
```

<sub>`api.ogt` line 3231</sub>

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

<sub>`api.ogt` line 3440</sub>

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

<sub>`api.ogt` line 3461</sub>

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
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame: position is the first baseline's origin in CSS pixels from the top left, y down, size in CSS pixels, rotation clockwise. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it. |
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
  field screen bool "Draw in screen space over the finished frame: position is the first baseline's origin in CSS pixels from the top left, y down, size in CSS pixels, rotation clockwise. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it."
  field clip Option.Array.f32.4 "Screen text: cut to [x, y, width, height] in CSS pixels, per pixel; null draws it whole."
}
```

<sub>`api.ogt` line 3480</sub>

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

<sub>`api.ogt` line 3514</sub>

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

<sub>`api.ogt` line 3525</sub>

<a id="type-SpriteDesc"></a>

### `SpriteDesc` (desc struct)

A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The texture from og.load.texture; null draws a solid quad of the colour. |
| `array` | `TextureArray \| null` | `null` | — | — | — | Draw from a layer of this texture array (og.load.textureArray) instead of `texture`: every sprite of one array is one draw whichever layer each shows, so a flipbook's frames packed as layers batch across frames. World and screen sprites alike, with flash, glow, blend and 2D lighting as a textured sprite. null uses `texture`. |
| `arrayLayer` | `number` | `0` | — | — | — | With `array`: the layer drawn, 0 to the array's layers minus 1 (refused past it). `uv` is the region within the layer. |
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
| `dropShadow` | `number` | `0` | 0 … 1 | — | — | A drop shadow this dark (0 none, 0.5 typical): the sprite's shape in black, flattened along the sun (the first og.DirectionalLight) onto the ground under it (dropShadowHeight, or the og.Terrain there), drawn before every sprite of its layer: pixel sprites standing in a 3D set cast shadows that agree with it. Where the ground already lies in the sun's cascaded shadow the drop shadow fades, so a shadow inside a shadow is no darker. None with the sun below the horizon; at most five times the sprite's height long under a low one. World sprites with a perspective camera over the xz ground; a 2D camera sees the ground edge on. |
| `dropShadowHeight` | `number` | `0` | — | `m` | — | The height of the level ground the drop shadow lies on (a floor, a deck, a step). Where an og.Terrain under the sprite's anchor is higher, the shadow lies on the terrain instead, on the plane fitted to its slope under the shadow, or bent over a hollow or a ridge narrower than the shadow (the terrain's heights under a 4 x 2 grid of the shadow's cells). 0 with no terrain is the ground plane y = 0. |
| `toneMapped` | `boolean` | `true` | — | — | — | As three.js Material.toneMapped: true draws the sprite into the lit scene, carried by exposure (auto exposure included), bloom and the tone curve like any surface. false keeps its colours as given: still placed in the world, depth tested and sorted with the other sprites, but shown unexposed and uncurved (clamped to 0 to 1), outside bloom and fog; the grade, vignette, grain and screen flash still apply. For world UI that must read the same at noon and at dusk: speech bubbles, name tags, markers. Alpha blending only: an additive sprite adds light and is always tone mapped. |
| `flipX` | `boolean` | `false` | — | — | — | Mirror the region left to right. |
| `flipY` | `boolean` | `false` | — | — | — | Mirror the region top to bottom. |
| `sortY` | `boolean` | `false` | — | — | — | Within its layer, draw by y: lower on the screen draws in front (top-down games). |
| `slice` | `[number, number, number, number] \| null` | `null` | — | — | — | Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole. |
| `sliceScale` | `number` | `0.0625` | ≥ 0 | — | — | Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame instead of the world: position x, y in CSS pixels from the top left with y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested (a HUD). flash, glow, blend and castShadow apply to world sprites only. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it. |
| `clip` | `[number, number, number, number] \| null` | `null` | — | — | — | Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole. |

```ogt
struct SpriteDesc {
  doc "A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene."
  desc true
  field texture Option.Texture "The texture from og.load.texture; null draws a solid quad of the colour."
  field array Option.TextureArray "Draw from a layer of this texture array (og.load.textureArray) instead of `texture`: every sprite of one array is one draw whichever layer each shows, so a flipbook's frames packed as layers batch across frames. World and screen sprites alike, with flash, glow, blend and 2D lighting as a textured sprite. null uses `texture`."
  field arrayLayer u32 "With `array`: the layer drawn, 0 to the array's layers minus 1 (refused past it). `uv` is the region within the layer."
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
  field dropShadow f32 "A drop shadow this dark (0 none, 0.5 typical): the sprite's shape in black, flattened along the sun (the first og.DirectionalLight) onto the ground under it (dropShadowHeight, or the og.Terrain there), drawn before every sprite of its layer: pixel sprites standing in a 3D set cast shadows that agree with it. Where the ground already lies in the sun's cascaded shadow the drop shadow fades, so a shadow inside a shadow is no darker. None with the sun below the horizon; at most five times the sprite's height long under a low one. World sprites with a perspective camera over the xz ground; a 2D camera sees the ground edge on." {
    min 0
    max 1
  }
  field dropShadowHeight f32 "The height of the level ground the drop shadow lies on (a floor, a deck, a step). Where an og.Terrain under the sprite's anchor is higher, the shadow lies on the terrain instead, on the plane fitted to its slope under the shadow, or bent over a hollow or a ridge narrower than the shadow (the terrain's heights under a 4 x 2 grid of the shadow's cells). 0 with no terrain is the ground plane y = 0." {
    unit m
  }
  field toneMapped bool "As three.js Material.toneMapped: true draws the sprite into the lit scene, carried by exposure (auto exposure included), bloom and the tone curve like any surface. false keeps its colours as given: still placed in the world, depth tested and sorted with the other sprites, but shown unexposed and uncurved (clamped to 0 to 1), outside bloom and fog; the grade, vignette, grain and screen flash still apply. For world UI that must read the same at noon and at dusk: speech bubbles, name tags, markers. Alpha blending only: an additive sprite adds light and is always tone mapped." {
    default true
  }
  field flipX bool "Mirror the region left to right."
  field flipY bool "Mirror the region top to bottom."
  field sortY bool "Within its layer, draw by y: lower on the screen draws in front (top-down games)."
  field slice Option.Array.f32.4 "Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole."
  field sliceScale f32 "Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets." {
    default 0.0625
    min 0
  }
  field screen bool "Draw in screen space over the finished frame instead of the world: position x, y in CSS pixels from the top left with y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested (a HUD). flash, glow, blend and castShadow apply to world sprites only. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it."
  field clip Option.Array.f32.4 "Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole."
}
```

<sub>`api.ogt` line 3532</sub>

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

<sub>`api.ogt` line 3720</sub>

<a id="type-BoxGeometryDesc"></a>

### `BoxGeometryDesc` (desc struct)

A box centred at the origin, as three.js BoxGeometry(width, height, depth).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its extent along x. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `depth` | `number` | `1` | ≥ 0 | `m` | — | Its extent along z. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160. |

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
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160." {
    default 1 1
  }
}
```

<sub>`api.ogt` line 3792</sub>

<a id="type-PlaneGeometryDesc"></a>

### `PlaneGeometryDesc` (desc struct)

A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its extent along x. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160. |

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
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160." {
    default 1 1
  }
}
```

<sub>`api.ogt` line 3815</sub>

<a id="type-SphereGeometryDesc"></a>

### `SphereGeometryDesc` (desc struct)

A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | Its radius. |
| `widthSegments` | `number` | `32` | ≥ 3 | — | — | Segments around the equator. |
| `heightSegments` | `number` | `16` | ≥ 2 | — | — | Rings from pole to pole. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160. |

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
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160." {
    default 1 1
  }
}
```

<sub>`api.ogt` line 3833</sub>

<a id="type-RenderCapabilities"></a>

### `RenderCapabilities` (struct)

What the renderer can do on this device. Backend limits are reported here, never by missing members.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tier` | `RenderTier` | `"webgpu"` | — | — | — | The backend. |
| `compute` | `boolean` | `false` | — | — | — | Whether compute passes run (GPU culling, GPU particles). |
| `antialias` | `Antialias` | `"tier"` | — | — | — | The antialiasing the device resolved RenderOptions.antialias to. |
| `maxLights` | `number` | `0` | — | — | — | How many point and spot lights one frame draws (MAX_LIGHTS on every tier). |
| `maxShadowedLights` | `number` | `0` | — | — | — | How many point and spot lights cast shadows in one frame (8 on the full tier, 4 on WebGL2); past it, the shadowed lights brightest at the camera keep theirs. |
| `pointShadows` | `PointShadows` | `"auto"` | — | — | — | How point lights' shadows are stored on this device: cube or paraboloid (what RenderOptions.pointShadows resolved to). |
| `width` | `number` | `0` | — | — | — | The render target width in device pixels. |
| `height` | `number` | `0` | — | — | — | The render target height in device pixels. |
| `stylise` | `readonly string[]` | `[]` | — | — | — | The Stylise and MaterialStyle primitives this renderer draws, by group: pixel, posterize, palette, dither, halftone, hatching, paint, ink, inkDepth, depthBands, retro3d, surfaceSpace (paintCanvasSpace, hatchingSpace and halftoneSpace surface) and objectSpace (the same object) of Stylise; toon, shadowTint, rim, outline, outlineDistance and flatShading of MaterialStyle. A look that uses one this list lacks still applies; that primitive is not drawn. |

```ogt
struct RenderCapabilities {
  doc "What the renderer can do on this device. Backend limits are reported here, never by missing members."
  field tier RenderTier "The backend."
  field compute bool "Whether compute passes run (GPU culling, GPU particles)."
  field antialias Antialias "The antialiasing the device resolved RenderOptions.antialias to."
  field maxLights u32 "How many point and spot lights one frame draws (MAX_LIGHTS on every tier)."
  field maxShadowedLights u32 "How many point and spot lights cast shadows in one frame (8 on the full tier, 4 on WebGL2); past it, the shadowed lights brightest at the camera keep theirs."
  field pointShadows PointShadows "How point lights' shadows are stored on this device: cube or paraboloid (what RenderOptions.pointShadows resolved to)."
  field width u32 "The render target width in device pixels."
  field height u32 "The render target height in device pixels."
  field stylise List.String "The Stylise and MaterialStyle primitives this renderer draws, by group: pixel, posterize, palette, dither, halftone, hatching, paint, ink, inkDepth, depthBands, retro3d, surfaceSpace (paintCanvasSpace, hatchingSpace and halftoneSpace surface) and objectSpace (the same object) of Stylise; toon, shadowTint, rim, outline, outlineDistance and flatShading of MaterialStyle. A look that uses one this list lacks still applies; that primitive is not drawn."
}
```

<sub>`api.ogt` line 3879</sub>

<a id="type-Shader"></a>

### `Shader` (handle)

A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader and the parameters they read.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Shader "A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader and the parameters they read."
```

<sub>`api.ogt` line 3907</sub>

<a id="type-BufferGeometryDesc"></a>

### `BufferGeometryDesc` (desc struct)

Geometry from attribute arrays, as three.js BufferGeometry with its attributes set: one item per vertex in each array, triangles three indices each (or, with no indices, every three vertices).

Missing normals are computed, as three.js computeVertexNormals (each vertex the area-weighted sum of its triangles' normals: shared vertices shade smooth, an unindexed geometry flat); missing tangents are generated when a normal map needs them. An unindexed geometry whose vertex count is not a multiple of three has no triangles: it draws nothing as an og.Mesh and is a vertex set for og.Lines and og.Points.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `positions` | `Float32Array` | — | — | — | — | The vertex positions, x y z per vertex (three.js `position`); at least one vertex. |
| `normals` | `Float32Array \| null` | `null` | — | — | — | Unit normals, x y z per vertex; null computes them. |
| `uvs` | `Float32Array \| null` | `null` | — | — | — | Texture coordinates, u v per vertex, v down the image (glTF's, as og-render's); null is 0 0. |
| `colors` | `Float32Array \| null` | `null` | — | — | — | Linear vertex colours, r g b or r g b a per vertex, multiplied into the material's colour (three.js `color` with vertexColors); null is white. |
| `tangents` | `Float32Array \| null` | `null` | — | — | — | Tangents, x y z w per vertex (glTF's: w the bitangent's sign); null generates them. |
| `indices` | `Uint32Array \| null` | `null` | — | — | — | Triangles, three vertex indices each, counter-clockwise seen from the front; null takes every three vertices as one. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it (a mesh made each tick: water, a deforming terrain, a trail of geometry). A static geometry is refused there. |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too, so an og.Collider with it as `mesh` (SHAPE_MESH or SHAPE_CONVEX) collides with what is drawn; equivalent to og.physics.setMesh with its positions and indices. An update re-registers it. |

```ogt
struct BufferGeometryDesc {
  doc "Geometry from attribute arrays, as three.js BufferGeometry with its attributes set: one item per vertex in each array, triangles three indices each (or, with no indices, every three vertices)."
  doc "Missing normals are computed, as three.js computeVertexNormals (each vertex the area-weighted sum of its triangles' normals: shared vertices shade smooth, an unindexed geometry flat); missing tangents are generated when a normal map needs them. An unindexed geometry whose vertex count is not a multiple of three has no triangles: it draws nothing as an og.Mesh and is a vertex set for og.Lines and og.Points."
  desc true
  field positions Float32Array "The vertex positions, x y z per vertex (three.js `position`); at least one vertex."
  field normals Option.Float32Array "Unit normals, x y z per vertex; null computes them."
  field uvs Option.Float32Array "Texture coordinates, u v per vertex, v down the image (glTF's, as og-render's); null is 0 0."
  field colors Option.Float32Array "Linear vertex colours, r g b or r g b a per vertex, multiplied into the material's colour (three.js `color` with vertexColors); null is white."
  field tangents Option.Float32Array "Tangents, x y z w per vertex (glTF's: w the bitangent's sign); null generates them."
  field indices Option.Uint32Array "Triangles, three vertex indices each, counter-clockwise seen from the front; null takes every three vertices as one."
  field dynamic bool "Whether og.render.updateGeometry may rewrite it (a mesh made each tick: water, a deforming terrain, a trail of geometry). A static geometry is refused there."
  field physics bool "Register it as a physics mesh too, so an og.Collider with it as `mesh` (SHAPE_MESH or SHAPE_CONVEX) collides with what is drawn; equivalent to og.physics.setMesh with its positions and indices. An update re-registers it." {
    default true
  }
}
```

<sub>`api.ogt` line 3962</sub>

<a id="type-GeometryInfo"></a>

### `GeometryInfo` (struct)

What a geometry holds (og.render.geometryInfo).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vertices` | `number` | `0` | — | — | — | Vertices. |
| `triangles` | `number` | `0` | — | — | — | Triangles. |
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bounds' least corner in the geometry's own space, as three.js boundingBox.min. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bounds' greatest corner. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it. |

```ogt
struct GeometryInfo {
  doc "What a geometry holds (og.render.geometryInfo)."
  field vertices u32 "Vertices."
  field triangles u32 "Triangles."
  field min Vector3 "The bounds' least corner in the geometry's own space, as three.js boundingBox.min." {
    unit m
  }
  field max Vector3 "The bounds' greatest corner." {
    unit m
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it."
}
```

<sub>`api.ogt` line 3978</sub>

<a id="type-CylinderGeometryDesc"></a>

### `CylinderGeometryDesc` (desc struct)

A cylinder (or a truncated cone) along +Y centred at the origin, as three.js CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radiusTop` | `number` | `1` | ≥ 0 | `m` | — | The radius at the top. |
| `radiusBottom` | `number` | `1` | ≥ 0 | `m` | — | The radius at the bottom. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `radialSegments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `heightSegments` | `number` | `1` | ≥ 1 | — | — | Rows along the height. |
| `openEnded` | `boolean` | `false` | — | — | — | Leave the ends open (no caps). |
| `thetaStart` | `number` | `0` | — | `rad` | — | Where the side starts around y, from +Z towards +X. |
| `thetaLength` | `number` | `6.283185307179586` | — | `rad` | — | How far around the side goes; 2 pi is whole. |

```ogt
struct CylinderGeometryDesc {
  doc "A cylinder (or a truncated cone) along +Y centred at the origin, as three.js CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength)."
  desc true
  field radiusTop f32 "The radius at the top." {
    default 1
    min 0
    unit m
  }
  field radiusBottom f32 "The radius at the bottom." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its extent along y." {
    default 1
    min 0
    unit m
  }
  field radialSegments u32 "Segments around." {
    default 32
    min 3
  }
  field heightSegments u32 "Rows along the height." {
    default 1
    min 1
  }
  field openEnded bool "Leave the ends open (no caps)."
  field thetaStart f32 "Where the side starts around y, from +Z towards +X." {
    unit rad
  }
  field thetaLength f32 "How far around the side goes; 2 pi is whole." {
    default 6.283185307179586
    unit rad
  }
}
```

<sub>`api.ogt` line 4013</sub>

<a id="type-ConeGeometryDesc"></a>

### `ConeGeometryDesc` (desc struct)

A cone along +Y centred at the origin, its tip up, as three.js ConeGeometry(radius, height, radialSegments, heightSegments, openEnded).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The base's radius. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `radialSegments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `heightSegments` | `number` | `1` | ≥ 1 | — | — | Rows along the height. |
| `openEnded` | `boolean` | `false` | — | — | — | Leave the base open. |

```ogt
struct ConeGeometryDesc {
  doc "A cone along +Y centred at the origin, its tip up, as three.js ConeGeometry(radius, height, radialSegments, heightSegments, openEnded)."
  desc true
  field radius f32 "The base's radius." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its extent along y." {
    default 1
    min 0
    unit m
  }
  field radialSegments u32 "Segments around." {
    default 32
    min 3
  }
  field heightSegments u32 "Rows along the height." {
    default 1
    min 1
  }
  field openEnded bool "Leave the base open."
}
```

<sub>`api.ogt` line 4049</sub>

<a id="type-CapsuleGeometryDesc"></a>

### `CapsuleGeometryDesc` (desc struct)

A capsule along +Y centred at the origin, as three.js CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments): a cylinder of `height` between two hemispheres, `height + 2 radius` long in all (an og.Collider capsule of halfHeight height / 2 and that radius matches it).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The radius. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | The straight middle's length. |
| `capSegments` | `number` | `4` | ≥ 1 | — | — | Rows in each hemisphere. |
| `radialSegments` | `number` | `8` | ≥ 3 | — | — | Segments around. |
| `heightSegments` | `number` | `1` | ≥ 1 | — | — | Rows along the middle. |

```ogt
struct CapsuleGeometryDesc {
  doc "A capsule along +Y centred at the origin, as three.js CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments): a cylinder of `height` between two hemispheres, `height + 2 radius` long in all (an og.Collider capsule of halfHeight height / 2 and that radius matches it)."
  desc true
  field radius f32 "The radius." {
    default 1
    min 0
    unit m
  }
  field height f32 "The straight middle's length." {
    default 1
    min 0
    unit m
  }
  field capSegments u32 "Rows in each hemisphere." {
    default 4
    min 1
  }
  field radialSegments u32 "Segments around." {
    default 8
    min 3
  }
  field heightSegments u32 "Rows along the middle." {
    default 1
    min 1
  }
}
```

<sub>`api.ogt` line 4073</sub>

<a id="type-TorusGeometryDesc"></a>

### `TorusGeometryDesc` (desc struct)

A torus in the xy plane around +Z, as three.js TorusGeometry(radius, tube, radialSegments, tubularSegments, arc).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | From the centre to the tube's centre. |
| `tube` | `number` | `0.4` | ≥ 0 | `m` | — | The tube's radius. |
| `radialSegments` | `number` | `12` | ≥ 2 | — | — | Segments around the tube. |
| `tubularSegments` | `number` | `48` | ≥ 3 | — | — | Segments around the ring. |
| `arc` | `number` | `6.283185307179586` | — | `rad` | — | How far around the ring goes. |

```ogt
struct TorusGeometryDesc {
  doc "A torus in the xy plane around +Z, as three.js TorusGeometry(radius, tube, radialSegments, tubularSegments, arc)."
  desc true
  field radius f32 "From the centre to the tube's centre." {
    default 1
    min 0
    unit m
  }
  field tube f32 "The tube's radius." {
    default 0.4
    min 0
    unit m
  }
  field radialSegments u32 "Segments around the tube." {
    default 12
    min 2
  }
  field tubularSegments u32 "Segments around the ring." {
    default 48
    min 3
  }
  field arc f32 "How far around the ring goes." {
    default 6.283185307179586
    unit rad
  }
}
```

<sub>`api.ogt` line 4100</sub>

<a id="type-RingGeometryDesc"></a>

### `RingGeometryDesc` (desc struct)

A flat ring in the xy plane facing +Z, as three.js RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `innerRadius` | `number` | `0.5` | ≥ 0 | `m` | — | The hole's radius. |
| `outerRadius` | `number` | `1` | ≥ 0 | `m` | — | The ring's radius. |
| `thetaSegments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `phiSegments` | `number` | `1` | ≥ 1 | — | — | Rings from the inside out. |
| `thetaStart` | `number` | `0` | — | `rad` | — | Where it starts, from +X towards +Y. |
| `thetaLength` | `number` | `6.283185307179586` | — | `rad` | — | How far around it goes. |

```ogt
struct RingGeometryDesc {
  doc "A flat ring in the xy plane facing +Z, as three.js RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength)."
  desc true
  field innerRadius f32 "The hole's radius." {
    default 0.5
    min 0
    unit m
  }
  field outerRadius f32 "The ring's radius." {
    default 1
    min 0
    unit m
  }
  field thetaSegments u32 "Segments around." {
    default 32
    min 3
  }
  field phiSegments u32 "Rings from the inside out." {
    default 1
    min 1
  }
  field thetaStart f32 "Where it starts, from +X towards +Y." {
    unit rad
  }
  field thetaLength f32 "How far around it goes." {
    default 6.283185307179586
    unit rad
  }
}
```

<sub>`api.ogt` line 4127</sub>

<a id="type-CircleGeometryDesc"></a>

### `CircleGeometryDesc` (desc struct)

A disc (or a sector) in the xy plane facing +Z, as three.js CircleGeometry(radius, segments, thetaStart, thetaLength): a 2D polygon's simplest shape.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The radius. |
| `segments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `thetaStart` | `number` | `0` | — | `rad` | — | Where it starts, from +X towards +Y. |
| `thetaLength` | `number` | `6.283185307179586` | — | `rad` | — | How far around it goes. |

```ogt
struct CircleGeometryDesc {
  doc "A disc (or a sector) in the xy plane facing +Z, as three.js CircleGeometry(radius, segments, thetaStart, thetaLength): a 2D polygon's simplest shape."
  desc true
  field radius f32 "The radius." {
    default 1
    min 0
    unit m
  }
  field segments u32 "Segments around." {
    default 32
    min 3
  }
  field thetaStart f32 "Where it starts, from +X towards +Y." {
    unit rad
  }
  field thetaLength f32 "How far around it goes." {
    default 6.283185307179586
    unit rad
  }
}
```

<sub>`api.ogt` line 4157</sub>

<a id="type-TubeGeometryDesc"></a>

### `TubeGeometryDesc` (desc struct)

A tube along a curve through points, as three.js TubeGeometry(new CatmullRomCurve3(points, closed, 'centripetal'), tubularSegments, radius, radialSegments, closed): u runs along it, v around. The points may come from anywhere: a path's samples, a rope's points, a spline a tool made.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | `m` | — | The points the curve passes through, in order (at least two). |
| `tubularSegments` | `number` | `64` | ≥ 1 | — | — | Segments along the tube. |
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The tube's radius. |
| `radialSegments` | `number` | `8` | ≥ 3 | — | — | Segments around it. |
| `closed` | `boolean` | `false` | — | — | — | Join the last point back to the first. |

```ogt
struct TubeGeometryDesc {
  doc "A tube along a curve through points, as three.js TubeGeometry(new CatmullRomCurve3(points, closed, 'centripetal'), tubularSegments, radius, radialSegments, closed): u runs along it, v around. The points may come from anywhere: a path's samples, a rope's points, a spline a tool made."
  desc true
  field points List.Vector3 "The points the curve passes through, in order (at least two)." {
    unit m
  }
  field tubularSegments u32 "Segments along the tube." {
    default 64
    min 1
  }
  field radius f32 "The tube's radius." {
    default 1
    min 0
    unit m
  }
  field radialSegments u32 "Segments around it." {
    default 8
    min 3
  }
  field closed bool "Join the last point back to the first."
}
```

<sub>`api.ogt` line 4178</sub>

<a id="type-LineMode"></a>

### `LineMode` (enum)

How a line set's points join, as three.js's three line objects.

A string: one of the variants below. The zero value is `"strip"`.

| Variant | Doc |
|---|---|
| `"strip"` | One polyline through every point, as three.js Line. |
| `"segments"` | Pairs of points, each its own segment, as three.js LineSegments. |
| `"loop"` | A polyline closed back to its first point, as three.js LineLoop. |

```ogt
enum LineMode {
  doc "How a line set's points join, as three.js's three line objects."
  variant strip "One polyline through every point, as three.js Line."
  variant segments "Pairs of points, each its own segment, as three.js LineSegments."
  variant loop "A polyline closed back to its first point, as three.js LineLoop."
}
```

<sub>`api.ogt` line 4249</sub>

<a id="type-LinesDesc"></a>

### `LinesDesc` (desc struct)

Lines through points, drawn this frame only (immediate mode, as og.render.sprite): submit them every tick they should show. three.js Line, LineSegments and LineLoop with a LineBasicMaterial or LineDashedMaterial, but any width on every tier (each segment is a quad in screen space with a feathered edge, so WebGL2's one-pixel lines do not apply).

World lines are drawn in the scene after the transparent meshes and particles, exposed, bloomed and tone mapped with it; screen lines over the finished frame, after every post effect and under the UI.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | `m` | — | The points, in order: world positions, or with screen CSS pixels from the top left of the play surface with y down (z ignored). |
| `colors` | `readonly Color[]` | `[]` | — | — | — | Per point linear colours, blended along each segment (three.js vertexColors); empty is `color` for all. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `mode` | `LineMode` | `"strip"` | — | — | — | How the points join. |
| `width` | `number` | `1` | ≥ 0 | — | — | The width: CSS pixels, or world units with worldUnits. |
| `worldUnits` | `boolean` | `false` | — | — | — | Measure width in world units (a line that thins with distance) instead of CSS pixels (as wide at any distance). |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry, as three.js Material.depthTest; false draws it over the scene. Screen lines are never depth tested. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame: points in CSS pixels, width in CSS pixels. |
| `dashed` | `boolean` | `false` | — | — | — | Dashes, as three.js LineDashedMaterial: dashSize on, gapSize off, along the line from its first point. |
| `dashSize` | `number` | `3` | ≥ 0 | — | — | A dash's length: world units (CSS pixels for screen lines). |
| `gapSize` | `number` | `1` | ≥ 0 | — | — | A gap's length. |

```ogt
struct LinesDesc {
  doc "Lines through points, drawn this frame only (immediate mode, as og.render.sprite): submit them every tick they should show. three.js Line, LineSegments and LineLoop with a LineBasicMaterial or LineDashedMaterial, but any width on every tier (each segment is a quad in screen space with a feathered edge, so WebGL2's one-pixel lines do not apply)."
  doc "World lines are drawn in the scene after the transparent meshes and particles, exposed, bloomed and tone mapped with it; screen lines over the finished frame, after every post effect and under the UI."
  desc true
  field points List.Vector3 "The points, in order: world positions, or with screen CSS pixels from the top left of the play surface with y down (z ignored)." {
    unit m
  }
  field colors List.Color "Per point linear colours, blended along each segment (three.js vertexColors); empty is `color` for all."
  field color Color "The linear colour." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field mode LineMode "How the points join."
  field width f32 "The width: CSS pixels, or world units with worldUnits." {
    default 1
    min 0
  }
  field worldUnits bool "Measure width in world units (a line that thins with distance) instead of CSS pixels (as wide at any distance)."
  field depthTest bool "Hidden behind nearer geometry, as three.js Material.depthTest; false draws it over the scene. Screen lines are never depth tested." {
    default true
  }
  field screen bool "Draw in screen space over the finished frame: points in CSS pixels, width in CSS pixels."
  field dashed bool "Dashes, as three.js LineDashedMaterial: dashSize on, gapSize off, along the line from its first point."
  field dashSize f32 "A dash's length: world units (CSS pixels for screen lines)." {
    default 3
    min 0
  }
  field gapSize f32 "A gap's length." {
    default 1
    min 0
  }
}
```

<sub>`api.ogt` line 4263</sub>

<a id="type-PointsDesc"></a>

### `PointsDesc` (desc struct)

A point cloud drawn this frame only (immediate mode), as three.js Points with a PointsMaterial: a square per point facing the camera. Submit it every tick it should show (og.Points keeps a static set).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | `m` | — | The positions: world, or with screen CSS pixels from the top left, y down. |
| `colors` | `readonly Color[]` | `[]` | — | — | — | Per point linear colours; empty is `color` for all. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `size` | `number` | `1` | ≥ 0 | — | — | The side of each square: world units with sizeAttenuation, else CSS pixels. |
| `sizeAttenuation` | `boolean` | `true` | — | — | — | Size in world units, smaller with distance, as three.js PointsMaterial.sizeAttenuation; false keeps every point `size` CSS pixels. |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry; false draws them over the scene. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame, positions and size in CSS pixels. |

```ogt
struct PointsDesc {
  doc "A point cloud drawn this frame only (immediate mode), as three.js Points with a PointsMaterial: a square per point facing the camera. Submit it every tick it should show (og.Points keeps a static set)."
  desc true
  field points List.Vector3 "The positions: world, or with screen CSS pixels from the top left, y down." {
    unit m
  }
  field colors List.Color "Per point linear colours; empty is `color` for all."
  field color Color "The linear colour." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field size f32 "The side of each square: world units with sizeAttenuation, else CSS pixels." {
    default 1
    min 0
  }
  field sizeAttenuation bool "Size in world units, smaller with distance, as three.js PointsMaterial.sizeAttenuation; false keeps every point `size` CSS pixels." {
    default true
  }
  field depthTest bool "Hidden behind nearer geometry; false draws them over the scene." {
    default true
  }
  field screen bool "Draw in screen space over the finished frame, positions and size in CSS pixels."
}
```

<sub>`api.ogt` line 4307</sub>

<a id="type-RaycastGeometryDesc"></a>

### `RaycastGeometryDesc` (desc struct)

Which meshes og.render.raycast tests, as three.js Raycaster's near, far and layers.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far the ray reaches (three.js Raycaster.far); 0 is unbounded. |
| `first` | `boolean` | `false` | — | — | — | Answer only the nearest hit (cheaper: the BVH stops early). |
| `layers` | `number` | `1` | — | — | — | A bit mask of the layers tested, as three.js Raycaster.layers: an entity's meshes are tested when its layers share a bit with it. Every mesh is on layer 1 (bit 0) until per-object layers land, so the default tests them all. |
| `invisible` | `boolean` | `false` | — | — | — | Test meshes whose visible is false too. |

```ogt
struct RaycastGeometryDesc {
  doc "Which meshes og.render.raycast tests, as three.js Raycaster's near, far and layers."
  desc true
  field maxDistance f32 "How far the ray reaches (three.js Raycaster.far); 0 is unbounded." {
    min 0
    unit m
  }
  field first bool "Answer only the nearest hit (cheaper: the BVH stops early)."
  field layers u32 "A bit mask of the layers tested, as three.js Raycaster.layers: an entity's meshes are tested when its layers share a bit with it. Every mesh is on layer 1 (bit 0) until per-object layers land, so the default tests them all." {
    default 1
  }
  field invisible bool "Test meshes whose visible is false too."
}
```

<sub>`api.ogt` line 4404</sub>

<a id="type-GeometryHit"></a>

### `GeometryHit` (struct)

Where a ray meets a drawn triangle, as three.js Raycaster's intersection.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Mesh's or og.Model's entity. |
| `geometry` | `Geometry` | `0` | — | — | — | The geometry hit. |
| `distance` | `number` | `0` | — | `m` | — | From the ray's origin along its normalised direction. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where, in world space. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The triangle's face normal in world space (its front side, counter-clockwise), unit length. |
| `uv` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The texture coordinates there, interpolated. |
| `faceIndex` | `number` | `0` | — | — | — | The triangle's index in the geometry, as three.js faceIndex. |

```ogt
struct GeometryHit {
  doc "Where a ray meets a drawn triangle, as three.js Raycaster's intersection."
  field entity Entity "The og.Mesh's or og.Model's entity."
  field geometry Geometry "The geometry hit."
  field distance f32 "From the ray's origin along its normalised direction." {
    unit m
  }
  field point Vector3 "Where, in world space." {
    unit m
  }
  field normal Vector3 "The triangle's face normal in world space (its front side, counter-clockwise), unit length."
  field uv Vector2 "The texture coordinates there, interpolated."
  field faceIndex u32 "The triangle's index in the geometry, as three.js faceIndex."
}
```

<sub>`api.ogt` line 4418</sub>

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
| <a id="column-Palette"></a>`Palette` | `Palette` | `Uint32Array` | 2 | A Palette handle field (0 is none). |
| <a id="column-DitherPattern"></a>`DitherPattern` | `DitherPattern` | `Uint32Array` | 1 | A DitherPattern field: the variant's index. |
| <a id="column-StyliseSpace"></a>`StyliseSpace` | `StyliseSpace` | `Uint32Array` | 1 | A StyliseSpace field: the variant's index. |
| <a id="column-LineMode"></a>`LineMode` | `LineMode` | `Uint32Array` | 1 | A LineMode field: the variant's index. |

```ogt
column Geometry {
  doc "A Geometry handle field."
  array Uint32Array
  stride 2
  value Geometry
}
```

<sub>`api.ogt` line 2242</sub>

```ogt
column Material {
  doc "A Material handle field."
  array Uint32Array
  stride 2
  value Material
}
```

<sub>`api.ogt` line 2249</sub>

```ogt
column ToneMapping {
  doc "A ToneMapping field: the variant's index."
  array Uint32Array
  stride 1
  value ToneMapping
}
```

<sub>`api.ogt` line 2618</sub>

```ogt
column Texture {
  doc "A Texture handle field (0 is none)."
  array Uint32Array
  stride 2
  value Texture
}
```

<sub>`api.ogt` line 2680</sub>

```ogt
column Lut {
  doc "A Lut handle field (0 is none)."
  array Uint32Array
  stride 2
  value Lut
}
```

<sub>`api.ogt` line 2687</sub>

```ogt
column EnvironmentMap {
  doc "An EnvironmentMap handle field (0 is none)."
  array Uint32Array
  stride 2
  value EnvironmentMap
}
```

<sub>`api.ogt` line 2694</sub>

```ogt
column ParticleEmitter {
  doc "A ParticleEmitter handle field (0 is none)."
  array Uint32Array
  stride 2
  value ParticleEmitter
}
```

<sub>`api.ogt` line 2721</sub>

```ogt
column AoMethod {
  doc "An AoMethod field: the variant's index."
  array Uint32Array
  stride 1
  value AoMethod
}
```

<sub>`api.ogt` line 2735</sub>

```ogt
column DofQuality {
  doc "A DofQuality field: the variant's index."
  array Uint32Array
  stride 1
  value DofQuality
}
```

<sub>`api.ogt` line 2748</sub>

```ogt
column SpriteBlend {
  doc "A SpriteBlend field: the variant's index."
  array Uint32Array
  stride 1
  value SpriteBlend
}
```

<sub>`api.ogt` line 2761</sub>

```ogt
column StripBlend {
  doc "A StripBlend field: the variant's index."
  array Uint32Array
  stride 1
  value StripBlend
}
```

<sub>`api.ogt` line 2774</sub>

```ogt
column Palette {
  doc "A Palette handle field (0 is none)."
  array Uint32Array
  stride 2
  value Palette
}
```

<sub>`api.ogt` line 2961</sub>

```ogt
column DitherPattern {
  doc "A DitherPattern field: the variant's index."
  array Uint32Array
  stride 1
  value DitherPattern
}
```

<sub>`api.ogt` line 2978</sub>

```ogt
column StyliseSpace {
  doc "A StyliseSpace field: the variant's index."
  array Uint32Array
  stride 1
  value StyliseSpace
}
```

<sub>`api.ogt` line 2992</sub>

```ogt
column LineMode {
  doc "A LineMode field: the variant's index."
  array Uint32Array
  stride 1
  value LineMode
}
```

<sub>`api.ogt` line 4256</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-PARTICLES_ARMED"></a>`PARTICLES_ARMED` | `number` | `4294967295` | ParticleSystem.startTick's default: never written. A looping emitter emits as if started at tick 0; a one-shot (`looping false`) emits nothing until its startTick is written, so a burst spawned in `start` waits for its first use. Write it back to re-arm a one-shot without firing it. |
| <a id="const-MAX_LIGHTS"></a>`MAX_LIGHTS` | `number` | `256` | The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept. |
| <a id="const-MAX_CLUSTER_LIGHTS"></a>`MAX_CLUSTER_LIGHTS` | `number` | `64` | The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there. |
| <a id="const-UI_LAYER"></a>`UI_LAYER` | `number` | `1000` | The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the screen transition included (a HUD element made of sprites sitting on a UI panel: draw it only while its screen shows), in their layers' order either way. |

```ogt
const PARTICLES_ARMED u32 4294967295 "ParticleSystem.startTick's default: never written. A looping emitter emits as if started at tick 0; a one-shot (`looping false`) emits nothing until its startTick is written, so a burst spawned in `start` waits for its first use. Write it back to re-arm a one-shot without firing it."
```

<sub>`api.ogt` line 3337</sub>

```ogt
const MAX_LIGHTS u32 256 "The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept."
```

<sub>`api.ogt` line 3875</sub>

```ogt
const MAX_CLUSTER_LIGHTS u32 64 "The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there."
```

<sub>`api.ogt` line 3876</sub>

```ogt
const UI_LAYER i32 1000 "The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the screen transition included (a HUD element made of sprites sitting on a UI panel: draw it only while its screen shows), in their layers' order either way."
```

<sub>`api.ogt` line 3877</sub>
