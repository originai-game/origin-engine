//! What the renderer draws of the terrains: their extraction, diagnostics and the seabed grid.

use super::*;

/// Presentation-only diagnostics of the extracted terrain, from the same
/// clipmap layout the renderer uploads. Never stored in the ECS.
/// The heights the id pass draws this frame's streamed terrains with
/// (`og_render::IdHeights`): each one's level-0 samples, hole bits and
/// all, decoded from its cooked tiles as they are read (the tiles
/// og.terrain.sample reads). A terrain whose field is in memory is read
/// from the scene.
pub(crate) fn id_heights(rs: &RenderState) -> impl Fn(usize, u32, u32) -> Option<f32> + '_ {
    type Sample<'a> = Box<dyn Fn(u32, u32) -> f32 + 'a>;
    let samplers: Vec<Option<Sample>> = rs
        .scene
        .world
        .terrains
        .iter()
        .map(|d| {
            let t = rs.terrain.loaded.iter().find(|t| t.render == d.terrain)?;
            Some(Box::new(t.tile_sampler().sample) as Sample)
        })
        .collect();
    move |k, c, r| Some(samplers.get(k)?.as_ref()?(c, r))
}

pub(crate) fn dump_render(rs: &RenderState, entity: u64) -> Option<Value> {
    let i = rs.ids.terrains.iter().position(|&e| e == entity)?;
    let t = rs.scene.world.terrains.get(i)?;
    let layout = rs.scene.terrain_layout(t.terrain)?;
    let height = match rs.backend.as_ref().map(|b| b.size()) {
        Some((_, ht)) if ht > 0 => ht,
        _ => crate::present::fx::viewport(rs).1 as u32,
    };
    let c = rs.scene.terrain_clipmap(
        i,
        &og_render::TerrainView::of(&rs.scene.frame.camera, height),
    )?;
    let regions: Vec<Value> = c
        .levels
        .iter()
        .enumerate()
        .map(|(level, l)| {
            let spacing = c.unit as f64 * l.step;
            let center = l.center.map(|v| v * c.unit as f64);
            let half = c.ring as f64 * spacing * 0.5;
            json!({
                "level": level, "ring": l.hole.is_some(), "holeOffset": l.hole,
                "localBoundsXZ": [center[0] - half - layout.size.x as f64 * 0.5,
                    center[1] - half - layout.size.y as f64 * 0.5,
                    center[0] + half - layout.size.x as f64 * 0.5,
                    center[1] + half - layout.size.y as f64 * 0.5],
                "triangleSpacingLocal": spacing,
                "triangleSpacingWorldXZ": [t.transform.x_axis.truncate().length() as f64 * spacing,
                    t.transform.z_axis.truncate().length() as f64 * spacing],
                "seam": {"morph": l.morph, "startQuads": l.morph_start,
                    "widthQuads": if l.morph { c.morph_width } else { 0.0 }},
            })
        })
        .collect();
    Some(json!({
        "visible": t.visible, "cell": layout.cell().to_array(), "ringQuads": c.ring,
        "regionsBeforeCulling": regions, "skirts": false,
        "heightBlend": t.height_blend,
        "layers": t.layers.iter().enumerate().map(|(i, l)| json!({
            "index": i, "triplanar": l.triplanar, "tile": l.tile,
            "slope": l.slope, "slopeFade": l.slope_fade,
            "altitude": l.altitude, "altitudeFade": l.altitude_fade,
        })).collect::<Vec<_>>(),
        "projectionWeights": {"view": "terrainProjection", "rgb": ["X", "Y", "Z"],
            "normal": "filtered height-field normal", "exponent": 4},
        "verticalTexelDensity": {"view": "terrainDensity", "rgb": ["X", "Y", "Z"],
            "layer": "dominant raw layer", "units": "base-level albedo texels per vertical screen pixel",
            "encoding": "linear RGB d/(1+d), before projection weighting"},
        "normalMip": {"view": "terrainMip", "measurement": "isotropic derivative estimate",
            "colors": ["green", "yellow", "red", "cyan", "purple", "orange"]},
    }))
}

fn rgb(c: &Color) -> Vec3 {
    Vec3::new(c.r, c.g, c.b)
}

