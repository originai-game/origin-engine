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

use super::codec::{Layout, Rec, Row, Slot as FieldSlot};
use glam::{Mat4, Quat, Vec3};
use og_ecs::{Entity, QueryDesc, World};
use og_render::{FontHandle, MaterialHandle, MeshHandle, RenderScene};
use serde_json::Value;
use std::collections::BTreeMap;

/// A renderer the host attached to an engine (an OffscreenCanvas surface on
/// the web, an offscreen target on native).
pub trait RenderBackend {
    /// The IDL `RenderTier` (`webgpu`, `webgl2`, `metal`, `vulkan`,
    /// `vulkan-cpu`), og-render's `tier_name`, which also names the
    /// reference frames.
    fn tier(&self) -> String;
    /// The render target in device pixels.
    fn size(&self) -> (u32, u32);
    /// What `RenderOptions.antialias` resolved to: `off`, `msaa4`, `taa`
    /// or `fxaa` ([`antialias_name`]).
    fn antialias(&self) -> String {
        "off".into()
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
    /// Draw one frame of the scene to the display target.
    fn render(&mut self, scene: &RenderScene) -> Result<(), String>;
    /// The last frame's text characters its fonts lack (og-render's
    /// `FrameStats::missing_glyphs`).
    fn missing_glyphs(&self) -> u32 {
        0
    }
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
    /// The last frame's main-pass triangles (og-render's
    /// `FrameStats::triangles`): `ProbeTimings.triangles`.
    fn triangles(&self) -> u64 {
        0
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
}

/// One model's parts: per part the mesh, the material and the node's
/// transform within the model.
pub(crate) type Parts = Vec<(MeshHandle, MaterialHandle, Mat4)>;

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
}

impl ModelLods {
    /// The model's error per level, level 0 first (0): the largest of its
    /// parts' at that level, a part with fewer levels standing at its
    /// coarsest.
    pub fn errors(&self) -> Vec<f32> {
        let n = self.parts.iter().map(|p| p.len()).max().unwrap_or(0);
        std::iter::once(0.0)
            .chain((0..n).map(|k| {
                self.parts
                    .iter()
                    .filter_map(|p| p.get(k).or(p.last()))
                    .map(|x| x.1)
                    .fold(0.0, f32::max)
            }))
            .collect()
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
    pub models: BTreeMap<u64, Parts>,
    /// The models' levels of detail by asset handle bits (models with one
    /// level have none).
    pub model_lods: BTreeMap<u64, ModelLods>,
    /// The level each og.Model entity drew last (by entity bits): the
    /// hysteresis of the next choice, and what dump_entity reports.
    pub lod_drawn: BTreeMap<u64, u8>,
    /// The baked reflection probes' captures tried (`super::probes`).
    pub probe_bakes: super::probes::ProbeBakes,
    /// Fonts by Font handle index.
    pub fonts: Vec<FontPages>,
    /// Text queued during the current tick, and that tick.
    pub texts: Vec<QueuedText>,
    pub texts_tick: u64,
    /// Wall time the probe's screenshots took (render, read back, PNG)
    /// since the step began: not the game's step time.
    pub capture_ms: f64,
    /// The effects games declare as data (`super::fx`), with the sprites
    /// and ribbons queued during the current tick.
    pub fx: super::fx::FxState,
    /// Textures by Texture handle index (`og.load.texture`).
    pub textures: Vec<og_render::TextureHandle>,
    /// Texture arrays by TextureArray handle index (`og.load.textureArray`).
    pub texture_arrays: Vec<super::assets::LoadedArray>,
    /// Videos by Video handle index (`og.load.video`), and og.VideoPlayer's layout.
    pub videos: Vec<super::video::LoadedVideo>,
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
    /// The command channel's `cover`: where a host's control would cover
    /// the surface (`Viewport.covered`), over whatever the host reports,
    /// so a native run lays out as the published page with its Back does.
    pub cover: Option<og_ui::Rect>,
    /// The engine's UI, drawn over every frame, and its atlas once added.
    pub ui: Option<og_ui::SharedUi>,
    pub ui_atlas: Option<og_render::TextureHandle>,
    /// A UI font's first atlas page → every page, so text is drawn per page.
    pub ui_pages: BTreeMap<FontHandle, Vec<FontHandle>>,
    /// Fonts (by index into `fonts`) that render missing characters at run
    /// time from their file (`super::glyphs`).
    pub runtime_fonts: BTreeMap<usize, super::glyphs::RuntimeFont>,
    /// Warnings of the last extraction, for the engine's log.
    pub notes: Vec<String>,
    /// The characters a frame drew that their fonts lack, each warned of
    /// once, on the tick it was first drawn.
    pub missing_warned: std::collections::BTreeSet<char>,
    /// The feel modules' drawing state (`super::feel`).
    pub feel: super::feel::FeelRender,
    /// The atmosphere's and the physics extras' drawing state (`super::atmosphere`).
    pub atmo: super::atmosphere::AtmoState,
    /// The environment maps loaded (`super::environment`).
    pub env: super::environment::EnvState,
    /// The terrains loaded (`super::terrain`).
    pub terrain: super::terrain::TerrainState,
    /// The last frame's hierarchy, kept for its allocations.
    pub hierarchy: Hierarchy,
    /// Runtime geometry, lines, points, debug draw (`super::geometry`).
    pub geo: super::geometry::GeoState,
    /// The entity of each drawable of the last extraction (`super::describe`).
    pub ids: super::describe::DrawIds,
    /// Render targets, secondary cameras and custom post passes (`super::views`).
    pub views: super::views::ViewState,
    /// Baked lighting of the scenes instantiated (`super::baked`).
    pub baked: super::baked::BakedState,
    /// The review camera in force (`view`, `turntable`: `super::review`).
    pub review: Option<super::review::ReviewView>,
}

impl RenderState {
    pub fn new(options: Value) -> RenderState {
        RenderState {
            scene: RenderScene::new(),
            backend: None,
            materials: Vec::new(),
            geometries: Vec::new(),
            models: BTreeMap::new(),
            model_lods: BTreeMap::new(),
            lod_drawn: BTreeMap::new(),
            probe_bakes: Default::default(),
            fonts: Vec::new(),
            texts: Vec::new(),
            texts_tick: 0,
            capture_ms: 0.0,
            fx: super::fx::FxState::default(),
            textures: Vec::new(),
            texture_arrays: Vec::new(),
            videos: Vec::new(),
            video_player: None,
            camera: None,
            settings: None,
            draw: DrawLayouts::default(),
            options,
            viewport: None,
            cover: None,
            ui: None,
            ui_atlas: None,
            ui_pages: BTreeMap::new(),
            runtime_fonts: BTreeMap::new(),
            notes: Vec::new(),
            missing_warned: Default::default(),
            feel: Default::default(),
            atmo: Default::default(),
            env: Default::default(),
            terrain: Default::default(),
            hierarchy: Hierarchy::default(),
            geo: Default::default(),
            ids: Default::default(),
            views: Default::default(),
            baked: Default::default(),
            review: None,
        }
    }
}

fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |x| x as f32)
}

