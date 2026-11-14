//! RenderScene's frame data by group: plain data, no GPU types, so a
//! format crate can take each group whole.

use super::*;

/// The frame's look: its post effects and stylise, the custom post passes
/// and their parameters, and the colour filter.
#[derive(Debug)]
pub struct SceneLook {
    pub post: PostEffects,
    /// Stylisation after (and, for depth bands, with) the post stack: off
    /// by default ([`crate::Stylise`]).
    pub stylise: crate::Stylise,
    /// Custom post passes ([`RenderScene::add_post`], `post_custom.rs`) and
    /// their parameters, by handle.
    pub posts: Vec<crate::PostShader>,
    pub post_params: Vec<[[f32; 4]; crate::MAX_SHADER_PARAMS]>,
    /// The custom post passes the frame runs, in order (at most
    /// [`crate::MAX_CUSTOM_POSTS`]), after the built-in effects and before
    /// exposure, bloom and the tone curve (`PostEffects.custom` in the API).
    pub custom_post: Vec<crate::PostHandle>,
    /// The player's colour vision filter over the finished frame, scene and
    /// overlay alike (`None`: none; crate::ColorFilter).
    pub color_filter: Option<crate::ColorFilter>,
    /// The fog of war over the lit image (`post/sight.rs`; `None`: none).
    pub fog_of_war: Option<crate::FogOfWar>,
}

/// The overlay (its UI tree and canvases) and the UI's vector paths and
/// materials.
#[derive(Debug)]
pub struct SceneUi {
    /// Screen-space sprites and text over the finished frame (`None`: none).
    pub overlay: Option<Overlay>,
    /// Cooked vectors the overlay's UI draws ([`RenderScene::add_ui_vector`]).
    pub ui_vectors: Vec<crate::ui::UiVectorData>,
    /// Custom materials UI layers composite through
    /// ([`RenderScene::add_ui_material`]).
    pub ui_materials: Vec<crate::ui::UiMaterialShader>,
}

/// The air and the weather: fog, volumetric fog, clouds, aurora, the eye's
/// adaptation, the surface weather, the wind and what pushes the grass.
#[derive(Debug)]
pub struct SceneAtmosphere {
    /// Fog over the main pass (`None`: none).
    pub fog: Option<crate::Fog>,
    /// Fog lit as a participating medium (`volumetric.rs`; `None`: none).
    /// It replaces [`RenderScene::fog`] over the frame while it is set
    /// (and [`crate::RenderQuality::volumetric_fog`] lets it).
    pub volumetric_fog: Option<crate::VolumetricFog>,
    /// Fog inside regions (og.FogVolume), drawn in the volumetric medium:
    /// in [`Self::volumetric_fog`], or alone over [`Self::fog`] where the
    /// scene has none.
    pub fog_volumes: Vec<crate::FogVolume>,
    /// Clouds in the procedural sky (`clouds.rs`; `None`: none). Drawn
    /// only with [`crate::Background::Procedural`].
    pub clouds: Option<crate::Clouds>,
    /// An aurora in the procedural sky (`aurora.rs`; `None`: none). Drawn
    /// only with [`crate::Background::Procedural`].
    pub aurora: Option<crate::Aurora>,
    /// How much of the image-based light's strength
    /// ([`crate::Environment::intensity`]) is the eye's adaptation to shade
    /// rather than the sky's own light: the day-night cycle's twilight lift
    /// ([`crate::DayNightLook::adaptation`]). It lights surfaces, but a
    /// mirror of the sky (the sea's reflection) divides it out, so it shows
    /// the sky as bright as the sky is drawn. 1: none.
    pub adaptation: f32,
    /// Rain, puddles and snow on the lit surfaces (`None`: dry).
    pub surface_weather: Option<crate::SurfaceWeather>,
    /// The wind that sways the scattered copies (`None` or strength 0:
    /// still).
    pub wind: Option<crate::Wind>,
    /// Local pushes on the simulated wind field (og.WindMotor,
    /// `wind_field.rs`): the nearest [`crate::wind_field::MAX_MOTORS`] drive it.
    pub wind_motors: Vec<crate::WindMotor>,
    /// What presses the grass and scattered copies (og.GrassPusher,
    /// `trample.rs`): the nearest [`crate::trample::MAX_PUSHERS`] press.
    pub grass_pushers: Vec<crate::GrassPusher>,
}

