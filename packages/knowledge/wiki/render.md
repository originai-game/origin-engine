<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `render` module

What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.

- **TypeScript**: functions are the namespace `og.render`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module render {
  doc "What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 3794</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Geometry`](render/type-Geometry.md) | handle |  | Vertex and index data, as three.js BufferGeometry. |
| [`Material`](render/type-Material.md) | handle |  | A material instance. |
| [`Texture`](render/type-Texture.md) | handle |  | A cooked texture. |
| [`Geometry()`](#column-Geometry) | column kind |  | A Geometry handle field. |
| [`Material()`](#column-Material) | column kind |  | A Material handle field. |
| [`Side`](render/type-Side.md) | enum |  | Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide. |
| [`RenderTier`](render/type-RenderTier.md) | enum |  | The backend the renderer runs on; the probe's hello reports it. |
| [`Antialias`](render/type-Antialias.md) | enum |  | Antialiasing, as og-render's Antialias. |
| [`PointShadows`](render/type-PointShadows.md) | enum |  | How a point light's shadow map is stored, as og-render's PointShadows. |
| [`TaaHistory`](render/type-TaaHistory.md) | enum |  | How TAA resamples its history each frame, as og-render's TaaHistory. |
| [`Upscaler`](render/type-Upscaler.md) | enum |  | The temporal resolve under TAA (RenderSettings.upscaler), as og-render's Upscaler: what turns the jittered frame, drawn at the render scale, into the output. |
| [`Upscaler()`](#column-Upscaler) | column kind |  | An Upscaler field: the variant's index. |
| [`DebugView`](render/type-DebugView.md) | enum |  | What the debug overlay shows besides og.debug's draws (og.debug.setView, RenderOptions.debugView, `debug:<view>`), drawn over the finished frame; the lit image's views (cascades, ao, albedo, iblDiffu… |
| [`Transparency`](render/type-Transparency.md) | enum |  | How the main camera's blended surfaces (transparent materials, alpha billboards, alpha particles and trails) are composited, as og-render's Transparency: in any order, never sorted. |
| [`RenderOptions`](render/type-RenderOptions.md) | desc struct |  | How the renderer is created for a game (GameDesc.render). |
| [`FoliageDesc`](render/type-FoliageDesc.md) | desc struct |  | Leaf lighting. Zero keeps ordinary opaque-surface lighting; this does not enable glass transmission or change alpha coverage. |
| [`StandardMaterialDesc`](render/type-StandardMaterialDesc.md) | desc struct |  | A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names. |
| [`og.Mesh`](render/record-Mesh.md) | component |  | Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh. |
| [`og.DirectionalLight`](render/record-DirectionalLight.md) | component |  | The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). |
| [`og.PointLight`](render/record-PointLight.md) | component |  | A light shining in every direction from the entity's position, as three.js PointLight. |
| [`og.SpotLight`](render/record-SpotLight.md) | component |  | A cone of light along the entity's local -Z, as three.js SpotLight. |
| [`ToneMapping`](render/type-ToneMapping.md) | enum |  | The curve from the lit HDR image to the display, as three.js toneMapping. |
| [`ToneMapping()`](#column-ToneMapping) | column kind |  | A ToneMapping field: the variant's index. |
| [`GraphicsQuality`](render/type-GraphicsQuality.md) | enum |  | A graphics preset: a row of RenderSettings' quality fields that og.render.applyQuality writes (its doc has the table). |
| [`AoResolutionSetting`](render/type-AoResolutionSetting.md) | enum |  | The resolution RenderSettings.aoResolution draws ambient occlusion at. |
| [`AoResolutionSetting()`](#column-AoResolutionSetting) | column kind |  | An AoResolutionSetting field: the variant's index. |
| [`QualityPreset`](render/type-QualityPreset.md) | enum |  | The preset RenderSettings' quality fields were last written with (RenderSettings.quality). |
| [`QualityPreset()`](#column-QualityPreset) | column kind |  | A QualityPreset field: the variant's index. |
| [`og.RenderSettings`](render/record-RenderSettings.md) | resource |  | The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping, and what the frame may cost: the quality fields (rectShadows to upscaler) that og.render.applyQuali… |
| [`og.Interpolation`](render/record-Interpolation.md) | component |  | How the renderer draws this entity between ticks while RenderSettings.interpolate is on; an entity without it is blended. |
| [`og.render.snap`](render/fn-snap.md) | function | `(world: World, entity: Entity): Result<void>` | Draw entity, and everything under it through og.Parent, at its tick transform on the frames until the next tick instead of blending from where it was (RenderSettings.interpolate): call it on the tick… |
| [`og.render.snapCamera`](render/fn-snapCamera.md) | function | `(world: World): void` | Draw og.Camera at its tick pose on the frames until the next tick instead of blending from where it was: a cut. |
| [`Lut`](render/type-Lut.md) | handle |  | A colour lookup table for PostEffects.lut, from og.load.lut (a `.cube` file). |
| [`EnvironmentMap`](render/type-EnvironmentMap.md) | handle |  | A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`, a KTX2 cube map, or six face images). |
| [`ParticleEmitter`](render/type-ParticleEmitter.md) | handle |  | A particle emitter asset for ParticleSystem.emitter, from og.load.emitter (an `.ogt` of `type particles`). |
| [`Texture()`](#column-Texture) | column kind |  | A Texture handle field (0 is none). |
| [`Lut()`](#column-Lut) | column kind |  | A Lut handle field (0 is none). |
| [`EnvironmentMap()`](#column-EnvironmentMap) | column kind |  | An EnvironmentMap handle field (0 is none). |
| [`og.Environment`](render/record-Environment.md) | resource |  | Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. |
| [`ParticleEmitter()`](#column-ParticleEmitter) | column kind |  | A ParticleEmitter handle field (0 is none). |
| [`AoMethod`](render/type-AoMethod.md) | enum |  | Which ambient occlusion algorithm runs. |
| [`AoMethod()`](#column-AoMethod) | column kind |  | An AoMethod field: the variant's index. |
| [`AoResolution`](render/type-AoResolution.md) | enum |  | The resolution ambient occlusion is searched at. |
| [`AoResolution()`](#column-AoResolution) | column kind |  | An AoResolution field: the variant's index. |
| [`DofQuality`](render/type-DofQuality.md) | enum |  | How many taps depth of field gathers per pixel. |
| [`DofQuality()`](#column-DofQuality) | column kind |  | A DofQuality field: the variant's index. |
| [`ScreenBlur`](render/type-ScreenBlur.md) | enum |  | Which way PostEffects.screenBlur smears the whole frame. |
| [`ScreenBlur()`](#column-ScreenBlur) | column kind |  | A ScreenBlur field: the variant's index. |
| [`SpriteBlend`](render/type-SpriteBlend.md) | enum |  | How a sprite is blended over what is below it. |
| [`SpriteBlend()`](#column-SpriteBlend) | column kind |  | A SpriteBlend field: the variant's index. |
| [`SpriteFacing`](render/type-SpriteFacing.md) | enum |  | How a world sprite turns towards the camera that draws it: a billboard (three.js Sprite). |
| [`SpriteFacing()`](#column-SpriteFacing) | column kind |  | A SpriteFacing field: the variant's index. |
| [`StripBlend`](render/type-StripBlend.md) | enum |  | How a trail, ribbon or beam is blended. |
| [`StripBlend()`](#column-StripBlend) | column kind |  | A StripBlend field: the variant's index. |
| [`og.PostEffects`](render/record-PostEffects.md) | resource |  | The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. |
| [`og.LensFlare`](render/record-LensFlare.md) | resource |  | Lens flare, as Filament's screen-space flare (after John Chapman): ghosts and a halo mirror the bright parts of the bloom chain's first level through the centre of the frame with chromatic aberration… |
| [`GiMethod`](render/type-GiMethod.md) | enum |  | How GlobalIllumination bounces the light. |
| [`GiMethod()`](#column-GiMethod) | column kind |  | A GiMethod field: the variant's index. |
| [`og.GlobalIllumination`](render/record-GlobalIllumination.md) | resource |  | Global illumination: light bounces between surfaces, so shade is lit by what stands around it. |
| [`Palette`](render/type-Palette.md) | handle |  | A colour palette for Stylise.palette, from og.load.palette (a `.hex` or GIMP `.gpl` file of up to 256 colours). |
| [`Look`](render/type-Look.md) | handle |  | A look from og.load.look (an `.ogt` of `type look`): the frame's whole art direction as data, applied with og.render.applyLook. |
| [`Palette()`](#column-Palette) | column kind |  | A Palette handle field (0 is none). |
| [`Look()`](#column-Look) | column kind |  | A Look handle field (0 is none): a look og.load.look answered, kept for og.render.applyLook. |
| [`DitherPattern`](render/type-DitherPattern.md) | enum |  | The threshold pattern Stylise.dither spreads the steps of posterize and palette with. |
| [`DitherPattern()`](#column-DitherPattern) | column kind |  | A DitherPattern field: the variant's index. |
| [`MaterialSurface`](render/type-MaterialSurface.md) | enum |  | A made surface MaterialStyle.surface lays on every lit material: the hand-made materials of the shape styles, drawn by the material's shading itself (its normal, its colour and how it takes the light… |
| [`MaterialSurface()`](#column-MaterialSurface) | column kind |  | A MaterialSurface field: the variant's index. |
| [`OutlinePlace`](render/type-OutlinePlace.md) | enum |  | Where MaterialStyle's outlines lie against an object's silhouette (MaterialStyle.outlinePlace). |
| [`OutlinePlace()`](#column-OutlinePlace) | column kind |  | An OutlinePlace field: the variant's index. |
| [`StyliseSpace`](render/type-StyliseSpace.md) | enum |  | Where Stylise lays a pattern (the paint canvas, the hatching, the halftone). |
| [`StyliseSpace()`](#column-StyliseSpace) | column kind |  | A StyliseSpace field: the variant's index. |
| [`PosterizeMode`](render/type-PosterizeMode.md) | enum |  | How Stylise.posterize steps a colour. |
| [`PosterizeMode()`](#column-PosterizeMode) | column kind |  | A PosterizeMode field: the variant's index. |
| [`HalftoneMode`](render/type-HalftoneMode.md) | enum |  | How Stylise.halftone inks the frame. |
| [`HalftoneMode()`](#column-HalftoneMode) | column kind |  | A HalftoneMode field: the variant's index. |
| [`HalftoneRegion`](render/type-HalftoneRegion.md) | enum |  | Which tones Stylise.halftone prints on. |
| [`HalftoneRegion()`](#column-HalftoneRegion) | column kind |  | A HalftoneRegion field: the variant's index. |
| [`og.Stylise`](render/record-Stylise.md) | resource |  | Stylisation primitives that move the frame away from the photographic: a low-resolution pixel grid scaled up by whole numbers, fewer tones or a fixed palette through an ordered or blue-noise dither, … |
| [`og.MaterialStyle`](render/record-MaterialStyle.md) | resource |  | Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls … |
| [`LookInfo`](render/type-LookInfo.md) | struct |  | A loaded look: where it came from and what applying it sets. |
| [`og.Lighting2D`](render/record-Lighting2D.md) | resource |  | 2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus e… |
| [`Light2DKind`](render/type-Light2DKind.md) | enum |  | What shape of light a Light2D is. |
| [`Light2DKind()`](#column-Light2DKind) | column kind |  | A Light2DKind field: the variant's index. |
| [`og.Light2D`](render/record-Light2D.md) | component |  | A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff (faded over its last tenth, so even a falloff below… |
| [`og.Occluder2D`](render/record-Occluder2D.md) | component |  | A shape that blocks 2D light (walls, crates, characters), centred on the entity's position and turned and scaled with it about z: a width by height rectangle, or the polygon `points` gives. |
| [`og.LightMask2D`](render/record-LightMask2D.md) | component |  | The light layers that light the entity's sprite, flipbook, 2D rig or tilemap: a Light2D lights it when their `layers` share a bit. |
| [`og.Decal`](render/record-Decal.md) | component |  | A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface in… |
| [`PARTICLES_ARMED`](#const-PARTICLES_ARMED) | constant |  | ParticleSystem.startTick's default: never written. |
| [`og.ParticleSystem`](render/record-ParticleSystem.md) | component |  | Emits particles from an og.load.emitter asset at the entity's MatrixWorld. |
| [`ParticleEventKind`](render/type-ParticleEventKind.md) | enum |  | What happened to a particle (og.ParticleEvent.kind; an emitter's `events` and `subEmitter` lines name them). |
| [`og.ParticleEvent`](render/record-ParticleEvent.md) | event |  | A particle of an og.ParticleSystem was born, died or hit something in the last tick (read in the tick after, like every event): a bullet's spark that lands, a firework's shell that bursts, debris tha… |
| [`og.Trail`](render/record-Trail.md) | component |  | A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail. |
| [`og.Beam`](render/record-Beam.md) | component |  | A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter). |
| [`RibbonPoint`](render/type-RibbonPoint.md) | desc struct |  | One point of a ribbon. |
| [`RibbonDesc`](render/type-RibbonDesc.md) | desc struct |  | An explicit camera-facing strip through points, drawn this frame only (immediate mode, as og.render.sprite): submit it every tick it should show. |
| [`og.render.ribbon`](render/fn-ribbon.md) | function | `(world: World, desc?: RibbonDesc): Result<void>` | Draw a ribbon this frame. |
| [`TextDesc`](render/type-TextDesc.md) | desc struct |  | A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show). |
| [`TextMeasureDesc`](render/type-TextMeasureDesc.md) | desc struct |  | Text to measure with og.render.measureText, laid out as og.render.text lays it out. |
| [`TextMetrics`](render/type-TextMetrics.md) | struct |  | The extent of a laid-out text, in world units. |
| [`SpriteDesc`](render/type-SpriteDesc.md) | desc struct |  | A textured or solid quad in the world's xy plane facing +z (or turned to the camera, `facing`), drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. |
| [`og.render.sprite`](render/fn-sprite.md) | function | `(world: World, desc?: SpriteDesc): Result<void>` | Draw a sprite this frame. |
| [`og.Sprite`](render/record-Sprite.md) | component |  | A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z, in the xy plane or turned to the camera by `facing`; its scale multiplies width and height, so squash and s… |
| [`og.Afterimage`](render/record-Afterimage.md) | component |  | Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by co… |
| [`FloatingTextDesc`](render/type-FloatingTextDesc.md) | desc struct |  | A damage number or pickup text that pops at a point, rises, spreads and fades by itself (no per-tick submission). |
| [`og.render.floatingText`](render/fn-floatingText.md) | function | `(world: World, desc?: FloatingTextDesc): Result<void>` | Spawn a floating text (damage numbers, +coins, `blocked`). |
| [`og.render.measureText`](render/fn-measureText.md) | function | `(world: World, desc?: TextMeasureDesc): Result<TextMetrics>` | The size text takes when og.render.text draws it, for centring, wrapping and layout. |
| [`BoxGeometryDesc`](render/type-BoxGeometryDesc.md) | desc struct |  | A box centred at the origin, as three.js BoxGeometry(width, height, depth). |
| [`PlaneGeometryDesc`](render/type-PlaneGeometryDesc.md) | desc struct |  | A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor. |
| [`SphereGeometryDesc`](render/type-SphereGeometryDesc.md) | desc struct |  | A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments). |
| [`og.render.boxGeometry`](render/fn-boxGeometry.md) | function | `(world: World, desc?: BoxGeometryDesc): Result<Geometry>` | Make a box geometry for og.Mesh. |
| [`og.render.planeGeometry`](render/fn-planeGeometry.md) | function | `(world: World, desc?: PlaneGeometryDesc): Result<Geometry>` | Make a plane geometry for og.Mesh. |
| [`og.render.sphereGeometry`](render/fn-sphereGeometry.md) | function | `(world: World, desc?: SphereGeometryDesc): Result<Geometry>` | Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh). |
| [`MAX_LIGHTS`](#const-MAX_LIGHTS) | constant |  | The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept. |
| [`MAX_CLUSTER_LIGHTS`](#const-MAX_CLUSTER_LIGHTS) | constant |  | The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there. |
| [`UI_LAYER`](#const-UI_LAYER) | constant |  | The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the s… |
| [`RenderCapabilities`](render/type-RenderCapabilities.md) | struct |  | What the renderer can do on this device. |
| [`MAX_REFLECTION_PROBES`](#const-MAX_REFLECTION_PROBES) | constant |  | The most og.ReflectionProbe components one frame samples, nearest the camera first; the rest are skipped. |
| [`ReflectionProbeMode`](render/type-ReflectionProbeMode.md) | enum |  | When a reflection probe's cube is rendered. |
| [`ReflectionProbeMode()`](#column-ReflectionProbeMode) | column kind |  | A ReflectionProbeMode field: the variant's index. |
| [`og.ReflectionProbe`](render/record-ReflectionProbe.md) | component |  | A local reflection: a cube of what is around the entity's position, sampled by every lit surface inside its box with box projection (the reflected ray meets the box, so a room's walls reflect where t… |
| [`og.render.standardMaterial`](render/fn-standardMaterial.md) | function | `(world: World, desc?: StandardMaterialDesc): Result<Material>` | Create a standard material. |
| [`og.render.updateMaterial`](render/fn-updateMaterial.md) | function | `(world: World, material: Material, desc?: StandardMaterialDesc): Result<void>` | Change some parameters of a standard material. |
| [`Shader`](render/type-Shader.md) | handle |  | A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader, or with `mode full` its own vertex and fragment entries, and the parameters they read. |
| [`Shader()`](#column-Shader) | column kind |  | A Shader handle field (0 is none): a material shader og.load.shader answered, kept for og.render.shaderMaterial. |
| [`og.render.shaderMaterial`](render/fn-shaderMaterial.md) | function | `(world: World, shader: Shader, desc?: StandardMaterialDesc): Result<Material>` | Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive), its parameters at their defaults. |
| [`og.render.setMaterialShader`](render/fn-setMaterialShader.md) | function | `(world: World, material: Material, shader: Shader \| null): Result<void>` | Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's paramete… |
| [`og.render.setShaderParam`](render/fn-setShaderParam.md) | function | `(world: World, material: Material, name: string, value: readonly number[]): Result<void>` | Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color. |
| [`og.render.applyLook`](render/fn-applyLook.md) | function | `(world: World, look: Look): Result<void>` | Apply a look, in GameDesc.start (or name it in GameDesc.look) or at any tick for a level's mood, a flashback or a photo mode: every resource the look names is replaced whole (the fields it sets take … |
| [`og.render.applyQuality`](render/fn-applyQuality.md) | function | `(world: World, preset: GraphicsQuality): Result<void>` | Apply a graphics preset: write its row of RenderSettings' quality fields (the rest of RenderSettings keeps its values) and record it in RenderSettings.quality. |
| [`FrameCap`](render/type-FrameCap.md) | enum |  | How many frames a second the display draws at most (DisplaySettings.frameCap). |
| [`PowerPreference`](render/type-PowerPreference.md) | enum |  | Which GPU the browser's adapter is asked for (requestAdapter's powerPreference), on a laptop with two. |
| [`DisplaySettings`](render/type-DisplaySettings.md) | desc struct |  | The player's graphics settings, as a settings page shows them and og.render.setDisplay sets them: the graphics preset, the frame cap, battery saver, render scale and automatic quality, fullscreen and… |
| [`GpuMemory`](render/type-GpuMemory.md) | struct |  | The GPU memory the renderer holds, counted where it creates each texture and buffer (sizes as allocated, without the driver's alignment and padding). |
| [`DisplayInfo`](render/type-DisplayInfo.md) | struct |  | What the display draws now (og.render.display): the settings, and what they resolved to on this device. |
| [`og.render.setDisplay`](render/fn-setDisplay.md) | function | `(world: World, settings?: DisplaySettings): Result<void>` | Set the player's graphics settings (DisplaySettings: preset, frame cap, battery saver, render scale, automatic quality, fullscreen, power preference). |
| [`og.render.resetDisplay`](render/fn-resetDisplay.md) | function | `(world: World): Result<void>` | Put every graphics setting back to its default (DisplaySettings' zero value: the game's own quality, frame cap auto, render scale 1, automatic quality on, no battery saver, windowed, high performance… |
| [`og.render.display`](render/fn-display.md) | function | `(world: World): DisplayInfo` | What the display draws now: the settings, the frame cap resolved, the render scale automatic quality chose, the scene's size, the device losses survived, the GPU memory, and what the browser decides … |
| [`og.render.look`](render/fn-look.md) | function | `(world: World): LookInfo \| null` | The look last applied (by og.render.applyLook or GameDesc.look), as og.load.lookInfo describes it; null before any. |
| [`og.render.capabilities`](render/fn-capabilities.md) | function | `(world: World): RenderCapabilities` | The renderer's tier and limits on this device. |
| [`og.render.text`](render/fn-text.md) | function | `(world: World, desc?: TextDesc): Result<void>` | Draw text this frame. |
| [`BufferGeometryDesc`](render/type-BufferGeometryDesc.md) | desc struct |  | Geometry from attribute arrays, as three.js BufferGeometry with its attributes set: one item per vertex in each array, triangles three indices each (or, with no indices, every three vertices). |
| [`GeometryInfo`](render/type-GeometryInfo.md) | struct |  | What a geometry holds (og.render.geometryInfo). |
| [`og.render.bufferGeometry`](render/fn-bufferGeometry.md) | function | `(world: World, desc?: BufferGeometryDesc): Result<Geometry>` | Make a geometry from attribute arrays (three.js BufferGeometry): procedural levels, 2D polygons, meshes a tool made. |
| [`og.render.updateGeometry`](render/fn-updateGeometry.md) | function | `(world: World, geometry: Geometry, desc?: BufferGeometryDesc): Result<void>` | Rewrite a dynamic geometry's attributes in place (desc as og.render.bufferGeometry's, every attribute given again; `dynamic` and `physics` keep their values from when it was made). |
| [`og.render.geometryInfo`](render/fn-geometryInfo.md) | function | `(world: World, geometry: Geometry): Result<GeometryInfo>` | A geometry's counts and bounds. |
| [`CylinderGeometryDesc`](render/type-CylinderGeometryDesc.md) | desc struct |  | A cylinder (or a truncated cone) along +Y centred at the origin, as three.js CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength). |
| [`ConeGeometryDesc`](render/type-ConeGeometryDesc.md) | desc struct |  | A cone along +Y centred at the origin, its tip up, as three.js ConeGeometry(radius, height, radialSegments, heightSegments, openEnded). |
| [`CapsuleGeometryDesc`](render/type-CapsuleGeometryDesc.md) | desc struct |  | A capsule along +Y centred at the origin, as three.js CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments): a cylinder of `height` between two hemispheres, `height + 2 radius`… |
| [`TorusGeometryDesc`](render/type-TorusGeometryDesc.md) | desc struct |  | A torus in the xy plane around +Z, as three.js TorusGeometry(radius, tube, radialSegments, tubularSegments, arc). |
| [`RingGeometryDesc`](render/type-RingGeometryDesc.md) | desc struct |  | A flat ring in the xy plane facing +Z, as three.js RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength). |
| [`CircleGeometryDesc`](render/type-CircleGeometryDesc.md) | desc struct |  | A disc (or a sector) in the xy plane facing +Z, as three.js CircleGeometry(radius, segments, thetaStart, thetaLength): a 2D polygon's simplest shape. |
| [`TubeGeometryDesc`](render/type-TubeGeometryDesc.md) | desc struct |  | A tube along a curve, as three.js TubeGeometry(curve, tubularSegments, radius, radialSegments, closed): its rings are evenly spaced along the curve's length (three.js getPointAt), u runs along it, v … |
| [`og.render.cylinderGeometry`](render/fn-cylinderGeometry.md) | function | `(world: World, desc?: CylinderGeometryDesc): Result<Geometry>` | Make a cylinder geometry for og.Mesh (give its body a cylinder-like Collider: a capsule, a convex Collider.mesh of it, or the geometry itself). |
| [`og.render.coneGeometry`](render/fn-coneGeometry.md) | function | `(world: World, desc?: ConeGeometryDesc): Result<Geometry>` | Make a cone geometry for og.Mesh. |
| [`og.render.capsuleGeometry`](render/fn-capsuleGeometry.md) | function | `(world: World, desc?: CapsuleGeometryDesc): Result<Geometry>` | Make a capsule geometry for og.Mesh (its body's Collider is a capsule of the same radius and half the height). |
| [`og.render.torusGeometry`](render/fn-torusGeometry.md) | function | `(world: World, desc?: TorusGeometryDesc): Result<Geometry>` | Make a torus geometry for og.Mesh. |
| [`og.render.ringGeometry`](render/fn-ringGeometry.md) | function | `(world: World, desc?: RingGeometryDesc): Result<Geometry>` | Make a ring geometry for og.Mesh (one-sided: give it a double-sided material to see it from behind). |
| [`og.render.circleGeometry`](render/fn-circleGeometry.md) | function | `(world: World, desc?: CircleGeometryDesc): Result<Geometry>` | Make a circle geometry for og.Mesh (one-sided, as ringGeometry). |
| [`og.render.tubeGeometry`](render/fn-tubeGeometry.md) | function | `(world: World, desc?: TubeGeometryDesc): Result<Geometry>` | Make a tube along a curve through points for og.Mesh: pipes, cables, rails, a path shown as a ribbon of geometry. |
| [`ShapePoints`](render/type-ShapePoints.md) | struct |  | A shape as points, what og.Shape.extractPoints returns and what shapeGeometry, extrudeGeometry and sweepGeometry read: the outline, then each hole. |
| [`ShapeGeometryDesc`](render/type-ShapeGeometryDesc.md) | desc struct |  | A flat shape with holes in the xy plane facing +Z, as three.js ShapeGeometry(shapes, curveSegments): triangulated by earcut (three.js's own port), the outline made clockwise and the holes counter-clo… |
| [`ExtrudeGeometryDesc`](render/type-ExtrudeGeometryDesc.md) | desc struct |  | Shapes pushed along +Z, or along a path, as three.js ExtrudeGeometry(shapes, options) with its option names and defaults: a closed solid, its lids from earcut and its walls ring by ring, rounded by a… |
| [`LatheGeometryDesc`](render/type-LatheGeometryDesc.md) | desc struct |  | A profile turned round +Y, as three.js LatheGeometry(points, segments, phiStart, phiLength): vases, bottles, columns, wheels. |
| [`PolyhedronGeometryDesc`](render/type-PolyhedronGeometryDesc.md) | desc struct |  | A polyhedron from vertices and faces, every face subdivided `detail` times and every vertex pushed onto the sphere of `radius`, as three.js PolyhedronGeometry(vertices, indices, radius, detail): unin… |
| [`SolidGeometryDesc`](render/type-SolidGeometryDesc.md) | desc struct |  | A regular solid, as three.js IcosahedronGeometry, DodecahedronGeometry, OctahedronGeometry and TetrahedronGeometry(radius, detail): PolyhedronGeometryDesc with the solid's own vertices and faces. |
| [`SweepKey`](render/type-SweepKey.md) | struct |  | A value at a fraction of a sweep's path: linear between keys, held before the first and past the last. |
| [`SweepGeometryDesc`](render/type-SweepGeometryDesc.md) | desc struct |  | A section swept along a path, scaled and twisted over it: horns, tusks, handles, blades, tapered pipes, twisted columns (img2threejs's tapered sweep; three.js has no class for it). |
| [`MirrorAxis`](render/type-MirrorAxis.md) | enum |  | The world axis a part is mirrored along, after its placement. |
| [`GeometryPart`](render/type-GeometryPart.md) | desc struct |  | A geometry placed as an Object3D is (scale, then rotate, then move), then mirrored: one part of og.render.mergeGeometries or an operand of og.render.csgGeometry. |
| [`CsgOp`](render/type-CsgOp.md) | enum |  | A boolean operation on two solids. |
| [`CsgGeometryDesc`](render/type-CsgGeometryDesc.md) | desc struct |  | Two closed geometries combined by a boolean operation, each placed first: a box with a hole bored through it, a dice, a window cut in a wall. |
| [`MergeGeometriesDesc`](render/type-MergeGeometriesDesc.md) | desc struct |  | Geometries placed and merged into one, as three.js BufferGeometryUtils.mergeGeometries after applying each part's matrix: a prop of parts drawn in one draw, a mirrored half, an array (the same part a… |
| [`og.render.shapeGeometry`](render/fn-shapeGeometry.md) | function | `(world: World, desc?: ShapeGeometryDesc): Result<Geometry>` | Make a flat shape for og.Mesh (one-sided, as circleGeometry): an og.Shape with holes, or a points array. |
| [`og.render.extrudeGeometry`](render/fn-extrudeGeometry.md) | function | `(world: World, desc?: ExtrudeGeometryDesc): Result<Geometry>` | Make an extruded solid for og.Mesh: a three.js extrusion is a paste (`og.render.extrudeGeometry(world, { shapes: shape, depth: 0.2, bevelEnabled: true })`). |
| [`og.render.latheGeometry`](render/fn-latheGeometry.md) | function | `(world: World, desc?: LatheGeometryDesc): Result<Geometry>` | Make a solid of revolution for og.Mesh: a profile turned round +Y. |
| [`og.render.polyhedronGeometry`](render/fn-polyhedronGeometry.md) | function | `(world: World, desc?: PolyhedronGeometryDesc): Result<Geometry>` | Make a polyhedron from vertices and faces for og.Mesh, subdivided onto a sphere. |
| [`og.render.icosahedronGeometry`](render/fn-icosahedronGeometry.md) | function | `(world: World, desc?: SolidGeometryDesc): Result<Geometry>` | Make an icosahedron (twenty faces) for og.Mesh; with detail, a geodesic sphere. |
| [`og.render.dodecahedronGeometry`](render/fn-dodecahedronGeometry.md) | function | `(world: World, desc?: SolidGeometryDesc): Result<Geometry>` | Make a dodecahedron (twelve pentagons, thirty-six triangles) for og.Mesh. |
| [`og.render.octahedronGeometry`](render/fn-octahedronGeometry.md) | function | `(world: World, desc?: SolidGeometryDesc): Result<Geometry>` | Make an octahedron (eight faces) for og.Mesh. |
| [`og.render.tetrahedronGeometry`](render/fn-tetrahedronGeometry.md) | function | `(world: World, desc?: SolidGeometryDesc): Result<Geometry>` | Make a tetrahedron (four faces) for og.Mesh. |
| [`og.render.sweepGeometry`](render/fn-sweepGeometry.md) | function | `(world: World, desc?: SweepGeometryDesc): Result<Geometry>` | Make a section swept along a path for og.Mesh, tapered and twisted by keys. |
| [`og.render.csgGeometry`](render/fn-csgGeometry.md) | function | `(world: World, desc?: CsgGeometryDesc): Result<Geometry>` | Make the union, difference or intersection of two placed geometries for og.Mesh. |
| [`og.render.mergeGeometries`](render/fn-mergeGeometries.md) | function | `(world: World, desc?: MergeGeometriesDesc): Result<Geometry>` | Make one geometry of several placed ones for og.Mesh: one draw instead of many, mirrored halves, arrays. |
| [`LineMode`](render/type-LineMode.md) | enum |  | How a line set's points join, as three.js's three line objects. |
| [`LineMode()`](#column-LineMode) | column kind |  | A LineMode field: the variant's index. |
| [`LinesDesc`](render/type-LinesDesc.md) | desc struct |  | Lines through points, drawn this frame only (immediate mode, as og.render.sprite): submit them every tick they should show. |
| [`og.render.lines`](render/fn-lines.md) | function | `(world: World, desc?: LinesDesc): Result<void>` | Draw lines this frame. |
| [`PointsDesc`](render/type-PointsDesc.md) | desc struct |  | A point cloud drawn this frame only (immediate mode), as three.js Points with a PointsMaterial: a square per point facing the camera. |
| [`og.render.points`](render/fn-points.md) | function | `(world: World, desc?: PointsDesc): Result<void>` | Draw a point cloud this frame. |
| [`og.Lines`](render/record-Lines.md) | component |  | Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a… |
| [`og.Points`](render/record-Points.md) | component |  | Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. |
| [`RaycastGeometryDesc`](render/type-RaycastGeometryDesc.md) | desc struct |  | Which meshes og.render.raycast tests, as three.js Raycaster's near, far and layers. |
| [`GeometryHit`](render/type-GeometryHit.md) | struct |  | Where a ray meets a drawn triangle, as three.js Raycaster's intersection. |
| [`og.render.raycast`](render/fn-raycast.md) | function | `(world: World, ray: Ray, desc?: RaycastGeometryDesc): Result<readonly GeometryHit[]>` | Every drawn triangle a ray crosses, nearest first, as three.js Raycaster.intersectObjects: what is drawn, not the colliders (og.physics.raycast answers those). |
| [`og.render.light2dAt`](render/fn-light2dAt.md) | function | `(world: World, point: Vector2, layers?: number): Result<Color>` | The 2D light reaching a point, for game rules (a stealth game's "is the player lit"): the sum of every og.Light2D whose layers share a bit with `layers`, each its color times intensity times its fall… |
| [`RenderTargetFormat`](render/type-RenderTargetFormat.md) | enum |  | How a render target's colour is stored. |
| [`RenderTargetFormat()`](#column-RenderTargetFormat) | column kind |  | A RenderTargetFormat field: the variant's index. |
| [`RenderTarget`](render/type-RenderTarget.md) | handle |  | A texture a camera draws into (og.render.renderTarget), as three.js WebGLRenderTarget. |
| [`RenderTarget()`](#column-RenderTarget) | column kind |  | A RenderTarget handle field (0 is none). |
| [`RenderTargetDesc`](render/type-RenderTargetDesc.md) | desc struct |  | A render target's size and storage, as three.js WebGLRenderTarget's width, height, type and depthTexture. |
| [`RenderTargetInfo`](render/type-RenderTargetInfo.md) | struct |  | What og.render.renderTarget made. |
| [`og.render.renderTarget`](render/fn-renderTarget.md) | function | `(world: World, desc?: RenderTargetDesc): Result<RenderTargetInfo>` | Make a render target: a texture an og.CameraView or og.Mirror draws into, its colour (RenderTargetInfo.texture) usable by any map, sprite or UI image, as three.js WebGLRenderTarget and its texture. |
| [`og.CameraView`](render/record-CameraView.md) | component |  | A secondary camera, as a second three.js camera rendered with setRenderTarget or setViewport: it draws the scene from the entity's MatrixWorld (looking down its local -Z, as og.Camera) into a render … |
| [`og.Mirror`](render/record-Mirror.md) | component |  | A planar mirror: the main camera reflected in the entity's xy plane (its local +Z the mirror's facing side), drawn into target fitted to a width by height rect centred on the entity, as a mirrored og… |
| [`Post`](render/type-Post.md) | handle |  | A custom post pass from og.load.post (an `.ogt` of `type post`): a WGSL `post` function over the lit image and the parameters it reads. |
| [`Post()`](#column-Post) | column kind |  | A Post handle field (0 is none). |
| [`og.render.setPostParam`](render/fn-setPostParam.md) | function | `(world: World, post: Post, name: string, value: readonly number[]): Result<void>` | Set a parameter of a custom post pass, by the name its asset declares: a number for float, two to four for vec2, vec3, vec4 and color. |
| [`PostError`](render/type-PostError.md) | struct |  | A custom post pass the device failed to compile. |
| [`og.render.postErrors`](render/fn-postErrors.md) | function | `(world: World): readonly PostError[]` | The custom post passes PostEffects.custom names that failed to compile on this device, each skipped (the frame draws as without it). |
| [`VolumetricFog`](render/type-VolumetricFog.md) | enum |  | How the device draws Fog.volumetric (RenderCapabilities.volumetricFog). |
| [`og.RectLight`](render/record-RectLight.md) | component |  | A rectangle of light, as three.js RectAreaLight: width by height metres centred on the entity in its local xy plane, facing along its local -Z (aim it with transform.lookAt, as a spot), emitting colo… |
| [`og.LightVolume`](render/record-LightVolume.md) | component |  | A box of baked light probes (an irradiance volume): og bake lightmaps traces the light arriving from the scene (bounced off its static models, from the sky, from emissive surfaces) at a grid of point… |
| [`og.RealtimeLightVolume`](render/record-RealtimeLightVolume.md) | component |  | Relights the LightVolume on the same entity while the game runs, so its probes follow moving lights, opening doors, a sun crossing the sky and switched lamps: each probe is relit by capturing the sce… |
| [`BakedScene`](render/type-BakedScene.md) | struct |  | One instantiated scene's baked lighting, as og.render.bakedLighting reports it. |
| [`og.render.bakedLighting`](render/fn-bakedLighting.md) | function | `(world: World): readonly BakedScene[]` | The baked lighting of every scene instantiated in the world, in instantiation order: whether it is drawn, and why not. |

## Column kinds

Each is a factory, `Geometry(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Geometry"></a>`Geometry` | `Geometry` | `Uint32Array` | 2 | A Geometry handle field. |
| <a id="column-Material"></a>`Material` | `Material` | `Uint32Array` | 2 | A Material handle field. |
| <a id="column-Upscaler"></a>`Upscaler` | `Upscaler` | `Uint32Array` | 1 | An Upscaler field: the variant's index. |
| <a id="column-ToneMapping"></a>`ToneMapping` | `ToneMapping` | `Uint32Array` | 1 | A ToneMapping field: the variant's index. |
| <a id="column-AoResolutionSetting"></a>`AoResolutionSetting` | `AoResolutionSetting` | `Uint32Array` | 1 | An AoResolutionSetting field: the variant's index. |
| <a id="column-QualityPreset"></a>`QualityPreset` | `QualityPreset` | `Uint32Array` | 1 | A QualityPreset field: the variant's index. |
| <a id="column-Texture"></a>`Texture` | `Texture` | `Uint32Array` | 2 | A Texture handle field (0 is none). |
| <a id="column-Lut"></a>`Lut` | `Lut` | `Uint32Array` | 2 | A Lut handle field (0 is none). |
| <a id="column-EnvironmentMap"></a>`EnvironmentMap` | `EnvironmentMap` | `Uint32Array` | 2 | An EnvironmentMap handle field (0 is none). |
| <a id="column-ParticleEmitter"></a>`ParticleEmitter` | `ParticleEmitter` | `Uint32Array` | 2 | A ParticleEmitter handle field (0 is none). |
| <a id="column-AoMethod"></a>`AoMethod` | `AoMethod` | `Uint32Array` | 1 | An AoMethod field: the variant's index. |
| <a id="column-AoResolution"></a>`AoResolution` | `AoResolution` | `Uint32Array` | 1 | An AoResolution field: the variant's index. |
| <a id="column-DofQuality"></a>`DofQuality` | `DofQuality` | `Uint32Array` | 1 | A DofQuality field: the variant's index. |
| <a id="column-ScreenBlur"></a>`ScreenBlur` | `ScreenBlur` | `Uint32Array` | 1 | A ScreenBlur field: the variant's index. |
| <a id="column-SpriteBlend"></a>`SpriteBlend` | `SpriteBlend` | `Uint32Array` | 1 | A SpriteBlend field: the variant's index. |
| <a id="column-SpriteFacing"></a>`SpriteFacing` | `SpriteFacing` | `Uint32Array` | 1 | A SpriteFacing field: the variant's index. |
| <a id="column-StripBlend"></a>`StripBlend` | `StripBlend` | `Uint32Array` | 1 | A StripBlend field: the variant's index. |
| <a id="column-GiMethod"></a>`GiMethod` | `GiMethod` | `Uint32Array` | 1 | A GiMethod field: the variant's index. |
| <a id="column-Palette"></a>`Palette` | `Palette` | `Uint32Array` | 2 | A Palette handle field (0 is none). |
| <a id="column-Look"></a>`Look` | `Look` | `Uint32Array` | 2 | A Look handle field (0 is none): a look og.load.look answered, kept for og.render.applyLook. |
| <a id="column-DitherPattern"></a>`DitherPattern` | `DitherPattern` | `Uint32Array` | 1 | A DitherPattern field: the variant's index. |
| <a id="column-MaterialSurface"></a>`MaterialSurface` | `MaterialSurface` | `Uint32Array` | 1 | A MaterialSurface field: the variant's index. |
| <a id="column-OutlinePlace"></a>`OutlinePlace` | `OutlinePlace` | `Uint32Array` | 1 | An OutlinePlace field: the variant's index. |
| <a id="column-StyliseSpace"></a>`StyliseSpace` | `StyliseSpace` | `Uint32Array` | 1 | A StyliseSpace field: the variant's index. |
| <a id="column-PosterizeMode"></a>`PosterizeMode` | `PosterizeMode` | `Uint32Array` | 1 | A PosterizeMode field: the variant's index. |
| <a id="column-HalftoneMode"></a>`HalftoneMode` | `HalftoneMode` | `Uint32Array` | 1 | A HalftoneMode field: the variant's index. |
| <a id="column-HalftoneRegion"></a>`HalftoneRegion` | `HalftoneRegion` | `Uint32Array` | 1 | A HalftoneRegion field: the variant's index. |
| <a id="column-Light2DKind"></a>`Light2DKind` | `Light2DKind` | `Uint32Array` | 1 | A Light2DKind field: the variant's index. |
| <a id="column-ReflectionProbeMode"></a>`ReflectionProbeMode` | `ReflectionProbeMode` | `Uint32Array` | 1 | A ReflectionProbeMode field: the variant's index. |
| <a id="column-Shader"></a>`Shader` | `Shader` | `Uint32Array` | 2 | A Shader handle field (0 is none): a material shader og.load.shader answered, kept for og.render.shaderMaterial. |
| <a id="column-LineMode"></a>`LineMode` | `LineMode` | `Uint32Array` | 1 | A LineMode field: the variant's index. |
| <a id="column-RenderTargetFormat"></a>`RenderTargetFormat` | `RenderTargetFormat` | `Uint32Array` | 1 | A RenderTargetFormat field: the variant's index. |
| <a id="column-RenderTarget"></a>`RenderTarget` | `RenderTarget` | `Uint32Array` | 2 | A RenderTarget handle field (0 is none). |
| <a id="column-Post"></a>`Post` | `Post` | `Uint32Array` | 2 | A Post handle field (0 is none). |

```ogt
column Geometry {
  doc "A Geometry handle field."
  array Uint32Array
  stride 2
  value Geometry
}
```

<sub>`api.ogt` line 3801</sub>

```ogt
column Material {
  doc "A Material handle field."
  array Uint32Array
  stride 2
  value Material
}
```

<sub>`api.ogt` line 3808</sub>

```ogt
column Upscaler {
  doc "An Upscaler field: the variant's index."
  array Uint32Array
  stride 1
  value Upscaler
}
```

<sub>`api.ogt` line 3861</sub>

```ogt
column ToneMapping {
  doc "A ToneMapping field: the variant's index."
  array Uint32Array
  stride 1
  value ToneMapping
}
```

<sub>`api.ogt` line 4323</sub>

```ogt
column AoResolutionSetting {
  doc "An AoResolutionSetting field: the variant's index."
  array Uint32Array
  stride 1
  value AoResolutionSetting
}
```

<sub>`api.ogt` line 4344</sub>

```ogt
column QualityPreset {
  doc "A QualityPreset field: the variant's index."
  array Uint32Array
  stride 1
  value QualityPreset
}
```

<sub>`api.ogt` line 4359</sub>

```ogt
column Texture {
  doc "A Texture handle field (0 is none)."
  array Uint32Array
  stride 2
  value Texture
}
```

<sub>`api.ogt` line 4515</sub>

```ogt
column Lut {
  doc "A Lut handle field (0 is none)."
  array Uint32Array
  stride 2
  value Lut
}
```

<sub>`api.ogt` line 4522</sub>

```ogt
column EnvironmentMap {
  doc "An EnvironmentMap handle field (0 is none)."
  array Uint32Array
  stride 2
  value EnvironmentMap
}
```

<sub>`api.ogt` line 4529</sub>

```ogt
column ParticleEmitter {
  doc "A ParticleEmitter handle field (0 is none)."
  array Uint32Array
  stride 2
  value ParticleEmitter
}
```

<sub>`api.ogt` line 4596</sub>

```ogt
column AoMethod {
  doc "An AoMethod field: the variant's index."
  array Uint32Array
  stride 1
  value AoMethod
}
```

<sub>`api.ogt` line 4611</sub>

```ogt
column AoResolution {
  doc "An AoResolution field: the variant's index."
  array Uint32Array
  stride 1
  value AoResolution
}
```

<sub>`api.ogt` line 4624</sub>

```ogt
column DofQuality {
  doc "A DofQuality field: the variant's index."
  array Uint32Array
  stride 1
  value DofQuality
}
```

<sub>`api.ogt` line 4637</sub>

```ogt
column ScreenBlur {
  doc "A ScreenBlur field: the variant's index."
  array Uint32Array
  stride 1
  value ScreenBlur
}
```

<sub>`api.ogt` line 4652</sub>

```ogt
column SpriteBlend {
  doc "A SpriteBlend field: the variant's index."
  array Uint32Array
  stride 1
  value SpriteBlend
}
```

<sub>`api.ogt` line 4665</sub>

```ogt
column SpriteFacing {
  doc "A SpriteFacing field: the variant's index."
  array Uint32Array
  stride 1
  value SpriteFacing
}
```

<sub>`api.ogt` line 4679</sub>

```ogt
column StripBlend {
  doc "A StripBlend field: the variant's index."
  array Uint32Array
  stride 1
  value StripBlend
}
```

<sub>`api.ogt` line 4692</sub>

```ogt
column GiMethod {
  doc "A GiMethod field: the variant's index."
  array Uint32Array
  stride 1
  value GiMethod
}
```

<sub>`api.ogt` line 5092</sub>

```ogt
column Palette {
  doc "A Palette handle field (0 is none)."
  array Uint32Array
  stride 2
  value Palette
}
```

<sub>`api.ogt` line 5162</sub>

```ogt
column Look {
  doc "A Look handle field (0 is none): a look og.load.look answered, kept for og.render.applyLook."
  array Uint32Array
  stride 2
  value Look
}
```

<sub>`api.ogt` line 5169</sub>

```ogt
column DitherPattern {
  doc "A DitherPattern field: the variant's index."
  array Uint32Array
  stride 1
  value DitherPattern
}
```

<sub>`api.ogt` line 5186</sub>

```ogt
column MaterialSurface {
  doc "A MaterialSurface field: the variant's index."
  array Uint32Array
  stride 1
  value MaterialSurface
}
```

<sub>`api.ogt` line 5201</sub>

```ogt
column OutlinePlace {
  doc "An OutlinePlace field: the variant's index."
  array Uint32Array
  stride 1
  value OutlinePlace
}
```

<sub>`api.ogt` line 5215</sub>

```ogt
column StyliseSpace {
  doc "A StyliseSpace field: the variant's index."
  array Uint32Array
  stride 1
  value StyliseSpace
}
```

<sub>`api.ogt` line 5229</sub>

```ogt
column PosterizeMode {
  doc "A PosterizeMode field: the variant's index."
  array Uint32Array
  stride 1
  value PosterizeMode
}
```

<sub>`api.ogt` line 5242</sub>

```ogt
column HalftoneMode {
  doc "A HalftoneMode field: the variant's index."
  array Uint32Array
  stride 1
  value HalftoneMode
}
```

<sub>`api.ogt` line 5255</sub>

```ogt
column HalftoneRegion {
  doc "A HalftoneRegion field: the variant's index."
  array Uint32Array
  stride 1
  value HalftoneRegion
}
```

<sub>`api.ogt` line 5269</sub>

```ogt
column Light2DKind {
  doc "A Light2DKind field: the variant's index."
  array Uint32Array
  stride 1
  value Light2DKind
}
```

<sub>`api.ogt` line 5649</sub>

```ogt
column ReflectionProbeMode {
  doc "A ReflectionProbeMode field: the variant's index."
  array Uint32Array
  stride 1
  value ReflectionProbeMode
}
```

<sub>`api.ogt` line 6406</sub>

```ogt
column Shader {
  doc "A Shader handle field (0 is none): a material shader og.load.shader answered, kept for og.render.shaderMaterial."
  array Uint32Array
  stride 2
  value Shader
}
```

<sub>`api.ogt` line 6463</sub>

```ogt
column LineMode {
  doc "A LineMode field: the variant's index."
  array Uint32Array
  stride 1
  value LineMode
}
```

<sub>`api.ogt` line 7256</sub>

```ogt
column RenderTargetFormat {
  doc "A RenderTargetFormat field: the variant's index."
  array Uint32Array
  stride 1
  value RenderTargetFormat
}
```

<sub>`api.ogt` line 7466</sub>

```ogt
column RenderTarget {
  doc "A RenderTarget handle field (0 is none)."
  array Uint32Array
  stride 2
  value RenderTarget
}
```

<sub>`api.ogt` line 7475</sub>

```ogt
column Post {
  doc "A Post handle field (0 is none)."
  array Uint32Array
  stride 2
  value Post
}
```

<sub>`api.ogt` line 7611</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-PARTICLES_ARMED"></a>`PARTICLES_ARMED` | `number` | `4294967295` | ParticleSystem.startTick's default: never written. A looping emitter emits as if started at tick 0; a one-shot (`looping false`) emits nothing until its startTick is written, so a burst spawned in `start` waits for its first use. Write it back to re-arm a one-shot without firing it. |
| <a id="const-MAX_LIGHTS"></a>`MAX_LIGHTS` | `number` | `256` | The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept. |
| <a id="const-MAX_CLUSTER_LIGHTS"></a>`MAX_CLUSTER_LIGHTS` | `number` | `64` | The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there. |
| <a id="const-UI_LAYER"></a>`UI_LAYER` | `number` | `1000` | The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the screen transition included (a HUD element made of sprites sitting on a UI panel: draw it only while its screen shows), in their layers' order either way. |
| <a id="const-MAX_REFLECTION_PROBES"></a>`MAX_REFLECTION_PROBES` | `number` | `8` | The most og.ReflectionProbe components one frame samples, nearest the camera first; the rest are skipped. |

```ogt
const PARTICLES_ARMED u32 4294967295 "ParticleSystem.startTick's default: never written. A looping emitter emits as if started at tick 0; a one-shot (`looping false`) emits nothing until its startTick is written, so a burst spawned in `start` waits for its first use. Write it back to re-arm a one-shot without firing it."
```

<sub>`api.ogt` line 5773</sub>

```ogt
const MAX_LIGHTS u32 256 "The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept."
```

<sub>`api.ogt` line 6359</sub>

```ogt
const MAX_CLUSTER_LIGHTS u32 64 "The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there."
```

<sub>`api.ogt` line 6360</sub>

```ogt
const UI_LAYER i32 1000 "The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the screen transition included (a HUD element made of sprites sitting on a UI panel: draw it only while its screen shows), in their layers' order either way."
```

<sub>`api.ogt` line 6361</sub>

```ogt
const MAX_REFLECTION_PROBES u32 8 "The most og.ReflectionProbe components one frame samples, nearest the camera first; the rest are skipped."
```

<sub>`api.ogt` line 6398</sub>
