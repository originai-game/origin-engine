//! The world → `RenderScene` extraction and the renderer a host attached.
//!
//! Rendering reads the world after the tick and never writes it back
//! (AGENTS.md: rendering never feeds back into simulation). Everything here
//! takes the world shared, so the probe's `Capture` hook can extract the
//! frame of the current tick from the `&World` it is handed.
//!
//! What is drawn:
//! - entities with `og.Model` (a loaded glTF's default scene) at their world
//!   transform, composed from `og.Transform` down the `og.Parent` chain;
//! - entities with `og.Mesh` (geometry and material handles), when og-core
//!   can register its bool fields;
//! - the first `og.DirectionalLight` (else the renderer's default sun), the
//!   `og.PointLight` / `og.SpotLight` (og-render culls and clusters them);
//! - the `og.Camera` and `og.RenderSettings` resources;
//! - the text `og.render.text` queued during the tick, with cooked
//!   multi-page MSDF fonts mapped to one `MsdfFont` per page.

mod frame;
mod hierarchy;
mod wind;

use crate::{
    author::review, bind::modules, bind::skeleton2d, bind::terrain, bind::video, present::art,
    present::atmosphere, present::baked, present::environment, present::feel, present::fx,
    present::geometry, present::glyphs, present::interpolate, present::probes, present::views,
    world::assets,
};
pub(crate) use frame::extract;
pub use hierarchy::Hierarchy;
pub(crate) use hierarchy::{Blend, blend};

use crate::host::codec::{Cells, Col, ColRow, Layout, Rec, Slot as FieldSlot};
use crate::host::wire::FromJson;
use glam::{Mat4, Quat, Vec3};
use og_core::IdMap;
use og_ecs::{Entity, QueryDesc, World};
use og_render::{FontHandle, MaterialHandle, MeshHandle, RenderScene};
use serde_json::Value;
use std::collections::BTreeMap;