/// The 3D lighting: the sun and the local lights, their cookies and the
/// baked light and light volumes, the reflection probes, the hemisphere and
/// the environment.
#[derive(Debug)]
pub struct SceneLighting {
    pub light: DirectionalLight,
    /// Point and spot lights; the first [`MAX_LIGHTS`] are drawn.
    pub lights: Vec<Light>,
    /// Light cookies of point and spot lights, by index into
    /// [`RenderScene::lights`] (`area.rs`).
    pub light_cookies: Vec<crate::LightCookie>,
    /// Baked lighting: lightmap pages, the lightmap uvs of the meshes drawn
    /// with them, light volumes (`lightmap.rs`). Empty by default.
    pub baked: crate::BakedLighting,
    /// Light volumes relit while the game runs (`probe_grid.rs`), sampled
    /// over the baked ones; the first [`crate::PROBE_VOLUMES`] with the
    /// baked ones fit. Empty by default.
    pub light_volumes: Vec<crate::RealtimeLightVolume>,
    /// Reflection probes: boxes reflecting a cube captured inside them
    /// instead of the environment (the first [`crate::MAX_REFLECTION_PROBES`]).
    pub reflection_probes: Vec<crate::ReflectionProbe>,
    /// A hemisphere light, as three.js `HemisphereLight`: diffuse-only
    /// ambient irradiance `mix(ground, sky, 0.5 n.y + 0.5) * intensity`
    /// (sky, ground linear RGB; intensity 0: off). Unlike light baked into
    /// the environment map it adds no reflection.
    pub hemisphere: ([f32; 3], [f32; 3], f32),
    pub environment: Environment,
    /// A cooked environment map lighting the scene instead of
    /// [`Environment::sky`] (three.js `scene.environment`), drawn as the
    /// background by [`Background::Sky`]; `None`: the sky lights it.
    pub environment_map: Option<crate::EnvironmentMap>,
}

/// The 3D world's ground and waters: water, terrains and their stamps,
/// scattered copies, grass, 3D fluids and the colliders they meet.
#[derive(Debug)]
pub struct SceneWorld {
    /// Planar 3D water surfaces, drawn after the opaque scene and the sky.
    pub water: Vec<crate::Water>,
    /// Terrains drawn this frame (clipmaps of [`RenderScene::add_terrain`] data).
    pub terrains: Vec<crate::Terrain>,
    /// Meshes drawn as part of this frame's terrains (cliffs, arches,
    /// overhangs placed as terrain stamps): shaded with their terrain's
    /// layers and blended into its ground (`terrain.rs`).
    pub terrain_stamps: Vec<crate::TerrainStampDraw>,
    /// Copies scattered over this frame's terrains, placed and drawn by
    /// the renderer from cached chunks.
    pub scatters: Vec<crate::ScatterDraw>,
    /// Fields of grass over this frame's terrains, generated and drawn by
    /// the renderer on the GPU (`grass.rs`).
    pub grass: Vec<crate::GrassDraw>,
    /// 3D particle fluids, drawn as one screen-space surface after the
    /// opaque scene, the sky and water.
    pub fluids_3d: Vec<crate::Fluid3d>,
    /// The scene's solid shapes ([`crate::SceneCollider`]): what
    /// CPU-simulated particles with collision bounce off (the CPU path's
    /// stand-in for the depth buffer the compute path reads) and what
    /// visual bodies ([`crate::VisualBody`]) meet on the GPU. Empty by
    /// default.
    pub colliders: Vec<crate::SceneCollider>,
}

/// The effects drawn among the scene: particles and their history, trails,
/// ribbons and beams, effect meshes, lines and points (each with its
/// layers), and the decals with their atlas.
#[derive(Debug)]
pub struct SceneEffects {
    /// Running particle systems (state is kept by their id).
    pub particles: Vec<ParticleSystem>,
    /// Each running system's ticks by its id, as each tick ended
    /// ([`crate::ParticleHistory`]): the ticks a system has not been drawn
    /// at step as they were. A system absent here steps by the frame's
    /// system alone.
    pub particle_history: og_core::IdMap<u64, std::sync::Arc<crate::ParticleHistory>>,
    /// Systems the host simulated itself (its tick ran
    /// [`crate::CpuParticles::advance`] to read their events), by
    /// [`ParticleSystem::id`]: the renderer draws these particles and their
    /// sub-emitters' as they are instead of simulating them, so what is
    /// drawn is what the game read. A system absent here simulates as usual.
    pub hosted_particles: Vec<(u64, std::sync::Arc<crate::CpuParticles>)>,
    /// The same for `particles` (billboards and 2D sprite particles alike).
    pub particle_layers: Vec<u32>,
    pub trails: Vec<Trail>,
    /// Trails drawn in the sprite order at a layer (2D) instead of with the
    /// 3D scene: `(Trail::id, layer)`.
    pub trail_layers: Vec<(u64, i32)>,
    pub ribbons: Vec<Ribbon>,
    pub beams: Vec<Beam>,
    /// Meshes drawn with an effect material this frame (`effect.rs`).
    pub effect_meshes: Vec<crate::EffectMesh>,
    /// Line sets drawn this frame (`lines.rs`): in the scene or over the
    /// finished frame (debug draw, screen lines).
    pub lines: Vec<crate::Lines>,
    /// The same for `lines` and `points` (scene sets; overlay and screen
    /// sets draw on the main frame whatever their masks).
    pub line_layers: Vec<u32>,
    /// Point clouds drawn this frame (`points.rs`).
    pub points: Vec<crate::Points>,
    pub point_layers: Vec<u32>,
    /// Projected decals (the first [`crate::MAX_DECALS`] are drawn).
    pub decals: Vec<Decal>,
    /// The texture decals take their `uv_rect` from (`None`: solid colour).
    pub decal_atlas: Option<TextureHandle>,
}

