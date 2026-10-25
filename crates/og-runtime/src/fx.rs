//! The effects a game declares as data, extracted into og-render's scene
//! (the IDL's `PostEffects`, `Lighting2D`, `Light2D`, `Occluder2D`, `Decal`,
//! `ParticleSystem`, `Trail`, `Beam`, `og.render.ribbon`, the sprite effect
//! and screen fields, and skinned models posed by og-anim), and the loads of
//! their assets (`og.load.lut`, `og.load.emitter`). Like the rest of
//! extraction it reads the world and never writes it.

use super::codec::Layout;
use super::extract::{Hierarchy, RenderState};
use glam::{Mat4, Vec2, Vec3};
use og_ecs::{Entity, QueryDesc, World};
use og_render::{
    AmbientOcclusion, AoMethod, Beam, CameraShake, ChromaticAberration, ColorCurve, ColorGrading,
    Curve, Decal, DepthOfField, DofQuality, FilmGrain, Glow, GodRays, JointPalette, Light2d,
    Lighting2d, MeshHandle, MorphTargets, MotionBlur, Occluder2d, Outline, ParticleBlend,
    ParticleSystem, RibbonUv, ScreenFlash, SkinData, SkinnedInstance, Trail, TrailStyle,
    TrailStyleHandle, Vignette,
};
use serde_json::Value;
use std::collections::BTreeMap;

/// The layouts of the effect records the engine registered.
#[derive(Default)]
pub(crate) struct FxLayouts {
    pub post: Option<Layout>,
    pub lighting_2d: Option<Layout>,
    pub light_2d: Option<Layout>,
    pub occluder_2d: Option<Layout>,
    pub decal: Option<Layout>,
    pub particles: Option<Layout>,
    pub trail: Option<Layout>,
    pub beam: Option<Layout>,
    pub skeleton: Option<Layout>,
    pub stylise: Option<Layout>,
    pub material_style: Option<Layout>,
}