/// A renderer the host attached to an engine (an OffscreenCanvas surface on
/// the web, an offscreen target on native).
pub trait RenderBackend {
    /// The IDL `RenderBackend` (`webgpu`, `metal`, `vulkan`, `vulkan-cpu`,
    /// `dx12`), og-render's `tier_name`, which also names the
    /// reference frames.
    fn tier(&self) -> String;
    /// The render target in device pixels.
    fn size(&self) -> (u32, u32);
    /// What `RenderOptions.antialias` resolved to: `off`, `msaa4`, `taa`
    /// or `fxaa` ([`antialias_name`]).
    fn antialias(&self) -> String {
        "off".into()
    }
    /// The pointer's shape the UI resolved this frame ([`crate::host::ui_cursor`]),
    /// given every frame: a backend that shows the frames in a window (the
    /// native host's shared frame) hands it on when it changes.
    fn cursor(&mut self, shape: &crate::host::ui_cursor::CursorShape) {
        let _ = shape;
    }
    /// The frames drawn so far and the frames TAA's history holds
    /// (og-render's `Renderer::history`); `None` for a backend without one.
    fn history(&self) -> Option<(u64, u64)> {
        None
    }
    /// What the renderer runs on and resolved its options to (the
    /// capabilities' shadow limits); `None` for a backend without one.
    fn info(&self) -> Option<og_render::RenderInfo> {
        None
    }
    /// The texture encodings the device takes (og-render's
    /// `RenderInfo::texture_formats`): loads transcode to them. `None`: RGBA8.
    fn texture_formats(&self) -> Option<og_render::TextureFormats> {
        None
    }
    /// The renderer's device and queue, for compute that runs beside the
    /// frames (og.motion's model); `None` for a backend without a GPU.
    /// After a lost device is replaced ([`RenderBackend::recover`]) this
    /// answers the new one.
    fn compute(&self) -> Option<(og_render::wgpu::Device, og_render::wgpu::Queue)> {
        None
    }
    /// Draw one frame of the scene to the display target.
    fn render(&mut self, scene: &RenderScene) -> Result<(), String>;
    /// Times each render-graph pass's encoding with `clock` (og-render's
    /// `Renderer::set_clock`); a backend without passes ignores it.
    fn set_clock(&mut self, _clock: Option<fn() -> f64>) {}
    /// The last frame's passes in execution order, each with the start
    /// and length of its CPU encoding (og-render's
    /// `Renderer::pass_timings`); empty without a clock.
    fn pass_timings(&self) -> Vec<og_render::PassTiming> {
        Vec::new()
    }
    /// The frames timed on the GPU read back since the last call, oldest
    /// first (og-render's `Renderer::gpu_timings`; `wait` waits for those
    /// in flight, natively); empty without a clock or timestamp queries.
    fn gpu_timings(&mut self, _wait: bool) -> Vec<og_render::GpuFrame> {
        Vec::new()
    }
    /// Why passes are not timed on the GPU here (og-render's
    /// `Renderer::gpu_timing_absent`), or `None` when they are.
    fn gpu_timing_absent(&self) -> Option<&'static str> {
        Some("this backend has no GPU passes")
    }
    /// What the device said when it refused the visibility buffer's
    /// resolve (og-render's `Renderer::visibility_buffer_refused`), or
    /// `None`: `RenderCapabilities.visibilityBuffer`.
    fn visibility_buffer_refused(&self) -> Option<String> {
        None
    }
    /// What the last frame submitted (og-render's `FrameStats`): its
    /// triangles are `ProbeTimings.triangles`, the rest the render dump's.
    fn frame_stats(&self) -> og_render::FrameStats {
        og_render::FrameStats::default()
    }
    /// Reflection probe `id`'s capture read back (og-render's
    /// `Renderer::read_reflection_probe`): native hosts, for `og bake probes`.
    fn read_reflection_probe(&self, _id: u64) -> Option<Result<og_render::ProbeCapture, String>> {
        None
    }
    /// The custom post passes of `scene` the device failed to compile, by
    /// post index, with its message (og-render's `Renderer::post_errors`).
    fn post_errors(&self, _scene: &RenderScene) -> Vec<(u32, String)> {
        Vec::new()
    }
    /// The material shaders whose sprite pipelines failed to compile, by
    /// shader index, with the message (og-render's
    /// `Renderer::sprite_shader_errors`).
    fn sprite_shader_errors(&self) -> Vec<(usize, String)> {
        Vec::new()
    }
    /// Draw the scene offscreen at `width`×`height` (0×0: the target's size)
    /// and read it back. Native hosts block on the readback; the web host
    /// answers a frame it requested earlier (request on tick N, answer on
    /// N + 1) and `None` while the copy is in flight.
    fn capture(
        &mut self,
        scene: &RenderScene,
        width: u32,
        height: u32,
    ) -> Option<og_render::RgbaImage>;
    /// Whether `capture` answers on a later call (the web host): a probe
    /// waits for the frame instead of failing on the first `None`.
    fn deferred(&self) -> bool {
        false
    }
    /// Why the last `capture` answered `None` (a readback that timed out,
    /// a lost device), for the probe's message; `None` when it gave a frame
    /// or does not say.
    fn capture_error(&self) -> Option<String> {
        None
    }
    /// og-render's `Renderer::save_state` (the adapted exposure) as its
    /// number, for a probe snapshot; 0 keeps nothing.
    fn save_state(&mut self) -> u64 {
        0
    }
    /// og-render's `Renderer::restore_state` of what `save_state` answered.
    fn restore_state(&mut self, state: u64) {
        let _ = state;
    }
    /// og-render's `Renderer::warm`: make one pipeline no frame has used
    /// yet, in the host's idle time; false when there is none left.
    fn warm(&mut self) -> bool {
        false
    }
    /// The last frame `render` drew, read back without drawing it again
    /// (`og render`'s every-frame capture); `None` when nothing was drawn,
    /// or on the web until that frame's readback has landed (the canvas's
    /// frames are kept offscreen once a reader asked; ask again before the
    /// next frame).
    fn read_frame(&mut self) -> Option<og_render::RgbaImage> {
        None
    }
    /// Whether it presents to a display (the web canvas): only then are
    /// the render scale and automatic quality applied (`crate::present::display`);
    /// a native run draws at its fixed size and scale 1.
    fn presents(&self) -> bool {
        false
    }
    /// og-render's `Renderer::set_render_scale`, from the next frame.
    fn set_render_scale(&mut self, _scale: f32) {}
    /// The size the 3D scene was drawn at last (og-render's
    /// `Renderer::scene_size_drawn`).
    fn scene_size(&self) -> (u32, u32) {
        self.size()
    }
    /// The GPU memory the renderer holds (og-render's `Renderer::gpu_memory`).
    fn gpu_memory(&self) -> Option<og_render::GpuMemory> {
        None
    }
    /// Before a frame: when the device was lost, make the renderer again
    /// on a new one (natively at once; the web host asks for one and
    /// draws nothing until it lands). `Ok(true)` while there is no device
    /// to draw on. The runtime calls it before it extracts the frame, so
    /// textures the new device takes in other encodings are decoded again
    /// first (`crate::world::assets::retranscode`); `render` recovers too.
    fn recover(&mut self) -> Result<bool, String> {
        Ok(false)
    }
    /// How many times the device was lost and the renderer made again on
    /// a new one (each backend recovers in its own `render`).
    fn device_losses(&self) -> u32 {
        0
    }
    /// A test hook: destroy the device (og-render's `Renderer::lose_device`);
    /// false where there is none.
    fn lose_device(&mut self) -> bool {
        false
    }
}

/// One loaded font: a cooked MTSDF font as one `MsdfFont` per atlas page.
/// Every page's font carries every glyph's advance, and the plane bounds only
/// of its own page's glyphs (the others are blank), so drawing the same
/// string once per page lays every page out identically.
pub(crate) struct FontPages {
    pub pages: Vec<FontHandle>,
}

/// A queued `og.render.text`.
#[derive(Clone, Debug)]
pub(crate) struct QueuedText {
    pub font: usize,
    pub text: String,
    pub position: Vec3,
    pub size: f32,
    pub rotation: f32,
    pub color: [f32; 4],
    pub layer: i32,
    /// In the overlay (CSS pixels, y down), cut to this clip.
    pub screen: bool,
    pub clip: Option<[f32; 4]>,
    /// World text: an outline (width as a fraction of the distance range) and its colour.
    pub outline: f32,
    pub outline_color: [f32; 4],
    /// World text: a billboard (`TextDesc.facing`) turning about the point
    /// `align` of each line along it (`og_render::TextFacing`).
    pub facing: og_render::SpriteFacing,
    pub align: f32,
}

/// One model's parts: per part the mesh, the material and the node's
/// transform within the model.
pub(crate) type Parts = Vec<(MeshHandle, MaterialHandle, Mat4)>;

/// The coarser levels a scattered mesh draws (`og_render::ScatterDraw::lods`).
#[cfg(not(feature = "profile2d"))]
pub(crate) type ScatterLods = [Option<(MeshHandle, f32)>; 2];

