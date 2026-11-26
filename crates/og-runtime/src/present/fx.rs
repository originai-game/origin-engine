//! The effects a game declares as data, extracted into og-render's scene
//! (the IDL's `PostEffects`, `Lighting2D`, `Light2D`, `Occluder2D`, `Decal`,
//! `ParticleSystem`, `Trail`, `Beam`, `og.render.ribbon`, the sprite effect
//! and screen fields, and skinned models posed by og-anim), and the loads of
//! their assets (`og.load.lut`, `og.load.emitter`). Like the rest of
//! extraction it reads the world and never writes it.

use crate::host::codec::{ColRow, Layout, Rec, Slot};
use crate::host::wire::{self, FromJson, record};
use crate::present::extract::{Hierarchy, RenderState};
use glam::{Mat4, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::{Entity, World};
use og_render::{
    AmbientOcclusion, AoMethod, AoResolution, Beam, CameraShake, ChromaticAberration, ColorAdjust,
    ColorCurve, ColorGrading, Curve, Decal, DepthOfField, DofQuality, FilmGrain, Glow, GodRays,
    JointPalette, LensDistortion, LensFlare, Light2d, Light2dKind, Lighting2d, MeshHandle,
    MorphTargets, MotionBlur, Occluder2d, Outline, ParticleBlend, ParticleSystem, RibbonUv,
    ScreenBlur, ScreenBlurKind, ScreenFlash, SkinData, SkinnedInstance, Trail, TrailStyle,
    TrailStyleHandle, Vignette,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

mod loading;
mod look;
mod models;
mod motion_history;

pub(crate) use loading::*;
pub(crate) use look::*;
pub(crate) use models::*;
pub(crate) use motion_history::*;

/// Screen-space GI's settings: a lazy resource (crate::engine::lazy), so a world
/// that never names it keeps its type table and hash.
pub(crate) const GLOBAL_ILLUMINATION: &str = "og.GlobalIllumination";
/// The lens flare's settings: lazy as well, read by extraction alone.
pub(crate) const LENS_FLARE: &str = "og.LensFlare";

/// The layouts of the effect records the engine registered.
#[derive(Default)]
pub(crate) struct FxLayouts {
    pub post: Option<Layout>,
    pub lighting_2d: Option<Layout>,
    pub global_illumination: Option<Layout>,
    pub lens_flare: Option<Layout>,
    pub light_2d: Option<Layout>,
    pub occluder_2d: Option<Layout>,
    pub light_mask_2d: Option<Layout>,
    pub decal: Option<Layout>,
    pub particles: Option<Layout>,
    pub trail: Option<Layout>,
    pub beam: Option<Layout>,
    pub effect_mesh: Option<Layout>,
    pub vfx: Option<Layout>,
    pub skeleton: Option<Layout>,
    pub crowd: Option<Layout>,
    pub stylise: Option<Layout>,
    pub material_style: Option<Layout>,
    /// The art contract's resources the runtime reads (`crate::present::art`).
    pub art_line: Option<Layout>,
    pub art_pixels: Option<Layout>,
    pub art_camera: Option<Layout>,
    pub art_palette: Option<Layout>,
    pub art_motion: Option<Layout>,
}

impl FxLayouts {
    pub fn from(get: impl Fn(&str) -> Option<Layout>) -> FxLayouts {
        FxLayouts {
            post: get("og.PostEffects"),
            lighting_2d: get("og.Lighting2D"),
            global_illumination: get(GLOBAL_ILLUMINATION),
            lens_flare: get(LENS_FLARE),
            light_2d: get("og.Light2D"),
            occluder_2d: get("og.Occluder2D"),
            light_mask_2d: get("og.LightMask2D"),
            decal: get("og.Decal"),
            particles: get("og.ParticleSystem"),
            trail: get("og.Trail"),
            beam: get("og.Beam"),
            effect_mesh: get("og.EffectMesh"),
            vfx: get(crate::bind::vfx::COMPONENT),
            skeleton: get("og.Skeleton"),
            crowd: get("og.CrowdAnimation"),
            stylise: get("og.Stylise"),
            material_style: get("og.MaterialStyle"),
            art_line: get("og.ArtLine"),
            art_pixels: get("og.ArtPixels"),
            art_camera: get("og.ArtCamera"),
            art_palette: get("og.ArtPalette"),
            art_motion: get("og.ArtMotion"),
        }
    }
}

/// A sprite queued by `og.render.sprite` with its effect and screen fields.
#[derive(Clone, Debug)]
pub(crate) struct QueuedSprite {
    pub sprite: og_render::Sprite,
    /// Flash colour and amount, glow, additive.
    pub flash: [f32; 4],
    pub glow: f32,
    pub additive: bool,
    pub cast_shadow: bool,
    /// Drawn in the overlay (CSS pixels, y down) with this clip.
    pub screen: bool,
    pub clip: Option<[f32; 4]>,
    /// The sort key within its layer (`sortY`: -y); 0 keeps submission order.
    pub order: f32,
    /// Drawn from this layer of a texture array instead of its texture.
    pub array: Option<(og_render::TextureArrayHandle, u32)>,
    /// `SpriteDesc.dropShadow`: a flattened dark copy on the ground (0 none).
    pub drop_shadow: f32,
    /// `SpriteDesc.dropShadowHeight`: the level ground under it (a higher
    /// terrain wins, `terrain::ground_drop_shadows`).
    pub drop_shadow_height: f32,
    /// `SpriteDesc.toneMapped`: false keeps it out of exposure, bloom and
    /// the tone curve ([`og_render::SpriteFx::tone_mapped`]).
    pub tone_mapped: bool,
    /// `SpriteDesc.facing`: a billboard (world sprites only).
    pub facing: og_render::SpriteFacing,
}

/// One `og.load.lut`: its renderer LUT while resident.
#[derive(Clone, Debug)]
pub(crate) struct LoadedLut {
    pub id: String,
    /// `None` while its streaming group is not resident: released when the
    /// group unloaded, added again once it is resident (`resolve_luts`).
    pub handle: Option<og_render::LutHandle>,
}

/// One loaded model's primitives, for skinning: the scene mesh of each
/// primitive, the glTF mesh of each primitive, and
/// the node of each of its parts (`og_bundle::runtime::ModelParts::nodes`)
/// and whether each is drawn at rest (`ModelParts::shown`).
#[derive(Clone, Debug)]
pub(crate) struct ModelInfo {
    pub id: String,
    /// The scene mesh of each primitive (shared with other models when
    /// identical: not consecutive).
    pub meshes: Vec<og_render::MeshHandle>,
    pub prim_mesh: Vec<u32>,
    pub part_nodes: Vec<usize>,
    pub part_offsets: Vec<Mat4>,
    pub part_shown: Vec<bool>,
    /// Per `KHR_materials_variants` variant, each part's material
    /// (`ModelParts::variants`): og.Model.variant draws with these.
    pub part_variants: Vec<Vec<og_render::MaterialHandle>>,
    /// A skinned model's coarser levels by primitive (empty for a model
    /// without skins): each level's mesh and, per vertex of it, the full
    /// mesh's vertex it is (`CookedPrimitive::lod_sources`), which gives
    /// the level's skinned copy its joints and weights.
    pub lod_sources: Vec<Vec<(og_render::MeshHandle, Vec<u32>)>>,
}

impl ModelInfo {
    /// The primitive drawn with scene mesh `mesh` (`usize::MAX` for none).
    pub fn primitive(&self, mesh: og_render::MeshHandle) -> usize {
        self.meshes
            .iter()
            .position(|&h| h == mesh)
            .unwrap_or(usize::MAX)
    }
}

/// A model's skin and morph data from its `#anim` blob, and the skinned
/// copies of its primitives once made (by primitive; `None`: not skinned).
#[derive(Clone, Debug, Default)]
pub(crate) struct SkinnedModel {
    pub skeleton: og_anim::SkeletonAsset,
    pub streams: Vec<og_anim::cooked::SkinStream>,
    pub morphs: Vec<og_anim::cooked::MorphTargets>,
    pub meshes: Option<Vec<Option<MeshHandle>>>,
    /// The skinned copies of each primitive's coarser levels, made with
    /// `meshes` (`ModelInfo::lod_sources`): what a crowd draws far away.
    pub levels: Vec<Vec<MeshHandle>>,
}

/// The effect side of a render state.
#[derive(Default)]
pub(crate) struct FxState {
    /// `og.load.lut` by load index.
    pub luts: Vec<LoadedLut>,
    /// `og.load.emitter` handles by load index.
    pub emitters: Vec<og_render::EmitterHandle>,
    /// The texture array (its load index in RenderState::texture_arrays)
    /// of each emitter that draws from one, by the emitter's load index:
    /// the renderer's emitter follows the array's handle, and a system of
    /// it draws nothing while the array's streaming group is not resident.
    pub emitter_arrays: IdMap<usize, usize>,
    /// Source bundle index for each loaded emitter, by emitter load index.
    pub emitter_bundles: IdMap<usize, usize>,
    pub by_path: BTreeMap<String, usize>,
    /// Built-in mesh particles' shapes (`render mesh box`) and effect
    /// meshes (`vfx:ring ...`), made once.
    pub particle_meshes: BTreeMap<String, MeshHandle>,
    /// Each loaded model's materials by glTF material index, those no
    /// part draws with included (`ModelParts::materials`: a fracture's
    /// mortar), by the model's asset handle.
    pub model_materials: BTreeMap<u64, Vec<og_render::MaterialHandle>>,
    /// The debris layer's rubble chips (og_render::chip) and their hulls'
    /// points, made at the first fine piece drawn.
    pub chip_meshes: Vec<(MeshHandle, Vec<glam::Vec3>)>,
    /// Emitters being loaded, outermost first: a sub-emitter naming one of
    /// them is a cycle.
    pub loading: Vec<String>,
    /// og.particles' state (`crate::bind::particles`): the systems the
    /// tick simulates, drawn as they stand. Set at the first emitter load.
    pub host: Option<Arc<std::sync::Mutex<crate::bind::particles::Hosted>>>,
    /// Per-trail-id mutable style handles, updated in-place via
    /// `RenderScene::set_trail_style` so animated styles never leak or reset
    /// GPU history rings.
    pub entity_trail_styles: IdMap<u64, TrailStyleHandle>,
    /// Recycled trail style handles returned when a trail entity disappears.
    pub free_trail_styles: Vec<TrailStyleHandle>,
    /// Trail ids extracted during the current frame (for reclaiming unused handles).
    pub active_trails: Vec<u64>,
    /// Ribbons and sprites queued this tick (`og.render.ribbon`, `.sprite`).
    pub ribbons: Vec<og_render::Ribbon>,
    pub sprites: Vec<QueuedSprite>,
    /// Loaded models by asset handle bits, and skin data by model id.
    pub models: IdMap<u64, ModelInfo>,
    pub skins: BTreeMap<String, SkinnedModel>,
    /// The extracted Skeleton asset per entity, and any current draw issue.
    pub pose_sources: IdMap<u64, u64>,
    pub pose_issues: IdMap<u64, Value>,
    /// Looks and palettes (`crate::present::look`).
    pub look: crate::present::look::LookState,
    /// The GI switch of the last quality preset applied while the world
    /// had no og.GlobalIllumination to write it to (og.render.applyQuality;
    /// `None` once a preset is written there): presentation state, never
    /// the world's.
    pub gi_preset: Option<bool>,
    /// og-anim on this world (its palettes and blendshape weights).
    pub anim: Option<og_anim::AnimationSystem>,
    /// Instance transforms by (entity, part) of the last two ticks, for
    /// per-object motion blur.
    pub tick: u64,
    pub motion_clock: MotionClock,
    /// In instance order; a key's last entry is its transform.
    pub current: Vec<((u64, u32), Mat4)>,
    pub previous: Vec<((u64, u32), Mat4)>,
    /// `previous` by key, made when a frame with object motion blur first
    /// asks for it in its tick (most frames never do).
    pub previous_by_key: Option<BTreeMap<(u64, u32), Mat4>>,
    /// The last display frame's instance keys and their transforms as drawn
    /// (blended between ticks), side by side: what a display frame between
    /// ticks moves from. Apart, a frame whose keys are the last one's takes
    /// the transforms whole, with no copy of the keys or lookup by key.
    pub drawn_keys: Vec<(u64, u32)>,
    pub drawn: Vec<Mat4>,
    /// The number of the instance list `drawn` holds the transforms of
    /// (`crate::present::extract::KeptModels`; 0: none known).
    pub drawn_list: u64,
    /// The last frame's joints of each posed model by entity, in the world
    /// (its instance's transform times its palette): TAA's skinned motion.
    pub drawn_joints: IdMap<u64, Vec<Mat4>>,
    /// The last frame's blendshape weights of each posed model's skinned
    /// mesh, by (entity, mesh index): TAA's motion of a morphing face.
    pub drawn_weights: BTreeMap<(u64, usize), [f32; og_render::MAX_MORPH_TARGETS]>,
    /// og.Garment's welds and the entities the last tick posed (`crate::bind::garment`).
    pub garments: crate::bind::garment::Garments,
    /// This frame's `deformed_vertices` entries' (entity, primitive) and
    /// instance transform, in their order.
    pub deformed_keys: Vec<((u64, usize), Mat4)>,
    /// Last frame's draw inputs and transform, retained without CPU re-skinning.
    pub drawn_deformed: BTreeMap<(u64, usize), (Mat4, og_render::DeformedVertices)>,
    /// Clips baked for og.CrowdAnimation, by (skeleton, skin, clip, rate
    /// bits): their index in `RenderScene::baked_clips`, its frames and
    /// the clip's seconds.
    pub baked: BTreeMap<(u64, u32, u64, u32), Baked>,
    /// Each crowd entity's baked clip, frame and transform as the last
    /// frame drew it (TAA's motion), and this frame's.
    pub drawn_baked: IdMap<u64, (u32, f32, Mat4)>,
    pub baked_now: IdMap<u64, (u32, f32, Mat4)>,
}

// Its readers are 3D (terrain.rs).
pub(crate) fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |x| x as f32)
}

