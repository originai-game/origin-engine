//! Per-frame scene extraction and particle colliders.

use super::*;

/// The render target's size, (1, 1) without a renderer.
fn backend_size(rs: &RenderState) -> (u32, u32) {
    rs.backend.as_ref().map_or((1, 1), |b| b.size())
}

/// Rebuild the per-frame part of `rs.scene` from the world as of its tick.
// Rendering only: the drawn cameras' field of view.
#[allow(clippy::disallowed_methods)]
pub(crate) fn extract(world: &World, rs: &mut RenderState) {
    let mut h = std::mem::take(&mut rs.hierarchy);
    h.refresh(world);
    // The art contract this frame (nothing without project.ogt's `look`).
    super::art::frame(world, rs);
    // A review camera frames what the last frame drew for its entity, so
    // the models below pick their levels for it (once more at the end).
    let ids = std::mem::take(&mut rs.ids.instances);
    let framed = super::review::bounds(rs, &h, &ids);
    rs.ids.instances = ids;
    let size = backend_size(rs);
    let camera = resource(world, rs.camera.as_ref());
    let settings = resource(world, rs.settings.as_ref());
    // A display frame between ticks: every world matrix and the camera
    // from here on are blended with the tick before (`super::interpolate`).
    super::interpolate::begin(world, rs, settings.as_deref(), &mut h);
    let layouts = &rs.draw;
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
    scene.sprite_outlines.clear();
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
    scene.text_facing.clear();
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
        let (position, rot) = super::interpolate::camera_pose(c);
        let (position, rot) = super::interpolate::camera(&rs.interp, position, rot);
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
        let radius = f(s, "bloomRadius", -1.0);
        scene.post.bloom_unreal = (radius >= 0.0).then(|| radius.min(1.0));
        scene.post.bloom_contrast = f(s, "bloomContrast", 1.0).clamp(0.5, 2.0);
        scene.post.tonemap = if on(s, "toneMapped", true) {
            match s["toneMapping"].as_u64().or_else(|| {
                s["toneMapping"]
                    .as_str()
                    .and_then(|n| {
                        ["aces", "agx", "none", "neutral"]
                            .iter()
                            .position(|v| *v == n)
                    })
                    .map(|i| i as u64)
            }) {
                Some(1) => og_render::Tonemap::Agx,
                Some(2) => og_render::Tonemap::None,
                Some(3) => og_render::Tonemap::Neutral,
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
            ao_resolution: [
                None,
                Some(og_render::AoResolution::Full),
                Some(og_render::AoResolution::Half),
            ][crate::fx::variant(&s["aoResolution"], AO_RESOLUTION_SETTINGS).min(2)],
            ssr_step_scale: f(s, "ssrStepScale", full.ssr_step_scale),
            volumetric_step_scale: f(s, "volumetricStepScale", full.volumetric_step_scale),
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
            let variant = slot("variant");
            let fx_models = &rs.fx.models;
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
                    let mut parts = lightmapped.or_else(|| models.get(&asset))?.clone();
                    // KHR_materials_variants: the variant's material on every
                    // part (the parts the model's own; -1 keeps the glTF's).
                    let v = v.num(&variant).unwrap_or(-1.0);
                    if v >= 0.0
                        && let Some(mats) = fx_models
                            .get(&asset)
                            .and_then(|i| i.part_variants.get(v as usize))
                            .filter(|m| m.len() == parts.len())
                    {
                        for (p, &m) in parts.iter_mut().zip(mats) {
                            p.1 = m;
                        }
                    }
                    Some((e, asset, parts, look, bias))
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
    let mut drawn = IdMap::new();
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
        // Drawn without a pose: a part whose node the glTF hides at rest
        // (KHR_node_visibility, a `.vox` frame after 0) is not drawn, when
        // the parts are the model's own (not a bake's).
        let shown = rs
            .fx
            .models
            .get(&asset)
            .map(|i| &i.part_shown)
            .filter(|s| s.len() == parts.len());
        for (k, &(mesh, material, local)) in parts.iter().enumerate() {
            if shown.is_some_and(|s| !s[k]) {
                continue;
            }
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
    // The environment's sky is the built-in one each frame until the
    // directional light (its sun) and og.Sky or og.DayNight (atmosphere.rs)
    // set it: kept from the last frame, a Sky turned off went on lighting
    // the scene and filling its background.
    scene.environment.sky = og_render::ProceduralSky::default();
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
        light.shadow_box = f(&v, "shadowBox", 0.0);
        light.shadow_focus = f(&v, "shadowFocus", 0.0);
        light.shadow_focus_drop = f(&v, "shadowFocusDrop", 0.0);
        light.shadow_bias = f(&v, "shadowBias", 0.0);
        light.shadow_normal_bias = f(&v, "shadowNormalBias", 0.0);
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
                if t.facing != og_render::SpriteFacing::None {
                    scene.text_facing.push(og_render::TextFacing {
                        text: scene.texts.len() as u32,
                        facing: t.facing,
                        align: t.align,
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
    super::interpolate::end(rs, &mut h);
    rs.hierarchy = h;
}

/// The colliders CPU-simulated particles bounce off
/// (`RenderScene::particle_colliders`): every og.Collider that is a box, a
/// sphere or a terrain's height field, placed as physics places it, while
/// a particle system of the frame has collision. The compute path reads the
/// depth buffer instead; `ParticleSimulation::Cpu` and sprite-mode
/// emitters read these, so their sparks bounce on the ground and on solid
/// shapes too.
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