/// A model's levels of detail (`og-render`'s `lod.rs` picks one per
/// entity): per part, the meshes of its coarser levels with their fraction
/// of the full mesh's triangles, and the model's bounding sphere in its
/// own space.
#[derive(Clone, Debug)]
pub(crate) struct ModelLods {
    /// Per part, its coarser levels' meshes with their errors in the
    /// model's units (og-cook's `Lod::error`, the part's node scale in).
    pub parts: Vec<Vec<(MeshHandle, f32)>>,
    pub center: Vec3,
    pub radius: f32,
    /// Raw primitive meshes paired with their placed, model-space scatter
    /// mesh and that mesh's coarser levels with their errors, placed alike:
    /// og.TerrainScatter's, so 3D only.
    #[cfg(not(feature = "profile2d"))]
    pub scatter_meshes: Vec<(MeshHandle, MeshHandle, ScatterLods)>,
    /// The model's impostor and its error: the level after the parts'
    /// coarsest (`og_render::Impostor`).
    pub impostor: Option<(og_render::Impostor, f32)>,
}

impl ModelLods {
    /// The model's error per level, level 0 first (0): the largest of its
    /// parts' at that level, a part with fewer levels standing at its
    /// coarsest; then its impostor's, when it has one. The impostor
    /// replaces the mesh levels whose error is no smaller than its own
    /// (it is the finer picture and the cheaper draw), so a model whose
    /// coarse levels simplify badly (a palm's fronds, metres off) still
    /// reaches its impostor rather than having its texel's error raised
    /// to theirs by `lod::select`.
    pub fn errors(&self) -> Vec<f32> {
        (0..self.mesh_levels())
            .map(|k| self.mesh_error(k))
            .chain(self.impostor.map(|i| i.1))
            .collect()
    }

    /// The level index of the impostor, when the model has one.
    pub fn impostor_level(&self) -> Option<usize> {
        self.impostor.map(|_| self.mesh_levels())
    }

    /// Mesh level `k`'s error (0: the full mesh, 0).
    fn mesh_error(&self, k: usize) -> f32 {
        if k == 0 {
            return 0.0;
        }
        self.parts
            .iter()
            .filter_map(|p| p.get(k - 1).or(p.last()))
            .map(|x| x.1)
            .fold(0.0, f32::max)
    }

    /// How many mesh levels the chain keeps (the full mesh counted): all
    /// of them, or with an impostor those finer than it.
    fn mesh_levels(&self) -> usize {
        let n = self.parts.iter().map(|p| p.len()).max().unwrap_or(0) + 1;
        let Some((_, limit)) = self.impostor else {
            return n;
        };
        let mut worst = 0.0f32;
        1 + (1..n)
            .take_while(|&k| {
                worst = worst.max(self.mesh_error(k));
                worst < limit
            })
            .count()
    }
}