fn vec3(v: &Value) -> Vec3 {
    Vec3::new(f(v, "x", 0.0), f(v, "y", 0.0), f(v, "z", 0.0))
}

fn color(v: &Value, default: f32) -> Vec3 {
    Vec3::new(f(v, "r", default), f(v, "g", default), f(v, "b", default))
}

fn on(v: &Value, key: &str, default: bool) -> bool {
    match &v[key] {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        _ => default,
    }
}

/// The value of a resource the world holds, decoded with its layout.
pub(crate) fn resource(world: &World, layout: Option<&Layout>) -> Option<Value> {
    let layout = layout?;
    let id = world.id_by_name(&layout.schema).ok()?;
    let bytes = world.resource_bytes(id).ok()?;
    Some(layout.decode(&bytes))
}

/// Local transforms and parents of every entity with `og.Transform`, for
/// composing world matrices.
///
/// Dense by entity index (an entity's slot holds its bits, so a stale or
/// unknown entity is told apart), read from the columns without decoding a
/// row: extraction asks it for every drawn entity on every frame, so a
/// lookup is an index, not a tree walk (owed row 50: 20,000 meshes spent
/// most of their frame in the maps this replaced).
#[derive(Default)]
pub struct Hierarchy {
    /// Per entity index: the entity's bits and what is known of its
    /// matrices, apart from them (`data`), so a read where nothing moved
    /// walks 16 bytes an entity, not the matrices' 190 (20,000 still
    /// meshes: 0.33 ms a read, read twice a frame).
    slots: Vec<Slot>,
    /// Per entity index: its transform's bytes, its local matrix and its
    /// world matrix once composed.
    data: Vec<SlotData>,
    /// Parents by child bits (only entities with `og.Parent`).
    parent: BTreeMap<u64, u64>,
    /// World matrices of chain members without `og.Transform` (no slot).
    loose: BTreeMap<u64, Mat4>,
    /// World matrices composed since the last read of the world (the
    /// raycast record's cost test counts them).
    pub(crate) composed: usize,
    /// Which slots had og.Transform at the last read (kept for its
    /// allocation).
    was: Vec<bool>,
    /// Per og.Parent archetype: its entities and its column as last read.
    parent_columns: Vec<(Vec<Entity>, Vec<u8>)>,
    /// Per og.Transform archetype, in query order: its entities and its
    /// position, quaternion and scale columns as last read, so an
    /// archetype where nothing moved is told by four comparisons.
    columns: Vec<(Vec<Entity>, [Vec<u8>; 3])>,
}

#[derive(Clone, Copy)]
struct Slot {
    bits: u64,
    /// Whether the entity has `og.Transform` this frame.
    local_set: bool,
    /// Whether `world` holds its world matrix: kept from the last read
    /// while its transform, its ancestors' and the parents are unchanged.
    world_set: bool,
    /// Whether its transform differs from the last read's (or it is new).
    changed: bool,
}

#[derive(Clone, Copy)]
struct SlotData {
    /// Its position, quaternion and scale as last read, as the columns'
    /// bytes (12, 16 and 12).
    trs: [u8; 40],
    /// Its local matrix, made from `trs` when first composed after a
    /// change ([`SlotData::local`]): the raycast record reads every transform
    /// as a tick starts and composes nothing unless a system raycasts.
    local: Option<Mat4>,
    world: Mat4,
}

impl SlotData {
    /// The local matrix of `trs`, made once per change.
    fn local(&mut self) -> Mat4 {
        *self.local.get_or_insert_with(|| {
            let t = &self.trs;
            let x =
                |i: usize| f32::from_le_bytes([t[i * 4], t[i * 4 + 1], t[i * 4 + 2], t[i * 4 + 3]]);
            Mat4::from_scale_rotation_translation(
                Vec3::new(x(7), x(8), x(9)),
                Quat::from_xyzw(x(3), x(4), x(5), x(6)),
                Vec3::new(x(0), x(1), x(2)),
            )
        })
    }
}

const EMPTY_SLOT: Slot = Slot {
    bits: 0,
    local_set: false,
    world_set: false,
    changed: true,
};

const EMPTY_DATA: SlotData = SlotData {
    trs: [0; 40],
    local: None,
    world: Mat4::IDENTITY,
};

impl Hierarchy {
    pub(crate) fn of(world: &World) -> Hierarchy {
        let mut h = Hierarchy::default();
        h.refresh(world);
        h
    }