/// og.ParticleSystem.startTick's default (the IDL's `PARTICLES_ARMED`):
/// never written, so a one-shot emitter has not fired.
const PARTICLES_ARMED: u32 = u32::MAX;

/// Every row of `rows` sorted by entity index, borrowed directly from the
/// world's packed archetype columns without JSON allocation.
pub(crate) fn sorted_column_rows<'a>(
    rows: &'a crate::present::extract::ColumnRows<'a>,
) -> Vec<(u64, ColRow<'a>)> {
    let mut out: Vec<_> = rows.iter().collect();
    og_core::sort::sort_by_key(&mut out, |(e, _)| Entity::from_bits(*e).index());
    out
}

/// Ensures an emitter's streamed texture array (if any) is resident and bound
/// on the renderer's `ParticleEmitter`. Returns `false` while unloaded.
pub(crate) fn emitter_ready(
    scene: &mut og_render::RenderScene,
    fx: &FxState,
    texture_arrays: &[crate::world::assets::LoadedArray],
    index: usize,
    emitter: og_render::EmitterHandle,
) -> bool {
    if let Some(&a) = fx.emitter_arrays.get(&index) {
        let Some(handle) = texture_arrays.get(a).and_then(|a| a.handle) else {
            return false;
        };
        if let Some(e) = scene.emitter(emitter).filter(|e| e.array != Some(handle)) {
            let e = og_render::ParticleEmitter {
                array: Some(handle),
                ..e.clone()
            };
            scene.set_emitter(emitter, e);
        }
    }
    true
}