/// The render side of an engine, shared with the probe's capture hook.
pub(crate) struct RenderState {
    pub scene: RenderScene,
    pub backend: Option<Box<dyn RenderBackend>>,
    /// Material handles (`og.render.standardMaterial`, glTF materials) by index.
    pub materials: Vec<MaterialHandle>,
    /// Geometry handles (glTF primitives) by index.
    pub geometries: Vec<MeshHandle>,
    /// Model parts by asset handle bits.
    pub models: IdMap<u64, Parts>,
    /// The models' levels of detail by asset handle bits (models with one
    /// level have none).
    pub model_lods: IdMap<u64, ModelLods>,
    /// The textures and meshes loaded models share, and the materials
    /// theirs draw in the batches of (assets.rs `insert_model`).
    pub shared: og_bundle::runtime::SharedParts,
    /// The level each og.Model entity drew last (by entity bits): the
    /// hysteresis of the next choice, and what dump_entity reports.
    pub lod_drawn: IdMap<u64, u8>,
    /// The models' wind (og.Model.sway): their tops and the wind copies
    /// of their materials (`wind.rs`).
    pub wind: wind::WindModels,
    /// The baked reflection probes' captures tried (`crate::present::probes`).
    pub probe_bakes: crate::present::probes::ProbeBakes,
    /// Fonts by Font handle index.
    pub fonts: Vec<FontPages>,
    /// Text queued during the current tick, and that tick.
    pub texts: Vec<QueuedText>,
    pub texts_tick: u64,
    /// Wall time the probe's screenshots took (render, read back, PNG)
    /// since the step began: not the game's step time.
    pub capture_ms: f64,
    /// The effects games declare as data (`crate::present::fx`), with the sprites
    /// and ribbons queued during the current tick.
    pub fx: crate::present::fx::FxState,
    /// Textures by Texture handle index (`og.load.texture`).
    pub textures: Vec<og_render::TextureHandle>,
    /// Texture arrays by TextureArray handle index (`og.load.textureArray`).
    pub texture_arrays: Vec<crate::world::assets::LoadedArray>,
    /// Videos by Video handle index (`og.load.video`), and og.VideoPlayer's layout.
    #[cfg(feature = "video")]
    pub videos: Vec<crate::bind::video::LoadedVideo>,
    #[cfg(feature = "video")]
    pub video_player: Option<Layout>,
    /// Layouts of the engine records extraction reads.
    pub camera: Option<Layout>,
    pub settings: Option<Layout>,
    pub draw: DrawLayouts,
    /// The game's RenderOptions (antialias, shadowMapSize).
    pub options: Value,
    /// The play surface the host reported (`set_viewport`); `None` uses the
    /// render target at one CSS pixel per device pixel.
    pub viewport: Option<og_ui::Viewport>,
    /// The viewport og.ui.previewViewport set (a preview, `og ui check`'s
    /// phone at 2 device pixels per CSS pixel): over the host's, the CSS
    /// surface the screen overlay is laid on ([`crate::present::fx::viewport`]), as
    /// the UI lays out on it.
    pub preview: Option<og_ui::Viewport>,
    /// The command channel's `cover`: where a host's control would cover
    /// the surface (`Viewport.covered`), over whatever the host reports,
    /// so a native run lays out as the published page with its Back does.
    pub cover: Option<og_ui::Rect>,
    /// The engine's UI, drawn over every frame.
    pub ui: Option<og_ui::SharedUi>,
    /// A UI font's first atlas page → every page, so text is drawn per page.
    pub ui_pages: BTreeMap<FontHandle, Vec<FontHandle>>,
    /// Fonts (by index into `fonts`) that render missing characters at run
    /// time from their file (`crate::present::glyphs`).
    pub runtime_fonts: IdMap<usize, crate::present::glyphs::RuntimeFont>,
    /// Warnings of the last extraction, for the engine's log.
    pub notes: Vec<String>,
    /// The engine this state draws (its id: the paths a river follows are
    /// its own, `timeline::path_of`).
    pub engine: u32,
    /// Each (font, character) a text drew, judged once a run: in the
    /// font, or warned of on the tick it was first drawn (`crate::present::glyphs`).
    pub glyphs_judged: std::collections::BTreeSet<(FontHandle, char)>,
    /// The next frame waits for every run-time glyph asked for (a capture
    /// is the frame a build that rendered them on the frame draws).
    pub glyphs_settle: bool,
    /// The inspector (development builds): its panel is drawn over the
    /// frame, above the game's UI.
    #[cfg(not(feature = "publish"))]
    pub inspector: Option<Box<crate::author::inspector::Inspector>>,
    /// The feel modules' drawing state (`crate::present::feel`).
    pub feel: crate::present::feel::FeelRender,
    /// The atmosphere's and the physics extras' drawing state (`crate::present::atmosphere`).
    pub atmo: crate::present::atmosphere::AtmoState,
    /// The effect materials loaded (`crate::present::effect`).
    pub effects: crate::present::effect::EffectState,
    /// The pixel terrains' and pieces' textures (`crate::bind::destruction2d`).
    #[cfg(feature = "destruction")]
    pub pixel: crate::bind::destruction2d::PixelRender,
    /// The environment maps loaded (`crate::present::environment`).
    pub env: crate::present::environment::EnvState,
    /// The terrains loaded (`crate::bind::terrain`).
    pub terrain: crate::bind::terrain::TerrainState,
    /// The last frame's hierarchy, kept for its allocations.
    pub hierarchy: Hierarchy,
    /// Where og.WorldOrigin put the world's zero for the last frame (R12):
    /// `scene.origin` is in the absolute world, so the world's place of a
    /// drawn position is it plus `scene.origin` less this.
    pub zero: Vec3,
    /// Runtime geometry, lines, points, debug draw (`crate::present::geometry`).
    pub geo: crate::present::geometry::GeoState,
    /// The entity of each drawable of the last extraction (`crate::author::describe`).
    pub ids: crate::author::describe::DrawIds,
    /// Render targets, secondary cameras and custom post passes (`crate::present::views`).
    pub views: crate::present::views::ViewState,
    /// Baked lighting of the scenes instantiated (`crate::present::baked`).
    pub baked: crate::present::baked::BakedState,
    /// The review camera in force (`view`, `turntable`: `crate::author::review`).
    pub review: Option<crate::author::review::ReviewView>,
    /// The art contract's runtime defaults (`crate::present::art`).
    pub art: crate::present::art::ArtState,
    /// The vectors and UI materials elements draw with (`crate::present::ui_draw`).
    pub ui_draw: crate::present::ui_draw::UiDrawState,
    /// Interpolation between ticks (`crate::present::interpolate`).
    pub interp: crate::present::interpolate::InterpState,
    /// How the frame is presented (`crate::present::display`).
    pub display: crate::present::display::DisplayState,
    /// The cooked textures in the renderer, decoded again for a new
    /// device's encodings (`crate::world::assets::retranscode`).
    pub decoded: crate::world::assets::Decoded,
    /// og.motion's model and the palettes it draws (`crate::bind::motion::present`).
    pub motion: crate::bind::motion::present::Presenter,
    /// Each particle system's ticks, by its id (`crate::present::presentation`).
    pub particle_history: IdMap<u64, std::sync::Arc<og_render::ParticleHistory>>,
    /// The presentation checkpoints kept for probe snapshots (`crate::present::presentation`).
    pub checkpoints: crate::present::presentation::Checkpoints,
}