/// The 2D scene: sprites and texts with their order, outlines, shadows,
/// shading, layers, light masks and facing, tile layers, sprite arrays,
/// sprite effects and meshes, the 2D lighting, lights and occluders, 2D
/// water and metaballs.
#[derive(Debug)]
pub struct SceneSprites {
    /// 2D quads drawn after the 3D scene, blended, sorted by layer.
    pub sprites: Vec<Sprite>,
    /// MSDF text, sorted with the sprites by layer.
    pub texts: Vec<Text>,
    /// Sort keys of single sprites within their layer (top-down y sorting):
    /// a layer draws its sprites by ascending key, then texture, then
    /// submission order; a sprite without one has key 0.
    pub sprite_order: Vec<SpriteOrder>,
    /// Outlines of single world texts, by text index.
    pub text_outlines: Vec<TextOutline>,
    /// Tile layers: chunked quads drawn through the sprite batch, each
    /// under the sprites of its layer.
    pub tile_layers: Vec<TileLayer>,
    /// Sprites drawn from a layer of a texture array instead of their
    /// texture, by sprite index (world sprites only; the last entry for an
    /// index wins). Sprites of one array draw as one run whatever their
    /// layers, so a sheet of frames in an array batches across frames.
    pub sprite_arrays: Vec<SpriteArray>,
    /// Drop shadows of single world sprites, by sprite index (the last
    /// entry for an index wins): each a flattened dark copy of the sprite
    /// on the ground plane y = 0 cast along [`RenderScene::light`], drawn
    /// before every sprite of its layer.
    pub sprite_shadows: Vec<SpriteShadow>,
    /// Pixel outlines of single world sprites, by sprite index (the last
    /// entry for an index wins; [`SpriteOutline`]).
    pub sprite_outlines: Vec<SpriteOutline>,
    /// Material shaders of single world sprites, by sprite index (the last
    /// entry for an index wins; [`SpriteShading`]).
    pub sprite_shading: Vec<SpriteShading>,
    /// The same for the world `sprites` (with their drop shadows, glow and
    /// afterimages drawn from them).
    pub sprite_layers: Vec<u32>,
    /// Each world sprite's 2D light layers (`og.LightMask2D`), by index in
    /// `sprites`: lit by the [`crate::Light2d`]s whose `layers` share a
    /// bit with it. An index past the end is 1 (layer 0, every light's
    /// default). A frame lights at most [`crate::MAX_LIGHT_MASKS_2D`]
    /// different masks apart; a further one is lit as mask 1.
    pub sprite_light_masks: Vec<u32>,
    /// How each world sprite faces the camera, by index in `sprites`
    /// (billboards): an index past the end is [`SpriteFacing::None`], the
    /// xy plane. A faced sprite without a [`RenderScene::sprite_order`] key
    /// (and tone mapped) draws among the transparent meshes, back to front
    /// with them by the view depth of its anchor from the eye that draws
    /// it, its drop shadow before them; with a key it keeps the sprites'
    /// (layer, key) order. Culled by its bounds (a sphere round the anchor
    /// through the farthest corner) against that camera's frustum. Its
    /// drop shadow is cast by the quad stood upright facing the sun, so it
    /// shows the sprite's whole shape from every camera.
    pub sprite_facing: Vec<SpriteFacing>,
    /// World texts turned to the camera ([`TextFacing`], by text index; the
    /// last entry for an index wins; a text without one stays in the xy
    /// plane), as [`RenderScene::sprite_facing`] turns sprites.
    pub text_facing: Vec<TextFacing>,
    /// Effects on single sprites (flash, glow, blend), by sprite index.
    pub sprite_fx: Vec<SpriteFx>,
    /// Textured triangles drawn in the sprite order (a 2D skeleton's
    /// attachments): [`crate::SpriteMesh`].
    pub sprite_meshes: Vec<crate::SpriteMesh>,
    /// 2D lighting of the sprite layers (`None`: sprites are unlit).
    pub lighting_2d: Option<Lighting2d>,
    pub lights_2d: Vec<Light2d>,
    pub occluders_2d: Vec<Occluder2d>,
    /// Side-on 2D water bodies, drawn in the sprite order.
    pub water_2d: Vec<crate::Water2d>,
    /// 2D fluids drawn as metaballs in the sprite order (the first
    /// [`crate::MAX_FLUIDS`]; fluids of one group are one surface).
    pub metaballs: Vec<crate::Metaballs>,
}

