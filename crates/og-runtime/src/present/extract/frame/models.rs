//! The frame's models and meshes.

use super::*;
use crate::present::extract::kept;

/// Baked lighting, then the models: a fractured destructible's by its
/// chunks, the rest at their level of detail, posed, as crowds, as
/// impostors or as their parts.
pub(super) fn models(
    world: &World,
    rs: &mut RenderState,
    h: &mut Hierarchy,
    b: &Begun,
    keys: &mut Vec<(u64, u32)>,
) {
    let settings = b.settings.as_ref();
    // The models fading in (`crate::present::reveal`): what appeared in
    // the world this tick left, unless the tick's own step saw it already,
    // and the frame's place among the ticks.
    crate::present::reveal::step(rs, world);
    let tick = world.time().tick as f64;
    let display = rs.interp.display;
    rs.reveal.frame(
        tick - h.frame_alpha().map_or(0.0, |a| 1.0 - f64::from(a)),
        display,
    );
    // Baked lighting (RenderSettings.bakedLighting, on by default).
    let baked = settings.is_none_or(|s| s.baked_lighting);
    super::baked::apply(world, &mut rs.scene, &mut rs.baked, baked);
    // The HLOD proxies lit by their models' bake draw with its pages too.
    if baked {
        let lit = rs.hlod.lit.values().flat_map(|l| l.meshes.iter().cloned());
        rs.scene.lighting.baked.meshes.extend(lit);
    }
    // The models' parts and their lightmapped ones, lent to the rows until
    // the end of this read: a row borrows its model's parts and copies them
    // only to change them (a variant's materials, wind copies), where every
    // row copied them each frame. Nothing below reads either through `rs`.
    let models = std::mem::take(&mut rs.models);
    let lightmapped_parts = std::mem::take(&mut rs.baked.overrides);
    let layouts = &rs.draw;
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
        ticks / f64::from(t.rate)
    };
    // Each model's own tint and flash ride in its instances' records
    // (kestrel-stair #11: a glTF's materials are shared by every instance),
    // with its shadow flags (on unless the model says otherwise). A hidden
    // model draws nothing (meridian-loop #14). The rows are read in place
    // by offset, the fields resolved once a frame: decoding each row to
    // JSON was about a third of Lantern Vale's extraction
    // (the former test sample
    // Each with its inputs, as the kept list compares them
    // (`crate::present::extract::kept`).
    type Row<'a> = (std::borrow::Cow<'a, Parts>, kept::RowIn);
    let model_rows = layouts
        .model
        .as_ref()
        .map(|layout| column_rows(world, layout));
    let model_parts: Vec<Row<'_>> = match (&layouts.model, &model_rows) {
        (Some(layout), Some(rows)) => {
            let slot = |key| FieldSlot::of(layout, key);
            let (visible, lod_bias, asset) = (slot("visible"), slot("lodBias"), slot("asset"));
            let (flash, flash_color, tint) = (slot("flash"), slot("flashColor"), slot("tint"));
            let (cast, receive) = (slot("castShadow"), slot("receiveShadow"));
            let is_static = slot("static");
            let variant = slot("variant");
            let sway = slot("sway");
            let fx_models = &rs.fx.models;
            rows.iter()
                .filter_map(|(e, v)| {
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
                    // Texture readiness is presentation only. Never expose
                    // a streamed model's untextured stand-in while its
                    // entities and colliders are already live in the world.
                    if rs.decoded.hidden_models.contains_key(&asset) {
                        return None;
                    }
                    let look = super::fx::ModelLook {
                        fx: og_render::InstanceFx {
                            flash: num(&flash, 0.0),
                            flash_color: rgb(&flash_color),
                            tint: rgb(&tint),
                            outline: 0,
                            reveal: 1.0,
                            fade_out: false,
                        },
                        cast: on(&cast, true),
                        receive: on(&receive, true),
                    };
                    // A static model with a bake draws its lightmapped parts.
                    let lightmapped = (baked && on(&is_static, false))
                        .then(|| lightmapped_parts.get(&e))
                        .flatten();
                    // A lightmapped model keeps still: its light was baked where it stands.
                    let sway = match lightmapped {
                        Some(_) => 0.0,
                        None => num(&sway, 0.0).max(0.0),
                    };
                    let source = lightmapped.or_else(|| models.get(&asset))?;
                    let mut parts = std::borrow::Cow::Borrowed(source);
                    // KHR_materials_variants: the variant's material on every
                    // part (the parts the model's own; -1 keeps the glTF's).
                    let v = v.num(&variant).unwrap_or(-1.0);
                    if v >= 0.0
                        && let Some(mats) = fx_models
                            .get(&asset)
                            .and_then(|i| i.part_variants.get(v as usize))
                            .filter(|m| m.len() == parts.len())
                    {
                        for (p, &m) in parts.to_mut().iter_mut().zip(mats) {
                            p.1 = m;
                        }
                    }
                    let row = kept::RowIn {
                        entity: e,
                        asset,
                        source: source.as_ptr() as usize,
                        variant: v.to_bits(),
                        look,
                        bias,
                        sway,
                        matrix: Mat4::IDENTITY,
                        posed: false,
                        crowd: false,
                    };
                    Some((parts, row))
                })
                .collect()
        }
        _ => Vec::new(),
    };
    drop(model_rows);
    // Levels of detail: per entity, the coarsest level whose error stays
    // under a pixel at its distance (og_render::lod), kept near a boundary
    // by the level it drew last. `RenderOptions.debugView lod` colours each
    // by its level.
    let settings_bias = settings.map_or(1.0, |s| s.lod_bias.max(0.0));
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
    // A level cross-fade's length in ticks, and how far one started at a
    // frame time is at this frame's (as an HLOD node's, `hlod.rs`).
    let mut lod_fades = std::mem::take(&mut rs.lod_fades);
    lod_fades.retain(|e, _| last_drawn.contains_key(e));
    let now = rs.reveal.now();
    let lod_ticks =
        f64::from(crate::present::reveal::SECONDS) / f64::from(world.time().dt().max(1e-6));
    let lod_progress = |at: f64| ((now - at) / lod_ticks).clamp(0.0, 1.0) as f32;
    let mut drawn = IdMap::new();
    // The meshlet path draws the level picked here too (one per model, with
    // hysteresis), so both paths draw the same picture.
    rs.scene.objects.instance_levels.clear();
    rs.scene.objects.debris.clear();
    rs.scene.objects.impostors.clear();
    // A fractured destructible's model is drawn by its chunks (og.Chunk):
    // each its node of the model at its own transform, with the model's look.
    let fractured: IdMap<u64, crate::bind::destruction::Drawn> =
        if crate::engine::engine(layouts.engine).is_ok_and(|e| {
            e.modules
                .destruction
                .enabled
                .load(std::sync::atomic::Ordering::Relaxed)
        }) {
            crate::bind::destruction::drawn(layouts.engine)
        } else {
            Vec::new()
        }
        .into_iter()
        .map(|d| (d.entity, d))
        .collect();
    let (chunked, model_parts): (Vec<_>, Vec<_>) = model_parts
        .into_iter()
        .partition(|m| fractured.contains_key(&m.1.entity));
    for (parts, r) in chunked {
        let (e, asset, look) = (r.entity, r.asset, r.look);
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
            throw_chips(rs, &piece, &materials, &look, keys);
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
                throw_crumb(rs, c, inner, mortar, &look, keys);
            }
        }
    }
    // The rows' drawn places and paths, then the rows against the last
    // frame's (`crate::present::extract::kept`).
    let (parts_of, rows): (Vec<_>, Vec<_>) = model_parts
        .into_iter()
        .map(|(parts, mut r)| {
            r.matrix = h.world_of(r.entity);
            r.posed = posed.contains(&r.entity);
            r.crowd = crowds.contains_key(&r.entity);
            (parts, r)
        })
        .unzip();
    // The rows still fading in carry how much of them is drawn.
    let mut rows = rows;
    if rs.reveal.fading() {
        for r in &mut rows {
            r.look.fx.reveal = rs.reveal.entity(r.entity);
        }
    }
    let view = kept::ModelsView::of(rs, view_height, settings_bias, debug_lod, baked);
    let mut build = std::mem::take(&mut rs.kept_models).begin(view, rows);
    // HLOD (`crate::present::hlod`): the proxies of this frame, and the
    // models they stand in for; the last frame's when nothing changed.
    let hlod_inputs = crate::present::hlod::inputs(world, rs, h);
    let hlod_from = (rs.scene.objects.instances.len(), keys.len());
    let settled = rs.hlod.settled;
    let replaced = if build.copy_hlod(&mut rs.scene, keys, &hlod_inputs, settled) {
        std::mem::take(&mut rs.hlod.replaced)
    } else {
        // A row with a variant's materials holds its own copy of its parts.
        let rows = || {
            let plain = |(parts, r): (&std::borrow::Cow<'_, Parts>, &kept::RowIn)| {
                r.look.fx.flash == 0.0
                    && r.look.fx.tint == Vec3::ONE
                    && matches!(parts, std::borrow::Cow::Borrowed(_))
            };
            let rows = parts_of.iter().zip(build.rows()).map(|row| {
                let (asset, plain) = (row.1.asset, plain(row));
                (row.1.entity, crate::present::hlod::Row { asset, plain })
            });
            rows.collect()
        };
        let view = crate::present::hlod::View {
            models: &models,
            rows: &rows,
            view_height,
            bias: settings_bias,
            debug: debug_lod,
            baked,
        };
        crate::present::hlod::draw(world, rs, h, &view, keys)
    };
    build.hlod_drawn(hlod_inputs, hlod_from, &rs.scene, keys);
    // The models cross-fading with a proxy (`crate::present::hlod`): how
    // much of each is drawn, and whether it is the side fading out. Empty
    // when the proxies were copied (a frame with a fade never is).
    let crossing = std::mem::take(&mut rs.hlod.crossing);
    for (i, mut parts) in parts_of.into_iter().enumerate() {
        let r = build.rows()[i];
        let (e, asset, bias, sway, m) = (r.entity, r.asset, r.bias, r.sway, r.matrix);
        let mut look = r.look;
        let is_replaced = replaced.contains_key(&e);
        let crossed = crossing.get(&e).copied();
        if let Some((reveal, fade_out)) = crossed {
            look.fx.reveal = look.fx.reveal.min(reveal);
            look.fx.fade_out = fade_out;
        }
        build.start(&rs.scene, keys);
        if let Some(level) = crossed
            .is_none()
            .then(|| build.copy_row(i, is_replaced, &mut rs.scene, keys))
            .flatten()
        {
            if let Some(level) = level {
                drawn.insert(e, level);
            }
            continue;
        }
        if is_replaced {
            build.built(true, u8::MAX, true);
            continue;
        }
        let (level, settled) = match model_lods.get(&asset) {
            Some(l) => {
                let scale = m.to_scale_rotation_translation().0.abs().max_element();
                let per_unit = og_render::lod::pixels_per_unit(
                    &rs.scene.frame.camera,
                    m.transform_point3(l.center),
                    l.radius * scale,
                    view_height,
                );
                let previous = last_drawn.get(&e).map(|&x| x as usize);
                let level = og_render::lod::select(
                    &l.errors(),
                    per_unit * scale,
                    bias * settings_bias,
                    previous,
                );
                (level, previous == Some(level))
            }
            None => (0, true),
        };
        // Where its level turns, the level it leaves and the one it takes
        // are both drawn for a fade's length, dithered against each other
        // (common.wgsl `dither_cut`): the new one draws a share of the
        // pixels, the old one exactly the others, as an HLOD proxy and
        // its models do. A turn back during the fade reverses it from
        // where it is. A model fading in, cross-fading with a proxy,
        // posed or in a crowd switches at once.
        let fading = (look.fx.reveal >= 1.0
            && crossed.is_none()
            && !posed.contains(&e)
            && !r.crowd
            && !debug_lod)
            .then(|| {
                let turned = last_drawn.get(&e).copied().filter(|&p| p as usize != level);
                let fade = match (turned, lod_fades.get(&e).copied()) {
                    (Some(p), Some((from, start))) if from as usize == level => {
                        // Back towards the level it was leaving.
                        let shown = lod_progress(start);
                        (p, now - f64::from(1.0 - shown) * lod_ticks)
                    }
                    (Some(p), _) => (p, now),
                    (None, Some(f)) => f,
                    (None, None) => return None,
                };
                let t = lod_progress(fade.1);
                (t < 1.0).then(|| {
                    lod_fades.insert(e, fade);
                    (fade.0 as usize, t)
                })
            })
            .flatten();
        if fading.is_none() {
            lod_fades.remove(&e);
        }
        // Copied by the next frame when its level held here (the
        // hysteresis then keeps it), nothing animates it and it is not
        // cross-fading.
        let copyable = settled && !r.posed && !r.crowd && crossed.is_none() && fading.is_none();
        build.built(false, level as u8, copyable);
        drawn.insert(e, level as u8);
        if debug_lod {
            look.fx.flash = 0.8;
            look.fx.flash_color = og_render::lod::debug_color(level);
        }
        // The levels drawn: its own, and the one it fades out from.
        let mut levels = vec![(level, look)];
        if let Some((from, t)) = fading {
            levels[0].1.fx.reveal = t;
            let mut out = look;
            out.fx.reveal = 1.0 - t;
            out.fx.fade_out = true;
            levels.push((from, out));
        }
        // A swaying model's parts draw with their materials' wind copies.
        if sway > 0.0
            && let Some(&top) = rs.wind.tops.get(&asset)
        {
            for p in parts.to_mut().iter_mut() {
                p.1 = rs.wind.material(&mut rs.scene, p.1, sway, top);
            }
        }
        for (level, look) in levels {
            // At its impostor's level a model at rest is one card
            // (og-render's `impostor`), drawn with the scattered copies'
            // passes.
            if let Some(l) = model_lods.get(&asset)
                && let Some((impostor, _)) = l.impostor
                && l.impostor_level() == Some(level)
                && !posed.contains(&e)
            {
                rs.scene.objects.impostors.push(og_render::ImpostorDraw {
                    impostor,
                    transform: m,
                    cast_shadows: look.cast,
                    sway,
                    top: rs.wind.tops.get(&asset).copied().unwrap_or(0.0),
                    reveal: look.fx.reveal,
                    fade_out: look.fx.fade_out,
                    foliage: parts.iter().map(|p| (p.0, p.1)).find(|&(m, _)| {
                        rs.scene
                            .mesh_foliage(m)
                            .is_some_and(|f| f.season.season > 0.0)
                    }),
                });
                continue;
            }
            if posed.contains(&e)
                && super::fx::posed_model(rs, e, asset, m, h.blend_alpha(e), &parts, look, keys)
            {
                continue;
            }
            if let Some(crowd) = crowds.get(&e) {
                let seconds = crowd_time(h.blend_alpha(e));
                let lods = model_lods.get(&asset);
                // A crowd draws whole: its cards read the look's flash word as is.
                let mut look = look;
                look.fx.reveal = 1.0;
                if super::fx::crowd_model(rs, e, asset, m, seconds, crowd, &parts, lods, look, keys)
                {
                    continue;
                }
            }
            let part_levels = model_lods.get(&asset).map(|l| &l.parts);
            // Its meshes never draw past their coarsest: a posed model at
            // its impostor's level draws that.
            let level = match model_lods.get(&asset).and_then(|l| l.impostor_level()) {
                Some(i) => level.min(i - 1),
                None => level,
            };
            // Drawn without a pose: a part whose node the glTF hides at
            // rest (KHR_node_visibility, a `.vox` frame after 0) is not
            // drawn, when the parts are the model's own (not a bake's).
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
                let mut selected_material = model_lods
                    .get(&asset)
                    .map_or(material, |l| l.material(k, level, material));
                if selected_material != material
                    && sway > 0.0
                    && let Some(&top) = rs.wind.tops.get(&asset)
                {
                    selected_material =
                        rs.wind
                            .material(&mut rs.scene, selected_material, sway, top);
                }
                let material = selected_material;
                // A part with fewer levels than the model draws its
                // coarsest. One the meshlet path draws keeps level 0's mesh
                // and is told the level (`RenderScene::instance_levels`):
                // its clusters hold every level, the same as the part's
                // meshes. One partly hidden is drawn off that path
                // (og-render's `batches.rs`), as its level's mesh.
                let meshlet = look.fx.reveal >= 1.0 && rs.scene.meshlet_drawable(mesh, material);
                if meshlet {
                    let at = rs.scene.objects.instances.len();
                    rs.scene.objects.instance_levels.resize(at + 1, u8::MAX);
                    rs.scene.objects.instance_levels[at] = level as u8;
                }
                let mesh = match (level, part_levels.and_then(|l| l.get(k))) {
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
    }
    rs.kept_models = build.finish(&rs.scene, keys, !fractured.is_empty());
    rs.hlod.replaced = replaced;
    rs.model_lods = model_lods;
    rs.lod_drawn = drawn;
    rs.lod_fades = lod_fades;
    rs.models = models;
    rs.baked.overrides = lightmapped_parts;
}

/// og.Mesh rows.
pub(super) fn meshes(
    world: &World,
    rs: &mut RenderState,
    h: &mut Hierarchy,
    keys: &mut Vec<(u64, u32)>,
) {
    let layouts = &rs.draw;
    let (geometries, materials) = (&rs.geometries, &rs.materials);
    let mesh_rows = layouts
        .mesh
        .as_ref()
        .map(|m| mesh_rows(world, m))
        .unwrap_or_default();
    let (geometries, materials) = (geometries.clone(), materials.clone());
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
}