impl RenderState {
    pub fn new(options: Value) -> RenderState {
        RenderState {
            engine: 0,
            scene: RenderScene::new(),
            backend: None,
            materials: Vec::new(),
            geometries: Vec::new(),
            models: IdMap::new(),
            model_lods: IdMap::new(),
            shared: Default::default(),
            lod_drawn: IdMap::new(),
            wind: Default::default(),
            probe_bakes: Default::default(),
            fonts: Vec::new(),
            texts: Vec::new(),
            texts_tick: 0,
            capture_ms: 0.0,
            fx: crate::present::fx::FxState::default(),
            textures: Vec::new(),
            texture_arrays: Vec::new(),
            #[cfg(feature = "video")]
            videos: Vec::new(),
            #[cfg(feature = "video")]
            video_player: None,
            camera: None,
            settings: None,
            draw: DrawLayouts::default(),
            options,
            viewport: None,
            preview: None,
            cover: None,
            ui: None,
            ui_pages: BTreeMap::new(),
            runtime_fonts: IdMap::new(),
            notes: Vec::new(),
            glyphs_judged: Default::default(),
            glyphs_settle: false,
            #[cfg(not(feature = "publish"))]
            inspector: None,
            feel: Default::default(),
            atmo: Default::default(),
            effects: Default::default(),
            #[cfg(feature = "destruction")]
            pixel: Default::default(),
            env: Default::default(),
            terrain: Default::default(),
            hierarchy: Hierarchy::default(),
            zero: Vec3::ZERO,
            geo: Default::default(),
            ids: Default::default(),
            views: Default::default(),
            baked: Default::default(),
            review: None,
            art: Default::default(),
            ui_draw: Default::default(),
            interp: Default::default(),
            display: Default::default(),
            decoded: Default::default(),
            motion: Default::default(),
            particle_history: IdMap::new(),
            checkpoints: Default::default(),
        }
    }
}

/// The IDL `AoResolutionSetting`'s variants, in order: RenderSettings.aoResolution.
const AO_RESOLUTION_SETTINGS: &[&str] = &["postEffects", "full", "half"];

/// RenderSettings.aoResolution's variant for a quality's AO resolution.
pub(crate) fn ao_resolution_setting(r: Option<og_render::AoResolution>) -> &'static str {
    AO_RESOLUTION_SETTINGS[match r {
        None => 0,
        Some(og_render::AoResolution::Full) => 1,
        Some(og_render::AoResolution::Half) => 2,
    }]
}

/// The IDL `Upscaler`'s variants, in order: RenderSettings.upscaler and
/// RenderCapabilities.upscalers.
pub(crate) const UPSCALERS: &[&str] = &["taau", "fsr3"];

/// RenderSettings.upscaler's variant for a quality's upscaler.
pub(crate) fn upscaler_setting(u: og_render::Upscaler) -> &'static str {
    UPSCALERS[match u {
        og_render::Upscaler::Taau => 0,
        og_render::Upscaler::Fsr3 => 1,
    }]
}

/// The value of a resource the world holds, decoded with its layout (kept
/// while its bytes are unchanged: [`Layout::decode_kept`]).
pub(crate) fn resource(world: &World, layout: Option<&Layout>) -> Option<std::sync::Arc<Value>> {
    let layout = layout?;
    let id = world.id_by_name(&layout.schema).ok()?;
    let bytes = world.resource_bytes(id).ok()?;
    Some(layout.decode_kept(bytes))
}

/// Every row of one component, read in place from its archetypes' columns
/// in query order ([`column_rows`]).
pub(crate) struct ColumnRows<'a> {
    layout: &'a Layout,
    /// Per archetype holding a row: its entities and their values.
    archetypes: Vec<(&'a [Entity], Cells<'a>)>,
}

impl ColumnRows<'_> {
    pub(crate) fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }

    /// `(entity bits, row)` for every row, in query order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (u64, ColRow<'_>)> + '_ {
        let layout = self.layout;
        self.archetypes.iter().flat_map(move |(entities, cells)| {
            entities
                .iter()
                .enumerate()
                .map(move |(i, e)| (e.to_bits(), ColRow::new(layout, cells, i)))
        })
    }
}

/// The rows of `layout`'s component, borrowed from the world's columns: no
/// row is copied or looked up by its entity (a packed copy of each row,
/// found through its entity, was most of reading 20,000 sprites or models).
/// An archetype one of whose columns a view holds mutably has no rows here,
/// as its rows had no bytes before.
pub(crate) fn column_rows<'a>(world: &'a World, layout: &'a Layout) -> ColumnRows<'a> {
    let mut archetypes = Vec::new();
    let q = QueryDesc {
        read: vec![layout.schema.clone()],
        ..QueryDesc::default()
    };
    if let (Ok(id), Ok(all)) = (world.id_by_name(&layout.schema), world.query(&q)) {
        for a in all {
            if a.is_empty() {
                continue;
            }
            let Ok(columns) = a.component_bytes(id) else {
                continue;
            };
            let fields = &layout.fields;
            let cells = if columns.len() == fields.len()
                && columns
                    .iter()
                    .zip(fields.iter())
                    .all(|(c, f)| c.len() == a.len() * f.ty.size())
            {
                Cells::Columns(
                    columns
                        .into_iter()
                        .zip(fields.iter())
                        .map(|(c, f)| Col::of(c, f))
                        .collect(),
                )
            } else {
                // Not the layout's fields: each row packed as the world
                // packs it, read by the layout's offsets as it was.
                let stride = columns.iter().map(|c| c.len()).sum::<usize>() / a.len();
                let mut bytes = Vec::with_capacity(stride * a.len());
                for row in 0..a.len() {
                    for c in &columns {
                        let n = c.len() / a.len();
                        bytes.extend_from_slice(&c[row * n..(row + 1) * n]);
                    }
                }
                Cells::Rows { bytes, stride }
            };
            archetypes.push((a.entities(), cells));
        }
    }
    ColumnRows { layout, archetypes }
}

