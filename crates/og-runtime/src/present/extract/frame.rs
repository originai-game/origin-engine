//! Per-frame scene extraction and particle colliders.

use super::*;

mod colliders;
mod debris;
mod lights;
mod models;
mod origin;

use colliders::*;
use debris::*;
use lights::*;
use models::*;
pub(crate) use origin::*;

/// The render target's size, (1, 1) without a renderer.
fn backend_size(rs: &RenderState) -> (u32, u32) {
    rs.backend.as_ref().map_or((1, 1), |b| b.size())
}

/// The frame's beginning, as the stages after it read it.
struct Begun {
    /// What a review camera frames of the last frame, in this frame's world.
    framed: Option<(Vec3, Vec3)>,
    /// The render target's size.
    size: (u32, u32),
    camera: Option<std::sync::Arc<Value>>,
    settings: Option<crate::host::wire::record::RenderSettings>,
    /// The world's zero in the absolute world (og.WorldOrigin).
    zero: glam::DVec3,
    /// The queued sprites and texts are this tick's (or a command's since).
    fresh: bool,
}

/// Where the frame is seen from: the camera's exposure, the render origin
/// and the camera's forward.
struct View {
    exposure: f32,
    origin: Vec3,
    look_along: Option<Vec3>,
}

/// Rebuild the per-frame part of `rs.scene` from the world as of its tick,
/// stage by stage: the frame begins (the lists cleared, the display frame
/// blended), then the camera, the settings, the models, the meshes, the
/// lights, the effects and the world's other drawables, the text, and the
/// frame's end (queued sprites, lines, motion, views, the UI, the render
/// origin).
pub(crate) fn extract(world: &World, rs: &mut RenderState) {
    let mut h = std::mem::take(&mut rs.hierarchy);
    let b = begin(world, rs, &mut h);
    let view = camera(rs, &b);
    settings(world, rs, &b, view.exposure);
    let mut keys: Vec<(u64, u32)> = Vec::new();
    models(world, rs, &mut h, &b, &mut keys);
    meshes(world, rs, &mut h, &mut keys);
    lights(world, rs, &mut h);
    let (vp, tick) = effects(world, rs, &mut h, &mut keys);
    texts(rs, b.fresh, vp);
    finish(world, rs, h, &b, &view, &keys, vp, tick);
}

/// The frame begins: the hierarchy, timelines' materials and the art
/// contract; the per-frame lists cleared; the display frame between ticks.
fn begin(world: &World, rs: &mut RenderState, h: &mut Hierarchy) -> Begun {
    h.refresh(world);
    // Timelines' material tracks, at their time (presentation only).
    crate::bind::timeline::present(world, rs);
    // The art contract this frame (nothing without project.ogt's `look`).
    super::art::frame(world, rs);
    // A review camera frames what the last frame drew for its entity, so
    // the models below pick their levels for it (once more at the end).
    let ids = std::mem::take(&mut rs.ids.instances);
    // The last frame was drawn from its render origin; this one is built
    // in the world and moved to its own at the end (`rebase`).
    // In the absolute world (R12's og.WorldOrigin moves the world's zero
    // there); the world's place is it less the zero.
    let drawn_at = std::mem::take(&mut rs.scene.frame.origin);
    let zero = crate::world::origin::zero(world, rs.draw.origin.as_ref());
    let back = drawn_at - zero.as_vec3();
    let framed = super::review::bounds(rs, h, &ids).map(|(a, b)| (a + back, b + back));
    rs.ids.instances = ids;
    let size = backend_size(rs);
    let camera = resource(world, rs.camera.as_ref());
    let settings = resource(world, rs.settings.as_ref())
        .and_then(|s| crate::host::wire::record::RenderSettings::from_json(&s).ok());
    // A display frame between ticks: every world matrix and the camera
    // from here on are blended with the tick before (`super::interpolate`).
    super::interpolate::begin(world, rs, settings.as_ref(), h);
    let blended = rs.interp.drawn;
    rs.atmo.palettes.clear();
    rs.ids.sprites.clear();
    let scene = &mut rs.scene;
    scene.objects.instances.clear();
    scene.lighting.lights.clear();
    scene.sprite.texts.clear();
    scene.sprite.sprites.clear();
    scene.sprite.sprite_meshes.clear();
    scene.ui.overlay = None;
    scene.look.color_filter = None;
    scene.objects.joint_palettes.clear();
    scene.objects.skinned_instances.clear();
    scene.objects.crowds.clear();
    scene.objects.deformed_vertices.clear();
    scene.objects.previous_deformed_vertices.clear();
    rs.fx.deformed_keys.clear();
    rs.fx.drawn_baked = std::mem::take(&mut rs.fx.baked_now);
    scene.sprite.sprite_order.clear();
    scene.sprite.sprite_fx.clear();
    scene.sprite.sprite_arrays.clear();
    scene.sprite.sprite_shadows.clear();
    scene.sprite.sprite_outlines.clear();
    scene.sprite.sprite_shading.clear();
    scene.sprite.text_outlines.clear();
    scene.sprite.tile_layers.clear();
    scene.effects.trail_layers.clear();
    scene.effects.lines.clear();
    scene.effects.points.clear();
    // Layer masks of the sprites, lines, points and particle systems
    // (views.rs): filled by what pushes them, 1 past their ends.
    scene.sprite.sprite_layers.clear();
    // 2D light masks of the sprites (og.LightMask2D): 1 past the end.
    scene.sprite.sprite_light_masks.clear();
    // Billboards (sprites' facing): set by what pushes them, none past the end.
    scene.sprite.sprite_facing.clear();
    scene.sprite.text_facing.clear();
    scene.effects.line_layers.clear();
    scene.effects.point_layers.clear();
    scene.effects.particle_layers.clear();
    let time = world.time();
    scene.frame.tick = time.tick;
    scene.frame.tick_rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    // The sea's clock between ticks (og_render's `RenderScene::interpolation`).
    scene.frame.interpolation = blended;
    // og.render.sprite and og.render.text are immediate mode: what the last
    // tick submitted (or a command since it) is drawn, and a tick that
    // submitted nothing draws nothing. The queues are cleared lazily by the
    // next submission, so a queue from an older tick is stale here.
    let now = world.time().tick;
    let fresh = rs.texts_tick == now || rs.texts_tick + 1 == now;

    Begun {
        framed,
        size,
        camera,
        settings,
        zero,
        fresh,
    }
}

