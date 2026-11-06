//! The effects a game declares as data, extracted into og-render's scene
//! (the IDL's `PostEffects`, `Lighting2D`, `Light2D`, `Occluder2D`, `Decal`,
//! `ParticleSystem`, `Trail`, `Beam`, `og.render.ribbon`, the sprite effect
//! and screen fields, and skinned models posed by og-anim), and the loads of
//! their assets (`og.load.lut`, `og.load.emitter`). Like the rest of
//! extraction it reads the world and never writes it.

use super::codec::Layout;
use super::extract::{Hierarchy, RenderState};
use glam::{Mat4, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::{Entity, QueryDesc, World};
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
    /// Built-in mesh particles' shapes (`render mesh box`), made once.
    pub particle_meshes: BTreeMap<&'static str, MeshHandle>,
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

pub(crate) fn vec2(v: &Value, default: Vec2) -> Vec2 {
    Vec2::new(f(v, "x", default.x), f(v, "y", default.y))
}

pub(crate) fn vec3(v: &Value) -> Vec3 {
    Vec3::new(f(v, "x", 0.0), f(v, "y", 0.0), f(v, "z", 0.0))
}

/// The load index of a handle field (`0` is none).
pub(crate) fn handle_index(v: &Value) -> Option<usize> {
    super::assets::index_of(v.as_f64()?)
}

/// A u32 bit mask field.
fn mask(v: &Value, key: &str, default: u32) -> u32 {
    v[key].as_u64().map_or(default, |x| x as u32)
}

/// Each og.LightMask2D's layers by entity (empty until a game names it:
/// og.LightMask2D is lazy). A receiver without one is lit as mask 1.
pub(crate) fn light_masks(world: &World, layouts: &FxLayouts) -> IdMap<u64, u32> {
    rows(world, layouts.light_mask_2d.as_ref())
        .into_iter()
        .map(|(e, v)| (e, mask(&v, "layers", 1)))
        .collect()
}

/// Whether the quality lets global illumination draw (screen-space GI and
/// realtime light volumes): og.GlobalIllumination's `allowed`, where the
/// presets write it, and the switch of a preset applied before the world
/// had the resource (`FxState::gi_preset`).
pub(crate) fn gi_allowed(gi: Option<&Value>, preset: Option<bool>) -> bool {
    gi.is_none_or(|g| on(g, "allowed", true)) && preset.unwrap_or(true)
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
    let mut entities: Vec<Entity> = all.flat_map(|a| a.entities().to_vec()).collect();
    og_core::sort::sort_by_key(&mut entities, |e| e.index());
    entities
        .into_iter()
        .filter_map(|e| Some((e.to_bits(), layout.decode(&world.get_bytes(e, id).ok()?))))
        .collect()
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
    scene.decal_atlas = resource(world, render_settings.as_ref())
        .and_then(|s| handle_index(&s["decalAtlas"]))
        .and_then(|i| textures.get(i).copied());
    let post = &mut scene.post;
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
    scene.shake = CameraShake::default();
    scene.lighting_2d = None;
    if let Some(p) = resource(world, layouts.post.as_ref()) {
        if on(&p, "ao", false) {
            post.ao = Some(AmbientOcclusion {
                radius: f(&p, "aoRadius", 0.6),
                intensity: f(&p, "aoIntensity", 1.0),
                method: [
                    AoMethod::Auto,
                    AoMethod::Gtao,
                    AoMethod::Ssao,
                    AoMethod::Screen,
                ][variant(&p["aoMethod"], &["auto", "gtao", "ssao", "screen"]).min(3)],
                resolution: [AoResolution::Full, AoResolution::Half]
                    [variant(&p["aoResolution"], &["full", "half"]).min(1)],
                direct: f(&p, "aoDirect", 0.0).clamp(0.0, 1.0),
                radius_scale: f(&p, "aoRadiusScale", 0.0).max(0.0),
            });
        }
        if on(&p, "outline", false) {
            let c = color(&p["outlineColor"], Vec3::new(0.02, 0.02, 0.03));
            post.outline = Some(Outline {
                color: [c.x, c.y, c.z, 1.0],
                width: f(&p, "outlineWidth", 2.0),
                depth_threshold: f(&p, "outlineDepthThreshold", 0.04),
                normal_threshold: f(&p, "outlineNormalThreshold", 0.4),
            });
        }
        if on(&p, "depthOfField", false) {
            post.depth_of_field = Some(DepthOfField {
                focus_distance: f(&p, "dofFocusDistance", 10.0),
                focus_range: f(&p, "dofFocusRange", 2.0),
                max_blur: f(&p, "dofMaxBlur", 0.012),
                falloff: f(&p, "dofFalloff", 4.0),
                quality: if variant(&p["dofQuality"], &["low", "high"]) == 1 {
                    DofQuality::High
                } else {
                    DofQuality::Low
                },
            });
        }
        if on(&p, "motionBlur", false) {
            post.motion_blur = Some(MotionBlur {
                shutter: f(&p, "motionBlurShutter", 0.5),
                samples: u(&p, "motionBlurSamples", 12),
                max_pixels: f(&p, "motionBlurMaxPixels", 40.0),
            });
        }
        let blur_kind = variant(&p["screenBlur"], &["none", "zoom", "spin", "directional"]);
        let blur_strength = f(&p, "screenBlurStrength", 0.0);
        if blur_kind > 0 && blur_strength != 0.0 {
            post.screen_blur = Some(ScreenBlur {
                kind: [
                    ScreenBlurKind::Zoom,
                    ScreenBlurKind::Spin,
                    ScreenBlurKind::Directional,
                ][(blur_kind - 1).min(2)],
                strength: blur_strength,
                center: vec2(&p["screenBlurCenter"], Vec2::splat(0.5)),
                angle: f(&p, "screenBlurAngle", 0.0).to_radians(),
                samples: u(&p, "screenBlurSamples", 16),
            });
        }
        if on(&p, "godRays", false) {
            post.god_rays = Some(GodRays {
                intensity: f(&p, "godRaysIntensity", 0.5),
                decay: f(&p, "godRaysDecay", 0.965),
                density: f(&p, "godRaysDensity", 1.0),
                tint: color(&p["godRaysTint"], Vec3::ONE),
                samples: u(&p, "godRaysSamples", 48),
                threshold: f(&p, "godRaysThreshold", 2.0),
            });
        }
        if on(&p, "ssr", false) {
            post.ssr = Some(og_render::ScreenSpaceReflections {
                max_distance: f(&p, "ssrMaxDistance", 100.0),
                thickness: f(&p, "ssrThickness", 0.3),
                steps: u(&p, "ssrSteps", 64),
                fade: f(&p, "ssrFade", 0.15),
                intensity: f(&p, "ssrIntensity", 1.0),
                max_roughness: f(&p, "ssrMaxRoughness", 0.6),
                resolution: if p["ssrResolution"].is_null() {
                    AoResolution::Half
                } else {
                    [AoResolution::Full, AoResolution::Half]
                        [variant(&p["ssrResolution"], &["full", "half"]).min(1)]
                },
            });
        }
        if on(&p, "glow", false) {
            post.glow = Some(Glow {
                intensity: f(&p, "glowIntensity", 1.0),
            });
        }
        if let Some(lut) = handle_index(&p["lut"])
            .and_then(|i| fx.luts.get(i))
            .and_then(|l| l.handle)
        {
            post.color_grading = Some(ColorGrading {
                lut,
                intensity: f(&p, "lutIntensity", 1.0),
            });
        }
        let adjust = ColorAdjust {
            temperature: f(&p, "temperature", 0.0),
            tint: f(&p, "tint", 0.0),
            brightness: f(&p, "brightness", 0.0),
            contrast: f(&p, "contrast", 0.0),
            shadows: vec3(&p["balanceShadows"]),
            midtones: vec3(&p["balanceMidtones"]),
            highlights: vec3(&p["balanceHighlights"]),
            preserve_luminosity: on(&p, "preserveLuminosity", true),
            hue: f(&p, "hue", 0.0),
            saturation: f(&p, "saturation", 0.0),
            vibrance: f(&p, "vibrance", 0.0),
            grayscale: f(&p, "grayscale", 0.0),
            invert: f(&p, "invert", 0.0),
            threshold: f(&p, "threshold", 0.0),
        };
        post.color_adjust = (!adjust.is_identity()).then_some(adjust);
        if on(&p, "vignette", false) {
            post.vignette = Some(Vignette {
                intensity: f(&p, "vignetteIntensity", 0.35),
                radius: f(&p, "vignetteRadius", 0.55),
                smoothness: f(&p, "vignetteSmoothness", 0.45),
                color: color(&p["vignetteColor"], Vec3::ZERO),
            });
        }
        let aberration = f(&p, "chromaticAberration", 0.0);
        if aberration > 0.0 {
            post.chromatic_aberration = Some(ChromaticAberration {
                intensity: aberration,
            });
        }
        let lens = f(&p, "lensDistortion", 0.0);
        if lens != 0.0 {
            post.lens_distortion = Some(LensDistortion {
                amount: lens,
                center: vec2(&p["lensDistortionCenter"], Vec2::splat(0.5)),
            });
        }
        let grain = f(&p, "filmGrain", 0.0);
        if grain > 0.0 {
            post.film_grain = Some(FilmGrain {
                intensity: grain,
                size: f(&p, "filmGrainSize", 1.5),
            });
        }
        let sharpen = f(&p, "sharpen", -1.0);
        if sharpen >= 0.0 {
            post.sharpen = Some(sharpen);
        }
        let flash = f(&p, "flash", 0.0);
        if flash > 0.0 {
            post.flash = Some(ScreenFlash {
                color: color(&p["flashColor"], Vec3::ONE),
                amount: flash,
            });
        }
        let trauma = f(&p, "shakeTrauma", 0.0);
        if trauma > 0.0 {
            scene.shake = CameraShake::trauma(
                trauma,
                scene.tick,
                scene.tick_rate,
                u64::from(u(&p, "shakeSeed", 0)),
                f(&p, "shakeMaxOffset", 0.05),
                f(&p, "shakeMaxRoll", 3.0),
                f(&p, "shakeFrequency", 12.0),
            );
        }
    }
    scene.stylise = stylise(
        resource(world, layouts.stylise.as_ref()),
        resource(world, layouts.material_style.as_ref()),
        &fx.look.palettes,
        textures,
    );
    // Painted 2D (super::art::texel_paint): the cook painted and washed
    // each texture in its own texels, and the frame draws neither pass.
    if art.texel_paint {
        scene.stylise.paint = None;
    }
    if let Some(p) = resource(world, layouts.lens_flare.as_ref())
        && on(&p, "enabled", false)
    {
        let d = LensFlare::default();
        post.lens_flare = Some(LensFlare {
            intensity: f(&p, "intensity", d.intensity),
            ghosts: u(&p, "ghosts", d.ghosts),
            ghost_spacing: f(&p, "ghostSpacing", d.ghost_spacing),
            ghost_threshold: f(&p, "ghostThreshold", d.ghost_threshold),
            halo_radius: f(&p, "haloRadius", d.halo_radius),
            halo_thickness: f(&p, "haloThickness", d.halo_thickness),
            halo_threshold: f(&p, "haloThreshold", d.halo_threshold),
            chromatic_aberration: f(&p, "chromaticAberration", d.chromatic_aberration),
            starburst: on(&p, "starburst", d.starburst),
            sun: f(&p, "sun", d.sun),
            lights: f(&p, "lights", d.lights),
        });
    }
    // The art contract's line round meshes (ArtLine, art.rs).
    super::art::mesh_line(&art.frame, &mut scene.stylise.material);
    // Under the art contract a frame that quantises without a palette of
    // its own takes the contract's (ArtPalette.colors): the colours the
    // cook put the art on are the ones the frame keeps.
    if let Some(q) = scene.stylise.quantize.as_mut()
        && q.palette.is_none()
    {
        q.palette = art.frame.palette;
    }
    if let Some(g) = resource(world, layouts.global_illumination.as_ref())
        && on(&g, "enabled", false)
        && gi_allowed(Some(&g), fx.gi_preset)
    {
        // GiMethod: screen, probes, both.
        let method = variant(&g["method"], &["screen", "probes", "both"]).min(2);
        if method >= 1 {
            post.gi_probes = Some(og_render::ProbeGlobalIllumination {
                spacing: f(&g, "probeSpacing", 1.0).max(0.05),
                cascades: u(&g, "probeCascades", 3).clamp(1, 4),
                rays: u(&g, "probeRays", 64).clamp(32, 256),
                intensity: f(&g, "intensity", 1.0).max(0.0),
            });
        }
    }
    if let Some(g) = resource(world, layouts.global_illumination.as_ref())
        && on(&g, "enabled", false)
        && gi_allowed(Some(&g), fx.gi_preset)
        && variant(&g["method"], &["screen", "probes", "both"]) != 1
    {
        post.gi = Some(og_render::ScreenSpaceGlobalIllumination {
            intensity: f(&g, "intensity", 1.0),
            max_distance: f(&g, "maxDistance", 16.0),
            thickness: f(&g, "thickness", 0.5),
            steps: u(&g, "steps", 12),
            rays: u(&g, "rays", 4),
            occlusion: f(&g, "occlusion", 1.0).clamp(0.0, 1.0),
            resolution: [AoResolution::Full, AoResolution::Half]
                [variant(&g["resolution"], &["full", "half"]).min(1)],
        });
    }
    if let Some(l) = resource(world, layouts.lighting_2d.as_ref())
        && on(&l, "enabled", false)
    {
        scene.lighting_2d = Some(Lighting2d {
            ambient: color(&l["ambient"], Vec3::splat(0.15)),
            min_layer: l["minLayer"].as_i64().unwrap_or(-1_000_000) as i32,
            max_layer: l["maxLayer"].as_i64().unwrap_or(99) as i32,
            resolution: f(&l, "resolution", 0.5),
            shadow_angles: u(&l, "shadowAngles", 1024),
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
    let rows = match scene.stylise.pixel.filter(|p| p.snap && p.height > 0) {
        Some(p) => Some(p.height),
        None if art.pixel_perfect && art.logical_height > 0 => Some(art.logical_height),
        None => None,
    };
    let og_render::Projection::Orthographic { height } = scene.camera.projection else {
        return;
    };
    let unit = match rows {
        Some(rows) => height / rows as f32,
        None if art.pixel_perfect && art.unit > 0.0 => 1.0 / art.unit,
        None => return,
    };
    let forward = (scene.camera.target - scene.camera.position).normalize_or_zero();
    if forward.z > -0.999 {
        return;
    }
    if unit <= 0.0 {
        return;
    }
    let snap = |v: f32| (v / unit).round() * unit;
    let c = &mut scene.camera;
    let d = c.target - c.position;
    c.position.x = snap(c.position.x);
    c.position.y = snap(c.position.y);
    c.target = c.position + d;
    for s in &mut scene.sprites {
        s.position.x = snap(s.position.x);
        s.position.y = snap(s.position.y);
    }
}

/// `Stylise` and `MaterialStyle` as og-render's description (every
/// primitive `None` or zero unless its field turns it on).
fn stylise(
    s: Option<Arc<Value>>,
    m: Option<Arc<Value>>,
    palettes: &[(String, Option<og_render::PaletteHandle>)],
    textures: &[og_render::TextureHandle],
) -> og_render::Stylise {
    let rgb = |v: &Value, d: Vec3| {
        let c = color(v, d);
        [c.x, c.y, c.z]
    };
    let texture = |v: &Value| handle_index(v).and_then(|i| textures.get(i).copied());
    let mut out = og_render::Stylise::default();
    if let Some(s) = s {
        let height = u(&s, "pixelHeight", 0);
        if height > 0 {
            out.pixel = Some(og_render::PixelScale {
                height,
                upscale: u(&s, "pixelUpscale", 0),
                snap: on(&s, "pixelSnap", true),
            });
        }
        let posterize = u(&s, "posterize", 0);
        // A palette whose streaming group is not resident quantises to none.
        let palette = handle_index(&s["palette"]).and_then(|i| palettes.get(i)?.1);
        let dither = match variant(
            &s["dither"],
            &[
                "none",
                "bayer2",
                "bayer4",
                "bayer8",
                "blueNoise",
                "interleavedGradient",
            ],
        ) {
            1 => og_render::DitherPattern::Bayer2,
            2 => og_render::DitherPattern::Bayer4,
            3 => og_render::DitherPattern::Bayer8,
            4 => og_render::DitherPattern::BlueNoise,
            5 => og_render::DitherPattern::InterleavedGradient,
            _ => og_render::DitherPattern::None,
        };
        let space = |v: &Value| match variant(v, &["screen", "surface", "object"]) {
            1 => og_render::StyliseSpace::Surface,
            2 => og_render::StyliseSpace::Object,
            _ => og_render::StyliseSpace::Screen,
        };
        if posterize > 0 || palette.is_some() || dither != og_render::DitherPattern::None {
            out.quantize = Some(og_render::Quantize {
                posterize,
                mode: match variant(&s["posterizeMode"], &["channel", "tone"]) {
                    1 => og_render::PosterizeMode::Tone,
                    _ => og_render::PosterizeMode::Channel,
                },
                palette,
                dither,
                dither_strength: f(&s, "ditherStrength", 1.0),
                dither_scale: u(&s, "ditherScale", 1).max(1),
                space: space(&s["ditherSpace"]),
            });
        }
        let halftone = f(&s, "halftone", 0.0);
        if halftone > 0.0 {
            out.halftone = Some(og_render::Halftone {
                strength: halftone,
                size: f(&s, "halftoneSize", 6.0),
                angle_degrees: f(&s, "halftoneAngle", 45.0),
                color: rgb(&s["halftoneColor"], Vec3::new(0.02, 0.02, 0.03)),
                space: space(&s["halftoneSpace"]),
                mode: match variant(&s["halftoneMode"], &["ink", "cmyk"]) {
                    1 => og_render::HalftoneMode::Cmyk,
                    _ => og_render::HalftoneMode::Ink,
                },
                misregistration: f(&s, "halftoneMisregistration", 0.0),
                region: match variant(&s["halftoneRegion"], &["all", "shade", "highlight"]) {
                    1 => og_render::HalftoneRegion::Shade,
                    2 => og_render::HalftoneRegion::Highlight,
                    _ => og_render::HalftoneRegion::All,
                },
            });
        }
        let hatching = f(&s, "hatching", 0.0);
        if hatching > 0.0 {
            out.hatching = Some(og_render::Hatching {
                strength: hatching,
                scale: f(&s, "hatchingScale", 6.0),
                levels: u(&s, "hatchingLevels", 4),
                angle_degrees: f(&s, "hatchingAngle", 45.0),
                color: rgb(&s["hatchingColor"], Vec3::new(0.04, 0.035, 0.03)),
                paper: rgb(&s["hatchingPaper"], Vec3::ONE),
                space: space(&s["hatchingSpace"]),
                depth: f(&s, "hatchingDepth", 0.0),
                keep_color: on(&s, "hatchingKeepColor", false),
            });
        }
        let paint = og_render::Paint {
            radius: f(&s, "paintRadius", 0.0),
            anisotropy: f(&s, "paintAnisotropy", 0.0),
            edge_darkening: f(&s, "paintEdgeDarkening", 0.0),
            bleed: f(&s, "paintBleed", 0.0),
            canvas: texture(&s["paintCanvas"]),
            canvas_strength: f(&s, "paintCanvasStrength", 0.0),
            canvas_scale: f(&s, "paintCanvasScale", 1.0),
            canvas_space: space(&s["paintCanvasSpace"]),
            canvas_world_scale: f(&s, "paintCanvasWorldScale", 128.0),
            detail: f(&s, "paintDetail", og_render::DEFAULT_PAINT_DETAIL),
            character_detail: f(
                &s,
                "paintCharacterDetail",
                og_render::DEFAULT_CHARACTER_DETAIL,
            ),
            watercolor: og_render::Watercolor {
                edge_darkening: f(&s, "watercolorEdgeDarkening", 0.0),
                granulation: f(&s, "watercolorGranulation", 0.0),
                turbulence: f(&s, "watercolorTurbulence", 0.0),
                dry_brush: f(&s, "watercolorDryBrush", 0.0),
                wobble: f(&s, "watercolorWobble", 0.0),
                paper: rgb(&s["watercolorPaper"], Vec3::ONE),
            },
        };
        if paint.paints() || paint.watercolor.active() {
            out.paint = Some(paint);
        }
        if on(&s, "ink", false) {
            out.ink = Some(og_render::Ink {
                threshold: f(&s, "inkThreshold", 0.1),
                width: f(&s, "inkWidth", 1.0),
                color: rgb(&s["inkColor"], Vec3::new(0.02, 0.02, 0.03)),
                depth: f(&s, "inkDepth", 0.0),
                depth_threshold: f(&s, "inkDepthThreshold", 0.04),
                normal_threshold: f(&s, "inkNormalThreshold", 0.4),
            });
        }
        let bands = u(&s, "depthBands", 0);
        if bands > 0 {
            out.depth_bands = Some(og_render::DepthBands {
                count: bands,
                near: f(&s, "depthBandsNear", 5.0),
                far: f(&s, "depthBandsFar", 60.0),
                near_color: rgb(&s["depthBandsNearColor"], Vec3::new(0.02, 0.02, 0.03)),
                far_color: rgb(&s["depthBandsFarColor"], Vec3::new(0.55, 0.6, 0.7)),
                flatten: f(&s, "depthBandsFlatten", 1.0),
            });
        }
        let retro = og_render::Retro3d {
            vertex_snap: f(&s, "vertexSnap", 0.0),
            affine: f(&s, "affineTexturing", 0.0),
            depth_bits: u(&s, "depthBits", 0),
        };
        if retro.vertex_snap > 0.0 || retro.affine > 0.0 || retro.depth_bits > 0 {
            out.retro = Some(retro);
        }
    }
    if let Some(m) = m {
        out.material = og_render::MaterialStyle {
            toon: on(&m, "toon", false),
            gradient_map: texture(&m["gradientMap"]),
            shadow_tint: rgb(&m["shadowTint"], Vec3::new(0.18, 0.16, 0.32)),
            shadow_tint_amount: f(&m, "shadowTintAmount", 0.0),
            shadow_darkness: f(&m, "shadowDarkness", 0.0),
            rim_color: rgb(&m["rimColor"], Vec3::ONE),
            rim_intensity: f(&m, "rimIntensity", 0.0),
            rim_power: f(&m, "rimPower", 3.0),
            outline_color: rgb(&m["outlineColor"], Vec3::new(0.02, 0.02, 0.03)),
            outline_width: f(&m, "outlineWidth", 0.0),
            outline_distance: f(&m, "outlineDistance", 0.0),
            flat_shading: on(&m, "flatShading", false),
            surface: og_render::MaterialSurface::from_index(variant(
                &m["surface"],
                &["none", "clay", "paper", "plastic"],
            )),
            surface_strength: f(&m, "surfaceStrength", 1.0),
            surface_scale: f(&m, "surfaceScale", 0.0),
            toon_ambient: f(&m, "toonAmbient", 0.0),
            toon_softness: f(&m, "toonSoftness", og_render::DEFAULT_TOON_SOFTNESS),
            outline_place: match variant(&m["outlinePlace"], &["inside", "outside", "both"]) {
                1 => og_render::OutlinePlace::Outside,
                2 => og_render::OutlinePlace::Both,
                _ => og_render::OutlinePlace::Inside,
            },
            wireframe: on(&m, "wireframe", false),
            wireframe_angle: f(&m, "wireframeAngle", og_render::DEFAULT_WIREFRAME_ANGLE),
            wireframe_color: rgb(&m["wireframeColor"], Vec3::ZERO),
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
    scene.lights_2d.clear();
    scene.occluders_2d.clear();
    scene.decals.clear();
    scene.particles.clear();
    scene.particle_layers.clear();
    scene.hosted_particles.clear();
    let hosted = fx.host.as_ref().map(|h| super::particle_events::lock(h));
    scene.trails.clear();
    scene.beams.clear();
    scene.ribbons.clear();
    let texture = |v: &Value| handle_index(v).and_then(|i| textures.get(i).copied());
    // Time groups (og.TimeGroups), once a game uses them.
    let groups = world.time_groups();
    for (e, v) in rows(world, layouts.light_2d.as_ref()) {
        let m = h.world_of(e);
        let (pos, _, _) = planar(m);
        // A spot's and a global light's axis: the entity's local +x in
        // the plane.
        let axis = m.x_axis.truncate().truncate();
        scene.lights_2d.push(Light2d {
            position: pos,
            color: color(&v["color"], Vec3::ONE),
            intensity: f(&v, "intensity", 1.0),
            radius: f(&v, "radius", 5.0),
            falloff: f(&v, "falloff", 2.0),
            source_radius: f(&v, "sourceRadius", 0.25),
            cast_shadows: on(&v, "castShadows", true),
            kind: match variant(&v["kind"], &["point", "spot", "global"]) {
                1 => Light2dKind::Spot,
                2 => Light2dKind::Global,
                _ => Light2dKind::Point,
            },
            direction: axis.normalize_or(Vec2::X),
            inner_angle: f(&v, "innerAngle", 0.35),
            outer_angle: f(&v, "outerAngle", 0.6),
            height: f(&v, "height", 1.0),
            layers: mask(&v, "layers", u32::MAX),
        });
    }
    for (e, v) in rows(world, layouts.occluder_2d.as_ref()) {
        let (pos, rot, scale) = planar(h.world_of(e));
        // The same outline light2dAt reads (`super::light2d_at`).
        let points: Vec<f32> = v["points"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_f64)
                    .map(|x| x as f32)
                    .collect()
            })
            .unwrap_or_default();
        let polygon: Vec<Vec2> = if points.len() >= 6 {
            super::light2d_at::local_outline(&points, 0.0, 0.0)
                .into_iter()
                .map(|p| p * scale)
                .collect()
        } else {
            Vec::new()
        };
        scene.occluders_2d.push(Occluder2d {
            half_size: Vec2::new(f(&v, "width", 1.0), f(&v, "height", 1.0)) * scale * 0.5,
            layers: mask(&v, "layers", u32::MAX),
            ..Occluder2d::polygon(pos, rot, &polygon)
        });
    }
    for (e, v) in rows(world, layouts.decal.as_ref()) {
        if !on(&v, "visible", true) {
            continue;
        }
        let c = color(&v["color"], Vec3::ONE);
        let (lo, hi) = (vec2(&v["uvMin"], Vec2::ZERO), vec2(&v["uvMax"], Vec2::ONE));
        scene.decals.push(Decal {
            transform: h.world_of(e),
            uv_rect: [lo.x, lo.y, hi.x, hi.y],
            color: [c.x, c.y, c.z, f(&v, "opacity", 1.0)],
            emissive: color(&v["emissive"], Vec3::ZERO),
            roughness: f(&v, "roughness", -1.0),
            angle_fade_degrees: f(&v, "angleFade", 75.0),
            normal_rect: {
                let (lo, hi) = (
                    vec2(&v["normalMin"], Vec2::ZERO),
                    vec2(&v["normalMax"], Vec2::ZERO),
                );
                [lo.x, lo.y, hi.x, hi.y]
            },
            normal_scale: f(&v, "normalScale", 1.0),
        });
    }
    for (e, v) in rows(world, layouts.particles.as_ref()) {
        if !on(&v, "visible", true) {
            continue;
        }
        let Some(index) = handle_index(&v["emitter"]) else {
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
        let stop = u(&v, "stopTick", 0);
        // An unwritten startTick (PARTICLES_ARMED): a looping emitter runs
        // as if started at tick 0, a one-shot waits for its tick.
        let start = match u(&v, "startTick", PARTICLES_ARMED) {
            PARTICLES_ARMED if scene.emitter(emitter).is_some_and(|e| !e.looping) => continue,
            PARTICLES_ARMED => 0,
            t => t,
        };
        let layers = v["layers"].as_u64().map_or(1, |x| x as u32);
        if layers != 1 {
            scene.particle_layers.resize(scene.particles.len(), 1);
            scene.particle_layers.push(layers);
        }
        if let Some(h) = hosted.as_ref().and_then(|h| h.systems.get(&e)) {
            scene.hosted_particles.push((e, Arc::new(h.cpu.clone())));
        }
        scene.particles.push(ParticleSystem {
            id: e,
            emitter,
            transform: h.world_of(e),
            seed: u64::from(u(&v, "seed", 0)),
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
        let c0 = color(&v["color"], Vec3::ONE);
        let c1 = color(&v["colorEnd"], Vec3::ONE);
        let tile = f(&v, "uvTile", 0.0);
        let additive = variant(&v["blend"], &["additive", "alpha"]) == 0;
        let style = TrailStyle {
            lifetime: f(&v, "lifetime", 0.4),
            min_distance: f(&v, "minDistance", 0.05),
            width: Curve::linear(f(&v, "widthStart", 0.15), f(&v, "widthEnd", 0.0)),
            color: ColorCurve::new(&[
                (0.0, [c0.x, c0.y, c0.z, f(&v, "opacity", 1.0)]),
                (1.0, [c1.x, c1.y, c1.z, f(&v, "opacityEnd", 0.0)]),
            ]),
            texture: texture(&v["texture"]),
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
        if on(&v, "layered", false) {
            scene
                .trail_layers
                .push((e, v["layer"].as_i64().unwrap_or(0) as i32));
        }
        scene.trails.push(Trail {
            id: e,
            style: handle,
            position: h.world_of(e).w_axis.truncate(),
            tick,
        });
    }
    for (e, v) in rows(world, layouts.beam.as_ref()) {
        let c = color(&v["color"], Vec3::ONE);
        scene.beams.push(Beam {
            start: h.world_of(e).w_axis.truncate(),
            end: vec3(&v["end"]),
            width: f(&v, "width", 0.1),
            color: [c.x, c.y, c.z, f(&v, "opacity", 1.0)],
            texture: texture(&v["texture"]),
            tile: f(&v, "tile", 1.0),
            scroll: f(&v, "scroll", 0.0),
            segments: u(&v, "segments", 1).max(1),
            jitter: f(&v, "jitter", 0.0),
            seed: u64::from(u(&v, "seed", 0)),
            blend: if variant(&v["blend"], &["additive", "alpha"]) == 0 {
                ParticleBlend::Additive
            } else {
                ParticleBlend::Alpha
            },
        });
    }
}

/// A trail style's identity: the bits of its fields.
fn style_key(s: &TrailStyle) -> Vec<u32> {
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
    scene.ribbons.extend(fx.ribbons.iter().cloned());
    for q in &fx.sprites {
        if q.screen {
            let o = scene
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
        let index = scene.sprites.len() as u32;
        super::feel::set_facing(&mut scene.sprite_facing, index as usize, q.facing);
        scene.sprites.push(q.sprite);
        if let Some((array, layer)) = q.array {
            scene.sprite_arrays.push(og_render::SpriteArray {
                sprite: index,
                array,
                layer,
            });
        }
        if q.order != 0.0 {
            scene.sprite_order.push(og_render::SpriteOrder {
                sprite: index,
                key: q.order,
            });
        }
        if q.cast_shadow {
            scene
                .occluders_2d
                .push(og_render::Occluder2d::from_sprite(&q.sprite, 0.0));
        }
        if q.drop_shadow > 0.0 {
            scene.sprite_shadows.push(og_render::SpriteShadow {
                sprite: index,
                opacity: q.drop_shadow.min(1.0),
                ground: og_render::ShadowGround::level(q.drop_shadow_height),
            });
        }
        if q.flash[3] > 0.0 || q.glow > 0.0 || q.additive || !q.tone_mapped {
            scene.sprite_fx.push(og_render::SpriteFx {
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
        .map(|(e, v)| (e, v["asset"].as_f64().unwrap_or(0.0) as u64))
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
    let rs_motion = &motion.palettes;
    let Some(anim) = fx.anim.as_ref() else {
        return false;
    };
    let Some(mut out) = anim.output(Entity::from_bits(entity)) else {
        return false;
    };
    // og.motion's generated pose, made for this frame, when it has one.
    let matrices = match rs_motion.get(&entity) {
        Some(m) if m.len() == out.palette.len() => m.clone(),
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
    let palette = scene.joint_palettes.len() as u32;
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
    scene.joint_palettes.push(JointPalette { matrices });
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
                    inst.deformed = Some(scene.deformed_vertices.len() as u32);
                    scene.deformed_vertices.push(vertices);
                    fx.deformed_keys.push(((entity, p), world_matrix));
                } else if info.prim_mesh.get(p).copied() == Some(out.mesh) {
                    let n = inst.morph_weights.len().min(out.weights.len());
                    inst.morph_weights[..n].copy_from_slice(&out.weights[..n]);
                }
                scene.skinned_instances.push(inst);
            }
            None => {
                scene.instances.push(og_render::Instance {
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
    use super::codec::{Rec, Row, Slot};
    let Some(layout) = rs.draw.fx.crowd.as_ref() else {
        return IdMap::new();
    };
    let Ok(id) = world.id_by_name(&layout.schema) else {
        return IdMap::new();
    };
    let slot = |key| Slot::of(layout, key);
    let (skeleton, skin, clip) = (slot("skeleton"), slot("skin"), slot("clip"));
    let (offset, speed, looping, rate) =
        (slot("offset"), slot("speed"), slot("loop"), slot("rate"));
    super::feel::byte_rows(world, id, layout)
        .iter()
        .map(|(e, bytes)| {
            let v = Row::new(layout, bytes);
            let num = |s: &Slot, d: f64| v.num(s).unwrap_or(d);
            (
                *e,
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
    let index = scene.baked_clips.len() as u32;
    scene.baked_clips.push(og_render::BakedClip {
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
            .crowds
            .iter()
            .position(|c| c.cast_shadows == look.cast && c.parts == skinned);
        let row = match at {
            Some(i) => &mut scene.crowds[i],
            None => {
                scene.crowds.push(og_render::Crowd {
                    parts: skinned,
                    cast_shadows: look.cast,
                    ..Default::default()
                });
                scene.crowds.last_mut().expect("just pushed")
            }
        };
        row.members.push(member);
    }
    for (k, &(mesh, material, local)) in parts.iter().enumerate() {
        let p = info.primitive(mesh);
        match meshes.get(p).copied().flatten() {
            Some(_) => {}
            None => {
                scene.instances.push(og_render::Instance {
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
            .zip(&scene.instances)
            .map(|(k, i)| (*k, i.transform)),
    );
    scene.previous_transforms.clear();
    scene.previous_joint_palettes.clear();
    scene.previous_morph_weights.clear();
    scene.previous_deformed_vertices.clear();
    let blur = scene.post.motion_blur.is_some();
    let keyed = keys.len() == scene.instances.len();
    if blur && keyed {
        let previous = fx
            .previous_by_key
            .get_or_insert_with(|| fx.previous.iter().copied().collect());
        scene.previous_transforms = keys
            .iter()
            .zip(&scene.instances)
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
    joints(scene, atmo, fx);
    if !interp.display && !taa {
        return;
    }
    let from_drawn = if blur {
        interp.drawn.is_some() && !scene.previous_transforms.is_empty()
    } else {
        keyed
    };
    if from_drawn {
        let last: BTreeMap<(u64, u32), Mat4> = fx.drawn.iter().copied().collect();
        scene.previous_transforms = keys
            .iter()
            .zip(&scene.instances)
            .map(|(k, i)| last.get(k).copied().unwrap_or(i.transform))
            .collect();
    }
    if !cut.is_empty() {
        for ((k, i), p) in keys
            .iter()
            .zip(&scene.instances)
            .zip(&mut scene.previous_transforms)
        {
            if cut.contains_key(&k.0) {
                *p = i.transform;
            }
        }
    }
    fx.drawn.clear();
    fx.drawn.extend(
        keys.iter()
            .zip(&scene.instances)
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
    let mut entity_of = vec![None; scene.joint_palettes.len()];
    for (&e, &p) in &atmo.palettes {
        if let Some(slot) = entity_of.get_mut(p as usize) {
            *slot = Some(e);
        }
    }
    let mut at = vec![None; scene.joint_palettes.len()];
    for s in &scene.skinned_instances {
        if let Some(t) = at.get_mut(s.palette as usize) {
            t.get_or_insert(s.transform);
        }
    }
    let mut now = IdMap::new();
    scene.previous_joint_palettes = scene
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
    scene.previous_morph_weights = scene
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
    scene.previous_deformed_vertices = scene
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
    let host = super::particle_events::shared(eng.id);
    rs.fx.host = Some(host.clone());
    // An emitter that sends events: the tick simulates its systems.
    let definition = (!emitter.events.is_empty()).then(|| {
        let subs = emitter
            .sub_emitters
            .iter()
            .map(|s| rs.scene.emitter(s.emitter).cloned())
            .collect();
        let points = match emitter.shape {
            og_render::EmitterShape::Mesh { mesh } => rs
                .scene
                .mesh(mesh)
                .map(og_render::mesh_points)
                .unwrap_or_default(),
            _ => vec![[0.0; 4]; 2],
        };
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
        },
    )?;
    Ok((emitter, array))
}

/// A mesh particle's mesh: a built-in shape (made once) or a model's first
/// mesh, the model loaded with the emitter.
fn particle_mesh(
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
            rs.fx.particle_meshes.insert(name, h);
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

    /// A look's MaterialStyle.shadowDarkness reaches the renderer's
    /// material style; unset it is 0 (off), so no look moves without it.
    #[test]
    fn material_style_carries_the_shadow_darkness() {
        let m = Arc::new(serde_json::json!({ "toon": true, "shadowDarkness": 0.75 }));
        let s = stylise(None, Some(m), &[], &[]);
        assert_eq!(s.material.shadow_darkness, 0.75);
        assert!(s.material.toon);
        let off = stylise(
            None,
            Some(Arc::new(serde_json::json!({ "toon": true }))),
            &[],
            &[],
        );
        assert_eq!(off.material.shadow_darkness, 0.0);
    }

    #[test]
    fn the_material_style_s_surface_reaches_the_renderer() {
        let m = |v: Value| stylise(None, Some(Arc::new(v)), &[], &[]).material;
        // As the resource decodes it (the variant's index) and as a look
        // names it (the variant's name).
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