/// Decoded rows of one component: `(entity bits, value)`.
pub(crate) fn rows(world: &World, layout: &Layout) -> Vec<(u64, Value)> {
    // The bytes are read serially (the world is one thread's), each row
    // packed into one buffer; decoding them is a lane per row natively, the
    // rows staying in entity order.
    let rows = column_rows(world, layout);
    let mut bytes = Vec::new();
    let cells: Vec<(u64, std::ops::Range<usize>)> = rows
        .iter()
        .map(|(e, row)| {
            let at = bytes.len();
            row.pack(&mut bytes);
            (e, at..bytes.len())
        })
        .collect();
    let bytes = bytes.as_slice();
    if cfg!(target_arch = "wasm32") {
        // A row's decode is allocations (a JSON map per row), and the lanes
        // wasm has one allocator behind one lock: 20,000 og.Mesh rows split
        // over eight Workers spent 116 ms of a 137 ms frame waiting for the
        // lanes (owed row 50, the former test sample
        return cells
            .iter()
            .map(|(e, at)| (*e, layout.decode(&bytes[at.clone()])))
            .collect();
    }
    og_lanes::map(og_lanes::Site::Extract, &cells, ROW_GRAIN, |(e, at)| {
        (*e, layout.decode(&bytes[at.clone()]))
    })
}

/// One `og.Mesh` as extraction reads it.
pub(crate) struct MeshRow {
    pub entity: u64,
    /// Indices into `RenderState::geometries` and `::materials`.
    pub geometry: usize,
    material: usize,
    pub visible: bool,
    flash: f32,
    flash_color: Vec3,
    /// `materialLayer`: the texture-array layer its material reads (-1:
    /// the material's own).
    layer: i32,
    /// `castShadow`: drawn into the shadow maps.
    cast: bool,
    /// `receiveShadow`: darkened by the shadow maps.
    receive: bool,
    /// `layers`: its three.js layer mask (`crate::present::views`).
    pub layers: u32,
}

/// A handle field's index, as the JSON rows read it (`as_f64`, then the
/// low 32 bits).
fn handle_index(bits: u64) -> usize {
    ((bits as f64) as u64 & 0xffff_ffff) as usize
}

/// The `og.Mesh` rows in query order, read straight from the columns (a
/// mesh field decoded to JSON per row cost about a microsecond natively and
/// several on wasm: owed row 50); the JSON rows when the columns are not
/// the IDL's.
pub(crate) fn mesh_rows(world: &World, layout: &Layout) -> Vec<MeshRow> {
    let columnar = || -> Option<Vec<MeshRow>> {
        let id = world.id_by_name(&layout.schema).ok()?;
        let field = |n: &str| world.field_by_id(id, n).ok();
        let (g, m, v, f, c) = (
            field("geometry")?,
            field("material")?,
            field("visible")?,
            field("flash")?,
            field("flashColor")?,
        );
        let l = field("materialLayer")?;
        let s = field("castShadow")?;
        let r = field("receiveShadow")?;
        let ly = field("layers");
        let q = QueryDesc {
            read: vec![layout.schema.clone()],
            ..QueryDesc::default()
        };
        let mut out = Vec::new();
        for a in world.query(&q).ok()? {
            let (g, m, v, f, c) = (
                a.read::<u64>(g).ok()?,
                a.read::<u64>(m).ok()?,
                a.read::<og_core::reflect::Bool8>(v).ok()?,
                a.read::<f32>(f).ok()?,
                a.read::<Vec3>(c).ok()?,
            );
            let l = a.read::<i32>(l).ok()?;
            let s = a.read::<og_core::reflect::Bool8>(s).ok()?;
            let r = a.read::<og_core::reflect::Bool8>(r).ok()?;
            let ly = match ly {
                Some(ly) => Some(a.read::<u32>(ly).ok()?),
                None => None,
            };
            out.reserve(a.len());
            for (i, e) in a.entities().iter().enumerate() {
                out.push(MeshRow {
                    entity: e.to_bits(),
                    geometry: handle_index(g[i]),
                    material: handle_index(m[i]),
                    visible: v[i].get(),
                    flash: f[i],
                    flash_color: c[i],
                    layer: l[i],
                    cast: s[i].get(),
                    receive: r[i].get(),
                    layers: ly.as_ref().map_or(1, |ly| ly[i]),
                });
            }
        }
        Some(out)
    };
    if let Some(rows) = columnar() {
        return rows;
    }
    rows(world, layout)
        .into_iter()
        .filter_map(|(e, v)| {
            let v = crate::host::wire::record::Mesh::from_json(&v).ok()?;
            let index = |h: u64| (h & 0xffff_ffff) as usize;
            let c = &v.flash_color;
            Some(MeshRow {
                entity: e,
                geometry: index(v.geometry),
                material: index(v.material),
                visible: v.visible,
                flash: v.flash,
                flash_color: Vec3::new(c.r, c.g, c.b),
                layer: v.material_layer,
                cast: v.cast_shadow,
                receive: v.receive_shadow,
                layers: v.layers,
            })
        })
        .collect()
}

/// Rows per lane task when decoding a component's rows natively: a row decodes in
/// about 0.6 us, so a task is a few hundred microseconds of work and a
/// component splits from 1,024 rows (500 rows stay serial).
const ROW_GRAIN: usize = 512;