    /// Reads the world's transforms and parents again, keeping the
    /// allocations of the last frame and every world matrix that cannot
    /// have moved: an entity whose transform (bit for bit) and whose
    /// ancestors' transforms are as last read, under the same parents,
    /// keeps its world matrix, so a frame (or the raycast's tick record)
    /// where little moves composes only what moved. The ECS keeps no
    /// change ticks, so the columns are still read and compared.
    pub(crate) fn refresh(&mut self, world: &World) {
        self.composed = 0;
        // `local_set` now says "had og.Transform at the last read".
        let mut was: Vec<bool> = std::mem::take(&mut self.was);
        was.clear();
        was.extend(self.slots.iter().map(|s| s.local_set));
        for s in &mut self.slots {
            s.local_set = false;
        }
        self.loose.clear();
        if let Ok(tid) = world.id_by_name("og.Transform") {
            let fields = ["position", "quaternion", "scale"];
            let refs: Vec<_> = fields
                .iter()
                .filter_map(|n| world.field_by_id(tid, n).ok())
                .collect();
            let q = QueryDesc {
                read: vec!["og.Transform".into()],
                ..QueryDesc::default()
            };
            if let (Ok(all), 3) = (world.query(&q), refs.len()) {
                for (k, a) in all.enumerate() {
                    let (Ok(_), Ok(_), Ok(_), Ok(pb), Ok(rb), Ok(sb)) = (
                        a.read::<Vec3>(refs[0]),
                        a.read::<Quat>(refs[1]),
                        a.read::<Vec3>(refs[2]),
                        a.read_bytes(refs[0]),
                        a.read_bytes(refs[1]),
                        a.read_bytes(refs[2]),
                    ) else {
                        continue;
                    };
                    if self.columns.len() <= k {
                        self.columns.push(Default::default());
                    }
                    let (ents, cols) = &mut self.columns[k];
                    let still = ents.as_slice() == a.entities()
                        && cols[0] == *pb
                        && cols[1] == *rb
                        && cols[2] == *sb;
                    if !still {
                        ents.clear();
                        ents.extend_from_slice(a.entities());
                        for (c, b) in cols.iter_mut().zip([&pb, &rb, &sb]) {
                            c.clear();
                            c.extend_from_slice(b);
                        }
                    }
                    for (i, e) in a.entities().iter().enumerate() {
                        let index = e.index() as usize;
                        if still && was.get(index).copied().unwrap_or(false) {
                            // As last read, and the slot was this entity's then.
                            let slot = &mut self.slots[index];
                            slot.changed = false;
                            slot.local_set = true;
                            continue;
                        }
                        if index >= self.slots.len() {
                            self.slots.resize(index + 1, EMPTY_SLOT);
                            self.data.resize(index + 1, EMPTY_DATA);
                        }
                        let (slot, data) = (&mut self.slots[index], &mut self.data[index]);
                        let bits = e.to_bits();
                        let (pb, rb, sb) = (
                            &pb[i * 12..i * 12 + 12],
                            &rb[i * 16..i * 16 + 16],
                            &sb[i * 12..i * 12 + 12],
                        );
                        let same = was.get(index).copied().unwrap_or(false)
                            && slot.bits == bits
                            && data.trs[..12] == *pb
                            && data.trs[12..28] == *rb
                            && data.trs[28..] == *sb;
                        slot.changed = !same;
                        if !same {
                            slot.bits = bits;
                            data.trs[..12].copy_from_slice(pb);
                            data.trs[12..28].copy_from_slice(rb);
                            data.trs[28..].copy_from_slice(sb);
                            slot.world_set = false;
                            data.local = None;
                        }
                        slot.local_set = true;
                    }
                }
            }
        }
        // A slot that lost its transform changed; one that never had one
        // did not.
        let mut moved = false;
        for (i, s) in self.slots.iter_mut().enumerate() {
            if !s.local_set {
                s.changed = was.get(i).copied().unwrap_or(false);
                s.world_set = false;
            }
            moved |= s.changed;
        }
        self.was = was;
        let reparented = self.read_parents(world);
        if reparented {
            // Every world matrix is composed again.
            for s in &mut self.slots {
                s.world_set = false;
            }
        } else if moved {
            // A kept child under an ancestor whose slot changed (moved, new,
            // gone) is composed again; an ancestor without og.Transform is
            // the identity, so the walk goes on through it.
            for &child in self.parent.keys() {
                let index = Entity::from_bits(child).index() as usize;
                if !self
                    .slots
                    .get(index)
                    .is_some_and(|s| s.world_set && s.bits == child)
                {
                    continue;
                }
                let mut cur = child;
                let mut depth = 0;
                let mut stale = false;
                while let Some(&p) = self.parent.get(&cur) {
                    depth += 1;
                    let changed = self
                        .slots
                        .get(Entity::from_bits(p).index() as usize)
                        .is_some_and(|s| s.changed);
                    if changed || depth > 256 || p == child {
                        stale = true;
                        break;
                    }
                    cur = p;
                }
                if stale {
                    self.slots[index].world_set = false;
                }
            }
        }
    }

    /// Reads og.Parent into `parent`; whether it differs from the last
    /// read. An archetype whose entities and column are as last read is
    /// not read again.
    fn read_parents(&mut self, world: &World) -> bool {
        let mut views = Vec::new();
        if let Ok(pid) = world.id_by_name("og.Parent")
            && let Ok(field) = world.field_by_id(pid, "entity")
        {
            let q = QueryDesc {
                read: vec!["og.Parent".into()],
                ..QueryDesc::default()
            };
            if let Ok(all) = world.query(&q) {
                for a in all {
                    if let Ok(b) = a.read_bytes(field) {
                        views.push((a.entities(), b));
                    }
                }
            }
        }
        let same = views.len() == self.parent_columns.len()
            && views
                .iter()
                .zip(&self.parent_columns)
                .all(|((e, b), (pe, pb))| *e == pe.as_slice() && **b == *pb.as_slice());
        if same {
            return false;
        }
        self.parent_columns.clear();
        let mut parent = BTreeMap::new();
        for (e, b) in &views {
            for (entity, p) in e.iter().zip(b.as_chunks::<8>().0) {
                let p = u64::from_le_bytes(*p);
                parent.insert(entity.to_bits(), p);
            }
            self.parent_columns.push((e.to_vec(), b.to_vec()));
        }
        let changed = parent != self.parent;
        self.parent = parent;
        changed
    }

    /// The slot of the entity `e` (bits), when it has `og.Transform`.
    fn slot(&mut self, e: u64) -> Option<(&mut Slot, &mut SlotData)> {
        let index = Entity::from_bits(e).index() as usize;
        let s = self.slots.get_mut(index)?;
        (s.local_set && s.bits == e).then(|| (s, &mut self.data[index]))
    }

