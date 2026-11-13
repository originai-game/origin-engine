//! The effects a game declares as data, extracted into og-render's scene
//! (the IDL's `PostEffects`, `Lighting2D`, `Light2D`, `Occluder2D`, `Decal`,
//! `ParticleSystem`, `Trail`, `Beam`, `og.render.ribbon`, the sprite effect
//! and screen fields, and skinned models posed by og-anim), and the loads of
//! their assets (`og.load.lut`, `og.load.emitter`). Like the rest of
//! extraction it reads the world and never writes it.

use super::codec::Layout;
use super::extract::{Hierarchy, RenderState};
use super::wire::{self, FromJson, StripBlend, record};
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

/// Screen-space GI's settings: a lazy resource (crate::lazy), so a world
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
    pub vfx: Option<Layout>,
    pub skeleton: Option<Layout>,
    pub crowd: Option<Layout>,
    pub stylise: Option<Layout>,
    pub material_style: Option<Layout>,
    /// The art contract's resources the runtime reads (`super::art`).
    pub art_line: Option<Layout>,
    pub art_pixels: Option<Layout>,
    pub art_camera: Option<Layout>,
    pub art_palette: Option<Layout>,
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
            vfx: get(crate::vfx::COMPONENT),
            skeleton: get("og.Skeleton"),
            crowd: get("og.CrowdAnimation"),
            stylise: get("og.Stylise"),
            material_style: get("og.MaterialStyle"),
            art_line: get("og.ArtLine"),
            art_pixels: get("og.ArtPixels"),
            art_camera: get("og.ArtCamera"),
            art_palette: get("og.ArtPalette"),
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
    pub part_shown: Vec<bool>,
    /// Per `KHR_materials_variants` variant, each part's material
    /// (`ModelParts::variants`): og.Model.variant draws with these.
    pub part_variants: Vec<Vec<og_render::MaterialHandle>>,
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
    pub streams: Vec<og_bundle::runtime::SkinStream>,
    pub morphs: Vec<og_bundle::runtime::MorphTargets>,
    pub meshes: Option<Vec<Option<MeshHandle>>>,
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
    /// og.particles' state (`super::particle_events`): the systems the
    /// tick simulates, drawn as they stand. Set at the first emitter load.
    pub host: Option<Arc<std::sync::Mutex<super::particle_events::Hosted>>>,
    /// Trail styles made from Trail components, by their fields' bits.
    pub trail_styles: BTreeMap<Vec<u32>, TrailStyleHandle>,
    /// Ribbons and sprites queued this tick (`og.render.ribbon`, `.sprite`).
    pub ribbons: Vec<og_render::Ribbon>,
    pub sprites: Vec<QueuedSprite>,
    /// Loaded models by asset handle bits, and skin data by model id.
    pub models: IdMap<u64, ModelInfo>,
    pub skins: BTreeMap<String, SkinnedModel>,
    /// The extracted Skeleton asset per entity, and any current draw issue.
    pub pose_sources: IdMap<u64, u64>,
    pub pose_issues: IdMap<u64, Value>,
    /// Looks and palettes (`super::look`).
    pub look: super::look::LookState,
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
    /// In instance order; a key's last entry is its transform.
    pub current: Vec<((u64, u32), Mat4)>,
    pub previous: Vec<((u64, u32), Mat4)>,
    /// `previous` by key, made when a frame with object motion blur first
    /// asks for it in its tick (most frames never do).
    pub previous_by_key: Option<BTreeMap<(u64, u32), Mat4>>,
    /// The last display frame's instance transforms by key, as drawn (blended
    /// between ticks): what a display frame between ticks moves from.
    pub drawn: Vec<((u64, u32), Mat4)>,
    /// The last frame's joints of each posed model by entity, in the world
    /// (its instance's transform times its palette): TAA's skinned motion.
    pub drawn_joints: IdMap<u64, Vec<Mat4>>,
    /// The last frame's blendshape weights of each posed model's skinned
    /// mesh, by (entity, mesh index): TAA's motion of a morphing face.
    pub drawn_weights: BTreeMap<(u64, usize), [f32; og_render::MAX_MORPH_TARGETS]>,
    /// og.Garment's welds and the entities the last tick posed (`super::garment`).
    pub garments: super::garment::Garments,
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
#[cfg(not(feature = "profile2d"))]
pub(crate) fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |x| x as f32)
}

/// og.ParticleSystem.startTick's default (the IDL's `PARTICLES_ARMED`):
/// never written, so a one-shot emitter has not fired.
const PARTICLES_ARMED: u32 = u32::MAX;