/// og.Terrain components into `rs.scene.terrains`: each terrain placed by
/// its entity's world transform with its layers; og.TerrainScatter
/// components into `rs.scene.scatters`; then the streamed terrains' tiles.
/// The first frame that draws a streamed terrain settles its tiles as a
/// capture does, so no frame draws a terrain before any of its tiles
/// arrived (web findings #5: Lantern Vale's first frame drew no ground, on
/// one host at tick 0 and on the other at tick 1, and auto exposure
/// started from that frame's metering).
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) -> Vec<(u64, u32)> {
    rs.scene.world.terrains.clear();
    rs.ids.terrains.clear();
    let layout = rs.draw.terrain.clone();
    let mut settle = std::mem::take(&mut rs.terrain.settle) || rs.terrain.resident;
    let budget = rs.terrain.budget;
    let mut streamed = Vec::new();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Ok(v) = record::Terrain::from_json(&v) else {
            continue;
        };
        let Some(i) = crate::world::assets::index_of(v.data as f64) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes
                .push(format!("og.Terrain on entity {e} names no loaded terrain"));
            continue;
        };
        let world = h.world_of(e);
        let spacing = v.lod_spacing;
        let ring = v.lod_ring;
        let array = |i: Option<usize>| i.and_then(|i| rs.texture_arrays.get(i)?.handle);
        let layer_arrays = og_render::TerrainLayerArrays {
            albedo: array(t.layer_arrays[0]),
            normal: array(t.layer_arrays[1]),
        };
        rs.scene.world.terrains.push(og_render::Terrain {
            terrain: t.render,
            transform: world,
            layers: t.layers.clone(),
            layer_arrays,
            height_blend: t.height_blend,
            cast_shadows: v.cast_shadows && t.cast_shadows,
            lod: TerrainLod {
                spacing: if spacing > 0.0 {
                    spacing
                } else {
                    t.lod.spacing
                },
                ring: if ring > 0 { ring } else { t.lod.ring },
                budget: if budget > 0 { budget } else { t.lod.budget },
                error: v.lod_error.max(0.0),
            },
            fade_in: !settle,
            visible: v.visible,
            horizon_shadows: v.horizon_shadows,
            displacement_distance: v.displacement_distance.max(0.0),
        });
        rs.ids.terrains.push(e);
        if !t.edited {
            streamed.push((i, rs.scene.world.terrains.len() - 1));
        }
    }
    if streamed.iter().any(|(i, _)| !rs.terrain.drawn.contains(i)) {
        settle = true;
        for &(i, k) in &streamed {
            rs.terrain.drawn.insert(i);
            rs.scene.world.terrains[k].fade_in = false;
        }
    }
    // og.TerrainScatter: copies over the drawn terrain of the same data,
    // placed and drawn by the renderer.
    rs.scene.world.scatters.clear();
    rs.terrain.scatter_issues.clear();
    let layout = rs.draw.scatter.clone();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Ok(v) = record::TerrainScatter::from_json(&v) else {
            continue;
        };
        let Some(i) = crate::world::assets::index_of(v.data as f64) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes.push(format!(
                "og.TerrainScatter on entity {e} names no loaded terrain"
            ));
            continue;
        };
        let Some(terrain) = rs
            .scene
            .world
            .terrains
            .iter()
            .position(|d| d.terrain == t.render)
        else {
            continue;
        };
        let skip = |rs: &mut crate::present::extract::RenderState, issue: String| {
            rs.notes
                .push(format!("og.TerrainScatter on entity {e}: {issue}"));
            rs.terrain.scatter_issues.insert(e, issue);
        };
        // What a copy draws, in the model's space: every part of `model`,
        // or the one geometry and material.
        let whole = v.model;
        let mut parts: Vec<(
            og_render::MeshHandle,
            og_render::MaterialHandle,
            crate::present::extract::ScatterLods,
        )> = Vec::new();
        let impostor = if whole != 0 {
            let Some(model) = rs.models.get(&whole) else {
                skip(rs, "model names no loaded glTF (og.load.gltf)".into());
                continue;
            };
            let lods = rs.model_lods.get(&whole);
            let mut seen = Vec::new();
            let mut by_nodes = false;
            for (k, &(mesh, material, _)) in model.iter().enumerate() {
                if seen.contains(&mesh) {
                    continue;
                }
                seen.push(mesh);
                // Placed with the impostor's bake, or standing at the
                // model's origin as made, with the part's own levels.
                let placed = lods
                    .and_then(|l| l.scatter_meshes.iter().find(|p| p.0 == mesh))
                    .map(|p| (p.1, p.2))
                    .or_else(|| {
                        let made = model.iter().filter(|p| p.0 == mesh);
                        made.clone().all(|p| p.2 == glam::Mat4::IDENTITY).then(|| {
                            let mut levels = [None; 2];
                            let own = lods.and_then(|l| l.parts.get(k)).into_iter().flatten();
                            for (to, &level) in levels.iter_mut().zip(own) {
                                *to = Some(level);
                            }
                            (mesh, levels)
                        })
                    });
                match placed {
                    Some((m, levels)) => parts.push((m, material, levels)),
                    None => by_nodes = true,
                }
            }
            if by_nodes || parts.is_empty() {
                let issue = if by_nodes {
                    "model has parts its nodes move or scale; cook it with `model { impostor true }`, which places them (and draws far copies as one card)"
                } else {
                    "model has no mesh"
                };
                skip(rs, issue.into());
                continue;
            }
            lods.and_then(|l| l.impostor.map(|i| i.0))
        } else {
            let index = |h: u64| (h & 0xffff_ffff) as usize;
            let (Some(&mesh), Some(&material)) = (
                rs.geometries.get(index(v.geometry)),
                rs.materials.get(index(v.material)),
            ) else {
                continue;
            };
            let model = v.impostor;
            let lods = rs.model_lods.get(&model);
            let impostor = lods.and_then(|l| l.impostor.map(|i| i.0));
            let (mesh, levels) = lods
                .and_then(|l| l.scatter_meshes.iter().find(|p| p.0 == mesh))
                .map_or((mesh, [None; 2]), |p| (p.1, p.2));
            if let Some(card) = impostor {
                let Some((min, max)) = rs.scene.mesh(mesh).and_then(|m| m.drawn_bounds()) else {
                    continue;
                };
                let center = (min + max) * 0.5;
                let radius = ((max - min) * 0.5).length().max(1e-3);
                // Cooked positions are quantized to 16 bits; compare bounds in model space.
                let tolerance = card.radius * 1e-4;
                if center.distance(card.center) > tolerance
                    || (radius - card.radius).abs() > tolerance
                {
                    skip(
                        rs,
                        format!(
                            "geometry bounds (center {center:?}, radius {radius}) differ from impostor bounds (center {:?}, radius {}); use geometry from the same complete model, or scatter the model whole (`model`)",
                            card.center, card.radius
                        ),
                    );
                    continue;
                }
            }
            parts.push((mesh, material, levels));
            impostor
        };
        let scatter = og_render::TerrainScatter {
            layer: v.layer,
            density: v.density,
            distance: v.distance,
            fade: v.fade,
            scale: [v.min_scale, v.max_scale],
            align: v.align,
            seed: v.seed,
            sway: v.sway,
            noise_scale: v.noise_scale,
            noise_threshold: v.noise_threshold,
            footprint: v.footprint,
            priority: v.priority,
            avoid_water: v.avoid_water,
            density_map: crate::world::assets::index_of(v.density_map as f64)
                .and_then(|i| rs.textures.get(i).copied()),
            gpu: v.gpu_placement,
        };
        // A model's parts are one copy: the first places it, the rest
        // draw with it.
        let first = rs.scene.world.scatters.len();
        for (k, (mesh, material, lods)) in parts.into_iter().enumerate() {
            rs.scene.world.scatters.push(og_render::ScatterDraw {
                terrain,
                mesh,
                material,
                scatter,
                cast_shadows: v.cast_shadows,
                impostor,
                impostor_distance: v.impostor_distance,
                with: (k > 0).then_some(first),
                lods,
            });
        }
    }
    // og.TerrainStamp: meshes drawn with the drawn terrain of the same
    // data, in its material and blended into its ground.
    rs.scene.world.terrain_stamps.clear();
    rs.ids.stamps.clear();
    let layout = rs.draw.stamp.clone();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Ok(v) = record::TerrainStamp::from_json(&v) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let Some(t) =
            crate::world::assets::index_of(v.data as f64).and_then(|i| rs.terrain.loaded.get(i))
        else {
            rs.notes.push(format!(
                "og.TerrainStamp on entity {e} names no loaded terrain"
            ));
            continue;
        };
        let Some(terrain) = rs
            .scene
            .world
            .terrains
            .iter()
            .position(|d| d.terrain == t.render)
        else {
            continue;
        };
        let geometry = (v.geometry & 0xffff_ffff) as usize;
        let Some(&mesh) = rs.geometries.get(geometry) else {
            continue;
        };
        rs.scene
            .world
            .terrain_stamps
            .push(og_render::TerrainStampDraw {
                terrain,
                mesh,
                transform: h.world_of(e),
                blend: v.blend,
                cast_shadows: v.cast_shadows,
            });
        rs.ids.stamps.push(e);
    }
    // og.Grass: fields over the drawn terrain of the same data, generated
    // and drawn by the renderer.
    rs.scene.world.grass.clear();
    rs.ids.grass.clear();
    let layout = rs.draw.grass.clone();
    // og.GrassType: the kinds of blade fields list in `types`, by entity.
    let kinds: og_core::IdMap<u64, og_render::GrassType> =
        kept_rows(world, rs.draw.grass_type.as_ref())
            .into_iter()
            .filter_map(|(e, v)| {
                let v = record::GrassType::from_json(&v).ok()?;
                let kind = og_render::GrassType {
                    height: v.height,
                    width: v.width,
                    curve: v.curve,
                    base_color: rgb(&v.base_color),
                    tip_color: rgb(&v.tip_color),
                    share: v.share,
                };
                Some((e, kind))
            })
            .collect();
    let texture =
        |h: u64| crate::world::assets::index_of(h as f64).and_then(|i| rs.textures.get(i).copied());
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Ok(v) = record::Grass::from_json(&v) else {
            continue;
        };
        let Some(i) = crate::world::assets::index_of(v.data as f64) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes
                .push(format!("og.Grass on entity {e} names no loaded terrain"));
            continue;
        };
        let Some(terrain) = rs
            .scene
            .world
            .terrains
            .iter()
            .position(|d| d.terrain == t.render)
        else {
            continue;
        };
        let mut types = v.types.iter().filter_map(|t| kinds.get(t).copied());
        let types = [types.next(), types.next(), types.next()];
        let (type_map, density_map, height_map) = (
            texture(v.type_map),
            texture(v.density_map),
            texture(v.height_map),
        );
        rs.scene.world.grass.push(og_render::GrassDraw {
            terrain,
            grass: og_render::Grass {
                layer: v.layer,
                density: v.density,
                distance: v.distance,
                far_tint: v.far_tint,
                height: v.height,
                height_variation: v.height_variation,
                width: v.width,
                curve: v.curve,
                lean: v.lean,
                clump_size: v.clump_size,
                clump_lean: v.clump_lean,
                tufts: v.tufts,
                base_color: rgb(&v.base_color),
                tip_color: rgb(&v.tip_color),
                dry_color: rgb(&v.dry_color),
                color_variation: v.color_variation,
                dryness: v.dryness,
                translucency: v.translucency,
                sheen: v.sheen,
                roughness: v.roughness,
                sway: v.sway,
                stiffness: v.stiffness,
                flutter: v.flutter,
                cast_shadows: v.cast_shadows,
                seed: v.seed,
                recovery: v.recovery,
                types,
                type_map,
                density_map,
                height_map,
            },
        });
        rs.ids.grass.push(e);
    }
    // og.GrassPusher: what presses the grass and scattered copies (the
    // renderer's trample map), placed by their entities' drawn transforms.
    rs.scene.atmosphere.grass_pushers.clear();
    let layout = rs.draw.grass_pusher.clone();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Ok(v) = record::GrassPusher::from_json(&v) else {
            continue;
        };
        let m = h.world_of(e);
        rs.scene
            .atmosphere
            .grass_pushers
            .push(og_render::GrassPusher {
                position: m.transform_point3(glam::Vec3::ZERO),
                radius: v.radius,
                height: v.height,
                strength: v.strength,
            });
    }
    // The unedited terrains' tiles, as the camera needs them.
    if !streamed.is_empty() {
        let height = match rs.backend.as_ref().map(|b| b.size()) {
            Some((_, ht)) if ht > 0 => ht,
            _ => crate::present::fx::viewport(rs).1 as u32,
        };
        let view = og_render::TerrainView::of(&rs.scene.frame.camera, height);
        let terrains = rs.scene.world.terrains.clone();
        let details: Vec<f32> = (0..terrains.len())
            .map(|k| rs.scene.scatter_detail(k))
            .collect();
        let st = &mut rs.terrain;
        let drawn: Vec<crate::bind::terrain_stream::Drawn> = streamed
            .iter()
            .map(|&(i, k)| crate::bind::terrain_stream::Drawn {
                index: i,
                render: st.loaded[i].render,
                cooked: &st.loaded[i].cooked,
                terrain: &terrains[k],
                detail: details[k],
            })
            .collect();
        let hooks = crate::engine::hooks();
        st.streamer.update(
            &drawn,
            &mut rs.scene,
            view,
            settle,
            hooks.as_deref(),
            &mut rs.notes,
        );
    }
    Vec::new()
}