/// The camera (with a review camera's framing).
// Rendering only: the drawn cameras' field of view.
#[allow(clippy::disallowed_methods)]
fn camera(rs: &mut RenderState, b: &Begun) -> View {
    let (zero, size, framed) = (b.zero, b.size, b.framed);
    // An absent background lane read 0 by hand, where the IDL's default is
    // the dark blue: kept.
    let base = crate::host::wire::record::Camera {
        background: crate::host::wire::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        },
        ..Default::default()
    };
    let camera = b
        .camera
        .as_deref()
        .and_then(|c| crate::host::wire::record::Camera::from_json_over(c, base).ok());
    let scene = &mut rs.scene;
    // Camera.
    let mut exposure = 1.0f32;
    scene.frame.camera_layers = camera.as_ref().map_or(1, |c| c.layers);
    scene.frame.camera_cut = false;
    let (mut origin, mut look_along) = (Vec3::ZERO, None);
    if let Some(c) = &camera {
        let (position, rot) = super::interpolate::camera_pose(c);
        let (position, rot, cut) = super::interpolate::camera(&rs.interp, position, rot);
        scene.frame.camera_cut = cut;
        let cam = &mut scene.frame.camera;
        cam.position = position;
        cam.target = position + og_core::facing::camera_forward(rot);
        cam.up = og_core::facing::up(rot);
        origin = render_origin(position, zero);
        look_along = Some(og_core::facing::camera_forward(rot));
        cam.near = c.near;
        cam.far = c.far;
        let zoom = c.zoom.max(0.01);
        cam.projection = if c.orthographic {
            og_render::Projection::Orthographic {
                height: c.height / zoom,
            }
        } else {
            // three.js zoom narrows the field of view: tan(fov/2) / zoom.
            let half = (c.fov.to_radians() / 2.0).tan() / zoom;
            og_render::Projection::Perspective {
                fov_y_degrees: (2.0 * half.atan()).to_degrees(),
            }
        };
        exposure = c.exposure.max(0.0);
        let bg = &c.background;
        scene.lighting.environment.background =
            og_render::Background::Color(Vec3::new(bg.r, bg.g, bg.b));
    }
    super::review::apply(&mut scene.frame.camera, rs.review.as_ref(), size, framed);

    View {
        exposure,
        origin,
        look_along,
    }
}