/// Each og.LightMask2D's layers by entity (empty until a game names it:
/// og.LightMask2D is lazy). A receiver without one is lit as mask 1.
pub(crate) fn light_masks(world: &World, layouts: &FxLayouts) -> IdMap<u64, u32> {
    rows(world, layouts.light_mask_2d.as_ref())
        .into_iter()
        .filter_map(|(e, v)| Some((e, record::LightMask2D::from_json(&v).ok()?.layers)))
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
    let rows = super::extract::column_rows(world, layout);
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
    let rows = super::extract::column_rows(world, layout);
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

/// The effect components: 2D lights and occluders, decals, particle
/// systems, trails and beams; and the queued ribbons.
pub(crate) fn components(world: &World, rs: &mut RenderState, h: &mut Hierarchy, tick: u64) {
    let RenderState {
        scene,
        fx,
        textures,
        texture_arrays,
        draw,
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
    let hosted = fx.host.as_ref().map(|h| super::particle_events::lock(h));
    scene.effects.trails.clear();
    scene.effects.beams.clear();
    scene.effects.ribbons.clear();
    let texture = |h: u64| super::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    let rgb = |c: &wire::Color| Vec3::new(c.r, c.g, c.b);
    // Time groups (og.TimeGroups), once a game uses them.
    let groups = world.time_groups();
    for (e, v) in rows(world, layouts.light_2d.as_ref()) {
        let Ok(v) = record::Light2D::from_json(&v) else {
            continue;
        };
        let m = h.world_of(e);
        let (pos, _, _) = planar(m);
        // A spot's and a global light's axis: the entity's local +x in
        // the plane.
        let axis = m.x_axis.truncate().truncate();
        // A NaN angle (decoded as null) keeps its former default: with
        // both NaN og-render's `inner.clamp(0.0, outer)` would panic.
        let angle = |a: f32, d: f32| if a.is_nan() { d } else { a };
        scene.sprite.lights_2d.push(Light2d {
            position: pos,
            color: rgb(&v.color),
            intensity: v.intensity,
            radius: v.radius,
            falloff: v.falloff,
            source_radius: v.source_radius,
            cast_shadows: v.cast_shadows,
            kind: match v.kind {
                wire::Light2DKind::Spot => Light2dKind::Spot,
                wire::Light2DKind::Global => Light2dKind::Global,
                wire::Light2DKind::Point => Light2dKind::Point,
            },
            direction: axis.normalize_or(Vec2::X),
            inner_angle: angle(v.inner_angle, 0.35),
            outer_angle: angle(v.outer_angle, 0.6),
            height: v.height,
            layers: v.layers,
        });
    }
    for (e, v) in rows(world, layouts.occluder_2d.as_ref()) {
        let Ok(v) = record::Occluder2D::from_json(&v) else {
            continue;
        };
        let (pos, rot, scale) = planar(h.world_of(e));
        // The same outline light2dAt reads (`super::light2d_at`).
        let polygon: Vec<Vec2> = if v.points.len() >= 6 {
            super::light2d_at::local_outline(&v.points, 0.0, 0.0)
                .into_iter()
                .map(|p| p * scale)
                .collect()
        } else {
            Vec::new()
        };
        scene.sprite.occluders_2d.push(Occluder2d {
            half_size: Vec2::new(v.width, v.height) * scale * 0.5,
            layers: v.layers,
            ..Occluder2d::polygon(pos, rot, &polygon)
        });
    }
    for (e, v) in rows(world, layouts.decal.as_ref()) {
        let Ok(v) = record::Decal::from_json(&v) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let c = &v.color;
        let (lo, hi) = (v.uv_min, v.uv_max);
        scene.effects.decals.push(Decal {
            transform: h.world_of(e),
            uv_rect: [lo.x, lo.y, hi.x, hi.y],
            color: [c.r, c.g, c.b, v.opacity],
            emissive: rgb(&v.emissive),
            roughness: v.roughness,
            angle_fade_degrees: v.angle_fade,
            normal_rect: {
                let (lo, hi) = (v.normal_min, v.normal_max);
                [lo.x, lo.y, hi.x, hi.y]
            },
            normal_scale: v.normal_scale,
        });
    }
    let mut air: Option<og_core::air::Air> = None;
    for (e, v) in rows(world, layouts.particles.as_ref()) {
        let Ok(v) = record::ParticleSystem::from_json(&v) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let Some(index) = super::assets::index_of(v.emitter as f64) else {
            continue;
        };
        let Some(&emitter) = fx.emitters.get(index) else {
            continue;
        };
        // An emitter drawing from a texture array follows the array's
        // renderer handle (a streaming group may have released and added
        // it again), and draws nothing while it is not resident.
        if let Some(&a) = fx.emitter_arrays.get(&index) {
            let Some(handle) = texture_arrays.get(a).and_then(|a| a.handle) else {
                continue;
            };
            if let Some(e) = scene.emitter(emitter).filter(|e| e.array != Some(handle)) {
                let e = og_render::ParticleEmitter {
                    array: Some(handle),
                    ..e.clone()
                };
                scene.set_emitter(emitter, e);
            }
        }
        let stop = v.stop_tick;
        // An unwritten startTick (PARTICLES_ARMED): a looping emitter runs
        // as if started at tick 0, a one-shot waits for its tick.
        let start = match v.start_tick {
            PARTICLES_ARMED if scene.emitter(emitter).is_some_and(|e| !e.looping) => continue,
            PARTICLES_ARMED => 0,
            t => t,
        };
        let layers = v.layers;
        if layers != 1 {
            scene
                .effects
                .particle_layers
                .resize(scene.effects.particles.len(), 1);
            scene.effects.particle_layers.push(layers);
        }
        if let Some(h) = hosted.as_ref().and_then(|h| h.systems.get(&e)) {
            scene
                .effects
                .hosted_particles
                .push((e, Arc::new(h.cpu.clone())));
        }
        let transform = h.world_of(e);
        // The frame's air, read once for the first system it carries.
        let air = air.get_or_insert_with(|| draw.atmo.air(world, h));
        scene.effects.particles.push(ParticleSystem {
            id: e,
            emitter,
            transform,
            air: air.at(transform.w_axis.truncate()),
            seed: u64::from(v.seed),
            start_tick: u64::from(start),
            stop_tick: (stop > 0).then_some(u64::from(stop)),
            tick,
            // Its time group's scale (og.TimeGroup): slowed sparks.
            time_scale: groups
                .as_ref()
                .map_or(1.0, |g| world.group_scale(g, og_ecs::Entity::from_bits(e))),
        });
    }
    for (e, v) in rows(world, layouts.trail.as_ref()) {
        let Ok(v) = record::Trail::from_json(&v) else {
            continue;
        };
        let (c0, c1) = (v.color, v.color_end);
        let tile = v.uv_tile;
        let additive = v.blend == StripBlend::Additive;
        let style = TrailStyle {
            lifetime: v.lifetime,
            min_distance: v.min_distance,
            width: Curve::linear(v.width_start, v.width_end),
            color: ColorCurve::new(&[
                (0.0, [c0.r, c0.g, c0.b, v.opacity]),
                (1.0, [c1.r, c1.g, c1.b, v.opacity_end]),
            ]),
            texture: super::assets::index_of(v.texture as f64)
                .and_then(|i| textures.get(i).copied()),
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
            effect: None,
        };
        let key = style_key(&style);
        let handle = match fx.trail_styles.get(&key) {
            Some(h) => *h,
            None => {
                let h = scene.add_trail_style(style);
                fx.trail_styles.insert(key, h);
                h
            }
        };
        if v.layered {
            scene.effects.trail_layers.push((e, v.layer));
        }
        scene.effects.trails.push(Trail {
            id: e,
            style: handle,
            position: h.world_of(e).w_axis.truncate(),
            tick,
        });
    }
    for (e, v) in rows(world, layouts.beam.as_ref()) {
        let Ok(v) = record::Beam::from_json(&v) else {
            continue;
        };
        let c = &v.color;
        scene.effects.beams.push(Beam {
            start: h.world_of(e).w_axis.truncate(),
            end: v.end,
            width: v.width,
            color: [c.r, c.g, c.b, v.opacity],
            texture: texture(v.texture),
            tile: v.tile,
            scroll: v.scroll,
            segments: v.segments.max(1),
            jitter: v.jitter,
            seed: u64::from(v.seed),
            blend: if v.blend == StripBlend::Additive {
                ParticleBlend::Additive
            } else {
                ParticleBlend::Alpha
            },
            effect: None,
        });
    }
}

/// A trail style's identity: the bits of its fields.
pub(crate) fn style_key(s: &TrailStyle) -> Vec<u32> {
    let mut k = vec![s.lifetime.to_bits(), s.min_distance.to_bits()];
    k.extend(
        s.width
            .keys
            .iter()
            .flat_map(|(t, v)| [t.to_bits(), v.to_bits()]),
    );
    k.extend(
        s.color
            .keys
            .iter()
            .flat_map(|(t, c)| std::iter::once(t.to_bits()).chain(c.iter().map(|x| x.to_bits()))),
    );
    k.push(s.texture.map_or(u32::MAX, |t| t.index() as u32));
    k.push(match s.uv {
        RibbonUv::Stretch => 0,
        RibbonUv::Tile { length } => length.to_bits(),
    });
    k.push(s.blend as u32);
    k
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
        super::feel::set_facing(&mut scene.sprite.sprite_facing, index as usize, q.facing);
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

#[cfg(test)]
mod tests;