/// The layouts of the drawing components the engine registered.
#[derive(Default)]
pub(crate) struct DrawLayouts {
    pub model: Option<Layout>,
    pub mesh: Option<Layout>,
    pub directional: Option<Layout>,
    pub point: Option<Layout>,
    pub spot: Option<Layout>,
    /// og.RectLight (area lights).
    pub rect: Option<Layout>,
    /// The effect records (`crate::present::fx`).
    pub fx: crate::present::fx::FxLayouts,
    /// The atmosphere's and the physics extras' records (`crate::present::atmosphere`).
    pub atmo: crate::present::atmosphere::AtmoLayouts,
    /// og.Environment (`crate::present::environment`).
    pub environment: Option<Layout>,
    /// og.Terrain and og.TerrainScatter (`crate::bind::terrain`; the 2D
    /// profile draws neither).
    #[cfg(not(feature = "profile2d"))]
    pub terrain: Option<Layout>,
    #[cfg(not(feature = "profile2d"))]
    pub scatter: Option<Layout>,
    #[cfg(not(feature = "profile2d"))]
    pub grass: Option<Layout>,
    #[cfg(not(feature = "profile2d"))]
    pub stamp: Option<Layout>,
    /// og.GrassPusher (`crate::bind::terrain`, the trample map).
    #[cfg(not(feature = "profile2d"))]
    pub grass_pusher: Option<Layout>,
    /// og.GrassType (the kinds og.Grass fields list in `types`).
    #[cfg(not(feature = "profile2d"))]
    pub grass_type: Option<Layout>,
    /// og.Collider: what CPU-simulated particles bounce off.
    pub collider: Option<Layout>,
    /// og.Lines and og.Points (`crate::present::geometry`).
    pub lines: Option<Layout>,
    pub points: Option<Layout>,
    /// og.ReflectionProbe (`crate::present::probes`).
    pub probe: Option<Layout>,
    /// og.LightVolume and the lazy og.RealtimeLightVolume beside it
    /// (`crate::present::probes::extract_light_volumes`).
    pub light_volume: Option<Layout>,
    pub realtime_volume: Option<Layout>,
    /// og.Interpolation (`crate::present::interpolate`).
    pub interpolation: Option<Layout>,
    /// og.WorldOrigin (R12): the render origin's grid is in the absolute world.
    pub origin: Option<Layout>,
    /// og.CameraView and og.Mirror (`crate::present::views`).
    pub views: crate::present::views::ViewLayouts,
    /// The engine's physics, for the debug view (`crate::present::geometry`).
    pub physics: Option<og_physics::PhysicsSystem>,
    pub engine: u32,
}

/// A cooked font as one `MsdfFont` per atlas page (Render's question 1: the
/// mapping lives here). Pages are uploaded linear, without mips.
pub(crate) fn add_font(
    scene: &mut RenderScene,
    font: &og_bundle::runtime::CookedFont,
) -> FontPages {
    let meta = &font.meta;
    let line_height = meta.ascender - meta.descender + meta.line_gap;
    let kerning: Vec<og_render::Kern> = font
        .kerning
        .iter()
        .filter_map(|k| {
            Some(og_render::Kern {
                left: char::from_u32(k.left)?,
                right: char::from_u32(k.right)?,
                adjust: k.adjust,
            })
        })
        .collect();
    // The distance-field pages; the colour glyphs' silhouettes (drawing
    // their shadows, glow and outline, before and so under their images);
    // the colour glyphs' pages (a plain sRGB atlas with its mips, drawn as
    // sprites: a distance range of 0); those in the text's colour. Each
    // entry: its data, the blob page whose rows it draws, and its kind
    // (colour, foreground, silhouette).
    let (n, c) = (font.pages.len(), font.color_pages.len());
    let fields = font
        .color_fields
        .iter()
        .enumerate()
        .map(|(i, d)| (d, n + i))
        .chain(
            font.foreground_fields
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + c + i)),
        );
    let all: Vec<(&og_render::TextureData, usize, bool, bool, bool)> = font
        .pages
        .iter()
        .enumerate()
        .map(|(i, d)| (d, i, false, false, false))
        .chain(fields.map(|(d, p)| (d, p, false, false, true)))
        .chain(
            font.color_pages
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + i, true, false, false)),
        )
        .chain(
            font.foreground_pages
                .iter()
                .enumerate()
                .map(|(i, d)| (d, n + c + i, true, true, false)),
        )
        .collect();
    let mut pages = Vec::with_capacity(all.len());
    for (data, p, colour, foreground, silhouette) in all {
        let atlas = scene.add_texture(if colour || silhouette {
            data.clone()
        } else {
            og_render::TextureData {
                mips: og_render::Mips::None,
                color_space: og_render::ColorSpace::Linear,
                ..data.clone()
            }
        });
        let glyphs = font
            .glyphs
            .iter()
            .filter_map(|g| {
                let key = g.key()?;
                let own = g.page as usize == p;
                Some(og_render::Glyph {
                    key,
                    advance: g.advance,
                    plane_bounds: if own {
                        [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top]
                    } else {
                        [0.0; 4]
                    },
                    atlas_bounds: if own {
                        [
                            g.atlas_x as f32,
                            g.atlas_y as f32,
                            (g.atlas_x + g.atlas_w) as f32,
                            (g.atlas_y + g.atlas_h) as f32,
                        ]
                    } else {
                        [0.0; 4]
                    },
                })
            })
            .collect();
        pages.push(scene.add_font(og_render::MsdfFont {
            atlas,
            distance_range: if colour {
                0.0
            } else if silhouette {
                og_bundle::format::COLOR_FIELD_RANGE
            } else {
                meta.px_range
            },
            foreground,
            silhouette,
            line_height,
            glyphs,
            kerning: kerning.clone(),
        }));
    }
    FontPages { pages }
}