/// Each og.LightMask2D's layers by entity (empty until a game names it:
/// og.LightMask2D is lazy). A receiver without one is lit as mask 1.
pub(crate) fn light_masks(world: &World, layouts: &FxLayouts) -> IdMap<u64, u32> {
    let Some(layout) = layouts.light_mask_2d.as_ref() else {
        return IdMap::new();
    };
    let rows = crate::present::extract::column_rows(world, layout);
    if rows.is_empty() {
        return IdMap::new();
    }
    let layers = Slot::of(layout, "layers");
    rows.iter()
        .map(|(e, v)| (e, v.uint(&layers).map_or(1, |x| x as u32)))
        .collect()
}

/// Whether the quality lets global illumination draw (screen-space GI and
/// realtime light volumes): og.GlobalIllumination's `allowed`, where the
/// presets write it, and the switch of a preset applied before the world
/// had the resource (`FxState::gi_preset`).
pub(crate) fn gi_allowed(gi: Option<&Value>, preset: Option<bool>) -> bool {
    // A resource that does not read is skipped, as one not there.
    gi.is_none_or(|g| record::GlobalIllumination::from_json(g).map_or(true, |g| g.allowed))
        && preset.unwrap_or(true)
}

pub(crate) fn resource(world: &World, layout: Option<&Layout>) -> Option<Arc<Value>> {
    let layout = layout?;
    let id = world.id_by_name(&layout.schema).ok()?;
    Some(layout.decode_kept(world.resource_bytes(id).ok()?))
}

