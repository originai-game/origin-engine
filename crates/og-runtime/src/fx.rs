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

pub(crate) fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |x| x as f32)
}

/// og.ParticleSystem.startTick's default (the IDL's `PARTICLES_ARMED`):
/// never written, so a one-shot emitter has not fired.
const PARTICLES_ARMED: u32 = u32::MAX;

pub(crate) fn u(v: &Value, key: &str, default: u32) -> u32 {
    v[key].as_f64().map_or(default, |x| x as u32)
}

pub(crate) fn on(v: &Value, key: &str, default: bool) -> bool {
    match &v[key] {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        _ => default,
    }
}

pub(crate) fn color(v: &Value, default: Vec3) -> Vec3 {
    if !v.is_object() {
        return default;
    }
    Vec3::new(
        f(v, "r", default.x),
        f(v, "g", default.y),
        f(v, "b", default.z),
    )
}

pub(crate) fn vec3(v: &Value) -> Vec3 {
    Vec3::new(f(v, "x", 0.0), f(v, "y", 0.0), f(v, "z", 0.0))
}

/// The load index of a handle field (`0` is none).
pub(crate) fn handle_index(v: &Value) -> Option<usize> {
    super::assets::index_of(v.as_f64()?)
}

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

pub(crate) fn variant(v: &Value, names: &[&str]) -> usize {
    match v {
        Value::String(s) => names.iter().position(|n| n == s).unwrap_or(0),
        Value::Number(n) => n.as_u64().unwrap_or(0) as usize,
        _ => 0,
    }
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

/// `PostEffects` and `Lighting2D` into the scene's post stack, shake and 2D lighting.
pub(crate) fn settings(world: &World, rs: &mut RenderState) {
    let RenderState {
        fx,
        scene,
        draw,
        settings: render_settings,
        textures,
        art,
        ..
    } = rs;
    let layouts = &draw.fx;
    // RenderSettings.decalAtlas: the texture Decal components sample.
    scene.effects.decal_atlas = resource(world, render_settings.as_ref())
        .and_then(|s| record::RenderSettings::from_json(&s).ok())
        .and_then(|s| super::assets::index_of(s.decal_atlas as f64))
        .and_then(|i| textures.get(i).copied());
    let post = &mut scene.look.post;
    post.ao = None;
    post.outline = None;
    post.depth_of_field = None;
    post.motion_blur = None;
    post.god_rays = None;
    post.lens_flare = None;
    post.ssr = None;
    post.gi = None;
    post.gi_probes = None;
    post.glow = None;
    post.color_grading = None;
    post.color_adjust = None;
    post.vignette = None;
    post.chromatic_aberration = None;
    post.lens_distortion = None;
    post.screen_blur = None;
    post.film_grain = None;
    post.sharpen = None;
    post.flash = None;
    scene.frame.shake = CameraShake::default();
    scene.sprite.lighting_2d = None;
    if let Some(p) = resource(world, layouts.post.as_ref())
        && let Ok(p) = record::PostEffects::from_json(&p)
    {
        let rgb = |c: &wire::Color| Vec3::new(c.r, c.g, c.b);
        let resolution = |r: wire::AoResolution| match r {
            wire::AoResolution::Full => AoResolution::Full,
            wire::AoResolution::Half => AoResolution::Half,
        };
        if p.ao {
            post.ao = Some(AmbientOcclusion {
                radius: p.ao_radius,
                intensity: p.ao_intensity,
                method: match p.ao_method {
                    wire::AoMethod::Auto => AoMethod::Auto,
                    wire::AoMethod::Gtao => AoMethod::Gtao,
                    wire::AoMethod::Ssao => AoMethod::Ssao,
                    wire::AoMethod::Screen => AoMethod::Screen,
                },
                resolution: resolution(p.ao_resolution),
                direct: p.ao_direct.clamp(0.0, 1.0),
                radius_scale: p.ao_radius_scale.max(0.0),
            });
        }
        if p.outline {
            let c = rgb(&p.outline_color);
            post.outline = Some(Outline {
                color: [c.x, c.y, c.z, 1.0],
                width: p.outline_width,
                depth_threshold: p.outline_depth_threshold,
                normal_threshold: p.outline_normal_threshold,
            });
        }
        if p.depth_of_field {
            post.depth_of_field = Some(DepthOfField {
                focus_distance: p.dof_focus_distance,
                focus_range: p.dof_focus_range,
                max_blur: p.dof_max_blur,
                falloff: p.dof_falloff,
                quality: match p.dof_quality {
                    wire::DofQuality::High => DofQuality::High,
                    wire::DofQuality::Low => DofQuality::Low,
                },
            });
        }
        if p.motion_blur {
            post.motion_blur = Some(MotionBlur {
                shutter: p.motion_blur_shutter,
                samples: p.motion_blur_samples,
                max_pixels: p.motion_blur_max_pixels,
            });
        }
        let blur_kind = match p.screen_blur {
            wire::ScreenBlur::None => None,
            wire::ScreenBlur::Zoom => Some(ScreenBlurKind::Zoom),
            wire::ScreenBlur::Spin => Some(ScreenBlurKind::Spin),
            wire::ScreenBlur::Directional => Some(ScreenBlurKind::Directional),
        };
        let blur_strength = p.screen_blur_strength;
        if let Some(kind) = blur_kind
            && blur_strength != 0.0
        {
            post.screen_blur = Some(ScreenBlur {
                kind,
                strength: blur_strength,
                center: p.screen_blur_center,
                angle: p.screen_blur_angle.to_radians(),
                samples: p.screen_blur_samples,
            });
        }
        if p.god_rays {
            post.god_rays = Some(GodRays {
                intensity: p.god_rays_intensity,
                decay: p.god_rays_decay,
                density: p.god_rays_density,
                tint: rgb(&p.god_rays_tint),
                samples: p.god_rays_samples,
                threshold: p.god_rays_threshold,
            });
        }
        if p.ssr {
            post.ssr = Some(og_render::ScreenSpaceReflections {
                max_distance: p.ssr_max_distance,
                thickness: p.ssr_thickness,
                steps: p.ssr_steps,
                fade: p.ssr_fade,
                intensity: p.ssr_intensity,
                max_roughness: p.ssr_max_roughness,
                resolution: resolution(p.ssr_resolution),
            });
        }
        if p.glow {
            post.glow = Some(Glow {
                intensity: p.glow_intensity,
            });
        }
        if let Some(lut) = super::assets::index_of(p.lut as f64)
            .and_then(|i| fx.luts.get(i))
            .and_then(|l| l.handle)
        {
            post.color_grading = Some(ColorGrading {
                lut,
                intensity: p.lut_intensity,
            });
        }
        let adjust = ColorAdjust {
            temperature: p.temperature,
            tint: p.tint,
            brightness: p.brightness,
            contrast: p.contrast,
            shadows: p.balance_shadows,
            midtones: p.balance_midtones,
            highlights: p.balance_highlights,
            preserve_luminosity: p.preserve_luminosity,
            hue: p.hue,
            saturation: p.saturation,
            vibrance: p.vibrance,
            grayscale: p.grayscale,
            invert: p.invert,
            threshold: p.threshold,
        };
        post.color_adjust = (!adjust.is_identity()).then_some(adjust);
        if p.vignette {
            post.vignette = Some(Vignette {
                intensity: p.vignette_intensity,
                radius: p.vignette_radius,
                smoothness: p.vignette_smoothness,
                color: rgb(&p.vignette_color),
            });
        }
        let aberration = p.chromatic_aberration;
        if aberration > 0.0 {
            post.chromatic_aberration = Some(ChromaticAberration {
                intensity: aberration,
            });
        }
        let lens = p.lens_distortion;
        if lens != 0.0 {
            post.lens_distortion = Some(LensDistortion {
                amount: lens,
                center: p.lens_distortion_center,
            });
        }
        let grain = p.film_grain;
        if grain > 0.0 {
            post.film_grain = Some(FilmGrain {
                intensity: grain,
                size: p.film_grain_size,
            });
        }
        let sharpen = p.sharpen;
        if sharpen >= 0.0 {
            post.sharpen = Some(sharpen);
        }
        let flash = p.flash;
        if flash > 0.0 {
            post.flash = Some(ScreenFlash {
                color: rgb(&p.flash_color),
                amount: flash,
            });
        }
        let trauma = p.shake_trauma;
        if trauma > 0.0 {
            scene.frame.shake = CameraShake::trauma(
                trauma,
                scene.frame.tick,
                scene.frame.tick_rate,
                u64::from(p.shake_seed),
                p.shake_max_offset,
                p.shake_max_roll,
                p.shake_frequency,
            );
        }
    }
    scene.look.stylise = stylise(
        resource(world, layouts.stylise.as_ref())
            .and_then(|s| record::Stylise::from_json(&s).ok())
            .as_ref(),
        resource(world, layouts.material_style.as_ref())
            .and_then(|m| record::MaterialStyle::from_json(&m).ok())
            .as_ref(),
        &fx.look.palettes,
        textures,
    );
    // Painted 2D (super::art::texel_paint): the cook painted and washed
    // each texture in its own texels, and the frame draws neither pass.
    if art.texel_paint {
        scene.look.stylise.paint = None;
    }
    if let Some(p) = resource(world, layouts.lens_flare.as_ref())
        && let Ok(p) = record::LensFlare::from_json(&p)
        && p.enabled
    {
        post.lens_flare = Some(LensFlare {
            intensity: p.intensity,
            ghosts: p.ghosts,
            ghost_spacing: p.ghost_spacing,
            ghost_threshold: p.ghost_threshold,
            halo_radius: p.halo_radius,
            halo_thickness: p.halo_thickness,
            halo_threshold: p.halo_threshold,
            chromatic_aberration: p.chromatic_aberration,
            starburst: p.starburst,
            sun: p.sun,
            lights: p.lights,
        });
    }
    // The art contract's line round meshes (ArtLine, art.rs).
    super::art::mesh_line(&art.frame, &mut scene.look.stylise.material);
    // Under the art contract a frame that quantises without a palette of
    // its own takes the contract's (ArtPalette.colors): the colours the
    // cook put the art on are the ones the frame keeps.
    if let Some(q) = scene.look.stylise.quantize.as_mut()
        && q.palette.is_none()
    {
        q.palette = art.frame.palette;
    }
    if let Some(v) = resource(world, layouts.global_illumination.as_ref())
        && let Ok(g) = record::GlobalIllumination::from_json(&v)
        && g.enabled
        && gi_allowed(Some(&v), fx.gi_preset)
    {
        if g.method != wire::GiMethod::Screen {
            post.gi_probes = Some(og_render::ProbeGlobalIllumination {
                spacing: g.probe_spacing.max(0.05),
                cascades: g.probe_cascades.clamp(1, 4),
                rays: g.probe_rays.clamp(32, 256),
                intensity: g.intensity.max(0.0),
            });
        }
        if g.method != wire::GiMethod::Probes {
            post.gi = Some(og_render::ScreenSpaceGlobalIllumination {
                intensity: g.intensity,
                max_distance: g.max_distance,
                thickness: g.thickness,
                steps: g.steps,
                rays: g.rays,
                occlusion: g.occlusion.clamp(0.0, 1.0),
                resolution: match g.resolution {
                    wire::AoResolution::Full => AoResolution::Full,
                    wire::AoResolution::Half => AoResolution::Half,
                },
            });
        }
    }
    if let Some(l) = resource(world, layouts.lighting_2d.as_ref())
        && let Ok(l) = record::Lighting2D::from_json(&l)
        && l.enabled
    {
        scene.sprite.lighting_2d = Some(Lighting2d {
            ambient: Vec3::new(l.ambient.r, l.ambient.g, l.ambient.b),
            min_layer: l.min_layer,
            max_layer: l.max_layer,
            resolution: l.resolution,
            shadow_angles: l.shadow_angles,
        });
    }
}

/// Pixel snap (`Stylise.pixelSnap`): on a logical pixel grid with the 2D
/// camera (orthographic, looking down -z), the camera and every world
/// sprite move by whole logical pixels, so a scrolling scene never swims
/// between pixel rows and sprites keep their edges. Under a project's art
/// contract, `ArtCamera.pixelPerfect` does the same on the contract's grid.
pub(crate) fn snap_to_grid(rs: &mut RenderState) {
    let art = &rs.art.frame;
    let scene = &mut rs.scene;
    // The frame's grid (Stylise.pixelHeight with pixelSnap), else the art
    // contract's (ArtCamera.pixelPerfect: ArtPixels.logicalHeight rows of
    // the view, or 1 / ArtPixels.unit world units).
    let rows = match scene.look.stylise.pixel.filter(|p| p.snap && p.height > 0) {
        Some(p) => Some(p.height),
        None if art.pixel_perfect && art.logical_height > 0 => Some(art.logical_height),
        None => None,
    };
    let og_render::Projection::Orthographic { height } = scene.frame.camera.projection else {
        return;
    };
    let unit = match rows {
        Some(rows) => height / rows as f32,
        None if art.pixel_perfect && art.unit > 0.0 => 1.0 / art.unit,
        None => return,
    };
    let forward = (scene.frame.camera.target - scene.frame.camera.position).normalize_or_zero();
    if forward.z > -0.999 {
        return;
    }
    if unit <= 0.0 {
        return;
    }
    let snap = |v: f32| (v / unit).round() * unit;
    let c = &mut scene.frame.camera;
    let d = c.target - c.position;
    c.position.x = snap(c.position.x);
    c.position.y = snap(c.position.y);
    c.target = c.position + d;
    for s in &mut scene.sprite.sprites {
        s.position.x = snap(s.position.x);
        s.position.y = snap(s.position.y);
    }
}

/// `Stylise` and `MaterialStyle` as og-render's description (every
/// primitive `None` or zero unless its field turns it on).
fn stylise(
    s: Option<&record::Stylise>,
    m: Option<&record::MaterialStyle>,
    palettes: &[(String, Option<og_render::PaletteHandle>)],
    textures: &[og_render::TextureHandle],
) -> og_render::Stylise {
    let rgb = |c: &wire::Color| [c.r, c.g, c.b];
    let texture = |h: u64| super::assets::index_of(h as f64).and_then(|i| textures.get(i).copied());
    let mut out = og_render::Stylise::default();
    if let Some(s) = s {
        let height = s.pixel_height;
        if height > 0 {
            out.pixel = Some(og_render::PixelScale {
                height,
                upscale: s.pixel_upscale,
                snap: s.pixel_snap,
            });
        }
        let posterize = s.posterize;
        // A palette whose streaming group is not resident quantises to none.
        let palette = super::assets::index_of(s.palette as f64).and_then(|i| palettes.get(i)?.1);
        let dither = match s.dither {
            wire::DitherPattern::Bayer2 => og_render::DitherPattern::Bayer2,
            wire::DitherPattern::Bayer4 => og_render::DitherPattern::Bayer4,
            wire::DitherPattern::Bayer8 => og_render::DitherPattern::Bayer8,
            wire::DitherPattern::BlueNoise => og_render::DitherPattern::BlueNoise,
            wire::DitherPattern::InterleavedGradient => {
                og_render::DitherPattern::InterleavedGradient
            }
            wire::DitherPattern::None => og_render::DitherPattern::None,
        };
        let space = |v: wire::StyliseSpace| match v {
            wire::StyliseSpace::Surface => og_render::StyliseSpace::Surface,
            wire::StyliseSpace::Object => og_render::StyliseSpace::Object,
            wire::StyliseSpace::Screen => og_render::StyliseSpace::Screen,
        };
        if posterize > 0 || palette.is_some() || dither != og_render::DitherPattern::None {
            out.quantize = Some(og_render::Quantize {
                posterize,
                mode: match s.posterize_mode {
                    wire::PosterizeMode::Tone => og_render::PosterizeMode::Tone,
                    wire::PosterizeMode::Channel => og_render::PosterizeMode::Channel,
                },
                palette,
                dither,
                dither_strength: s.dither_strength,
                dither_scale: s.dither_scale.max(1),
                space: space(s.dither_space),
            });
        }
        let halftone = s.halftone;
        if halftone > 0.0 {
            out.halftone = Some(og_render::Halftone {
                strength: halftone,
                size: s.halftone_size,
                angle_degrees: s.halftone_angle,
                color: rgb(&s.halftone_color),
                space: space(s.halftone_space),
                mode: match s.halftone_mode {
                    wire::HalftoneMode::Cmyk => og_render::HalftoneMode::Cmyk,
                    wire::HalftoneMode::Ink => og_render::HalftoneMode::Ink,
                },
                misregistration: s.halftone_misregistration,
                region: match s.halftone_region {
                    wire::HalftoneRegion::Shade => og_render::HalftoneRegion::Shade,
                    wire::HalftoneRegion::Highlight => og_render::HalftoneRegion::Highlight,
                    wire::HalftoneRegion::All => og_render::HalftoneRegion::All,
                },
            });
        }
        let hatching = s.hatching;
        if hatching > 0.0 {
            out.hatching = Some(og_render::Hatching {
                strength: hatching,
                scale: s.hatching_scale,
                levels: s.hatching_levels,
                angle_degrees: s.hatching_angle,
                color: rgb(&s.hatching_color),
                paper: rgb(&s.hatching_paper),
                space: space(s.hatching_space),
                depth: s.hatching_depth,
                keep_color: s.hatching_keep_color,
            });
        }
        let paint = og_render::Paint {
            radius: s.paint_radius,
            anisotropy: s.paint_anisotropy,
            edge_darkening: s.paint_edge_darkening,
            bleed: s.paint_bleed,
            canvas: texture(s.paint_canvas),
            canvas_strength: s.paint_canvas_strength,
            canvas_scale: s.paint_canvas_scale,
            canvas_space: space(s.paint_canvas_space),
            canvas_world_scale: s.paint_canvas_world_scale,
            detail: s.paint_detail,
            character_detail: s.paint_character_detail,
            watercolor: og_render::Watercolor {
                edge_darkening: s.watercolor_edge_darkening,
                granulation: s.watercolor_granulation,
                turbulence: s.watercolor_turbulence,
                dry_brush: s.watercolor_dry_brush,
                wobble: s.watercolor_wobble,
                paper: rgb(&s.watercolor_paper),
            },
        };
        if paint.paints() || paint.watercolor.active() {
            out.paint = Some(paint);
        }
        if s.ink {
            out.ink = Some(og_render::Ink {
                threshold: s.ink_threshold,
                width: s.ink_width,
                color: rgb(&s.ink_color),
                depth: s.ink_depth,
                depth_threshold: s.ink_depth_threshold,
                normal_threshold: s.ink_normal_threshold,
            });
        }
        let bands = s.depth_bands;
        if bands > 0 {
            out.depth_bands = Some(og_render::DepthBands {
                count: bands,
                near: s.depth_bands_near,
                far: s.depth_bands_far,
                near_color: rgb(&s.depth_bands_near_color),
                far_color: rgb(&s.depth_bands_far_color),
                flatten: s.depth_bands_flatten,
            });
        }
        let retro = og_render::Retro3d {
            vertex_snap: s.vertex_snap,
            affine: s.affine_texturing,
            depth_bits: s.depth_bits,
        };
        if retro.vertex_snap > 0.0 || retro.affine > 0.0 || retro.depth_bits > 0 {
            out.retro = Some(retro);
        }
    }
    if let Some(m) = m {
        out.material = og_render::MaterialStyle {
            toon: m.toon,
            gradient_map: texture(m.gradient_map),
            shadow_tint: rgb(&m.shadow_tint),
            shadow_tint_amount: m.shadow_tint_amount,
            shadow_darkness: m.shadow_darkness,
            rim_color: rgb(&m.rim_color),
            rim_intensity: m.rim_intensity,
            rim_power: m.rim_power,
            outline_color: rgb(&m.outline_color),
            outline_width: m.outline_width,
            outline_distance: m.outline_distance,
            flat_shading: m.flat_shading,
            surface: og_render::MaterialSurface::from_index(m.surface as usize),
            surface_strength: m.surface_strength,
            surface_scale: m.surface_scale,
            toon_ambient: m.toon_ambient,
            toon_softness: m.toon_softness,
            outline_place: match m.outline_place {
                wire::OutlinePlace::Outside => og_render::OutlinePlace::Outside,
                wire::OutlinePlace::Both => og_render::OutlinePlace::Both,
                wire::OutlinePlace::Inside => og_render::OutlinePlace::Inside,
            },
            wireframe: m.wireframe,
            wireframe_angle: m.wireframe_angle,
            wireframe_color: rgb(&m.wireframe_color),
        };
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
        scene.effects.particles.push(ParticleSystem {
            id: e,
            emitter,
            transform: h.world_of(e),
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

/// Makes the skinned copies of a model's primitives the first time it is
/// drawn posed (both og.load.gltf and og.load.animation of it are needed).
fn skinned_meshes(
    scene: &mut og_render::RenderScene,
    info: &ModelInfo,
    skin: &mut SkinnedModel,
    prims: usize,
) -> Vec<Option<MeshHandle>> {
    if let Some(m) = &skin.meshes {
        return m.clone();
    }
    let mut out = vec![None; prims];
    for (p, slot) in out.iter_mut().enumerate() {
        let stream = skin.streams.iter().find(|s| s.primitive == p);
        let morph = skin.morphs.iter().find(|m| m.primitive == p);
        if stream.is_none() && morph.is_none() {
            continue;
        }
        let Some(mesh) = info.meshes.get(p).and_then(|h| scene.mesh(*h)).cloned() else {
            continue;
        };
        let n = mesh.vertices.len();
        let data = match stream {
            Some(s) if s.joints.len() == n => SkinData {
                joints: s.joints.clone(),
                weights: s.weights.clone(),
            },
            // Morph targets alone: every vertex on joint 0 with full weight
            // (an identity palette keeps the bind pose).
            _ => SkinData {
                joints: vec![[0; 4]; n],
                weights: vec![[1.0, 0.0, 0.0, 0.0]; n],
            },
        };
        let h = scene.add_skinned_mesh(mesh, data);
        if let Some(m) = morph {
            let take = m.positions.len().min(og_render::MAX_MORPH_TARGETS);
            scene.set_morph_targets(
                h,
                MorphTargets {
                    positions: m.positions[..take].to_vec(),
                    normals: m.normals.as_ref().map(|n| n[..take].to_vec()),
                },
            );
        }
        *slot = Some(h);
    }
    skin.meshes = Some(out.clone());
    out
}

/// The entities with og.Skeleton (to draw their models posed).
pub(crate) fn skeletons(
    world: &World,
    layout: Option<&Layout>,
    fx: &mut FxState,
) -> std::collections::BTreeSet<u64> {
    fx.pose_issues.clear();
    fx.pose_sources = rows(world, layout)
        .into_iter()
        .filter_map(|(e, v)| Some((e, record::Skeleton::from_json(&v).ok()?.asset)))
        .collect();
    fx.pose_sources.keys().copied().collect()
}

/// One og.Model's own look and shadow flags, carried by each of its
/// instances: its tint and flash, `castShadow` and `receiveShadow`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ModelLook {
    pub fx: og_render::InstanceFx,
    pub cast: bool,
    pub receive: bool,
}

/// Draws a model posed by og-anim: its skinned primitives as skinned
/// instances under the entity's palette (and blendshape weights), the rest
/// as instances, each with the model's own tint, flash and shadow flags
/// (`look`). A part whose node the pose hides (a visibility channel, or
/// a hidden ancestor: `AnimOutput::drawn`) is not drawn at all: a `.vox`
/// frame folded away costs no draw. `false`
/// when it cannot (no palette yet, no skin data): the caller draws it at
/// its bind pose. On a frame between ticks the pose is blended with the
/// tick before's at `alpha` (`super::interpolate::pose`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn posed_model(
    rs: &mut RenderState,
    entity: u64,
    asset: u64,
    world_matrix: Mat4,
    alpha: Option<f32>,
    parts: &[(MeshHandle, og_render::MaterialHandle, Mat4)],
    look: ModelLook,
    keys: &mut Vec<(u64, u32)>,
) -> bool {
    let physics = rs.draw.physics.clone();
    let RenderState {
        scene,
        fx,
        atmo,
        interp,
        motion,
        notes,
        ..
    } = rs;
    let Some(anim) = fx.anim.as_ref() else {
        return false;
    };
    let Some(mut out) = anim.output(Entity::from_bits(entity)) else {
        return false;
    };
    // og.motion's generated pose, when it has one: between ticks, blended
    // from the tick before's like og-anim's.
    let matrices = match motion.drawn(entity, alpha) {
        Some(m) if m.len() == out.palette.len() => m,
        _ => super::interpolate::pose(interp, entity, alpha, &out.palette, &mut out.weights),
    };
    let Some(info) = fx.models.get(&asset).cloned() else {
        return false;
    };
    let source = fx.pose_sources.get(&entity).and_then(|&h| anim.skeleton(h));
    let reason = match fx.skins.get(&info.id) {
        None => Some("model skin data is not loaded; load its animation asset".to_string()),
        Some(model) if !model.streams.is_empty() => {
            match model.skeleton.skins.get(out.skin as usize) {
                None => Some(format!("model has no skin {}", out.skin)),
                Some(expected) if expected.joints.len() != matrices.len() => Some(format!(
                    "model skin {} needs {} joints but the palette has {}",
                    out.skin, expected.joints.len(), matrices.len(),
                )),
                Some(expected) => source.as_ref().and_then(|source| {
                    let actual = source.skins.get(out.skin as usize)?;
                    let different_order = expected.joints.iter().zip(&actual.joints).any(|(&a, &b)| {
                        model.skeleton.nodes[a as usize].name != source.nodes[b as usize].name
                    });
                    (expected.inverse_bind != actual.inverse_bind
                        || different_order)
                        .then(|| format!("model skin {} and skeleton have different joint order or inverse bind matrices", out.skin))
                }),
            }
        }
        Some(_) => None,
    };
    if let Some(reason) = reason {
        notes.push(format!("og.Model {} on entity {entity}: {reason}", info.id));
        fx.pose_issues.insert(
            entity,
            serde_json::json!({
                "model": info.id, "skin": out.skin, "paletteJoints": matrices.len(),
                "reason": reason,
            }),
        );
    }
    let Some(skin) = fx.skins.get_mut(&info.id) else {
        return false;
    };
    let meshes = skinned_meshes(scene, &info, skin, info.prim_mesh.len());
    let palette = scene.objects.joint_palettes.len() as u32;
    atmo.palettes.insert(entity, palette);
    // og.Garment: its primitives drawn at the simulated points, posed by
    // this frame's palette (`super::garment::deformed`).
    let mut garment = super::garment::deformed(
        fx,
        scene,
        physics.as_ref(),
        entity,
        asset,
        world_matrix,
        &matrices,
    );
    scene.objects.joint_palettes.push(JointPalette { matrices });
    // Parts and nodes line up unless the parts are not the model's own
    // (a bake's lightmapped parts): then every part is drawn.
    let nodes = (info.part_nodes.len() == parts.len()).then_some(&info.part_nodes);
    for (k, &(mesh, material, local)) in parts.iter().enumerate() {
        if nodes.is_some_and(|n| out.drawn.get(n[k]) == Some(&false)) {
            continue;
        }
        let p = info.primitive(mesh);
        match meshes.get(p).copied().flatten() {
            Some(skinned) => {
                let mut inst = SkinnedInstance::new(skinned, material, world_matrix, palette);
                inst.fx = look.fx;
                inst.cast_shadows = look.cast;
                inst.receive_shadows = look.receive;
                if let Some((zero, vertices)) = garment.remove(&p) {
                    inst.mesh = zero;
                    inst.deformed = Some(scene.objects.deformed_vertices.len() as u32);
                    scene.objects.deformed_vertices.push(vertices);
                    fx.deformed_keys.push(((entity, p), world_matrix));
                } else if info.prim_mesh.get(p).copied() == Some(out.mesh) {
                    let n = inst.morph_weights.len().min(out.weights.len());
                    inst.morph_weights[..n].copy_from_slice(&out.weights[..n]);
                }
                scene.objects.skinned_instances.push(inst);
            }
            None => {
                scene.objects.instances.push(og_render::Instance {
                    mesh,
                    material,
                    transform: world_matrix * local,
                    material_layer: None,
                    fx: look.fx,
                    cast_shadows: look.cast,
                    receive_shadows: look.receive,
                    sway: 0.0,
                });
                keys.push((entity, k as u32));
            }
        }
    }
    true
}

/// A baked clip of og.CrowdAnimation: its index in
/// `RenderScene::baked_clips`, its frames and the clip's seconds.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Baked {
    pub index: u32,
    pub frames: usize,
    pub seconds: f32,
}

/// One og.CrowdAnimation's fields.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CrowdPlay {
    pub skeleton: u64,
    pub skin: u32,
    pub clip: u64,
    pub offset: f32,
    pub speed: f32,
    pub looping: bool,
    pub rate: f32,
}

/// The entities with og.CrowdAnimation and its fields, read in place (a
/// crowd is thousands of rows: no JSON a row).
pub(crate) fn crowds(world: &World, rs: &RenderState) -> IdMap<u64, CrowdPlay> {
    use super::codec::{Rec, Slot};
    let Some(layout) = rs.draw.fx.crowd.as_ref() else {
        return IdMap::new();
    };
    let slot = |key| Slot::of(layout, key);
    let (skeleton, skin, clip) = (slot("skeleton"), slot("skin"), slot("clip"));
    let (offset, speed, looping, rate) =
        (slot("offset"), slot("speed"), slot("loop"), slot("rate"));
    super::extract::column_rows(world, layout)
        .iter()
        .map(|(e, v)| {
            let num = |s: &Slot, d: f64| v.num(s).unwrap_or(d);
            (
                e,
                CrowdPlay {
                    skeleton: num(&skeleton, 0.0) as u64,
                    skin: num(&skin, 0.0) as u32,
                    clip: num(&clip, 0.0) as u64,
                    offset: num(&offset, 0.0) as f32,
                    speed: num(&speed, 1.0) as f32,
                    looping: v.flag(&looping).unwrap_or(true),
                    rate: num(&rate, 30.0) as f32,
                },
            )
        })
        .collect()
}

/// The baked clip of an og.CrowdAnimation (`crowd`): baked from og-anim's
/// skeleton and clip the first time it is drawn, at `rate` frames a
/// second evenly over the clip, and shared from then on; `None` while the
/// skeleton is not loaded, or its skin has no joints.
fn baked_clip(
    fx: &mut FxState,
    scene: &mut og_render::RenderScene,
    crowd: &CrowdPlay,
) -> Option<Baked> {
    let (skeleton, clip, skin) = (crowd.skeleton, crowd.clip, crowd.skin);
    let rate = if crowd.rate.is_finite() {
        crowd.rate.clamp(1.0, 120.0)
    } else {
        30.0
    };
    let key = (skeleton, skin, clip, rate.to_bits());
    if let Some(&b) = fx.baked.get(&key) {
        return Some(b);
    }
    let anim = fx.anim.as_ref()?;
    let skel = anim.skeleton(skeleton)?;
    let clip = anim.clip(clip);
    let seconds = clip.as_ref().map_or(0.0, |c| c.duration.max(0.0));
    let count = (((seconds * rate).ceil() as usize) + 1).clamp(2, 4096);
    let binding = clip.as_ref().map(|c| og_anim::bind(&skel, c));
    let mut frames = Vec::new();
    let mut palette = Vec::new();
    for k in 0..count {
        let mut pose = og_anim::Pose::rest(&skel);
        if let (Some(c), Some(b)) = (&clip, &binding) {
            let t = seconds * k as f32 / (count - 1) as f32;
            og_anim::sample(&skel, c, b, t, og_anim::NONE, &mut pose);
        }
        skel.palette(skin, &pose.nodes, &mut palette);
        frames.extend_from_slice(&palette);
    }
    if palette.is_empty() {
        return None;
    }
    let index = scene.objects.baked_clips.len() as u32;
    scene.objects.baked_clips.push(og_render::BakedClip {
        joints: palette.len() as u32,
        frames,
    });
    let b = Baked {
        index,
        frames: count,
        seconds,
    };
    fx.baked.insert(key, b);
    Some(b)
}

/// Draws a model played by og.CrowdAnimation (`crowd`) on the GPU: its
/// skinned primitives as skinned instances posed by the shared baked clip
/// at the frame of `seconds` (the world's time, between ticks on a frame
/// drawn between them), with the frame before's clip, frame and transform
/// for TAA; its other parts as instances, as [`posed_model`] draws them.
/// `false` when it cannot (no skin data or skeleton yet): the caller draws
/// the model at its bind pose.
#[allow(clippy::too_many_arguments)]
pub(crate) fn crowd_model(
    rs: &mut RenderState,
    entity: u64,
    asset: u64,
    world_matrix: Mat4,
    seconds: f64,
    crowd: &CrowdPlay,
    parts: &[(MeshHandle, og_render::MaterialHandle, Mat4)],
    look: ModelLook,
    keys: &mut Vec<(u64, u32)>,
) -> bool {
    let RenderState { scene, fx, .. } = rs;
    let Some(info) = fx.models.get(&asset).cloned() else {
        return false;
    };
    let Some(Baked {
        index: clip,
        frames: count,
        seconds: length,
    }) = baked_clip(fx, scene, crowd)
    else {
        return false;
    };
    let Some(skin) = fx.skins.get_mut(&info.id) else {
        return false;
    };
    let meshes = skinned_meshes(scene, &info, skin, info.prim_mesh.len());
    // The clip's time, then its frame (in f64: a world hours old keeps
    // its sub-frame precision).
    let t = f64::from(crowd.offset) + f64::from(crowd.speed) * seconds;
    let length = f64::from(length);
    let t = if length <= 0.0 {
        0.0
    } else if crowd.looping {
        t.rem_euclid(length)
    } else {
        t.clamp(0.0, length)
    };
    let frame = if length > 0.0 {
        (t / length * (count - 1) as f64) as f32
    } else {
        0.0
    };
    let (previous_clip, previous_frame, previous_transform) = fx
        .drawn_baked
        .get(&entity)
        .copied()
        .unwrap_or((clip, frame, world_matrix));
    fx.baked_now.insert(entity, (clip, frame, world_matrix));
    let pose = og_render::BakedPose {
        clip,
        frame,
        previous_clip,
        previous_frame,
        previous_transform,
    };
    // Its skinned parts draw as a member of the frame's crowd of the same
    // parts (og-render's crowd.rs: culled, compacted and drawn on the GPU);
    // its rigid parts as instances.
    let skinned: Vec<og_render::CrowdPart> = parts
        .iter()
        .filter_map(|&(mesh, material, _)| {
            let p = info.primitive(mesh);
            meshes
                .get(p)
                .copied()
                .flatten()
                .map(|m| og_render::CrowdPart {
                    levels: vec![m],
                    material,
                })
        })
        .collect();
    if !skinned.is_empty() {
        let mut member = og_render::CrowdMember::new(world_matrix, clip, frame);
        member.pose = pose;
        member.fx = look.fx;
        member.receive_shadows = look.receive;
        let at = scene
            .objects
            .crowds
            .iter()
            .position(|c| c.cast_shadows == look.cast && c.parts == skinned);
        let row = match at {
            Some(i) => &mut scene.objects.crowds[i],
            None => {
                scene.objects.crowds.push(og_render::Crowd {
                    parts: skinned,
                    cast_shadows: look.cast,
                    ..Default::default()
                });
                scene.objects.crowds.last_mut().expect("just pushed")
            }
        };
        row.members.push(member);
    }
    for (k, &(mesh, material, local)) in parts.iter().enumerate() {
        let p = info.primitive(mesh);
        match meshes.get(p).copied().flatten() {
            Some(_) => {}
            None => {
                scene.objects.instances.push(og_render::Instance {
                    mesh,
                    material,
                    transform: world_matrix * local,
                    material_layer: None,
                    fx: look.fx,
                    cast_shadows: look.cast,
                    receive_shadows: look.receive,
                    sway: 0.0,
                });
                keys.push((entity, k as u32));
            }
        }
    }
    true
}

/// Per-object motion's state (`FxState`'s last two ticks and what the last
/// display frame drew), kept by a presentation checkpoint: the first frame
/// after a restore moves its objects from where the saved run's frame
/// before drew them, so TAA and motion blur reproject as that run did.
#[derive(Clone, Default)]
pub(crate) struct MotionCheckpoint {
    tick: u64,
    current: Vec<((u64, u32), Mat4)>,
    previous: Vec<((u64, u32), Mat4)>,
    drawn: Vec<((u64, u32), Mat4)>,
    drawn_joints: IdMap<u64, Vec<Mat4>>,
    drawn_weights: BTreeMap<(u64, usize), [f32; og_render::MAX_MORPH_TARGETS]>,
    drawn_deformed: BTreeMap<(u64, usize), (Mat4, og_render::DeformedVertices)>,
}

impl FxState {
    pub(crate) fn motion_checkpoint(&self) -> MotionCheckpoint {
        MotionCheckpoint {
            tick: self.tick,
            current: self.current.clone(),
            previous: self.previous.clone(),
            drawn: self.drawn.clone(),
            drawn_joints: self.drawn_joints.clone(),
            drawn_weights: self.drawn_weights.clone(),
            drawn_deformed: self.drawn_deformed.clone(),
        }
    }

    pub(crate) fn restore_motion(&mut self, c: &MotionCheckpoint) {
        self.tick = c.tick;
        self.current.clone_from(&c.current);
        self.previous.clone_from(&c.previous);
        self.previous_by_key = None;
        self.drawn.clone_from(&c.drawn);
        self.drawn_joints.clone_from(&c.drawn_joints);
        self.drawn_weights.clone_from(&c.drawn_weights);
        self.drawn_deformed.clone_from(&c.drawn_deformed);
    }
}

/// Whether the renderer resolved TAA, whose motion vectors want the
/// previous frame's transforms and joints whatever the post effects.
pub(crate) fn taa(rs: &RenderState) -> bool {
    rs.backend.as_ref().is_some_and(|b| b.antialias() == "taa")
}

/// Per-object motion: this tick's instance transforms, keyed, and last
/// tick's for `previous_transforms` (same order as the instances); a
/// display frame drawn between ticks (`super::interpolate`) moves from the
/// display frame before it instead, as drawn, since the shutter is a
/// fraction of the frame. An entity in `cut` (teleported into the frame:
/// `Hierarchy::cut`) moves from where it is drawn, so it does not smear.
/// Under TAA without motion blur every frame moves from the frame drawn
/// before it (TAA's history is that frame), instances and posed models'
/// joints alike (`previous_joint_palettes`). Only what is drawn is read:
/// the simulation never sees any of it.
pub(crate) fn motion(rs: &mut RenderState, keys: &[(u64, u32)], tick: u64, cut: &IdMap<u64, ()>) {
    let taa = taa(rs);
    let RenderState {
        scene,
        fx,
        interp,
        atmo,
        ..
    } = rs;
    if fx.tick != tick {
        std::mem::swap(&mut fx.previous, &mut fx.current);
        fx.previous_by_key = None;
        fx.tick = tick;
    }
    fx.current.clear();
    fx.current.extend(
        keys.iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| (*k, i.transform)),
    );
    scene.objects.previous_transforms.clear();
    scene.objects.previous_joint_palettes.clear();
    scene.objects.previous_morph_weights.clear();
    scene.objects.previous_deformed_vertices.clear();
    let blur = scene.look.post.motion_blur.is_some();
    let keyed = keys.len() == scene.objects.instances.len();
    if blur && keyed {
        let previous = fx
            .previous_by_key
            .get_or_insert_with(|| fx.previous.iter().copied().collect());
        scene.objects.previous_transforms = keys
            .iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| previous.get(k).copied().unwrap_or(i.transform))
            .collect();
    }
    // Kept only while motion blur or TAA is on (a copy of every instance's
    // transform a frame otherwise).
    if !blur && !taa {
        fx.drawn.clear();
        fx.drawn_joints.clear();
        fx.drawn_weights.clear();
        fx.drawn_deformed.clear();
        return;
    }
    // A capture (every extraction but a display frame's) draws the tick as
    // a still frame: its warm-up redraws one scene, so its instances and
    // joints move with nothing but the blur's tick before, and the frames
    // the display drew (or did not draw: `og run --capture-only`) neither
    // reach it nor are moved by it (docs/design.md §2.4).
    if !interp.display {
        return;
    }
    joints(scene, atmo, fx);
    let from_drawn = if blur {
        interp.drawn.is_some() && !scene.objects.previous_transforms.is_empty()
    } else {
        keyed
    };
    if from_drawn {
        let last: BTreeMap<(u64, u32), Mat4> = fx.drawn.iter().copied().collect();
        scene.objects.previous_transforms = keys
            .iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| last.get(k).copied().unwrap_or(i.transform))
            .collect();
    }
    if !cut.is_empty() {
        for ((k, i), p) in keys
            .iter()
            .zip(&scene.objects.instances)
            .zip(&mut scene.objects.previous_transforms)
        {
            if cut.contains_key(&k.0) {
                *p = i.transform;
            }
        }
    }
    fx.drawn.clear();
    fx.drawn.extend(
        keys.iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| (*k, i.transform)),
    );
}