    fn cached(&mut self, e: u64) -> Option<Mat4> {
        match self.slot(e) {
            Some((s, d)) => s.world_set.then_some(d.world),
            None => self.loose.get(&e).copied(),
        }
    }

    /// Whether `e` is `ancestor` or under it through og.Parent (bits).
    pub(crate) fn descends(&self, e: u64, ancestor: u64) -> bool {
        let mut cur = e;
        for _ in 0..=256 {
            if cur == ancestor {
                return true;
            }
            match self.parent.get(&cur) {
                Some(&p) => cur = p,
                None => return false,
            }
        }
        false
    }

    /// The world matrix of an entity's parent (the identity for a root).
    pub(crate) fn parent_world(&mut self, e: u64) -> Mat4 {
        match self.parent.get(&e).copied() {
            Some(p) => self.world_of(p),
            None => Mat4::IDENTITY,
        }
    }

    /// The world matrix of an entity: its local transform after its parents'.
    pub(crate) fn world_of(&mut self, e: u64) -> Mat4 {
        if let Some(m) = self.cached(e) {
            return m;
        }
        if !self.parent.contains_key(&e) {
            // A root: the chain below of one entity, without its vector.
            self.composed += 1;
            let mut m = Mat4::IDENTITY;
            match self.slot(e) {
                Some((s, d)) => {
                    m *= d.local();
                    d.world = m;
                    s.world_set = true;
                }
                None => {
                    m *= Mat4::IDENTITY;
                    self.loose.insert(e, m);
                }
            }
            return m;
        }
        let mut chain = vec![e];
        let mut cur = e;
        while let Some(&p) = self.parent.get(&cur) {
            if chain.contains(&p) || chain.len() > 256 {
                break;
            }
            chain.push(p);
            cur = p;
        }
        let mut m = Mat4::IDENTITY;
        for &id in chain.iter().rev() {
            if let Some(w) = self.cached(id) {
                m = w;
                continue;
            }
            self.composed += 1;
            match self.slot(id) {
                Some((s, d)) => {
                    m *= d.local();
                    d.world = m;
                    s.world_set = true;
                }
                None => {
                    m *= Mat4::IDENTITY;
                    self.loose.insert(id, m);
                }
            }
        }
        m
    }
}