/// Decoded rows of one component in entity order: `(entity bits, value)`.
pub(crate) fn rows(world: &World, layout: Option<&Layout>) -> Vec<(u64, Value)> {
    let Some(layout) = layout else {
        return Vec::new();
    };
    let rows = crate::present::extract::column_rows(world, layout);
    let mut rows: Vec<_> = rows.iter().collect();
    og_core::sort::sort_by_key(&mut rows, |(e, _)| Entity::from_bits(*e).index());
    let mut bytes = Vec::new();
    rows.into_iter()
        .map(|(e, row)| {
            bytes.clear();
            row.pack(&mut bytes);
            (e, layout.decode(&bytes))
        })
        .collect()
}

/// [`rows`] of a component extraction reads every frame whose rows are few
/// and seldom change (the terrains and what they draw over them, the
/// atmosphere's water, ropes, cloth, soft bodies, fluids and wind motors):
/// each row's JSON is kept with its bytes ([`Layout::kept_rows`], shared by
/// the layout's clones) and decoded again only when its bytes differ from
/// those of the row at its place in the last read, as [`resource`] keeps a
/// resource's. The same rows in the same order as [`rows`], each the JSON
/// its bytes decode to; the kept rows are this read's, so a row gone is
/// dropped with it (engine s110: decoding these rows, and dropping their
/// JSON after, was most of terrain and atmosphere extraction on the web).
pub(crate) fn kept_rows(world: &World, layout: Option<&Layout>) -> Vec<(u64, Arc<Value>)> {
    let Some(layout) = layout else {
        return Vec::new();
    };
    let rows = crate::present::extract::column_rows(world, layout);
    let mut rows: Vec<_> = rows.iter().collect();
    og_core::sort::sort_by_key(&mut rows, |(e, _)| Entity::from_bits(*e).index());
    let mut kept = layout
        .kept_rows
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let last = std::mem::take(&mut *kept);
    let mut out = Vec::with_capacity(rows.len());
    for (e, row) in rows {
        let mut bytes = Vec::new();
        row.pack(&mut bytes);
        let value = match last.get(out.len()) {
            Some((b, v)) if *b == bytes => v.clone(),
            _ => Arc::new(layout.decode(&bytes)),
        };
        kept.push((bytes, value.clone()));
        out.push((e, value));
    }
    out
}

/// The world's translation, rotation about z (2D) and scale of a matrix.
fn planar(m: Mat4) -> (Vec2, f32, Vec2) {
    let (scale, rot, pos) = m.to_scale_rotation_translation();
    let (axis, angle) = rot.to_axis_angle();
    let about_z = if axis.z < 0.0 { -angle } else { angle };
    (pos.truncate(), about_z, scale.truncate())
}

#[inline]
fn col_rgb(v: &ColRow<'_>, s: &Slot, d: Vec3) -> Vec3 {
    Vec3::new(
        v.lane(s, "r").map_or(d.x, |x| x as f32),
        v.lane(s, "g").map_or(d.y, |x| x as f32),
        v.lane(s, "b").map_or(d.z, |x| x as f32),
    )
}

#[inline]
fn col_vec2(v: &ColRow<'_>, s: &Slot, d: Vec2) -> Vec2 {
    Vec2::new(
        v.lane(s, "x").map_or(d.x, |x| x as f32),
        v.lane(s, "y").map_or(d.y, |x| x as f32),
    )
}

#[inline]
fn col_vec3(v: &ColRow<'_>, s: &Slot, d: Vec3) -> Vec3 {
    Vec3::new(
        v.lane(s, "x").map_or(d.x, |x| x as f32),
        v.lane(s, "y").map_or(d.y, |x| x as f32),
        v.lane(s, "z").map_or(d.z, |x| x as f32),
    )
}

#[inline]
fn col_blend_additive(v: &ColRow<'_>, s: &Slot, default_additive: bool) -> bool {
    match v.text(s).as_deref() {
        Some("additive") => true,
        Some("alpha") => false,
        _ => match v.uint(s) {
            Some(1) => true,
            Some(0) => false,
            _ => default_additive,
        },
    }
}

/// Lightweight tick-start extraction of `og.ParticleSystem` rows only, for
/// `presentation::before_tick` history recording without touching unrelated
/// scene lists.
pub(crate) fn extract_particles(
    world: &World,
    rs: &mut RenderState,
    h: &mut Hierarchy,
    tick: u64,
    out: &mut Vec<ParticleSystem>,
) {
    let RenderState {
        scene,
        fx,
        texture_arrays,
        draw,
        ..
    } = rs;
    let Some(layout) = draw.fx.particles.as_ref() else {
        return;
    };
    let rows = crate::present::extract::column_rows(world, layout);
    if rows.is_empty() {
        return;
    }
    let groups = world.time_groups();
    let mut air: Option<og_env::air::Air> = None;
    let visible_s = Slot::of(layout, "visible");
    let emitter_s = Slot::of(layout, "emitter");
    let seed_s = Slot::of(layout, "seed");
    let start_s = Slot::of(layout, "startTick");
    let stop_s = Slot::of(layout, "stopTick");
    for (e, v) in sorted_column_rows(&rows) {
        if !v.flag(&visible_s).unwrap_or(true) {
            continue;
        }
        let emitter_bits = v.uint(&emitter_s).unwrap_or(0);
        let Some(index) = crate::world::assets::index_of(emitter_bits as f64) else {
            continue;
        };
        let Some(&emitter) = fx.emitters.get(index) else {
            continue;
        };
        if !emitter_ready(scene, fx, texture_arrays, index, emitter) {
            continue;
        }
        let start_raw = v.uint(&start_s).map_or(PARTICLES_ARMED, |x| x as u32);
        let start = match start_raw {
            PARTICLES_ARMED if scene.emitter(emitter).is_some_and(|e| !e.looping) => continue,
            PARTICLES_ARMED => 0,
            t => t,
        };
        let stop = v.uint(&stop_s).map_or(0, |x| x as u32);
        let seed = v.uint(&seed_s).map_or(0, |x| x as u32);
        let transform = h.tick_world_of(e);
        let air = air.get_or_insert_with(|| draw.atmo.air(world, h));
        out.push(ParticleSystem {
            id: e,
            emitter,
            transform,
            air: air.at(transform.w_axis.truncate()),
            seed: u64::from(seed),
            start_tick: u64::from(start),
            stop_tick: (stop > 0).then_some(u64::from(stop)),
            tick,
            time_scale: groups
                .as_ref()
                .map_or(1.0, |g| world.group_scale(g, Entity::from_bits(e))),
            tint: Vec3::ONE,
            emission: 1.0,
            hue: 0.0,
        });
    }
}