/// TAA's skinned motion: each joint palette's joints as the frame before
/// drew them, in the world (`previous_joint_palettes`, in palette order;
/// a model new to the frame, or whose joint count changed, as it is drawn
/// now), and this frame's kept for the next.
fn joints(
    scene: &mut og_render::RenderScene,
    atmo: &super::atmosphere::AtmoState,
    fx: &mut FxState,
) {
    let mut entity_of = vec![None; scene.objects.joint_palettes.len()];
    for (&e, &p) in &atmo.palettes {
        if let Some(slot) = entity_of.get_mut(p as usize) {
            *slot = Some(e);
        }
    }
    let mut at = vec![None; scene.objects.joint_palettes.len()];
    for s in &scene.objects.skinned_instances {
        if let Some(t) = at.get_mut(s.palette as usize) {
            t.get_or_insert(s.transform);
        }
    }
    let mut now = IdMap::new();
    scene.objects.previous_joint_palettes = scene
        .objects
        .joint_palettes
        .iter()
        .zip(entity_of.iter().zip(&at))
        .map(|(p, (e, t))| {
            let t = t.unwrap_or(Mat4::IDENTITY);
            let world: Vec<Mat4> = p.matrices.iter().map(|m| t * *m).collect();
            let before = e
                .and_then(|e| fx.drawn_joints.get(&e))
                .filter(|b| b.len() == world.len())
                .cloned()
                .unwrap_or_else(|| world.clone());
            if let Some(e) = e {
                now.insert(*e, world);
            }
            og_render::JointPalette::from(before)
        })
        .collect();
    fx.drawn_joints = now;
    // Each skinned instance's weights the frame before (its own where it
    // was not drawn then: no morph motion), and this frame's kept.
    let mut weights = BTreeMap::new();
    scene.objects.previous_morph_weights = scene
        .objects
        .skinned_instances
        .iter()
        .map(|s| {
            let Some(e) = entity_of.get(s.palette as usize).copied().flatten() else {
                return s.morph_weights;
            };
            let key = (e, s.mesh.index());
            weights.insert(key, s.morph_weights);
            fx.drawn_weights
                .get(&key)
                .copied()
                .unwrap_or(s.morph_weights)
        })
        .collect();
    fx.drawn_weights = weights;
    // Retain the draw inputs, not CPU-skinned vertices. The fallback evaluates
    // the saved pose in this frame's instance space; compute keeps GPU history.
    let mut drawn = BTreeMap::new();
    scene.objects.previous_deformed_vertices = scene
        .objects
        .deformed_vertices
        .iter()
        .zip(&fx.deformed_keys)
        .map(|(d, &(key, world))| {
            let before = fx
                .drawn_deformed
                .get(&key)
                .filter(|(_, b)| b.vertex_count() == d.vertex_count())
                .map(|(old_world, b)| {
                    let transform = world.inverse() * *old_world;
                    let mut before = b.clone();
                    if let Some(g) = &mut before.garment {
                        g.output_transform = transform;
                    } else {
                        before
                            .positions
                            .iter_mut()
                            .for_each(|p| *p = transform.transform_point3(*p));
                    }
                    before
                })
                .unwrap_or_else(|| d.clone());
            drawn.insert(key, (world, d.clone()));
            before
        })
        .collect();
    fx.drawn_deformed = drawn;
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

/// Reads a cooked `.cube` table and adds it to the renderer.
fn add_lut(eng: &super::engine::Engine, id: &str) -> Result<og_render::LutHandle, String> {
    let lut = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        bundle.lut(id, bytes.bytes()).map_err(|e| e.to_string())?
    };
    Ok(eng.render.borrow_mut().scene.add_lut(lut))
}