/// RenderSettings: environment, sky, bloom, exposure, tone mapping and
/// the quality fields.
// Rendering only: the exposure's log.
#[allow(clippy::disallowed_methods)]
fn settings(world: &World, rs: &mut RenderState, b: &Begun, exposure: f32) {
    let settings = b.settings.as_ref();
    let mut exposure = exposure;
    let layouts = &rs.draw;
    let scene = &mut rs.scene;
    // Settings: environment, sky, bloom, exposure, tone mapping.
    if let Some(s) = settings {
        // RenderSettings.exposure scales every camera's, as three.js
        // toneMappingExposure does the whole frame's.
        exposure *= s.exposure.max(0.0);
        scene.lighting.environment.intensity = s.environment_intensity;
        if s.sky {
            scene.lighting.environment.background = og_render::Background::Sky;
        }
        scene.look.post.bloom = s.bloom.then_some(og_render::Bloom {
            threshold: s.bloom_threshold,
            knee: s.bloom_knee,
            intensity: s.bloom_intensity,
        });
        let radius = s.bloom_radius;
        scene.look.post.bloom_unreal = (radius >= 0.0).then(|| radius.min(1.0));
        scene.look.post.bloom_contrast = s.bloom_contrast.clamp(0.5, 2.0);
        scene.look.post.tonemap = if s.tone_mapped {
            match s.tone_mapping {
                crate::host::wire::ToneMapping::Agx => og_render::Tonemap::Agx,
                crate::host::wire::ToneMapping::None => og_render::Tonemap::None,
                crate::host::wire::ToneMapping::Neutral => og_render::Tonemap::Neutral,
                crate::host::wire::ToneMapping::Aces => og_render::Tonemap::Aces,
            }
        } else {
            og_render::Tonemap::None
        };
        scene.frame.camera.exposure = if s.auto_exposure {
            og_render::Exposure::Auto {
                compensation: exposure.max(1e-6).log2(),
                min_ev: -10.0,
                max_ev: 10.0,
                adapt_speed_up: s.adapt_speed_up.max(0.0),
                adapt_speed_down: s.adapt_speed_down.max(0.0),
            }
        } else {
            og_render::Exposure::linear(exposure)
        };
        // The quality fields, one to one (og.render.applyQuality writes a
        // preset's row of them; RenderSettings.quality is only its record).
        // The counts go through f32 as the hand read took them.
        scene.frame.quality = og_render::RenderQuality {
            rect_shadows: s.rect_shadows,
            shadowed_lights: s.shadowed_lights as f32 as u32,
            shadow_cascades: (s.shadow_cascades as f32).max(1.0) as u32,
            shadow_map_scale: s.shadow_map_scale,
            volumetric_fog: s.volumetric_fog,
            soft_particles: s.soft_particles,
            ssr: s.screen_space_reflections,
            ambient_occlusion: s.ambient_occlusion,
            // On the lazy og.GlobalIllumination's `allowed` (crate::present::fx).
            global_illumination: crate::present::fx::gi_allowed(
                crate::present::fx::resource(world, layouts.fx.global_illumination.as_ref())
                    .as_deref(),
                rs.fx.gi_preset,
            ),
            ao_resolution: match s.ao_resolution {
                crate::host::wire::AoResolutionSetting::PostEffects => None,
                crate::host::wire::AoResolutionSetting::Full => Some(og_render::AoResolution::Full),
                crate::host::wire::AoResolutionSetting::Half => Some(og_render::AoResolution::Half),
            },
            ssr_step_scale: s.ssr_step_scale,
            volumetric_step_scale: s.volumetric_step_scale,
            msaa_samples: (s.msaa_samples as f32).max(1.0) as u32,
            upscaler: match s.upscaler {
                crate::host::wire::Upscaler::Taau => og_render::Upscaler::Taau,
                crate::host::wire::Upscaler::Fsr3 => og_render::Upscaler::Fsr3,
            },
        };
    } else {
        scene.frame.camera.exposure = og_render::Exposure::linear(exposure);
        scene.frame.quality = og_render::RenderQuality {
            global_illumination: crate::present::fx::gi_allowed(
                crate::present::fx::resource(world, layouts.fx.global_illumination.as_ref())
                    .as_deref(),
                rs.fx.gi_preset,
            ),
            ..og_render::RenderQuality::default()
        };
    }
}