/// The 3D objects: mesh instances (layers, levels, previous transforms),
/// skinned instances with their palettes, morphs and baked clips, crowds,
/// deformed vertices, impostors, debris, and how meshes are drawn (meshlets,
/// LOD bias and debug, visibility buffer, cloth subdivisions).
#[derive(Debug)]
pub struct SceneObjects {
    /// Objects to draw this frame, in any order.
    pub instances: Vec<Instance>,
    /// Three.js layer masks of the instances, by index in `instances`
    /// (bit `n` set: on layer `n`): an index past the end is on layer 0
    /// alone (mask 1), as every object is in three.js. A camera draws the
    /// instances whose mask shares a bit with its own
    /// ([`RenderScene::camera_layers`], [`crate::View::layers`]); the shadow
    /// maps take every caster whatever its layers, so a body hidden from a
    /// first-person camera still casts its shadow.
    pub instance_layers: Vec<u32>,
    /// The level each clustered instance draws on the meshlet path, by
    /// instance index (same order as `instances`), when its caller picked
    /// it: og-runtime picks one level per model by its largest error and
    /// keeps it near a boundary (`lod.rs` [`crate::lod::select`] with
    /// hysteresis), so the meshlet path draws what the regular path draws
    /// for its parts. Every view and shadow pass draws that level (a
    /// level past the mesh's coarsest draws the coarsest); `u8::MAX`, or
    /// an instance past the end, leaves it to the renderer, which picks
    /// each instance's level by its own screen-space error per view.
    pub instance_levels: Vec<u8>,
    /// Each instance's transform in the previous frame (same order as
    /// `instances`), for per-object motion blur. Ignored unless its length
    /// equals `instances`'; then only the camera's motion blurs.
    pub previous_transforms: Vec<Mat4>,
    pub skinned_instances: Vec<SkinnedInstance>,
    /// The same for `skinned_instances`.
    pub skinned_layers: Vec<u32>,
    /// This frame's joint palettes ([`SkinnedInstance::palette`] indexes them).
    pub joint_palettes: Vec<JointPalette>,
    /// Each joint palette's joints as the previous frame drew them, in
    /// the world (the transform its instances were drawn at times each
    /// joint), same order as `joint_palettes`: TAA's motion vectors of
    /// skinned meshes. Ignored unless its length equals
    /// `joint_palettes`'; a palette whose joint count differs, or a mesh
    /// with morph targets alone, moves with the camera only.
    pub previous_joint_palettes: Vec<JointPalette>,
    /// Each skinned instance's blendshape weights as the previous frame drew
    /// them, same order as `skinned_instances`: TAA's motion vectors of a
    /// morphing mesh (the palette buffer's previous section, `skin.rs`).
    /// Ignored unless its length equals `skinned_instances`'; then the
    /// morph moves with the camera only.
    pub previous_morph_weights: Vec<[f32; crate::MAX_MORPH_TARGETS]>,
    /// Animation clips baked into joint palettes, played on the GPU by
    /// skinned instances with [`crate::SkinnedInstance::baked`] (crowds):
    /// kept from frame to frame; a frame uploads only the clips its
    /// instances use (`skin.rs`, the palette buffer's layout).
    pub baked_clips: Vec<crate::BakedClip>,
    /// Crowds: thousands of copies of one skinned model, each posed by a
    /// baked clip, culled, given a level and drawn on the GPU-driven path
    /// (`crowd.rs`).
    pub crowds: Vec<crate::Crowd>,
    /// The vertices of skinned instances that bring their own
    /// ([`crate::SkinnedInstance::deformed`] indexes it): a simulated
    /// garment's points, in each instance's space.
    pub deformed_vertices: Vec<crate::DeformedVertices>,
    /// Each `deformed_vertices` entry as the previous frame drew it, same
    /// order, in this frame's instance space: TAA's motion vectors of a
    /// swinging garment (the palette buffer's previous section, `skin.rs`).
    /// An entry whose count differs, or a list whose length differs, moves
    /// with the camera and the instance only.
    pub previous_deformed_vertices: Vec<crate::DeformedVertices>,
    /// Models drawn as their impostors this frame (`impostor.rs`): each a
    /// camera-facing card, drawn with the scattered copies' passes.
    pub impostors: Vec<crate::ImpostorDraw>,
    /// Whether instances of clustered meshes draw through the meshlet path
    /// (`meshlet.rs`): their level picked and their clusters culled on the
    /// GPU. True by default; false draws them as any mesh, level 0.
    pub meshlets: bool,
    /// The scene's level-of-detail bias for clustered meshes and scattered
    /// copies (`lod.rs`'s bias: 2 keeps detail twice as far, 0 the full
    /// mesh), 1 by default.
    pub lod_bias: f32,
    /// Colours each clustered instance by the level it draws (green the
    /// full mesh, yellow, orange, red: [`crate::lod::debug_color`]), as
    /// og-runtime's `debugView lod` colours the others.
    pub lod_debug: bool,
    /// Whether the meshlet path rasterises the camera's small clusters of
    /// opaque materials in compute into a visibility buffer and shades
    /// them in one resolve per batch (`meshlet.rs`); true by default.
    /// False draws every cluster by the hardware.
    pub visibility_buffer: bool,
    /// Draw segments per coarse cloth/soft-body edge; zero selects two, 1–8.
    pub cloth_subdivisions: u32,
    /// Fine pieces of broken props, simulated and drawn on the GPU only
    /// ([`crate::DebrisPiece`], `debris.rs`): each drawn by instances of
    /// [`RenderScene::instances`] whose transforms the GPU writes. They
    /// collide with [`RenderScene::colliders`]. Empty by default.
    pub debris: Vec<crate::DebrisPiece>,
}