/// `og.load.lut`: a cooked `.cube` table added to the renderer once. One
/// in a streaming group's pack loads once its group is resident (refused
/// before, naming the group); when the group unloads the renderer lets it
/// go (`super::stream`) and PostEffects.lut grades nothing with it until
/// the group is resident again, when it comes back under the same handle.
pub(crate) fn load_lut(eng: &super::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let key = format!("lut:{id}");
    if let Some(&i) = eng.render.borrow().fx.by_path.get(&key) {
        return Ok(i);
    }
    let h = add_lut(eng, &id)?;
    let mut rs = eng.render.borrow_mut();
    rs.fx.luts.push(LoadedLut {
        id,
        handle: Some(h),
    });
    let i = rs.fx.luts.len() - 1;
    rs.fx.by_path.insert(key, i);
    Ok(i)
}

/// At a tick boundary: adds again the LUTs whose streaming group is
/// resident again after an unload released them.
pub(crate) fn resolve_luts(eng: &super::engine::Engine) {
    let pending: Vec<(usize, String)> = eng
        .render
        .borrow()
        .fx
        .luts
        .iter()
        .enumerate()
        .filter(|(_, l)| l.handle.is_none())
        .map(|(i, l)| (i, l.id.clone()))
        .collect();
    for (i, id) in pending {
        if let Ok(h) = add_lut(eng, &id) {
            eng.render.borrow_mut().fx.luts[i].handle = Some(h);
        }
    }
}