/// The heights of a terrain under a sea (og.Water.seabed, W1): the terrain
/// at load index `index`, placed by `m`, sampled as world heights on a grid
/// over the world xz rectangle `lo` to `hi` (its part over the terrain),
/// at the terrain's own spacing or coarser (at most 1025 samples a side;
/// [`SeabedGrid::heights`]). Off the terrain the seabed is unknown and taken
/// to be far below. With a key for what makes it (the terrain's heights'
/// version, its placement, the rectangle), so the tick and the frame share
/// one field and rebuild it only when one changes.
pub(crate) fn seabed_grid(
    t: &TerrainState,
    index: usize,
    m: Mat4,
    lo: Vec2,
    hi: Vec2,
) -> Option<SeabedGrid> {
    let terrain = t.loaded.get(index)?;
    let size = terrain.size();
    let half = size * 0.5;
    let (along, _) = scales(m);
    // The terrain's footprint in the world, cut to the rectangle.
    let (mut tlo, mut thi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for (x, z) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let w = m.transform_point3(Vec3::new(x * half.x, 0.0, z * half.y));
        tlo = tlo.min(Vec2::new(w.x, w.z));
        thi = thi.max(Vec2::new(w.x, w.z));
    }
    let (lo, hi) = (lo.max(tlo), hi.min(thi));
    if !(hi.x > lo.x && hi.y > lo.y) {
        return None;
    }
    let l = terrain.cooked.layout;
    let spacing = (size.x / (l.columns.max(2) - 1) as f32) * along;
    let extent = (hi - lo).max_element();
    let cell = spacing.max(extent / 1024.0).max(1e-3);
    let columns = (((hi.x - lo.x) / cell).ceil() as u32 + 1).max(2);
    let rows = (((hi.y - lo.y) / cell).ceil() as u32 + 1).max(2);
    let mut key = og_core::hash::StableHasher::new();
    key.write_u64(terrain.generation);
    for v in m.to_cols_array().iter().chain(&[lo.x, lo.y, hi.x, hi.y]) {
        key.write_u32(v.to_bits());
    }
    Some(SeabedGrid {
        origin: lo,
        cell,
        columns,
        rows,
        key: key.finish(),
        index,
        m,
    })
}