/// The effect components: 2D lights and occluders, decals, particle
/// systems, trails and beams; and the queued ribbons.
pub(crate) fn components(world: &World, rs: &mut RenderState, h: &mut Hierarchy, tick: u64) {
    let RenderState {
        scene,
        fx,
        textures,
        texture_arrays,
        draw,
        effects,
        geometries,
        ..
    } = rs;
    let layouts = &draw.fx;
    scene.sprite.lights_2d.clear();
    scene.sprite.occluders_2d.clear();
    scene.effects.decals.clear();
    scene.effects.particles.clear();
    scene.effects.particle_layers.clear();
    scene.effects.effect_meshes.clear();
    scene.effects.hosted_particles.clear();
    scene.effects.trails.clear();
    scene.effects.beams.clear();
    scene.effects.ribbons.clear();
    let texture =
        |h: u64| crate::world::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    // og.load.effect's handle: its load index.
    let effect = |h: u64| {
        crate::world::assets::index_of(h as f64).and_then(|i| effects.handles.get(i).copied())
    };

    // ── og.Light2D ───────────────────────────────────────────────────────────
    if let Some(layout) = layouts.light_2d.as_ref() {
        let rows = crate::present::extract::column_rows(world, layout);
        if !rows.is_empty() {
            let color_s = Slot::of(layout, "color");
            let intensity_s = Slot::of(layout, "intensity");
            let distance_s = Slot::of(layout, "distance");
            let falloff_s = Slot::of(layout, "falloff");
            let source_radius_s = Slot::of(layout, "sourceRadius");
            let cast_shadows_s = Slot::of(layout, "castShadows");
            let kind_s = Slot::of(layout, "kind");
            let inner_angle_s = Slot::of(layout, "innerAngle");
            let outer_angle_s = Slot::of(layout, "outerAngle");
            let height_s = Slot::of(layout, "height");
            let layers_s = Slot::of(layout, "layers");
            for (e, v) in sorted_column_rows(&rows) {
                let m = h.world_of(e);
                let (pos, _, _) = planar(m);
                // A spot's and a global light's axis: the entity's local +x in the plane.
                let axis = m.x_axis.truncate().truncate();
                let kind = match v.text(&kind_s).as_deref() {
                    Some("spot") => Light2dKind::Spot,
                    Some("global") => Light2dKind::Global,
                    Some("point") => Light2dKind::Point,
                    _ => match v.uint(&kind_s) {
                        Some(1) => Light2dKind::Spot,
                        Some(2) => Light2dKind::Global,
                        _ => Light2dKind::Point,
                    },
                };
                // ColRow::num returns None for NaN, keeping defaults so
                // og-render's inner.clamp(0.0, outer) never sees NaN.
                scene.sprite.lights_2d.push(Light2d {
                    position: pos,
                    color: col_rgb(&v, &color_s, Vec3::ONE),
                    intensity: v.num(&intensity_s).map_or(1.0, |x| x as f32),
                    radius: v.num(&distance_s).map_or(5.0, |x| x as f32),
                    falloff: v.num(&falloff_s).map_or(2.0, |x| x as f32),
                    source_radius: v.num(&source_radius_s).map_or(0.25, |x| x as f32),
                    cast_shadows: v.flag(&cast_shadows_s).unwrap_or(true),
                    kind,
                    direction: axis.normalize_or(Vec2::X),
                    inner_angle: v.num(&inner_angle_s).map_or(0.35, |x| x as f32),
                    outer_angle: v.num(&outer_angle_s).map_or(0.6, |x| x as f32),
                    height: v.num(&height_s).map_or(1.0, |x| x as f32),
                    layers: v.uint(&layers_s).map_or(u32::MAX, |x| x as u32),
                });
            }
        }
    }

    // ── og.Occluder2D ────────────────────────────────────────────────────────
    if let Some(layout) = layouts.occluder_2d.as_ref() {
        let rows = crate::present::extract::column_rows(world, layout);
        if !rows.is_empty() {
            let width_s = Slot::of(layout, "width");
            let height_s = Slot::of(layout, "height");
            let points_s = Slot::of(layout, "points");
            let layers_s = Slot::of(layout, "layers");
            let mut pts = Vec::new();
            for (e, v) in sorted_column_rows(&rows) {
                let (pos, rot, scale) = planar(h.world_of(e));
                v.list_f32(&points_s, &mut pts);
                // The same outline light2dAt reads (`crate::present::light2d_at`).
                let polygon: Vec<Vec2> = if pts.len() >= 6 {
                    crate::present::light2d_at::local_outline(&pts, 0.0, 0.0)
                        .into_iter()
                        .map(|p| p * scale)
                        .collect()
                } else {
                    Vec::new()
                };
                scene.sprite.occluders_2d.push(Occluder2d {
                    half_size: Vec2::new(
                        v.num(&width_s).map_or(1.0, |x| x as f32),
                        v.num(&height_s).map_or(1.0, |x| x as f32),
                    ) * scale
                        * 0.5,
                    layers: v.uint(&layers_s).map_or(u32::MAX, |x| x as u32),
                    ..Occluder2d::polygon(pos, rot, &polygon)
                });
            }
        }
    }

    // ── og.Decal ─────────────────────────────────────────────────────────────
    if let Some(layout) = layouts.decal.as_ref() {
        let rows = crate::present::extract::column_rows(world, layout);
        if !rows.is_empty() {
            let visible_s = Slot::of(layout, "visible");
            let texture_s = Slot::of(layout, "texture");
            let normal_map_s = Slot::of(layout, "normalMap");
            let color_s = Slot::of(layout, "color");
            let opacity_s = Slot::of(layout, "opacity");
            let uv_min_s = Slot::of(layout, "uvMin");
            let uv_max_s = Slot::of(layout, "uvMax");
            let emissive_s = Slot::of(layout, "emissive");
            let roughness_s = Slot::of(layout, "roughness");
            let metallic_s = Slot::of(layout, "metallic");
            let wet_s = Slot::of(layout, "wet");
            let angle_fade_s = Slot::of(layout, "angleFade");
            let normal_scale_s = Slot::of(layout, "normalScale");
            for (e, v) in sorted_column_rows(&rows) {
                if !v.flag(&visible_s).unwrap_or(true) {
                    continue;
                }
                let c = col_rgb(&v, &color_s, Vec3::ONE);
                let lo = col_vec2(&v, &uv_min_s, Vec2::ZERO);
                let hi = col_vec2(&v, &uv_max_s, Vec2::ONE);
                scene.effects.decals.push(Decal {
                    transform: h.world_of(e),
                    texture: texture(v.uint(&texture_s).unwrap_or(0)),
                    normal_map: texture(v.uint(&normal_map_s).unwrap_or(0)),
                    uv_rect: [lo.x, lo.y, hi.x, hi.y],
                    color: [c.x, c.y, c.z, v.num(&opacity_s).map_or(1.0, |x| x as f32)],
                    emissive: col_rgb(&v, &emissive_s, Vec3::ZERO),
                    roughness: v.num(&roughness_s).map_or(-1.0, |x| x as f32),
                    metallic: v.num(&metallic_s).map_or(-1.0, |x| x as f32),
                    wet: v.flag(&wet_s).unwrap_or(false),
                    angle_fade_degrees: v.num(&angle_fade_s).map_or(75.0, |x| x as f32),
                    normal_scale: v.num(&normal_scale_s).map_or(1.0, |x| x as f32),
                });
            }
        }
    }

    // ── og.ParticleSystem ────────────────────────────────────────────────────
    if let Some(layout) = layouts.particles.as_ref() {
        let rows = crate::present::extract::column_rows(world, layout);
        if !rows.is_empty() {
            let hosted = fx.host.as_ref().map(|h| crate::bind::particles::lock(h));
            let groups = world.time_groups();
            let mut air: Option<og_env::air::Air> = None;
            let visible_s = Slot::of(layout, "visible");
            let emitter_s = Slot::of(layout, "emitter");
            let seed_s = Slot::of(layout, "seed");
            let start_s = Slot::of(layout, "startTick");
            let stop_s = Slot::of(layout, "stopTick");
            let layers_s = Slot::of(layout, "layers");
            for (e, v) in sorted_column_rows(&rows) {
                if !v.flag(&visible_s).unwrap_or(true) {
                    continue;
                }
                let emitter_bits = v.uint(&emitter_s).unwrap_or(0);
                let Some(index) = crate::world::assets::index_of(emitter_bits as f64) else {
                    continue;
                };
                let Some(&emitter) = fx.emitters.get(index) else {
                    continue;
                };
                if !emitter_ready(scene, fx, texture_arrays, index, emitter) {
                    continue;
                }
                let start_raw = v.uint(&start_s).map_or(PARTICLES_ARMED, |x| x as u32);
                // An unwritten startTick (PARTICLES_ARMED): a looping emitter runs
                // as if started at tick 0, a one-shot waits for its tick.
                let start = match start_raw {
                    PARTICLES_ARMED if scene.emitter(emitter).is_some_and(|e| !e.looping) => {
                        continue;
                    }
                    PARTICLES_ARMED => 0,
                    t => t,
                };
                let stop = v.uint(&stop_s).map_or(0, |x| x as u32);
                let seed = v.uint(&seed_s).map_or(0, |x| x as u32);
                let layers = v.uint(&layers_s).map_or(1, |x| x as u32);
                if layers != 1 {
                    scene
                        .effects
                        .particle_layers
                        .resize(scene.effects.particles.len(), 1);
                    scene.effects.particle_layers.push(layers);
                }
                if let Some(hs) = hosted.as_ref().and_then(|h| h.systems.get(&e)) {
                    scene
                        .effects
                        .hosted_particles
                        .push((e, Arc::new(hs.cpu.clone())));
                }
                // Uninterpolated tick transform keeps particle simulation deterministic
                // and independent of display-frame interpolation phase.
                let transform = h.tick_world_of(e);
                let air = air.get_or_insert_with(|| draw.atmo.air(world, h));
                scene.effects.particles.push(ParticleSystem {
                    id: e,
                    emitter,
                    transform,
                    air: air.at(transform.w_axis.truncate()),
                    seed: u64::from(seed),
                    start_tick: u64::from(start),
                    stop_tick: (stop > 0).then_some(u64::from(stop)),
                    tick,
                    // Its time group's scale (og.TimeGroup): slowed sparks.
                    time_scale: groups
                        .as_ref()
                        .map_or(1.0, |g| world.group_scale(g, Entity::from_bits(e))),
                    tint: Vec3::ONE,
                    emission: 1.0,
                    hue: 0.0,
                });
            }
        }
    }

    // ── og.Trail ─────────────────────────────────────────────────────────────
    fx.active_trails.clear();
    if let Some(layout) = layouts.trail.as_ref() {
        let rows = crate::present::extract::column_rows(world, layout);
        if !rows.is_empty() {
            let lifetime_s = Slot::of(layout, "lifetime");
            let min_dist_s = Slot::of(layout, "minDistance");
            let width_start_s = Slot::of(layout, "widthStart");
            let width_end_s = Slot::of(layout, "widthEnd");
            let color_s = Slot::of(layout, "color");
            let opacity_s = Slot::of(layout, "opacity");
            let color_end_s = Slot::of(layout, "colorEnd");
            let opacity_end_s = Slot::of(layout, "opacityEnd");
            let texture_s = Slot::of(layout, "texture");
            let effect_s = Slot::of(layout, "effect");
            let uv_tile_s = Slot::of(layout, "uvTile");
            let blend_s = Slot::of(layout, "blend");
            let layered_s = Slot::of(layout, "layered");
            let layer_s = Slot::of(layout, "layer");
            for (e, v) in sorted_column_rows(&rows) {
                let c0 = col_rgb(&v, &color_s, Vec3::ONE);
                let c1 = col_rgb(&v, &color_end_s, Vec3::ONE);
                let tile = v.num(&uv_tile_s).map_or(0.0, |x| x as f32);
                let additive = col_blend_additive(&v, &blend_s, true);
                let style = TrailStyle {
                    lifetime: v.num(&lifetime_s).map_or(0.4, |x| x as f32),
                    min_distance: v.num(&min_dist_s).map_or(0.05, |x| x as f32),
                    width: Curve::linear(
                        v.num(&width_start_s).map_or(0.15, |x| x as f32),
                        v.num(&width_end_s).map_or(0.0, |x| x as f32),
                    ),
                    color: ColorCurve::new(&[
                        (
                            0.0,
                            [
                                c0.x,
                                c0.y,
                                c0.z,
                                v.num(&opacity_s).map_or(1.0, |x| x as f32),
                            ],
                        ),
                        (
                            1.0,
                            [
                                c1.x,
                                c1.y,
                                c1.z,
                                v.num(&opacity_end_s).map_or(0.0, |x| x as f32),
                            ],
                        ),
                    ]),
                    texture: texture(v.uint(&texture_s).unwrap_or(0)),
                    uv: if tile > 0.0 {
                        RibbonUv::Tile { length: tile }
                    } else {
                        RibbonUv::Stretch
                    },
                    blend: if additive {
                        ParticleBlend::Additive
                    } else {
                        ParticleBlend::Alpha
                    },
                    effect: effect(v.uint(&effect_s).unwrap_or(0)),
                };
                let handle = fx.trail_style(scene, e, style);
                if v.flag(&layered_s).unwrap_or(false) {
                    scene
                        .effects
                        .trail_layers
                        .push((e, v.int(&layer_s).map_or(0, |x| x as i32)));
                }
                // Uninterpolated tick position keeps trail history ring samples
                // deterministic across varying display refresh rates.
                scene.effects.trails.push(Trail {
                    id: e,
                    style: handle,
                    position: h.tick_world_of(e).w_axis.truncate(),
                    tick,
                });
            }
        }
    }

    // ── og.Beam ──────────────────────────────────────────────────────────────
    if let Some(layout) = layouts.beam.as_ref() {
        let rows = crate::present::extract::column_rows(world, layout);
        if !rows.is_empty() {
            let end_s = Slot::of(layout, "end");
            let width_s = Slot::of(layout, "width");
            let color_s = Slot::of(layout, "color");
            let opacity_s = Slot::of(layout, "opacity");
            let target_s = Slot::of(layout, "target");
            let texture_s = Slot::of(layout, "texture");
            let effect_s = Slot::of(layout, "effect");
            let uv_tile_s = Slot::of(layout, "uvTile");
            let scroll_s = Slot::of(layout, "scroll");
            let segments_s = Slot::of(layout, "segments");
            let jitter_s = Slot::of(layout, "jitter");
            let seed_s = Slot::of(layout, "seed");
            let blend_s = Slot::of(layout, "blend");
            for (e, v) in sorted_column_rows(&rows) {
                let target = v.uint(&target_s).unwrap_or(0);
                // Its far end at its target's place, or `end`; a target gone draws no beam.
                let end = match target {
                    0 => col_vec3(&v, &end_s, Vec3::ZERO),
                    t if world.is_alive(Entity::from_bits(t)) => h.world_of(t).w_axis.truncate(),
                    _ => continue,
                };
                let c = col_rgb(&v, &color_s, Vec3::ONE);
                let tile = v.num(&uv_tile_s).map_or(1.0, |x| x as f32);
                let additive = col_blend_additive(&v, &blend_s, true);
                scene.effects.beams.push(Beam {
                    start: h.world_of(e).w_axis.truncate(),
                    end,
                    width: v.num(&width_s).map_or(0.1, |x| x as f32),
                    color: [c.x, c.y, c.z, v.num(&opacity_s).map_or(1.0, |x| x as f32)],
                    texture: texture(v.uint(&texture_s).unwrap_or(0)),
                    uv: if tile > 0.0 {
                        RibbonUv::Tile { length: tile }
                    } else {
                        RibbonUv::Stretch
                    },
                    scroll: v.num(&scroll_s).map_or(0.0, |x| x as f32),
                    segments: v.uint(&segments_s).map_or(1, |x| x as u32).max(1),
                    jitter: v.num(&jitter_s).map_or(0.0, |x| x as f32),
                    seed: u64::from(v.uint(&seed_s).map_or(0, |x| x as u32)),
                    blend: if additive {
                        ParticleBlend::Additive
                    } else {
                        ParticleBlend::Alpha
                    },
                    effect: effect(v.uint(&effect_s).unwrap_or(0)),
                });
            }
        }
    }

    // ── og.EffectMesh ────────────────────────────────────────────────────────
    if let Some(layout) = layouts.effect_mesh.as_ref() {
        let rows = crate::present::extract::column_rows(world, layout);
        if !rows.is_empty() {
            let geom_s = Slot::of(layout, "geometry");
            let effect_s = Slot::of(layout, "effect");
            let life_s = Slot::of(layout, "life");
            let color_s = Slot::of(layout, "color");
            let opacity_s = Slot::of(layout, "opacity");
            let seed_s = Slot::of(layout, "seed");
            let blend_s = Slot::of(layout, "blend");
            let billboard_s = Slot::of(layout, "billboard");
            for (e, v) in sorted_column_rows(&rows) {
                let geom_bits = v.uint(&geom_s).unwrap_or(0);
                let mesh = crate::world::assets::index_of(geom_bits as f64)
                    .and_then(|i| geometries.get(i).copied());
                let (Some(mesh), Some(material)) = (mesh, effect(v.uint(&effect_s).unwrap_or(0)))
                else {
                    continue;
                };
                let c = col_rgb(&v, &color_s, Vec3::ONE);
                let additive = col_blend_additive(&v, &blend_s, true);
                scene.effects.effect_meshes.push(og_render::EffectMesh {
                    id: e,
                    mesh,
                    material,
                    transform: h.world_of(e),
                    life: v.num(&life_s).map_or(0.0, |x| x as f32).clamp(0.0, 1.0),
                    seed: u64::from(v.uint(&seed_s).map_or(0, |x| x as u32)),
                    color: [c.x, c.y, c.z, v.num(&opacity_s).map_or(1.0, |x| x as f32)],
                    blend: if additive {
                        ParticleBlend::Additive
                    } else {
                        ParticleBlend::Alpha
                    },
                    layers: 1,
                    hue: 0.0,
                    billboard: v.flag(&billboard_s).unwrap_or(false),
                });
            }
        }
    }
}