/// `og.load.emitter`: a cooked emitter, its textures (or its texture
/// array), its mesh (`render mesh`) and its sub-emitters loaded with it.
pub(crate) fn load_emitter(eng: &super::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let key = format!("emitter:{id}");
    if let Some(&i) = eng.render.borrow().fx.by_path.get(&key) {
        return Ok(i);
    }
    if eng.render.borrow().fx.loading.contains(&id) {
        return Err(format!("{id}: a sub-emitter cycle ({id} fires itself)"));
    }
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .emitter_text(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    eng.render.borrow_mut().fx.loading.push(id.clone());
    let parsed = parse_linked(eng, &text);
    eng.render.borrow_mut().fx.loading.pop();
    let (emitter, array) = parsed.map_err(|e| format!("{id}: {e}"))?;
    let mut rs = eng.render.borrow_mut();
    let host = super::particle_events::shared(eng);
    rs.fx.host = Some(host.clone());
    // An emitter that sends events: the tick simulates its systems.
    let definition = (!emitter.events.is_empty()).then(|| {
        let subs = emitter
            .sub_emitters
            .iter()
            .map(|s| rs.scene.emitter(s.emitter).cloned())
            .collect();
        let mesh = match emitter.shape {
            og_render::EmitterShape::Mesh { mesh } => rs.scene.mesh(mesh),
            _ => None,
        };
        let points = og_render::shape_points(&emitter.shape, mesh);
        (emitter.clone(), subs, points)
    });
    let h = rs.scene.add_emitter(emitter);
    rs.fx.emitters.push(h);
    let i = rs.fx.emitters.len() - 1;
    if let Some((emitter, subs, points)) = definition {
        let d = super::particle_events::Definition {
            handle: h,
            emitter,
            subs,
            points,
        };
        super::particle_events::lock(&host)
            .emitters
            .insert(i, Arc::new(d));
    }
    if let Some(a) = array {
        rs.fx.emitter_arrays.insert(i, a);
    }
    rs.fx.by_path.insert(key, i);
    Ok(i)
}

/// An emitter's text with every reference loaded: the emitter and the load
/// index of the texture array it draws from.
fn parse_linked(
    eng: &super::engine::Engine,
    text: &str,
) -> Result<(og_render::ParticleEmitter, Option<usize>), String> {
    let mut array = None;
    let emitter = og_bundle::fx::parse_emitter_linked(
        text,
        &mut og_bundle::fx::EmitterLinks {
            texture: &mut |tex| {
                let i = super::assets::load_texture(eng, tex)?;
                Ok(eng.render.borrow().textures.get(i).copied())
            },
            array: &mut |a| {
                let i = super::assets::load_texture_array_index(eng, a)?;
                array = Some(i);
                Ok(eng
                    .render
                    .borrow()
                    .texture_arrays
                    .get(i)
                    .and_then(|a| a.handle))
            },
            mesh: &mut |m| particle_mesh(eng, m).map(Some),
            emitter: &mut |e| {
                let i = load_emitter(eng, e)?;
                Ok(eng.render.borrow().fx.emitters.get(i).copied())
            },
            effect: &mut |id| super::effect::load_effect(eng, id).map(Some),
        },
    )?;
    Ok((emitter, array))
}

/// A mesh particle's mesh: a built-in shape (made once) or a model's first
/// mesh, the model loaded with the emitter.
pub(crate) fn particle_mesh(
    eng: &super::engine::Engine,
    mesh: og_bundle::fx::ParticleMesh<'_>,
) -> Result<MeshHandle, String> {
    use og_bundle::fx::ParticleMesh;
    match mesh {
        ParticleMesh::Builtin(name) => {
            let name = og_bundle::fx::PARTICLE_MESHES
                .iter()
                .copied()
                .find(|n| *n == name)
                .ok_or_else(|| format!("`{name}` is not a built-in particle mesh"))?;
            let mut rs = eng.render.borrow_mut();
            if let Some(&h) = rs.fx.particle_meshes.get(name) {
                return Ok(h);
            }
            let data = match name {
                "box" => og_render::shapes::cuboid(glam::Vec3::ONE),
                "sphere" => og_render::shapes::sphere(12, 8),
                _ => og_render::shapes::tetrahedron(),
            };
            let h = rs.scene.add_mesh(data);
            rs.fx.particle_meshes.insert(name.to_string(), h);
            Ok(h)
        }
        ParticleMesh::Asset(model) => {
            let i = super::assets::load_gltf(eng, model)?;
            let geometry = match eng.assets.borrow().entries.get(i).map(|e| &e.payload) {
                Some(super::assets::Payload::Gltf { geometries, .. }) => {
                    geometries.first().copied()
                }
                _ => None,
            }
            .ok_or_else(|| format!("{model}: the model has no mesh"))?;
            let rs = eng.render.borrow();
            super::assets::index_of(geometry as f64)
                .and_then(|g| rs.geometries.get(g).copied())
                .ok_or_else(|| format!("{model}: the model's mesh is not loaded"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `kept_rows` answers what `rows` decodes, row for row, after each
    /// change it can see.
    fn kept_rows_decode(world: &World, layout: &Layout, kept: &[(u64, Arc<Value>)]) {
        let fresh = rows(world, Some(layout));
        assert_eq!(kept.len(), fresh.len());
        for ((ke, kv), (fe, fv)) in kept.iter().zip(&fresh) {
            assert_eq!((ke, &**kv), (fe, fv));
        }
    }

    /// A row's JSON is kept while its bytes stay those of the row at its
    /// place in the last read, decoded again when they change, and the kept
    /// rows are only the last read's: a row gone, or a new one, never leaves
    /// a row with another's JSON (engine s110).
    #[test]
    fn kept_rows_are_what_decoding_every_row_gives() {
        let decl = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == "og.WindMotor")
            .expect("og.WindMotor");
        let layout = Layout::from_decl(decl).unwrap();
        let mut world = World::new(&og_ecs::WorldDesc::default()).unwrap();
        let id = world.register_schema(layout.schema()).unwrap();
        let base = layout.encode(None, None).unwrap();
        let radius = |r: f64| {
            layout
                .encode(Some(&base), Some(&serde_json::json!({ "radius": r })))
                .unwrap()
        };
        let a = world.spawn_bytes(vec![(id, radius(1.0))]).unwrap();
        let b = world.spawn_bytes(vec![(id, radius(2.0))]).unwrap();
        world.spawn_bytes(vec![(id, radius(3.0))]).unwrap();
        let first = kept_rows(&world, Some(&layout));
        kept_rows_decode(&world, &layout, &first);
        assert_eq!(first.len(), 3);
        // Unchanged: every row's JSON is the one kept, by the layout's clones too.
        let again = kept_rows(&world, Some(&layout.clone()));
        kept_rows_decode(&world, &layout, &again);
        for (k, (f, g)) in first.iter().zip(&again).enumerate() {
            assert!(Arc::ptr_eq(&f.1, &g.1), "row {k} decoded again");
        }
        // One row changes: it alone is decoded again.
        world.insert_bytes(b, id, &radius(5.0)).unwrap();
        let changed = kept_rows(&world, Some(&layout));
        kept_rows_decode(&world, &layout, &changed);
        assert_eq!(changed[1].1["radius"], serde_json::json!(5.0));
        assert!(!Arc::ptr_eq(&again[1].1, &changed[1].1));
        assert!(Arc::ptr_eq(&again[0].1, &changed[0].1));
        assert!(Arc::ptr_eq(&again[2].1, &changed[2].1));
        // A row gone and a new one: the rows after them move.
        world.despawn(a).unwrap();
        let gone = kept_rows(&world, Some(&layout));
        kept_rows_decode(&world, &layout, &gone);
        world.spawn_bytes(vec![(id, radius(2.0))]).unwrap();
        let new = kept_rows(&world, Some(&layout));
        kept_rows_decode(&world, &layout, &new);
        assert_eq!(new.len(), 3);
        let kept = layout.kept_rows.lock().unwrap().len();
        assert_eq!(kept, 3, "the kept rows are the last read's");
        let again = kept_rows(&world, Some(&layout));
        assert!(Arc::ptr_eq(&new[1].1, &again[1].1));
        for (e, _) in again {
            world.despawn(Entity::from_bits(e)).unwrap();
        }
        assert!(kept_rows(&world, Some(&layout)).is_empty());
        assert!(layout.kept_rows.lock().unwrap().is_empty());
    }

    /// og.MaterialStyle as its row reads: `v` written over the IDL's
    /// defaults, then decoded.
    fn material_style(v: Value) -> record::MaterialStyle {
        let decl = og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == "og.MaterialStyle")
            .expect("og.MaterialStyle");
        let layout = Layout::from_decl(decl).unwrap();
        layout
            .read(&layout.encode(None, Some(&v)).unwrap())
            .unwrap()
    }

    /// A look's MaterialStyle.shadowDarkness reaches the renderer's
    /// material style; unset it is 0 (off), so no look moves without it.
    #[test]
    fn material_style_carries_the_shadow_darkness() {
        let m = material_style(serde_json::json!({ "toon": true, "shadowDarkness": 0.75 }));
        let s = stylise(None, Some(&m), &[], &[]);
        assert_eq!(s.material.shadow_darkness, 0.75);
        assert!(s.material.toon);
        let off = material_style(serde_json::json!({ "toon": true }));
        let off = stylise(None, Some(&off), &[], &[]);
        assert_eq!(off.material.shadow_darkness, 0.0);
    }

    #[test]
    fn the_material_style_s_surface_reaches_the_renderer() {
        let m = |v: Value| stylise(None, Some(&material_style(v)), &[], &[]).material;
        // Written as the variant's index and as its name (a look's).
        let clay = m(serde_json::json!({ "surface": 1, "surfaceStrength": 0.5 }));
        assert_eq!(clay.surface, og_render::MaterialSurface::Clay);
        assert_eq!(clay.surface_strength, 0.5);
        assert!(clay.is_active());
        let paper = m(serde_json::json!({ "surface": "paper", "surfaceScale": 0.3 }));
        assert_eq!(paper.surface, og_render::MaterialSurface::Paper);
        assert_eq!((paper.surface_strength, paper.surface_scale), (1.0, 0.3));
        assert_eq!(
            m(serde_json::json!({ "surface": 3 })).surface,
            og_render::MaterialSurface::Plastic
        );
        // None is no change.
        assert!(!m(serde_json::json!({})).is_active());
    }
}