/// Decoded rows of one component: `(entity bits, value)`.
pub(crate) fn rows(world: &World, layout: &Layout) -> Vec<(u64, Value)> {
    let Ok(id) = world.id_by_name(&layout.schema) else {
        return Vec::new();
    };
    let q = QueryDesc {
        read: vec![layout.schema.clone()],
        ..QueryDesc::default()
    };
    let Ok(all) = world.query(&q) else {
        return Vec::new();
    };
    // The bytes are read serially (the world is one thread's); decoding them
    // is a lane per row natively, the rows staying in entity order.
    let cells: Vec<(u64, Vec<u8>)> = all
        .flat_map(|a| a.entities().to_vec())
        .filter_map(|e| Some((e.to_bits(), world.get_bytes(e, id).ok()?)))
        .collect();
    if cfg!(target_arch = "wasm32") {
        // A row's decode is allocations (a JSON map per row), and the lanes
        // wasm has one allocator behind one lock: 20,000 og.Mesh rows split
        // over eight Workers spent 116 ms of a 137 ms frame waiting for the
        // lanes (owed row 50, tests/evidence/P4/perf/).
        return cells.iter().map(|(e, b)| (*e, layout.decode(b))).collect();
    }
    og_lanes::map(og_lanes::Site::Extract, &cells, ROW_GRAIN, |(e, bytes)| {
        (*e, layout.decode(bytes))
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
    /// `layers`: its three.js layer mask (`super::views`).
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
        .map(|(e, v)| {
            let index = |key: &str| (v[key].as_f64().unwrap_or(0.0) as u64 & 0xffff_ffff) as usize;
            MeshRow {
                entity: e,
                geometry: index("geometry"),
                material: index("material"),
                visible: on(&v, "visible", true),
                flash: v["flash"].as_f64().unwrap_or(0.0) as f32,
                flash_color: color(&v["flashColor"], 1.0),
                layer: v["materialLayer"].as_f64().unwrap_or(-1.0) as i32,
                cast: on(&v, "castShadow", false),
                receive: on(&v, "receiveShadow", true),
                layers: v["layers"].as_u64().map_or(1, |x| x as u32),
            }
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
    /// The effect records (`super::fx`).
    pub fx: super::fx::FxLayouts,
    /// The atmosphere's and the physics extras' records (`super::atmosphere`).
    pub atmo: super::atmosphere::AtmoLayouts,
    /// og.Environment (`super::environment`).
    pub environment: Option<Layout>,
    /// og.Terrain and og.TerrainScatter (`super::terrain`; the 2D
    /// profile draws neither).
    #[cfg(not(feature = "profile2d"))]
    pub terrain: Option<Layout>,
    #[cfg(not(feature = "profile2d"))]
    pub scatter: Option<Layout>,
    /// og.Collider: what CPU-simulated particles bounce off.
    pub collider: Option<Layout>,
    /// og.Lines and og.Points (`super::geometry`).
    pub lines: Option<Layout>,
    pub points: Option<Layout>,
    /// og.ReflectionProbe (`super::probes`).
    pub probe: Option<Layout>,
    /// og.CameraView and og.Mirror (`super::views`).
    pub views: super::views::ViewLayouts,
    /// The engine's physics, for the debug view (`super::geometry`).
    pub physics: Option<og_physics::PhysicsSystem>,
    pub engine: u32,
}

/// The render target's size, (1, 1) without a renderer.
fn backend_size(rs: &RenderState) -> (u32, u32) {
    rs.backend.as_ref().map_or((1, 1), |b| b.size())
}

/// Rebuild the per-frame part of `rs.scene` from the world as of its tick.
pub(crate) fn extract(world: &World, rs: &mut RenderState) {
    let mut h = std::mem::take(&mut rs.hierarchy);
    h.refresh(world);
    // A review camera frames what the last frame drew for its entity, so
    // the models below pick their levels for it (once more at the end).
    let ids = std::mem::take(&mut rs.ids.instances);
    let framed = super::review::bounds(rs, &h, &ids);
    rs.ids.instances = ids;
    let size = backend_size(rs);
    let layouts = &rs.draw;
    let camera = resource(world, rs.camera.as_ref());
    let settings = resource(world, rs.settings.as_ref());
    rs.atmo.palettes.clear();
    rs.ids.sprites.clear();
    let scene = &mut rs.scene;
    let (models, geometries, materials) = (&rs.models, &rs.geometries, &rs.materials);
    scene.instances.clear();
    scene.lights.clear();
    scene.texts.clear();
    scene.sprites.clear();
    scene.sprite_meshes.clear();
    scene.overlay = None;
    scene.joint_palettes.clear();
    scene.skinned_instances.clear();
    scene.sprite_order.clear();
    scene.sprite_fx.clear();
    scene.sprite_arrays.clear();
    scene.sprite_shadows.clear();
    scene.text_outlines.clear();
    scene.tile_layers.clear();
    scene.trail_layers.clear();
    scene.lines.clear();
    scene.points.clear();
    // Layer masks of the sprites, lines, points and particle systems
    // (views.rs): filled by what pushes them, 1 past their ends.
    scene.sprite_layers.clear();
    // Billboards (sprites' facing): set by what pushes them, none past the end.
    scene.sprite_facing.clear();
    scene.line_layers.clear();
    scene.point_layers.clear();
    scene.particle_layers.clear();
    let time = world.time();
    scene.tick = time.tick;
    scene.tick_rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    // og.render.sprite and og.render.text are immediate mode: what the last
    // tick submitted (or a command since it) is drawn, and a tick that
    // submitted nothing draws nothing. The queues are cleared lazily by the
    // next submission, so a queue from an older tick is stale here.
    let now = world.time().tick;
    let fresh = rs.texts_tick == now || rs.texts_tick + 1 == now;

    // Camera.
    let mut exposure = 1.0f32;
    scene.camera_layers = camera
        .as_ref()
        .and_then(|c| c["layers"].as_u64())
        .map_or(1, |x| x as u32);
    if let Some(c) = &camera {
        let q = &c["quaternion"];
        let rot = Quat::from_xyzw(
            f(q, "x", 0.0),
            f(q, "y", 0.0),
            f(q, "z", 0.0),
            f(q, "w", 1.0),
        )
        .normalize();
        let position = vec3(&c["position"]);
        let cam = &mut scene.camera;
        cam.position = position;
        cam.target = position + rot * Vec3::NEG_Z;
        cam.up = rot * Vec3::Y;
        cam.near = f(c, "near", 0.1);
        cam.far = f(c, "far", 2000.0);
        let zoom = f(c, "zoom", 1.0).max(0.01);
        cam.projection = if on(c, "orthographic", false) {
            og_render::Projection::Orthographic {
                height: f(c, "height", 10.0) / zoom,
            }
        } else {
            // three.js zoom narrows the field of view: tan(fov/2) / zoom.
            let half = (f(c, "fov", 50.0).to_radians() / 2.0).tan() / zoom;
            og_render::Projection::Perspective {
                fov_y_degrees: (2.0 * half.atan()).to_degrees(),
            }
        };
        exposure = f(c, "exposure", 1.0).max(0.0);
        scene.environment.background = og_render::Background::Color(color(&c["background"], 0.0));
    }
    super::review::apply(&mut scene.camera, rs.review.as_ref(), size, framed);

    // Settings: environment, sky, bloom, exposure, tone mapping.
    if let Some(s) = &settings {
        // RenderSettings.exposure scales every camera's, as three.js
        // toneMappingExposure does the whole frame's.
        exposure *= f(s, "exposure", 1.0).max(0.0);
        scene.environment.intensity = f(s, "environmentIntensity", 0.55);
        if on(s, "sky", false) {
            scene.environment.background = og_render::Background::Sky;
        }
        scene.post.bloom = on(s, "bloom", false).then(|| og_render::Bloom {
            threshold: f(s, "bloomThreshold", 1.0),
            knee: f(s, "bloomKnee", 0.5),
            intensity: f(s, "bloomIntensity", 0.6),
        });
        scene.post.tonemap = if on(s, "toneMapped", true) {
            match s["toneMapping"].as_u64().or_else(|| {
                s["toneMapping"]
                    .as_str()
                    .and_then(|n| ["aces", "agx", "none"].iter().position(|v| *v == n))
                    .map(|i| i as u64)
            }) {
                Some(1) => og_render::Tonemap::Agx,
                Some(2) => og_render::Tonemap::None,
                _ => og_render::Tonemap::Aces,
            }
        } else {
            og_render::Tonemap::None
        };
        scene.camera.exposure = if on(s, "autoExposure", false) {
            og_render::Exposure::Auto {
                compensation: exposure.max(1e-6).log2(),
                min_ev: -10.0,
                max_ev: 10.0,
                adapt_speed_up: f(s, "adaptSpeedUp", 0.0).max(0.0),
                adapt_speed_down: f(s, "adaptSpeedDown", 0.0).max(0.0),
            }
        } else {
            og_render::Exposure::linear(exposure)
        };
        // The quality fields, one to one (og.render.applyQuality writes a
        // preset's row of them; RenderSettings.quality is only its record).
        let full = og_render::RenderQuality::default();
        scene.quality = og_render::RenderQuality {
            rect_shadows: on(s, "rectShadows", full.rect_shadows),
            shadowed_lights: f(s, "shadowedLights", full.shadowed_lights as f32).max(0.0) as u32,
            shadow_map_scale: f(s, "shadowMapScale", full.shadow_map_scale),
            volumetric_fog: on(s, "volumetricFog", full.volumetric_fog),
            soft_particles: on(s, "softParticles", full.soft_particles),
            ssr: on(s, "screenSpaceReflections", full.ssr),
            ambient_occlusion: on(s, "ambientOcclusion", full.ambient_occlusion),
            msaa_samples: f(s, "msaaSamples", full.msaa_samples as f32).max(1.0) as u32,
        };
    } else {
        scene.camera.exposure = og_render::Exposure::linear(exposure);
        scene.quality = og_render::RenderQuality::default();
    }

    // Baked lighting (RenderSettings.bakedLighting, on by default).
    let baked = settings
        .as_ref()
        .is_none_or(|s| on(s, "bakedLighting", true));
    super::baked::apply(world, &mut rs.scene, &mut rs.baked, baked);
    // Models: posed by og-anim when the entity has a Skeleton.
    let posed = super::fx::skeletons(world, rs);
    let mut keys: Vec<(u64, u32)> = Vec::new();
    // Each model's own tint and flash ride in its instances' records
    // (kestrel-stair #11: a glTF's materials are shared by every instance),
    // with its shadow flags (on unless the model says otherwise). A hidden
    // model draws nothing (meridian-loop #14). The rows are read in place
    // by offset, the fields resolved once a frame: decoding each row to
    // JSON was about a third of Lantern Vale's extraction
    // (tests/evidence/r4/perf-ui/NOTES.md).
    let model_parts: Vec<(u64, u64, Parts, super::fx::ModelLook, f32)> = match &layouts.model {
        Some(layout) => {
            let slot = |key| FieldSlot::of(layout, key);
            let (visible, lod_bias, asset) = (slot("visible"), slot("lodBias"), slot("asset"));
            let (flash, flash_color, tint) = (slot("flash"), slot("flashColor"), slot("tint"));
            let (cast, receive) = (slot("castShadow"), slot("receiveShadow"));
            let is_static = slot("static");
            let rows = world
                .id_by_name(&layout.schema)
                .map(|id| super::feel::byte_rows(world, id, layout))
                .unwrap_or_default();
            rows.iter()
                .filter_map(|(e, bytes)| {
                    let (e, v) = (*e, Row::new(layout, bytes));
                    let num = |s: &FieldSlot, d: f32| v.num(s).map_or(d, |x| x as f32);
                    let on = |s: &FieldSlot, d: bool| v.flag(s).unwrap_or(d);
                    let rgb = |s: &FieldSlot| {
                        let lane = |l| v.lane(s, l).map_or(1.0, |x| x as f32);
                        Vec3::new(lane("r"), lane("g"), lane("b"))
                    };
                    if !on(&visible, true) {
                        return None;
                    }
                    let bias = num(&lod_bias, 1.0).max(0.0);
                    let asset = v.num(&asset).unwrap_or(0.0) as u64;
                    let look = super::fx::ModelLook {
                        fx: og_render::InstanceFx {
                            flash: num(&flash, 0.0),
                            flash_color: rgb(&flash_color),
                            tint: rgb(&tint),
                        },
                        cast: on(&cast, true),
                        receive: on(&receive, true),
                    };
                    // A static model with a bake draws its lightmapped parts.
                    let lightmapped = (baked && on(&is_static, false))
                        .then(|| rs.baked.overrides.get(&e))
                        .flatten();
                    lightmapped
                        .or_else(|| models.get(&asset))
                        .map(|p| (e, asset, p.clone(), look, bias))
                })
                .collect()
        }
        None => Vec::new(),
    };
    let mesh_rows = layouts
        .mesh
        .as_ref()
        .map(|m| mesh_rows(world, m))
        .unwrap_or_default();
    let (geometries, materials) = (geometries.clone(), materials.clone());
    // Levels of detail: per entity, the coarsest level whose error stays
    // under a pixel at its distance (og_render::lod), kept near a boundary
    // by the level it drew last. `RenderOptions.debugView lod` colours each
    // by its level.
    let settings_bias = settings
        .as_ref()
        .map_or(1.0, |s| f(s, "lodBias", 1.0).max(0.0));
    let debug_lod = super::geometry::current_view(rs) == "lod";
    let view_height = match rs.backend.as_ref().map(|b| b.size()) {
        Some((_, ht)) if ht > 0 => ht as f32,
        _ => super::fx::viewport(rs).1,
    };
    let model_lods = std::mem::take(&mut rs.model_lods);
    let last_drawn = std::mem::take(&mut rs.lod_drawn);
    let mut drawn = BTreeMap::new();
    for (e, asset, parts, mut look, bias) in model_parts {
        let m = h.world_of(e);
        let level = match model_lods.get(&asset) {
            Some(l) => {
                let scale = m.to_scale_rotation_translation().0.abs().max_element();
                let per_unit = og_render::lod::pixels_per_unit(
                    &rs.scene.camera,
                    m.transform_point3(l.center),
                    l.radius * scale,
                    view_height,
                );
                let previous = last_drawn.get(&e).map(|&x| x as usize);
                og_render::lod::select(
                    &l.errors(),
                    per_unit * scale,
                    bias * settings_bias,
                    previous,
                )
            }
            None => 0,
        };
        drawn.insert(e, level as u8);
        if debug_lod {
            look.fx.flash = 0.8;
            look.fx.flash_color = og_render::lod::debug_color(level);
        }
        if posed.contains(&e) && super::fx::posed_model(rs, e, asset, m, &parts, look, &mut keys) {
            continue;
        }
        let levels = model_lods.get(&asset).map(|l| &l.parts);
        for (k, &(mesh, material, local)) in parts.iter().enumerate() {
            // A part with fewer levels than the model draws its coarsest.
            let mesh = match (level, levels.and_then(|l| l.get(k))) {
                (0, _) | (_, None) => mesh,
                (_, Some(p)) if p.is_empty() => mesh,
                (l, Some(p)) => p[(l - 1).min(p.len() - 1)].0,
            };
            rs.scene.instances.push(og_render::Instance {
                mesh,
                material,
                transform: m * local,
                material_layer: None,
                fx: look.fx,
                cast_shadows: look.cast,
                receive_shadows: look.receive,
            });
            keys.push((e, k as u32));
        }
    }
    rs.model_lods = model_lods;
    rs.lod_drawn = drawn;
    let scene = &mut rs.scene;
    // Meshes.
    {
        scene.instances.reserve(mesh_rows.len());
        keys.reserve(mesh_rows.len());
        for r in mesh_rows {
            if !r.visible {
                continue;
            }
            let (Some(&g), Some(&m)) = (geometries.get(r.geometry), materials.get(r.material))
            else {
                continue;
            };
            let e = r.entity;
            let w = h.world_of(e);
            // A flashing mesh's flash rides in its instance's record, as
            // og.Model's does: one draw with its unflashed twins (row 85).
            scene.instances.push(og_render::Instance {
                mesh: g,
                material: m,
                transform: w,
                material_layer: u32::try_from(r.layer).ok(),
                fx: og_render::InstanceFx {
                    flash: r.flash,
                    flash_color: r.flash_color,
                    ..og_render::InstanceFx::NONE
                },
                cast_shadows: r.cast,
                receive_shadows: r.receive,
            });
            keys.push((e, 0));
        }
    }
    // Lights: the first directional light, then point and spot lights in
    // entity order (deterministic); og-render culls them to the view and
    // keeps at most MAX_LIGHTS.
    let forward = |m: Mat4| m.transform_vector3(Vec3::NEG_Z).normalize_or_zero();
    let layouts = &rs.draw;
    if let Some(dl) = &layouts.directional
        && let Some((e, v)) = rows(world, dl).into_iter().next()
    {
        let m = h.world_of(e);
        let light = &mut scene.light;
        let dir = forward(m);
        light.direction_to_light = if dir == Vec3::ZERO { Vec3::Y } else { -dir };
        light.color = color(&v["color"], 1.0);
        light.intensity = f(&v, "intensity", 3.0);
        light.cast_shadows = on(&v, "castShadow", true);
        light.max_shadow_distance = f(&v, "shadowDistance", 200.0);
        light.shadow_softness = f(&v, "shadowSoftness", 0.0);
        light.cascade_blend = f(&v, "cascadeBlend", 0.0);
        scene.environment.sky.sun_direction = light.direction_to_light.normalize_or_zero();
    }
    // Each light with its cookie (PointLight.cookie, SpotLight.cookie).
    let textures = &rs.textures;
    let cookie = |v: &Value| {
        v["cookie"]
            .as_f64()
            .and_then(super::assets::index_of)
            .and_then(|i| textures.get(i).copied())
    };
    let mut locals: Vec<(u64, og_render::Light, Option<og_render::TextureHandle>)> = Vec::new();
    if let Some(pl) = &layouts.point {
        for (e, v) in rows(world, pl) {
            let m = h.world_of(e);
            let c = cookie(&v);
            locals.push((
                e,
                og_render::Light {
                    kind: og_render::LightKind::Point,
                    position: m.transform_point3(Vec3::ZERO),
                    color: color(&v["color"], 1.0),
                    intensity: f(&v, "intensity", 10.0),
                    range: f(&v, "distance", 10.0),
                    cast_shadows: on(&v, "castShadow", false),
                },
                c,
            ));
        }
    }
    if let Some(sl) = &layouts.spot {
        for (e, v) in rows(world, sl) {
            let m = h.world_of(e);
            let c = cookie(&v);
            locals.push((
                e,
                og_render::Light {
                    kind: og_render::LightKind::Spot {
                        direction: forward(m),
                        angle_degrees: f(&v, "angle", std::f32::consts::FRAC_PI_6).to_degrees(),
                        penumbra: f(&v, "penumbra", 0.0),
                    },
                    position: m.transform_point3(Vec3::ZERO),
                    color: color(&v["color"], 1.0),
                    intensity: f(&v, "intensity", 10.0),
                    range: f(&v, "distance", 10.0),
                    cast_shadows: on(&v, "castShadow", false),
                },
                c,
            ));
        }
    }
    if let Some(rl) = &layouts.rect {
        for (e, v) in rows(world, rl) {
            let m = h.world_of(e);
            let light = super::baked::rect_light(m, &v);
            locals.push((e, light, None));
        }
    }
    og_core::sort::sort_by_key(&mut locals, |(e, ..)| Entity::from_bits(*e).index());
    scene.light_cookies = locals
        .iter()
        .enumerate()
        .filter_map(|(i, (_, _, c))| c.map(|texture| og_render::LightCookie { light: i, texture }))
        .collect();
    scene.lights = locals.iter().map(|(_, l, _)| *l).collect();
    // Cookies past RenderCapabilities.maxLightCookies layers are not drawn:
    // the log says which lights shine without theirs.
    let cookies = og_render::cookie_layers(scene);
    if !cookies.left_out.is_empty() {
        let lights: Vec<String> = cookies
            .left_out
            .iter()
            .map(|&i| Entity::from_bits(locals[i].0).to_string())
            .collect();
        rs.notes.push(format!(
            "light cookies: the frame's cookies take {} layers, past RenderCapabilities.maxLightCookies ({}; a SpotLight.cookie takes 1, a PointLight.cookie 6, a texture shared by lights of one kind once); {}: {}",
            cookies.asked,
            og_render::MAX_COOKIE_LAYERS,
            if lights.len() == 1 {
                "this light shines without its cookie"
            } else {
                "these lights shine without theirs"
            },
            lights.join(", ")
        ));
    }

    // The effects games declare: the post stack, shake and 2D lighting,
    // then the effect components.
    let tick = time.tick;
    super::fx::settings(world, rs);
    super::fx::components(world, rs, &mut h, tick);
    // The pixel grid's snap: after the sprites and the camera are in.
    super::fx::snap_to_grid(rs);
    // Cloth meshes are instances too: keyed for per-object motion.
    keys.extend(super::atmosphere::extract(world, rs, &mut h));
    super::environment::extract(world, rs);
    super::probes::extract(world, rs, &mut h);
    // Scattered copies are instances too, keyed by their terrain cell.
    keys.extend(super::terrain::extract(world, rs, &mut h));
    particle_colliders(world, rs, &mut h);
    let vp = super::fx::viewport(rs);
    // The camera's shake, punch, kick and zones (on the frame only), then
    // og.Sprite rows, floating texts and tile layers.
    let aspect = match rs.backend.as_ref().map(|b| b.size()) {
        Some((w, ht)) if ht > 0 => w as f32 / ht as f32,
        _ if vp.1 > 0.0 => vp.0 / vp.1,
        _ => 16.0 / 9.0,
    };
    super::feel::camera_frame(world, rs, aspect);
    super::feel::extract(world, rs, &mut h);
    super::skeleton2d::extract(world, rs, &mut h);

    // Characters the fonts lack, rendered from their files before any
    // text is laid out for this frame.
    super::glyphs::generate(rs);

    // Text, page by page; screen text into the overlay.
    let queued_texts: &[QueuedText] = if fresh { &rs.texts } else { &[] };
    for t in queued_texts {
        let Some(font) = rs.fonts.get(t.font) else {
            continue;
        };
        let scene = &mut rs.scene;
        for &page in &font.pages {
            let text = og_render::Text {
                font: page,
                text: t.text.clone(),
                position: t.position,
                size: t.size,
                rotation: t.rotation,
                color: t.color,
                layer: t.layer,
                clip: None,
                tracking: 0.0,
                weight: 0.0,
                glow: None,
                placements: Vec::new(),
            };
            if t.screen {
                let o = scene
                    .overlay
                    .get_or_insert_with(|| og_render::Overlay::new(vp.0, vp.1, 0.0));
                let clip = t.clip.map(|c| o.add_clip(c));
                o.texts.push(og_render::Text {
                    clip,
                    layer: og_render::screen_layer(text.layer),
                    ..text
                });
            } else {
                if t.outline > 0.0 {
                    scene.text_outlines.push(og_render::TextOutline {
                        text: scene.texts.len() as u32,
                        width: t.outline,
                        color: t.outline_color,
                    });
                }
                scene.texts.push(text);
            }
        }
    }

    // Sprites and ribbons queued this tick.
    super::fx::queued(rs, fresh, vp);
    // The review camera on this frame's instances of its entity.
    if rs.review.is_some() {
        let ids: Vec<u64> = keys.iter().map(|k| k.0).collect();
        let framed = super::review::bounds(rs, &h, &ids);
        super::review::apply(&mut rs.scene.camera, rs.review.as_ref(), size, framed);
    }
    // Lines, points and debug draw (after every instance is in, for the
    // bounds view).
    super::geometry::extract(world, rs, &mut h, vp);
    super::fx::motion(rs, &keys, tick);
    // Layer masks, secondary cameras and custom post passes.
    super::views::extract(world, rs, &keys, &mut h);
    // Which entity each instance came from, for the probe's id pass.
    rs.ids.instances.clear();
    rs.ids.instances.extend(keys.iter().map(|k| k.0));
    // Drop shadows over the terrain under their sprites.
    super::terrain::ground_drop_shadows(rs);

    // Each video's texture shows its player's frame.
    super::video::extract(world, rs);
    // The UI, over everything else, in the overlay.
    super::modules::emit(rs);
    rs.hierarchy = h;
}

/// A cooked font as one `MsdfFont` per atlas page (Render's question 1: the
/// mapping lives here). Pages are uploaded linear, without mips.
pub(crate) fn add_font(scene: &mut RenderScene, font: &og_cook::runtime::CookedFont) -> FontPages {
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
    let mut pages = Vec::with_capacity(font.pages.len());
    for (p, data) in font.pages.iter().enumerate() {
        let atlas = scene.add_texture(og_render::TextureData {
            mips: og_render::Mips::None,
            color_space: og_render::ColorSpace::Linear,
            ..data.clone()
        });
        let glyphs = font
            .glyphs
            .iter()
            .filter_map(|g| {
                let codepoint = char::from_u32(g.codepoint)?;
                let own = g.page as usize == p;
                Some(og_render::Glyph {
                    codepoint,
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
            distance_range: meta.px_range,
            line_height,
            glyphs,
            kerning: kerning.clone(),
        }));
    }
    FontPages { pages }
}

/// The colliders CPU-simulated particles bounce off
/// (`RenderScene::particle_colliders`): every og.Collider that is a box, a
/// sphere or a terrain's height field, placed as physics places it, while
/// a particle system of the frame has collision. The compute path reads the
/// depth buffer instead; WebGL2 (and `ParticleSimulation::Cpu`) reads these,
/// so sparks bounce on the ground and on solid shapes on every tier.
fn particle_colliders(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    rs.scene.particle_colliders.clear();
    let colliding = rs.scene.particles.iter().any(|s| {
        rs.scene
            .emitter(s.emitter)
            .is_some_and(|e| e.collision.is_some())
    });
    let Some(layout) = rs.draw.collider.clone().filter(|_| colliding) else {
        return;
    };
    for (e, v) in rows(world, &layout) {
        let world_m = h.world_of(e);
        let rotation = &v["rotation"];
        let q = Quat::from_xyzw(
            rotation["x"].as_f64().unwrap_or(0.0) as f32,
            rotation["y"].as_f64().unwrap_or(0.0) as f32,
            rotation["z"].as_f64().unwrap_or(0.0) as f32,
            rotation["w"].as_f64().unwrap_or(1.0) as f32,
        )
        .normalize();
        let local = Mat4::from_rotation_translation(q, vec3(&v["offset"]));
        let m = world_m * local;
        let collider = match v["shape"].as_u64().unwrap_or(0) {
            0 => og_render::ParticleCollider::Box {
                transform: m,
                half_extents: vec3(&v["size"]) * 0.5,
            },
            1 => {
                let scale = world_m.to_scale_rotation_translation().0.max_element();
                og_render::ParticleCollider::Sphere {
                    center: m.transform_point3(Vec3::ZERO),
                    radius: f(&v, "radius", 0.5) * scale,
                }
            }
            6 => {
                let geometry = v["mesh"].as_f64().unwrap_or(0.0) as u64;
                let Some(data) = super::terrain::field_of_geometry(&rs.terrain, geometry) else {
                    continue;
                };
                og_render::ParticleCollider::HeightField { data, transform: m }
            }
            _ => continue,
        };
        rs.scene.particle_colliders.push(collider);
    }
}