/// The effects games declare, the atmosphere, the environment, probes,
/// terrain, the camera's feel, sprites, tiles and the 2D drawables: the
/// viewport (CSS pixels) and the tick.
fn effects(
    world: &World,
    rs: &mut RenderState,
    h: &mut Hierarchy,
    keys: &mut Vec<(u64, u32)>,
) -> ((f32, f32), u64) {
    // The effects games declare: the post stack, shake and 2D lighting,
    // then the effect components.
    let tick = world.time().tick;
    super::fx::settings(world, rs);
    super::fx::components(world, rs, h, tick);
    crate::present::vfx::extract(world, rs, h, tick);
    rs.scene.effects.particle_history = rs.particle_history.clone();
    // The pixel grid's snap: after the sprites and the camera are in.
    super::fx::snap_to_grid(rs);
    // Cloth meshes are instances too: keyed for per-object motion.
    keys.extend(super::atmosphere::extract(world, rs, h));
    // The wind copies' parameters, from the og.Wind just read.
    let (wind, tick, rate) = (
        rs.scene.atmosphere.wind,
        rs.scene.frame.tick,
        rs.scene.frame.tick_rate,
    );
    rs.wind.present(&mut rs.scene, wind, tick, rate);
    super::environment::extract(world, rs);
    super::probes::extract(world, rs, h);
    super::probes::extract_light_volumes(world, rs, h);
    // Scattered copies are instances too, keyed by their terrain cell.
    keys.extend(super::terrain::extract(world, rs, h));
    colliders(world, rs, h);
    let vp = super::fx::viewport(rs);
    // The camera's shake, punch, kick and zones (on the frame only), then
    // og.Sprite rows, floating texts and tile layers.
    let aspect = match rs.backend.as_ref().map(|b| b.size()) {
        Some((w, ht)) if ht > 0 => w as f32 / ht as f32,
        _ if vp.1 > 0.0 => vp.0 / vp.1,
        _ => 16.0 / 9.0,
    };
    super::feel::camera_frame(world, rs, aspect);
    super::feel::extract(world, rs, h);
    crate::bind::destruction2d::extract(world, rs, h);
    super::skeleton2d::extract(world, rs, h);

    (vp, tick)
}

/// Characters the fonts lack, then the text queued for this frame.
fn texts(rs: &mut RenderState, fresh: bool, vp: (f32, f32)) {
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
                    .ui
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
                    scene.sprite.text_outlines.push(og_render::TextOutline {
                        text: scene.sprite.texts.len() as u32,
                        width: t.outline,
                        color: t.outline_color,
                    });
                }
                if t.facing != og_render::SpriteFacing::None {
                    scene.sprite.text_facing.push(og_render::TextFacing {
                        text: scene.sprite.texts.len() as u32,
                        facing: t.facing,
                        align: t.align,
                    });
                }
                scene.sprite.texts.push(text);
            }
        }
    }
}

/// The frame's end: queued sprites, the review camera, lines and points,
/// per-object motion, views, ids, drop shadows, video, the UI, and the
/// render origin.
#[allow(clippy::too_many_arguments)]
fn finish(
    world: &World,
    rs: &mut RenderState,
    mut h: Hierarchy,
    b: &Begun,
    view: &View,
    keys: &[(u64, u32)],
    vp: (f32, f32),
    tick: u64,
) {
    let (fresh, size, zero, camera) = (b.fresh, b.size, b.zero, b.camera.clone());
    let (origin, look_along) = (view.origin, view.look_along);
    // Sprites and ribbons queued this tick.
    super::fx::queued(rs, fresh, vp);
    // The review camera on this frame's instances of its entity.
    if rs.review.is_some() {
        let ids: Vec<u64> = keys.iter().map(|k| k.0).collect();
        let framed = super::review::bounds(rs, &h, &ids);
        super::review::apply(&mut rs.scene.frame.camera, rs.review.as_ref(), size, framed);
    }
    // Lines, points and debug draw (after every instance is in, for the
    // bounds view).
    super::geometry::extract(world, rs, &mut h, vp);
    // Teleported into this frame: drawn without per-object motion.
    let mut cut = IdMap::new();
    if rs.interp.drawn.is_some() && (rs.scene.look.post.motion_blur.is_some() || super::fx::taa(rs))
    {
        for &(e, _) in keys {
            if !cut.contains_key(&e) && h.cut(e) {
                cut.insert(e, ());
            }
        }
    }
    super::fx::motion(rs, keys, tick, &cut);
    // Layer masks, secondary cameras and custom post passes.
    super::views::extract(world, rs, keys, &mut h);
    // Which entity each instance came from, for the probe's id pass.
    rs.ids.instances.clear();
    rs.ids.instances.extend(keys.iter().map(|k| k.0));
    // Drop shadows over the terrain under their sprites.
    super::terrain::ground_drop_shadows(rs);

    // Each video's texture shows its player's frame.
    super::video::extract(world, rs);
    // The UI, over everything else, in the overlay.
    // Anchored UI moved with its entity's drawn place (`super::interpolate`).
    let shifts = super::interpolate::ui_shifts(rs, &mut h, camera.as_deref());
    super::modules::emit(rs, &shifts);
    // The inspector's panel over everything (development builds).
    #[cfg(not(feature = "publish"))]
    crate::author::inspector::emit(rs);
    super::interpolate::end(rs, &mut h);
    rebase(&mut rs.scene, origin, look_along);
    // The frame's zero is the grid point, in the absolute world.
    rs.scene.frame.origin += zero.as_vec3();
    rs.zero = zero.as_vec3();
    rs.hierarchy = h;
}