impl SeabedGrid {
    /// The world heights of its samples, row-major from `origin`.
    pub(crate) fn heights(&self, t: &TerrainState) -> Vec<f32> {
        let Some(terrain) = t.loaded.get(self.index) else {
            return vec![f32::MIN; (self.columns * self.rows) as usize];
        };
        let half = terrain.size() * 0.5;
        let inverse = self.m.inverse();
        let (deep, _) = terrain.range();
        let (m, lo, cell, columns, rows) =
            (self.m, self.origin, self.cell, self.columns, self.rows);
        terrain.with_heights(|h| {
            (0..rows)
                .flat_map(|r| (0..columns).map(move |c| (c, r)))
                .map(|(c, r)| {
                    let w = lo + Vec2::new(c as f32, r as f32) * cell;
                    let p = inverse.transform_point3(Vec3::new(w.x, 0.0, w.y));
                    let p = Vec2::new(p.x, p.z);
                    let inside = p.x.abs() <= half.x && p.y.abs() <= half.y;
                    let local = if inside { h(p) } else { deep - 1000.0 };
                    m.transform_point3(Vec3::new(p.x, local, p.y)).y
                })
                .collect()
        })
    }
}

/// [`seabed_grid`]'s grid: `columns` × `rows` samples from `origin` (world
/// xz) `cell` metres apart, and the key of what makes its heights.
pub(crate) struct SeabedGrid {
    pub origin: Vec2,
    pub cell: f32,
    pub columns: u32,
    pub rows: u32,
    pub key: u64,
    index: usize,
    m: Mat4,
}