/// The frame's clock and view: its tick and rate, interpolation, the camera
/// (its cut and layers), the viewport views, the world origin, the debug
/// view, the quality, the camera shake and the CSS height.
#[derive(Debug)]
pub struct SceneFrame {
    /// The game's tick (drives beams' scroll and film grain; particle
    /// systems and trails carry their own).
    pub tick: u64,
    /// Ticks per second (the fixed step of particles and trails).
    pub tick_rate: f32,
    /// A display frame drawn between the last two ticks (render
    /// interpolation): its alpha, 0 at the tick before `tick` and 1 at
    /// `tick`, as the blended transforms are drawn; `None` draws `tick`.
    /// The sea's clock runs at it, so waves move at the display's rate and
    /// stay in step with the floaters drawn on them.
    pub interpolation: Option<f32>,
    pub camera: Camera,
    /// The camera cut into this frame (a teleport or a cut between shots):
    /// motion blur takes no camera motion from the frame before, and TAA
    /// starts from this frame's samples ([`crate::Renderer::reset_history`]).
    pub camera_cut: bool,
    /// The main camera's layer mask (three.js `Camera.layers`): it draws
    /// the instances, skinned instances, world sprites, scene lines and
    /// points and particle systems whose masks share a bit with it, as each
    /// [`crate::View`] does with its own. 1 by default; 0 draws none of
    /// them. What carries no mask (the sky, terrain, scattered copies,
    /// water, trails, text, tile layers and sprite meshes) draws on every
    /// camera.
    pub camera_layers: u32,
    /// Secondary cameras drawn into render targets or viewports of the
    /// frame, before it in [`crate::View::order`] (`views.rs`).
    pub views: Vec<crate::View>,
    /// Where the zero of the scene's coordinates is in the caller's world
    /// ([`RenderScene::rebase`], `origin.rs`): the frame is drawn relative
    /// to it, and the shaders that read world position for its look add
    /// it back. Zero by default.
    pub origin: Vec3,
    pub debug: DebugView,
    /// What the frame may cost: switches and sizes over what the rest of
    /// the scene asks for (`quality.rs`). The default draws all of it.
    pub quality: crate::RenderQuality,
    /// Screen shake applied to the camera.
    pub shake: CameraShake,
    /// The output's height in CSS pixels, which line widths and point sizes
    /// are measured in (0: one CSS pixel per device pixel).
    pub css_height: f32,
}
