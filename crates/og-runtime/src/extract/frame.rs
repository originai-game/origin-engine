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
    // Timelines' material tracks, at their time (presentation only).
    crate::timeline::present(world, rs);
    // The art contract this frame (nothing without project.ogt's `look`).
    super::art::frame(world, rs);
    // A review camera frames what the last frame drew for its entity, so
    // the models below pick their levels for it (once more at the end).
    let ids = std::mem::take(&mut rs.ids.instances);
    // The last frame was drawn from its render origin; this one is built
    // in the world and moved to its own at the end (`rebase`).
    // In the absolute world (R12's og.WorldOrigin moves the world's zero
    // there); the world's place is it less the zero.
    let drawn_at = std::mem::take(&mut rs.scene.origin);
    let zero = crate::origin::zero(world, rs.draw.origin.as_ref());
    let back = drawn_at - zero.as_vec3();
    let framed = super::review::bounds(rs, &h, &ids).map(|(a, b)| (a + back, b + back));
    rs.ids.instances = ids;
    let size = backend_size(rs);
    let camera = resource(world, rs.camera.as_ref());
    let settings = resource(world, rs.settings.as_ref());
    // A display frame between ticks: every world matrix and the camera
    // from here on are blended with the tick before (`super::interpolate`).
    super::interpolate::begin(world, rs, settings.as_deref(), &mut h);
    let blended = rs.interp.drawn;
    let layouts = &rs.draw;
    rs.atmo.palettes.clear();
    rs.ids.sprites.clear();
    let scene = &mut rs.scene;
    let (models, geometries, materials) = (&rs.models, &rs.geometries, &rs.materials);
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
    scene.tick = time.tick;
    scene.tick_rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    // The sea's clock between ticks (og_render's `RenderScene::interpolation`).
    scene.interpolation = blended;
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
    scene.camera_cut = false;
    let (mut origin, mut look_along) = (Vec3::ZERO, None);
    if let Some(c) = &camera {
        let (position, rot) = super::interpolate::camera_pose(c);
        let (position, rot, cut) = super::interpolate::camera(&rs.interp, position, rot);
        scene.camera_cut = cut;
        let cam = &mut scene.camera;
        cam.position = position;
        cam.target = position + rot * Vec3::NEG_Z;
        cam.up = rot * Vec3::Y;
        origin = render_origin(position, zero);
        look_along = Some(rot * Vec3::NEG_Z);
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
        scene.lighting.environment.background =
            og_render::Background::Color(color(&c["background"], 0.0));
    }
    super::review::apply(&mut scene.camera, rs.review.as_ref(), size, framed);

    // Settings: environment, sky, bloom, exposure, tone mapping.
    if let Some(s) = &settings {
        // RenderSettings.exposure scales every camera's, as three.js
        // toneMappingExposure does the whole frame's.
        exposure *= f(s, "exposure", 1.0).max(0.0);
        scene.lighting.environment.intensity = f(s, "environmentIntensity", 0.55);
        if on(s, "sky", false) {
            scene.lighting.environment.background = og_render::Background::Sky;
        }
        scene.look.post.bloom = on(s, "bloom", false).then(|| og_render::Bloom {
            threshold: f(s, "bloomThreshold", 1.0),
            knee: f(s, "bloomKnee", 0.5),
            intensity: f(s, "bloomIntensity", 0.6),
        });
        let radius = f(s, "bloomRadius", -1.0);
        scene.look.post.bloom_unreal = (radius >= 0.0).then(|| radius.min(1.0));
        scene.look.post.bloom_contrast = f(s, "bloomContrast", 1.0).clamp(0.5, 2.0);
        scene.look.post.tonemap = if on(s, "toneMapped", true) {
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
            shadow_cascades: f(s, "shadowCascades", full.shadow_cascades as f32).max(1.0) as u32,
            shadow_map_scale: f(s, "shadowMapScale", full.shadow_map_scale),
            volumetric_fog: on(s, "volumetricFog", full.volumetric_fog),
            soft_particles: on(s, "softParticles", full.soft_particles),
            ssr: on(s, "screenSpaceReflections", full.ssr),
            ambient_occlusion: on(s, "ambientOcclusion", full.ambient_occlusion),
            // On the lazy og.GlobalIllumination's `allowed` (crate::fx).
            global_illumination: crate::fx::gi_allowed(
                crate::fx::resource(world, layouts.fx.global_illumination.as_ref()).as_deref(),
                rs.fx.gi_preset,
            ),
            ao_resolution: [
                None,
                Some(og_render::AoResolution::Full),
                Some(og_render::AoResolution::Half),
            ][crate::fx::variant(&s["aoResolution"], AO_RESOLUTION_SETTINGS).min(2)],
            ssr_step_scale: f(s, "ssrStepScale", full.ssr_step_scale),
            volumetric_step_scale: f(s, "volumetricStepScale", full.volumetric_step_scale),
            msaa_samples: f(s, "msaaSamples", full.msaa_samples as f32).max(1.0) as u32,
            upscaler: [og_render::Upscaler::Taau, og_render::Upscaler::Fsr3]
                [crate::fx::variant(&s["upscaler"], super::UPSCALERS).min(1)],
        };
    } else {
        scene.camera.exposure = og_render::Exposure::linear(exposure);
        scene.quality = og_render::RenderQuality {
            global_illumination: crate::fx::gi_allowed(
                crate::fx::resource(world, layouts.fx.global_illumination.as_ref()).as_deref(),
                rs.fx.gi_preset,
            ),
            ..og_render::RenderQuality::default()
        };
    }

    // Baked lighting (RenderSettings.bakedLighting, on by default).
    let baked = settings
        .as_ref()
        .is_none_or(|s| on(s, "bakedLighting", true));
    super::baked::apply(world, &mut rs.scene, &mut rs.baked, baked);
    // Models: posed by og-anim when the entity has a Skeleton.
    let posed = super::fx::skeletons(world, rs.draw.fx.skeleton.as_ref(), &mut rs.fx);
    // GPU crowds (og.CrowdAnimation): the world's time of this frame,
    // between ticks on a frame drawn between them.
    let crowds = super::fx::crowds(world, rs);
    let crowd_time = |alpha: Option<f32>| {
        let t = world.time();
        let ticks = match alpha {
            Some(a) => t.tick as f64 - 1.0 + f64::from(a),
            None => t.tick as f64,
        };
        ticks * f64::from(t.dt)
    };
    let mut keys: Vec<(u64, u32)> = Vec::new();
    // Each model's own tint and flash ride in its instances' records
    // (kestrel-stair #11: a glTF's materials are shared by every instance),
    // with its shadow flags (on unless the model says otherwise). A hidden
    // model draws nothing (meridian-loop #14). The rows are read in place
    // by offset, the fields resolved once a frame: decoding each row to
    // JSON was about a third of Lantern Vale's extraction
    // (the former test sample
    let model_parts: Vec<(u64, u64, Parts, super::fx::ModelLook, f32, f32)> = match &layouts.model {
        Some(layout) => {
            let slot = |key| FieldSlot::of(layout, key);
            let (visible, lod_bias, asset) = (slot("visible"), slot("lodBias"), slot("asset"));
            let (flash, flash_color, tint) = (slot("flash"), slot("flashColor"), slot("tint"));
            let (cast, receive) = (slot("castShadow"), slot("receiveShadow"));
            let is_static = slot("static");
            let variant = slot("variant");
            let sway = slot("sway");
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
                            outline: 0,
                        },
                        cast: on(&cast, true),
                        receive: on(&receive, true),
                    };
                    // A static model with a bake draws its lightmapped parts.
                    let lightmapped = (baked && on(&is_static, false))
                        .then(|| rs.baked.overrides.get(&e))
                        .flatten();
                    // A lightmapped model keeps still: its light was baked where it stands.
                    let sway = match lightmapped {
                        Some(_) => 0.0,
                        None => num(&sway, 0.0).max(0.0),
                    };
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
                    Some((e, asset, parts, look, bias, sway))
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
    // The clustered instances' levels are picked by the renderer, with the
    // settings' bias (a model's own lodBias applies to the others only).
    rs.scene.objects.lod_bias = settings_bias;
    rs.scene.objects.lod_debug = debug_lod;
    let view_height = match rs.backend.as_ref().map(|b| b.size()) {
        Some((_, ht)) if ht > 0 => ht as f32,
        _ => super::fx::viewport(rs).1,
    };
    let model_lods = std::mem::take(&mut rs.model_lods);
    let last_drawn = std::mem::take(&mut rs.lod_drawn);
    let mut drawn = IdMap::new();
    // The meshlet path draws the level picked here too (one per model, with
    // hysteresis), so both paths draw the same picture.
    rs.scene.objects.instance_levels.clear();
    rs.scene.objects.debris.clear();
    rs.scene.objects.impostors.clear();
    // A fractured destructible's model is drawn by its chunks (og.Chunk):
    // each its node of the model at its own transform, with the model's look.
    let fractured: IdMap<u64, crate::destruction::Drawn> =
        crate::destruction::drawn(layouts.engine)
            .into_iter()
            .map(|d| (d.entity, d))
            .collect();
    let (chunked, model_parts): (Vec<_>, Vec<_>) = model_parts
        .into_iter()
        .partition(|m| fractured.contains_key(&m.0));
    for (e, asset, parts, look, _, _) in chunked {
        let nodes = rs
            .fx
            .models
            .get(&asset)
            .map(|i| i.part_nodes.clone())
            .unwrap_or_default();
        for &(chunk, node, centre, outline) in &fractured[&e].chunks {
            let w = h.world_of(chunk) * Mat4::from_translation(-centre);
            for (k, &(mesh, material, local)) in parts.iter().enumerate() {
                if nodes.get(k) != Some(&(node as usize)) {
                    continue;
                }
                if rs.scene.meshlet_drawable(mesh, material) {
                    let at = rs.scene.objects.instances.len();
                    rs.scene.objects.instance_levels.resize(at + 1, u8::MAX);
                    rs.scene.objects.instance_levels[at] = 0;
                }
                rs.scene.objects.instances.push(og_render::Instance {
                    mesh,
                    material,
                    transform: w * local,
                    material_layer: None,
                    fx: og_render::InstanceFx { outline, ..look.fx },
                    cast_shadows: look.cast,
                    receive_shadows: look.receive,
                    sway: 0.0,
                });
                keys.push((chunk, k as u32));
            }
        }
        // Its fine pieces: instances the renderer's debris layer moves,
        // given where each broke off (og_render::DebrisPiece).
        for d in &fractured[&e].debris {
            let (position, rotation, scale) = d.pose;
            let body = Mat4::from_scale_rotation_translation(scale, rotation, position);
            let mut drawn_by = Vec::new();
            for (k, &(mesh, material, local)) in parts.iter().enumerate() {
                if nodes.get(k) != Some(&(d.node as usize)) {
                    continue;
                }
                let to_mesh = Mat4::from_translation(-d.centre) * local;
                if rs.scene.meshlet_drawable(mesh, material) {
                    let at = rs.scene.objects.instances.len();
                    rs.scene.objects.instance_levels.resize(at + 1, u8::MAX);
                    rs.scene.objects.instance_levels[at] = 0;
                }
                drawn_by.push((rs.scene.objects.instances.len() as u32, to_mesh));
                rs.scene.objects.instances.push(og_render::Instance {
                    mesh,
                    material,
                    transform: body * to_mesh,
                    material_layer: None,
                    fx: look.fx,
                    cast_shadows: look.cast,
                    receive_shadows: look.receive,
                    sway: 0.0,
                });
                // Keyed by the piece, so its previous transform is its own.
                keys.push((d.id, k as u32));
            }
            let piece = og_render::DebrisPiece {
                id: d.id,
                tick: d.tick,
                position,
                rotation,
                scale,
                velocity: d.velocity,
                angular_velocity: d.angular,
                points: d.points.clone(),
                friction: 0.6,
                restitution: 0.15,
                reach: DEBRIS_REACH,
                instances: drawn_by,
            };
            // Its node's materials in the cook's order: the cut faces' last.
            let materials: Vec<_> = parts
                .iter()
                .enumerate()
                .filter(|(k, _)| nodes.get(*k) == Some(&(d.node as usize)))
                .map(|(_, p)| p.1)
                .collect();
            throw_chips(rs, &piece, &materials, &look, &mut keys);
            rs.scene.objects.debris.push(piece);
        }
        // The grit its broken joints shed.
        let drawn = &fractured[&e];
        let mortar = rs
            .fx
            .model_materials
            .get(&asset)
            .and_then(|m| m.get(drawn.mortar as usize))
            .copied();
        for c in &drawn.crumbs {
            // The clay: the cut faces of the piece on the joint's first side.
            let inner = parts
                .iter()
                .enumerate()
                .rev()
                .find(|(k, _)| nodes.get(*k) == Some(&(c.node as usize)))
                .map(|(_, p)| p.1);
            if let Some(inner) = inner {
                throw_crumb(rs, c, inner, mortar, &look, &mut keys);
            }
        }
    }
    for (e, asset, mut parts, mut look, bias, sway) in model_parts {
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
        // At its impostor's level a model at rest is one card (og-render's
        // `impostor`), drawn with the scattered copies' passes.
        if og_render::WORLD_3D
            && let Some(l) = model_lods.get(&asset)
            && let Some((impostor, _)) = l.impostor
            && l.impostor_level() == Some(level)
            && !posed.contains(&e)
        {
            rs.scene.objects.impostors.push(og_render::ImpostorDraw {
                impostor,
                transform: m,
                cast_shadows: look.cast,
            });
            continue;
        }
        // A swaying model's parts draw with their materials' wind copies.
        if sway > 0.0
            && let Some(&top) = rs.wind.tops.get(&asset)
        {
            for p in parts.iter_mut() {
                p.1 = rs.wind.material(&mut rs.scene, p.1, sway, top);
            }
        }
        if posed.contains(&e)
            && super::fx::posed_model(rs, e, asset, m, h.blend_alpha(e), &parts, look, &mut keys)
        {
            continue;
        }
        if let Some(crowd) = crowds.get(&e) {
            let seconds = crowd_time(h.blend_alpha(e));
            if super::fx::crowd_model(rs, e, asset, m, seconds, crowd, &parts, look, &mut keys) {
                continue;
            }
        }
        let levels = model_lods.get(&asset).map(|l| &l.parts);
        // Its meshes never draw past their coarsest: a posed model (or one
        // in the 2D profile) at its impostor's level draws that.
        let level = match model_lods.get(&asset).and_then(|l| l.impostor_level()) {
            Some(i) => level.min(i - 1),
            None => level,
        };
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
            // One the meshlet path draws keeps level 0's mesh and is told
            // the level (`RenderScene::instance_levels`): its clusters
            // hold every level, the same as the part's meshes.
            let meshlet = rs.scene.meshlet_drawable(mesh, material);
            if meshlet {
                let at = rs.scene.objects.instances.len();
                rs.scene.objects.instance_levels.resize(at + 1, u8::MAX);
                rs.scene.objects.instance_levels[at] = level as u8;
            }
            let mesh = match (level, levels.and_then(|l| l.get(k))) {
                (0, _) | (_, None) => mesh,
                _ if meshlet => mesh,
                (_, Some(p)) if p.is_empty() => mesh,
                (l, Some(p)) => p[(l - 1).min(p.len() - 1)].0,
            };
            rs.scene.objects.instances.push(og_render::Instance {
                mesh,
                material,
                transform: m * local,
                material_layer: None,
                fx: look.fx,
                cast_shadows: look.cast,
                receive_shadows: look.receive,
                sway: 0.0,
            });
            keys.push((e, k as u32));
        }
    }
    rs.model_lods = model_lods;
    rs.lod_drawn = drawn;
    let scene = &mut rs.scene;
    // Meshes.
    {
        scene.objects.instances.reserve(mesh_rows.len());
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
            scene.objects.instances.push(og_render::Instance {
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
                sway: 0.0,
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
    scene.lighting.environment.sky = og_render::ProceduralSky::default();
    if let Some(dl) = &layouts.directional
        && let Some((e, v)) = rows(world, dl).into_iter().next()
    {
        let m = h.world_of(e);
        let light = &mut scene.lighting.light;
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
        scene.lighting.environment.sky.sun_direction = light.direction_to_light.normalize_or_zero();
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
    scene.lighting.light_cookies = locals
        .iter()
        .enumerate()
        .filter_map(|(i, (_, _, c))| c.map(|texture| og_render::LightCookie { light: i, texture }))
        .collect();
    scene.lighting.lights = locals.iter().map(|(_, l, _)| *l).collect();
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
    crate::vfx::extract(world, rs, &mut h, tick);
    rs.scene.effects.particle_history = rs.particle_history.clone();
    // The pixel grid's snap: after the sprites and the camera are in.
    super::fx::snap_to_grid(rs);
    // Cloth meshes are instances too: keyed for per-object motion.
    keys.extend(super::atmosphere::extract(world, rs, &mut h));
    // The wind copies' parameters, from the og.Wind just read.
    let (wind, tick, rate) = (rs.scene.atmosphere.wind, rs.scene.tick, rs.scene.tick_rate);
    rs.wind.present(&mut rs.scene, wind, tick, rate);
    super::environment::extract(world, rs);
    super::probes::extract(world, rs, &mut h);
    super::probes::extract_light_volumes(world, rs, &mut h);
    // Scattered copies are instances too, keyed by their terrain cell.
    keys.extend(super::terrain::extract(world, rs, &mut h));
    colliders(world, rs, &mut h);
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
    crate::destruction2d::extract(world, rs, &mut h);
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
    // Teleported into this frame: drawn without per-object motion.
    let mut cut = IdMap::new();
    if rs.interp.drawn.is_some() && (rs.scene.look.post.motion_blur.is_some() || super::fx::taa(rs))
    {
        for &(e, _) in &keys {
            if !cut.contains_key(&e) && h.cut(e) {
                cut.insert(e, ());
            }
        }
    }
    super::fx::motion(rs, &keys, tick, &cut);
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
    // Anchored UI moved with its entity's drawn place (`super::interpolate`).
    let shifts = super::interpolate::ui_shifts(rs, &mut h, camera.as_deref());
    super::modules::emit(rs, &shifts);
    // The inspector's panel over everything (development builds).
    #[cfg(not(feature = "publish"))]
    crate::inspector::emit(rs);
    super::interpolate::end(rs, &mut h);
    rebase(&mut rs.scene, origin, look_along);
    // The frame's zero is the grid point, in the absolute world.
    rs.scene.origin += zero.as_vec3();
    rs.zero = zero.as_vec3();
    rs.hierarchy = h;
}

/// The render origin's grid (m): a power of two, so moving a position near
/// a grid point to it is exact.
pub(crate) const RENDER_GRID: f32 = 8192.0;

/// The render origin for a camera at `position` in a world whose zero
/// og.WorldOrigin puts at `zero` of the absolute world (R12; zero without
/// one): the nearest point of the [`RENDER_GRID`] laid in the absolute
/// world, in the world's coordinates. The frame is drawn within 4096 m of
/// the camera on every axis; a shift of the world's zero moves the point
/// in the world's coordinates by the shift, so it stays where it was in the
/// absolute world and nothing drawn moves; and a scene without
/// og.WorldOrigin whose camera stays within 4096 m of the zero is drawn at
/// it, exactly as before render origins.
pub(crate) fn render_origin(position: Vec3, zero: glam::DVec3) -> Vec3 {
    if !position.is_finite() || !zero.is_finite() {
        return Vec3::ZERO;
    }
    let g = f64::from(RENDER_GRID);
    let at = position.as_dvec3() + zero;
    ((at / g).round() * g - zero).as_vec3()
}

/// Moves the frame, built in the world, to its render origin
/// (`og_render::RenderScene::rebase`): the GPU then works on positions
/// within a few kilometres of the camera, so nothing jitters however far
/// from the world's zero it is. The camera looks along `forward` (from its
/// rotation, exact) rather than at the target `position + forward`, which
/// was rounded at the camera's distance from the world's zero, unless
/// something since turned it (the review camera).
fn rebase(scene: &mut og_render::RenderScene, origin: Vec3, forward: Option<Vec3>) {
    if origin == Vec3::ZERO {
        return;
    }
    let cam = scene.camera;
    let look = cam.target - cam.position;
    // What rounding position + forward at this distance can do to it.
    let rounding = 4.0 * cam.position.abs().max_element() * f32::EPSILON + 1e-4;
    scene.rebase(origin);
    if let Some(f) = forward
        && (look - f).length() <= rounding
    {
        scene.camera.target = scene.camera.position + f;
    }
}

/// How far (m) a fine piece may fly from where it broke off before it is
/// no longer drawn (og_render::DebrisPiece::reach): its culling sphere.
const DEBRIS_REACH: f32 = 12.0;

/// A stream of 0..1 numbers from a seed (splitmix64): a debris chip's
/// every number, the same each frame.
struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A direction, evenly over the sphere.
    fn unit(&mut self) -> Vec3 {
        let z = self.next() * 2.0 - 1.0;
        let a = self.next() * std::f32::consts::TAU;
        let r = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(r * og_core::math::cos(a), z, r * og_core::math::sin(a))
    }
}

/// One rubble chip (og_render::chip) as the debris layer throws it.
struct Chip {
    id: u64,
    tick: u64,
    position: Vec3,
    /// Its size along its own axes (m): a chip is about 1 across.
    scale: Vec3,
    velocity: Vec3,
    angular: Vec3,
    material: og_render::MaterialHandle,
    reach: f32,
}

/// Draws `chip` as an instance the debris layer moves: one of the chip
/// shapes (made at the first) picked and turned by `seed`. False once the
/// layer is full.
fn push_chip(
    rs: &mut RenderState,
    chip: Chip,
    turn: Quat,
    seed: &mut Seeded,
    look: &super::fx::ModelLook,
    keys: &mut Vec<(u64, u32)>,
) -> bool {
    if rs.scene.objects.debris.len() + 1 >= og_render::MAX_DEBRIS {
        return false;
    }
    if rs.fx.chip_meshes.is_empty() {
        rs.fx.chip_meshes = (0..og_render::CHIP_VARIANTS)
            .map(|v| {
                let m = og_render::chip(v);
                // Its hull's points: the mesh's corners, each once.
                let mut points: Vec<Vec3> = Vec::new();
                for p in m.vertices.iter().map(|x| Vec3::from(x.position)) {
                    if !points.contains(&p) {
                        points.push(p);
                    }
                }
                (rs.scene.add_mesh(m), points)
            })
            .collect();
    }
    let variant = (seed.next() * og_render::CHIP_VARIANTS as f32) as u32 % og_render::CHIP_VARIANTS;
    let (mesh, points) = rs.fx.chip_meshes[variant as usize].clone();
    let axis = seed.unit();
    let rotation =
        og_core::math::quat_from_axis_angle(axis, seed.next() * std::f32::consts::TAU) * turn;
    let body = Mat4::from_scale_rotation_translation(chip.scale, rotation, chip.position);
    if rs.scene.meshlet_drawable(mesh, chip.material) {
        let at = rs.scene.objects.instances.len();
        rs.scene.objects.instance_levels.resize(at + 1, u8::MAX);
        rs.scene.objects.instance_levels[at] = 0;
    }
    let index = rs.scene.objects.instances.len() as u32;
    rs.scene.objects.instances.push(og_render::Instance {
        mesh,
        material: chip.material,
        transform: body,
        material_layer: None,
        fx: look.fx,
        cast_shadows: look.cast,
        receive_shadows: look.receive,
        sway: 0.0,
    });
    keys.push((chip.id, 0));
    rs.scene.objects.debris.push(og_render::DebrisPiece {
        id: chip.id,
        tick: chip.tick,
        position: chip.position,
        rotation,
        scale: chip.scale,
        velocity: chip.velocity,
        angular_velocity: chip.angular,
        points,
        friction: 0.7,
        restitution: 0.2,
        reach: chip.reach,
        instances: vec![(index, Mat4::IDENTITY)],
    });
    true
}

/// A chip's id: its source's, mixed with which of its chips it is.
fn chip_id(source: u64, k: u32) -> u64 {
    source ^ (u64::from(k) + 1).wrapping_mul(0xd6e8_feb8_6659_fd93)
}

/// The rubble chips a fine piece throws as it breaks off: four to six
/// lumps (og_render::chip) of its own cut faces' material, one in four of
/// its outer face's, each a third to two thirds of the piece across with
/// most of them small, at least half as thick as wide, flung from its
/// volume and tumbling. Visual only and the same each frame: every
/// number is the piece's id hashed, so a restored world or a renderer
/// made late throws the same chips, and the debris layer catches them up
/// from the tick the piece broke off as it does the piece.
fn throw_chips(
    rs: &mut RenderState,
    piece: &og_render::DebrisPiece,
    materials: &[og_render::MaterialHandle],
    look: &super::fx::ModelLook,
    keys: &mut Vec<(u64, u32)>,
) {
    let (Some(&inner), Some(&outer)) = (materials.last(), materials.first()) else {
        return;
    };
    let radius = piece
        .points
        .iter()
        .map(|p| (*p * piece.scale).length())
        .fold(0.0, f32::max);
    let mut r = Seeded(piece.id);
    let count = 4 + (r.next() * 3.0) as u32;
    for k in 0..count {
        let material = if materials.len() > 1 && r.next() < 0.25 {
            outer
        } else {
            inner
        };
        // Most chips small, a few near the piece's size.
        let u = r.next();
        let size = radius * (0.3 + 0.5 * u * u);
        let scale = Vec3::new(
            size,
            size * (0.65 + 0.35 * r.next()),
            size * (0.5 + 0.35 * r.next()),
        );
        let offset = r.unit() * radius * 0.5 * r.next().sqrt();
        let fling = r.unit() * (0.5 + 1.3 * r.next());
        let spin = r.unit() * (4.0 + 10.0 * r.next());
        let chip = Chip {
            id: chip_id(piece.id, k),
            tick: piece.tick,
            position: piece.position + offset,
            scale,
            velocity: piece.velocity + piece.angular_velocity.cross(offset) + fling,
            angular: piece.angular_velocity + spin,
            material,
            reach: piece.reach,
        };
        if !push_chip(rs, chip, piece.rotation, &mut r, look, keys) {
            return;
        }
    }
}

/// The grit a broken joint sheds: three or four small chips falling out of
/// the crack, one of the mortar where the prop has one
/// (`fracture.mortar`), the rest of its cut faces' clay (`inner`), from
/// one centimetre to five and a half across, knocked out along the joint's normal
/// either way and moving with the blow that broke it. Seeded by the
/// joint as [`throw_chips`] is by its piece.
fn throw_crumb(
    rs: &mut RenderState,
    c: &crate::destruction::DrawnCrumb,
    inner: og_render::MaterialHandle,
    mortar: Option<og_render::MaterialHandle>,
    look: &super::fx::ModelLook,
    keys: &mut Vec<(u64, u32)>,
) {
    let mut r = Seeded(c.id);
    let count = 3 + (r.next() * 2.0) as u32;
    let n = c.normal.normalize_or(Vec3::Y);
    for k in 0..count {
        let (material, size) = match mortar {
            Some(m) if k == 0 => (m, 0.012 + 0.016 * r.next()),
            _ => (inner, 0.02 + 0.035 * r.next() * r.next()),
        };
        let scale = Vec3::new(
            size,
            size * (0.6 + 0.4 * r.next()),
            size * (0.5 + 0.4 * r.next()),
        );
        let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
        // Anywhere on the joint's face, most near its rim (where the crack
        // shows): its centre is inside the prop, and a chip there would
        // ride hidden in the bricks.
        let along = r.unit();
        let flat = (along - n * along.dot(n)).normalize_or(n.any_orthonormal_vector());
        let offset = flat * c.half * (1.0 - 0.6 * r.next() * r.next()) + n * side * 0.01;
        let chip = Chip {
            id: chip_id(c.id, k),
            tick: c.tick,
            position: c.point + offset,
            scale,
            velocity: c.velocity
                + n * side * (0.3 + 0.9 * r.next())
                + flat * (0.4 + 0.8 * r.next()),
            angular: r.unit() * (6.0 + 14.0 * r.next()),
            material,
            reach: DEBRIS_REACH,
        };
        if !push_chip(rs, chip, Quat::IDENTITY, &mut r, look, keys) {
            return;
        }
    }
}

/// The colliders CPU-simulated particles bounce off, visual bodies meet and
/// the debris layer's pieces land on (`RenderScene::colliders`): every
/// og.Collider that is a box, a sphere, a capsule or a terrain's height
/// field, placed as physics places it, while the frame has debris, a
/// visual body or a particle system with collision. Compute particles
/// read these where their emitter names them (`collision ... colliders`)
/// and in sprite mode, the depth buffer otherwise; the CPU path always
/// reads these, so sparks bounce on the ground and on solid shapes.
fn colliders(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    rs.scene.world.colliders.clear();
    let colliding = !rs.scene.objects.debris.is_empty()
        || rs.scene.has_visual_bodies()
        || rs.scene.effects.particles.iter().any(|s| {
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
            0 => og_render::SceneCollider::Box {
                transform: m,
                half_extents: vec3(&v["size"]) * 0.5,
            },
            1 => {
                let scale = world_m.to_scale_rotation_translation().0.max_element();
                og_render::SceneCollider::Sphere {
                    center: m.transform_point3(Vec3::ZERO),
                    radius: f(&v, "radius", 0.5) * scale,
                }
            }
            2 => {
                let scale = world_m.to_scale_rotation_translation().0.max_element();
                let half = f(&v, "height", 1.0) * 0.5;
                og_render::SceneCollider::Capsule {
                    a: m.transform_point3(Vec3::new(0.0, -half, 0.0)),
                    b: m.transform_point3(Vec3::new(0.0, half, 0.0)),
                    radius: f(&v, "radius", 0.5) * scale,
                }
            }
            6 => {
                let geometry = v["mesh"].as_f64().unwrap_or(0.0) as u64;
                let Some(data) = super::terrain::field_of_geometry(&rs.terrain, geometry) else {
                    continue;
                };
                og_render::SceneCollider::HeightField { data, transform: m }
            }
            _ => continue,
        };
        rs.scene.world.colliders.push(collider);
    }
}