/// A multi-page font draws once per page with one layout: each overlay text
/// from `first` on whose font has more pages (`pages`, by its first) is
/// drawn again with each, its effects (shadows, outline, gradient fill: the
/// UI's `text_effects`, by text index) with it.
pub(crate) fn draw_other_pages(
    overlay: &mut og_render::Overlay,
    first: usize,
    pages: &std::collections::BTreeMap<og_render::FontHandle, Vec<og_render::FontHandle>>,
) {
    let effects: std::collections::HashMap<u32, og_render::UiTextEffects> = overlay
        .ui
        .text_effects
        .iter()
        .filter(|e| e.text as usize >= first)
        .map(|e| (e.text, e.clone()))
        .collect();
    let end = overlay.texts.len();
    for i in first..end {
        let t = &overlay.texts[i];
        let Some(rest) = pages.get(&t.font).and_then(|p| p.get(1..)) else {
            continue;
        };
        let t = t.clone();
        for &page in rest {
            let k = overlay.texts.len() as u32;
            overlay.texts.push(og_render::Text {
                font: page,
                ..t.clone()
            });
            if let Some(e) = effects.get(&(i as u32)) {
                overlay.ui.text_effects.push(og_render::UiTextEffects {
                    text: k,
                    ..e.clone()
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A text in a font of two pages is drawn again with the second, with
    /// the first's shadows and outline; a text in a one-page font, or one
    /// before `first`, is left alone.
    #[test]
    fn a_text_drawn_on_its_fonts_other_pages_keeps_its_effects() {
        let mut scene = RenderScene::new();
        let atlas = scene.add_texture(og_render::TextureData {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
            color_space: og_render::ColorSpace::Linear,
            mips: og_render::Mips::None,
        });
        let font = || og_render::MsdfFont {
            atlas,
            distance_range: 4.0,
            foreground: false,
            silhouette: false,
            line_height: 1.2,
            glyphs: Vec::new(),
            kerning: Vec::new(),
        };
        let (a, a2, b) = (
            scene.add_font(font()),
            scene.add_font(font()),
            scene.add_font(font()),
        );
        let mut overlay = og_render::Overlay::new(100.0, 100.0, 1.0);
        let text = og_render::Text::new;
        overlay.texts = vec![text(a, "before"), text(a, "styled"), text(b, "one page")];
        let shadow = og_render::UiTextShadow {
            color: [0.0, 0.0, 0.0, 1.0],
            offset: og_core::glam::Vec2::new(2.0, 2.0),
            blur: 3.0,
        };
        let effects = |text| og_render::UiTextEffects {
            text,
            shadows: vec![shadow],
            outline: None,
            fill: None,
            fill_box: [0.0; 4],
            transform: [1.0, 0.0, 0.0, 1.0],
        };
        overlay.ui.text_effects = vec![effects(0), effects(1)];
        let pages = BTreeMap::from([(a, vec![a, a2]), (b, vec![b])]);
        draw_other_pages(&mut overlay, 1, &pages);
        let drawn: Vec<_> = overlay
            .texts
            .iter()
            .map(|t| (t.font, t.text.as_str()))
            .collect();
        assert_eq!(
            drawn,
            [
                (a, "before"),
                (a, "styled"),
                (b, "one page"),
                (a2, "styled")
            ]
        );
        let on: Vec<u32> = overlay.ui.text_effects.iter().map(|e| e.text).collect();
        assert_eq!(on, [0, 1, 3]);
        assert_eq!(overlay.ui.text_effects[2].shadows, [shadow]);
    }

    /// A model's impostor is the level after its parts' coarsest finer
    /// than it: chosen by `lod::select` once its error (an atlas texel)
    /// projects under a pixel, and replacing the mesh levels coarser than
    /// it.
    #[test]
    fn the_impostor_ends_the_level_chain() {
        let mut scene = RenderScene::new();
        let mesh = scene.add_mesh(og_render::shapes::cuboid(Vec3::ONE));
        let impostor = og_render::Impostor {
            material: scene.add_standard_material(Default::default()),
            grid: 8,
            center: Vec3::ZERO,
            radius: 1.0,
            material_atlas: false,
        };
        let lods = ModelLods {
            // One part with levels 3 cm and 12 cm off, another with one.
            parts: vec![vec![(mesh, 0.03), (mesh, 0.12)], vec![(mesh, 0.02)]],
            center: Vec3::ZERO,
            radius: 1.0,
            #[cfg(not(feature = "profile2d"))]
            scatter_meshes: Vec::new(),
            impostor: Some((impostor, 0.05)),
        };
        // 5 cm: finer than level 2 (12 cm), which it replaces; drawn once
        // it would show under a pixel (20 px per unit), level 1 nearer.
        assert_eq!(lods.errors(), vec![0.0, 0.03, 0.05]);
        assert_eq!(lods.impostor_level(), Some(2));
        assert_eq!(og_render::lod::select(&lods.errors(), 30.0, 1.0, None), 1);
        assert_eq!(og_render::lod::select(&lods.errors(), 19.0, 1.0, None), 2);
        // Finer than every mesh level (a palm's, whose coarse fronds are
        // metres off): it follows the full mesh directly.
        let fine = ModelLods {
            impostor: Some((impostor, 2.0 / 128.0)),
            ..lods.clone()
        };
        assert_eq!(fine.errors(), vec![0.0, 2.0 / 128.0]);
        assert_eq!(fine.impostor_level(), Some(1));
        assert_eq!(og_render::lod::select(&fine.errors(), 60.0, 1.0, None), 1);
        // Coarser than every one: after the coarsest.
        let coarse = ModelLods {
            impostor: Some((impostor, 0.5)),
            ..lods.clone()
        };
        assert_eq!(coarse.errors(), vec![0.0, 0.03, 0.12, 0.5]);
        assert_eq!(coarse.impostor_level(), Some(3));
        let none = ModelLods {
            impostor: None,
            ..lods
        };
        assert_eq!(none.errors().len(), 3);
        assert_eq!(none.impostor_level(), None);
    }
}
