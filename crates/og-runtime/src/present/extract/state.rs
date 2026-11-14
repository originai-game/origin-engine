//! What extraction keeps between frames: fonts, queued text, models' levels of detail and the render state.

use super::*;

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
    pub(crate) fn mesh_error(&self, k: usize) -> f32 {
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
    pub(super) fn mesh_levels(&self) -> usize {
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