impl FxState {
    /// Returns the mutable style handle owned by trail `id`, updating it
    /// in-place via `set_trail_style` so animated parameters never leak handles
    /// or wipe GPU history rings.
    pub(crate) fn trail_style(
        &mut self,
        scene: &mut og_render::RenderScene,
        id: u64,
        style: TrailStyle,
    ) -> TrailStyleHandle {
        self.active_trails.push(id);
        if let Some(&h) = self.entity_trail_styles.get(&id) {
            scene.set_trail_style(h, style);
            return h;
        }
        let h = match self.free_trail_styles.pop() {
            Some(h) => {
                scene.set_trail_style(h, style);
                h
            }
            None => scene.add_trail_style(style),
        };
        self.entity_trail_styles.insert(id, h);
        h
    }

    /// Returns trail style handles of entities absent from the current frame
    /// to the free pool.
    pub(crate) fn reclaim_trail_styles(&mut self) {
        if self.active_trails.is_empty() {
            self.free_trail_styles
                .extend(self.entity_trail_styles.values().copied());
            self.entity_trail_styles.clear();
            return;
        }
        self.active_trails.sort_unstable();
        self.active_trails.dedup();
        let active = &self.active_trails;
        let free = &mut self.free_trail_styles;
        self.entity_trail_styles.retain(|id, h| {
            if active.binary_search(id).is_ok() {
                true
            } else {
                free.push(*h);
                false
            }
        });
        self.active_trails.clear();
    }
}