impl FxLayouts {
    pub fn from(get: impl Fn(&str) -> Option<Layout>) -> FxLayouts {
        FxLayouts {
            post: get("og.PostEffects"),
            lighting_2d: get("og.Lighting2D"),
            light_2d: get("og.Light2D"),
            occluder_2d: get("og.Occluder2D"),
            decal: get("og.Decal"),
            particles: get("og.ParticleSystem"),
            trail: get("og.Trail"),
            beam: get("og.Beam"),
            skeleton: get("og.Skeleton"),
            stylise: get("og.Stylise"),
            material_style: get("og.MaterialStyle"),
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
}

/// One `og.load.lut`: its renderer LUT while resident.
#[derive(Clone, Debug)]
pub(crate) struct LoadedLut {
    pub id: String,
    /// `None` while its streaming group is not resident: released when the
    /// group unloaded, added again once it is resident (`resolve_luts`).
    pub handle: Option<og_render::LutHandle>,
}

/// One loaded model's primitives, for skinning: the scene mesh of its first
/// primitive (they are consecutive), the glTF mesh of each primitive, and
/// the node of each of its parts (`og_cook::runtime::ModelParts::nodes`).
#[derive(Clone, Debug)]
pub(crate) struct ModelInfo {
    pub id: String,
    pub base: usize,
    pub prim_mesh: Vec<u32>,
    pub part_nodes: Vec<usize>,
}

/// A model's skin and morph data from its `#anim` blob, and the skinned
/// copies of its primitives once made (by primitive; `None`: not skinned).
#[derive(Clone, Debug, Default)]
pub(crate) struct SkinnedModel {
    pub streams: Vec<og_cook::runtime::SkinStream>,
    pub morphs: Vec<og_cook::runtime::MorphTargets>,
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
    pub emitter_arrays: BTreeMap<usize, usize>,
    pub by_path: BTreeMap<String, usize>,
    /// Trail styles made from Trail components, by their fields' bits.
    pub trail_styles: BTreeMap<Vec<u32>, TrailStyleHandle>,
    /// Ribbons and sprites queued this tick (`og.render.ribbon`, `.sprite`).
    pub ribbons: Vec<og_render::Ribbon>,
    pub sprites: Vec<QueuedSprite>,
    /// Loaded models by asset handle bits, and skin data by model id.
    pub models: BTreeMap<u64, ModelInfo>,
    pub skins: BTreeMap<String, SkinnedModel>,
    /// Looks and palettes (`super::look`).
    pub look: super::look::LookState,
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

fn variant(v: &Value, names: &[&str]) -> usize {
    match v {
        Value::String(s) => names.iter().position(|n| n == s).unwrap_or(0),
        Value::Number(n) => n.as_u64().unwrap_or(0) as usize,
        _ => 0,
    }
}

pub(crate) fn resource(world: &World, layout: Option<&Layout>) -> Option<Value> {
    let layout = layout?;
    let id = world.id_by_name(&layout.schema).ok()?;
    Some(layout.decode(&world.resource_bytes(id).ok()?))
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
    post.ssr = None;
    post.glow = None;
    post.color_grading = None;
    post.vignette = None;
    post.chromatic_aberration = None;
    post.film_grain = None;
    post.flash = None;
    scene.shake = CameraShake::default();
    scene.lighting_2d = None;
    if let Some(p) = resource(world, layouts.post.as_ref()) {
        if on(&p, "ao", false) {
            post.ao = Some(AmbientOcclusion {
                radius: f(&p, "aoRadius", 0.6),
                intensity: f(&p, "aoIntensity", 1.0),
                method: [AoMethod::Auto, AoMethod::Gtao, AoMethod::Ssao]
                    [variant(&p["aoMethod"], &["auto", "gtao", "ssao"]).min(2)],
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
                max_distance: f(&p, "ssrMaxDistance", 20.0),
                thickness: f(&p, "ssrThickness", 0.3),
                steps: u(&p, "ssrSteps", 32),
                fade: f(&p, "ssrFade", 0.15),
                intensity: f(&p, "ssrIntensity", 1.0),
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
        let grain = f(&p, "filmGrain", 0.0);
        if grain > 0.0 {
            post.film_grain = Some(FilmGrain {
                intensity: grain,
                size: f(&p, "filmGrainSize", 1.5),
            });
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
    if let Some(l) = resource(world, layouts.lighting_2d.as_ref())
        && on(&l, "enabled", false)
    {
        scene.lighting_2d = Some(Lighting2d {
            ambient: color(&l["ambient"], Vec3::splat(0.15)),
            min_layer: l["minLayer"].as_i64().unwrap_or(-1_000_000) as i32,
            max_layer: l["maxLayer"].as_i64().unwrap_or(99) as i32,
            resolution: f(&l, "resolution", 0.5),
        });
    }
}

/// Pixel snap (`Stylise.pixelSnap`): on a logical pixel grid with the 2D
/// camera (orthographic, looking down -z), the camera and every world
/// sprite move by whole logical pixels, so a scrolling scene never swims
/// between pixel rows and sprites keep their edges.
pub(crate) fn snap_to_grid(rs: &mut RenderState) {
    let scene = &mut rs.scene;
    let Some(p) = scene.stylise.pixel.filter(|p| p.snap && p.height > 0) else {
        return;
    };
    let og_render::Projection::Orthographic { height } = scene.camera.projection else {
        return;
    };
    let forward = (scene.camera.target - scene.camera.position).normalize_or_zero();
    if forward.z > -0.999 {
        return;
    }
    let unit = height / p.height as f32;
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
    s: Option<Value>,
    m: Option<Value>,
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
        if posterize > 0 || palette.is_some() || dither != og_render::DitherPattern::None {
            out.quantize = Some(og_render::Quantize {
                posterize,
                palette,
                dither,
                dither_strength: f(&s, "ditherStrength", 1.0),
                dither_scale: u(&s, "ditherScale", 1).max(1),
            });
        }
        let space = |v: &Value| match variant(v, &["screen", "surface", "object"]) {
            1 => og_render::StyliseSpace::Surface,
            2 => og_render::StyliseSpace::Object,
            _ => og_render::StyliseSpace::Screen,
        };
        let halftone = f(&s, "halftone", 0.0);
        if halftone > 0.0 {
            out.halftone = Some(og_render::Halftone {
                strength: halftone,
                size: f(&s, "halftoneSize", 6.0),
                angle_degrees: f(&s, "halftoneAngle", 45.0),
                color: rgb(&s["halftoneColor"], Vec3::new(0.02, 0.02, 0.03)),
                space: space(&s["halftoneSpace"]),
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
                space: space(&s["hatchingSpace"]),
                depth: f(&s, "hatchingDepth", 0.0),
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
        };
        if paint.radius > 0.0
            || paint.edge_darkening > 0.0
            || paint.bleed > 0.0
            || (paint.canvas.is_some() && paint.canvas_strength > 0.0)
        {
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
            rim_color: rgb(&m["rimColor"], Vec3::ONE),
            rim_intensity: f(&m, "rimIntensity", 0.0),
            rim_power: f(&m, "rimPower", 3.0),
            outline_color: rgb(&m["outlineColor"], Vec3::new(0.02, 0.02, 0.03)),
            outline_width: f(&m, "outlineWidth", 0.0),
            outline_distance: f(&m, "outlineDistance", 0.0),
            flat_shading: on(&m, "flatShading", false),
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
    scene.trails.clear();
    scene.beams.clear();
    scene.ribbons.clear();
    let texture = |v: &Value| handle_index(v).and_then(|i| textures.get(i).copied());
    for (e, v) in rows(world, layouts.light_2d.as_ref()) {
        let (pos, _, _) = planar(h.world_of(e));
        scene.lights_2d.push(Light2d {
            position: pos,
            color: color(&v["color"], Vec3::ONE),
            intensity: f(&v, "intensity", 1.0),
            radius: f(&v, "radius", 5.0),
            falloff: f(&v, "falloff", 2.0),
            source_radius: f(&v, "sourceRadius", 0.25),
            cast_shadows: on(&v, "castShadows", true),
        });
    }
    for (e, v) in rows(world, layouts.occluder_2d.as_ref()) {
        let (pos, rot, scale) = planar(h.world_of(e));
        scene.occluders_2d.push(Occluder2d {
            center: pos,
            half_size: Vec2::new(f(&v, "width", 1.0), f(&v, "height", 1.0)) * scale * 0.5,
            rotation: rot,
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
        scene.particles.push(ParticleSystem {
            id: e,
            emitter,
            transform: h.world_of(e),
            seed: u64::from(u(&v, "seed", 0)),
            start_tick: u64::from(start),
            stop_tick: (stop > 0).then_some(u64::from(stop)),
            tick,
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
        let Some(mesh) = scene
            .mesh_handle(info.base + p)
            .and_then(|h| scene.mesh(h))
            .cloned()
        else {
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
pub(crate) fn skeletons(world: &World, rs: &RenderState) -> std::collections::BTreeSet<u64> {
    rows(world, rs.draw.fx.skeleton.as_ref())
        .into_iter()
        .map(|(e, _)| e)
        .collect()
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
/// its bind pose.
pub(crate) fn posed_model(
    rs: &mut RenderState,
    entity: u64,
    asset: u64,
    world_matrix: Mat4,
    parts: &[(MeshHandle, og_render::MaterialHandle, Mat4)],
    look: ModelLook,
    keys: &mut Vec<(u64, u32)>,
) -> bool {
    let RenderState {
        scene, fx, atmo, ..
    } = rs;
    let Some(anim) = fx.anim.as_ref() else {
        return false;
    };
    let Some(out) = anim.output(Entity::from_bits(entity)) else {
        return false;
    };
    let Some(info) = fx.models.get(&asset).cloned() else {
        return false;
    };
    let Some(skin) = fx.skins.get_mut(&info.id) else {
        return false;
    };
    let meshes = skinned_meshes(scene, &info, skin, info.prim_mesh.len());
    let palette = scene.joint_palettes.len() as u32;
    atmo.palettes.insert(entity, palette);
    scene.joint_palettes.push(JointPalette {
        matrices: out.palette.clone(),
    });
    // Parts and nodes line up unless the parts are not the model's own
    // (a bake's lightmapped parts): then every part is drawn.
    let nodes = (info.part_nodes.len() == parts.len()).then_some(&info.part_nodes);
    for (k, &(mesh, material, local)) in parts.iter().enumerate() {
        if nodes.is_some_and(|n| out.drawn.get(n[k]) == Some(&false)) {
            continue;
        }
        let p = mesh.index().wrapping_sub(info.base);
        match meshes.get(p).copied().flatten() {
            Some(skinned) => {
                let mut inst = SkinnedInstance::new(skinned, material, world_matrix, palette);
                inst.fx = look.fx;
                inst.cast_shadows = look.cast;
                inst.receive_shadows = look.receive;
                if info.prim_mesh.get(p).copied() == Some(out.mesh) {
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
                });
                keys.push((entity, k as u32));
            }
        }
    }
    true
}

/// Per-object motion: this tick's instance transforms, keyed, and last
/// tick's for `previous_transforms` (same order as the instances).
pub(crate) fn motion(rs: &mut RenderState, keys: &[(u64, u32)], tick: u64) {
    let RenderState { scene, fx, .. } = rs;
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
    if scene.post.motion_blur.is_some() && keys.len() == scene.instances.len() {
        let previous = fx
            .previous_by_key
            .get_or_insert_with(|| fx.previous.iter().copied().collect());
        scene.previous_transforms = keys
            .iter()
            .zip(&scene.instances)
            .map(|(k, i)| previous.get(k).copied().unwrap_or(i.transform))
            .collect();
    }
}

/// The overlay's size in CSS pixels: the host's viewport, else the target.
pub(crate) fn viewport(rs: &RenderState) -> (f32, f32) {
    if let Some(v) = rs.viewport {
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
    let id = og_cook::runtime::asset_id(path).to_string();
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
/// array) loaded with it.
pub(crate) fn load_emitter(eng: &super::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    let key = format!("emitter:{id}");
    if let Some(&i) = eng.render.borrow().fx.by_path.get(&key) {
        return Ok(i);
    }
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .emitter_text(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let mut array = None;
    let emitter = og_cook::fx::parse_emitter_with(
        &text,
        &mut |tex| {
            let i = super::assets::load_texture(eng, tex)?;
            Ok(eng.render.borrow().textures.get(i).copied())
        },
        &mut |a| {
            let i = super::assets::load_texture_array_index(eng, a)?;
            array = Some(i);
            Ok(eng
                .render
                .borrow()
                .texture_arrays
                .get(i)
                .and_then(|a| a.handle))
        },
    )
    .map_err(|e| format!("{id}: {e}"))?;
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_emitter(emitter);
    rs.fx.emitters.push(h);
    let i = rs.fx.emitters.len() - 1;
    if let Some(a) = array {
        rs.fx.emitter_arrays.insert(i, a);
    }
    rs.fx.by_path.insert(key, i);
    Ok(i)
}