/// Queued sprites and ribbons into the scene (world sprites with their
/// effects, screen sprites into the overlay).
pub(crate) fn queued(rs: &mut RenderState, fresh: bool, viewport: (f32, f32)) {
    // (sprite_fx is cleared with the frame's lists in extract: og.Sprite rows add to it first.)
    let RenderState { scene, fx, .. } = rs;
    if !fresh {
        return;
    }
    scene.effects.ribbons.extend(fx.ribbons.iter().cloned());
    for q in &fx.sprites {
        if q.screen {
            let o = scene
                .ui
                .overlay
                .get_or_insert_with(|| og_render::Overlay::new(viewport.0, viewport.1, 0.0));
            let clip = q.clip.map(|c| o.add_clip(c));
            if let Some((array, layer)) = q.array {
                o.sprite_arrays.push(og_render::SpriteArray {
                    sprite: o.sprites.len() as u32,
                    array,
                    layer,
                });
            }
            o.sprites.push(og_render::Sprite {
                clip,
                layer: og_render::screen_layer(q.sprite.layer),
                ..q.sprite
            });
            continue;
        }
        let index = scene.sprite.sprites.len() as u32;
        crate::present::feel::set_facing(&mut scene.sprite.sprite_facing, index as usize, q.facing);
        scene.sprite.sprites.push(q.sprite);
        if let Some((array, layer)) = q.array {
            scene.sprite.sprite_arrays.push(og_render::SpriteArray {
                sprite: index,
                array,
                layer,
            });
        }
        if q.order != 0.0 {
            scene.sprite.sprite_order.push(og_render::SpriteOrder {
                sprite: index,
                key: q.order,
            });
        }
        if q.cast_shadow {
            scene
                .sprite
                .occluders_2d
                .push(og_render::Occluder2d::from_sprite(&q.sprite, 0.0));
        }
        if q.drop_shadow > 0.0 {
            scene.sprite.sprite_shadows.push(og_render::SpriteShadow {
                sprite: index,
                opacity: q.drop_shadow.min(1.0),
                ground: og_render::ShadowGround::level(q.drop_shadow_height),
            });
        }
        if q.flash[3] > 0.0 || q.glow > 0.0 || q.additive || !q.tone_mapped {
            scene.sprite.sprite_fx.push(og_render::SpriteFx {
                sprite: index,
                flash: q.flash,
                glow: q.glow,
                blend: if q.additive {
                    og_render::SpriteBlend::Additive
                } else {
                    og_render::SpriteBlend::Alpha
                },
                tone_mapped: q.tone_mapped,
            });
        }
    }
}

/// The overlay's size in CSS pixels: the host's viewport, else the target.
pub(crate) fn viewport(rs: &RenderState) -> (f32, f32) {
    if let Some(v) = rs.preview.or(rs.viewport) {
        return (v.width.max(1.0), v.height.max(1.0));
    }
    match rs.backend.as_ref().map(|b| b.size()) {
        Some((w, h)) if w > 0 && h > 0 => (w as f32, h as f32),
        _ => (1280.0, 720.0),
    }
}
