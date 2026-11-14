use super::*;

#[test]
fn a_diagonal_ridge_keeps_its_height_between_samples() {
    let mut field = TerrainData {
        size: Vec2::ONE,
        columns: 2,
        rows: 2,
        heights: [0.0, 300.0, 300.0, 100.0].into(),
        control: None,
        holes: None,
    };
    for (p, expected) in [
        (Vec2::ZERO, 300.0),
        (Vec2::splat(-0.25), 150.0),
        (Vec2::splat(0.25), 200.0),
    ] {
        assert_eq!(field.height(p), expected, "{p}");
    }
    field.heights = [300.0, 0.0, 100.0, 300.0].into();
    assert_eq!(field.height(Vec2::ZERO), 300.0);
    // Equal differences retain h00--h11, not the other diagonal's 250.
    field.heights = [0.0, 200.0, 300.0, 100.0].into();
    assert_eq!(field.height(Vec2::ZERO), 50.0);
}

/// The layer-array variant is the standard lit module with two arrays
/// declared in place of the eight layer textures, and its two layer
/// reads taking the layer from slope.w and altitude.w; nothing else.
#[test]
fn the_layer_array_variant_reads_two_arrays() {
    let standard = terrain_lit_source();
    let array = terrain_array_source(&standard);
    for gone in ["albedo0", "albedo3", "normal1", "normal3"] {
        assert!(standard.contains(gone));
        assert!(!array.contains(gone), "{gone}");
    }
    // The eight layer textures become the layer albedo and layer normal
    // arrays; every other texture (the frame's, the decal atlas) is the
    // standard variant's.
    let count = |s: &str, ty: &str| s.matches(ty).count();
    let (arrays, flat) = ("texture_2d_array<f32>;", "texture_2d<f32>;");
    assert_eq!(count(&array, arrays), count(&standard, arrays) + 2);
    assert_eq!(count(&array, flat) + 8, count(&standard, flat));
    assert!(array.contains("i32(terrain.layers[i].slope.w)"));
    assert!(array.contains("i32(terrain.layers[i].altitude.w)"));
    let tail = |s: &str| s[s.find("fn band(").unwrap()..].to_string();
    assert_eq!(tail(&array), tail(&standard));
    assert!(!terrain_shadow_array_source().contains("albedo0"));
}

/// The lit terrain module as the renderer compiles it.
fn lit_module() -> naga::Module {
    let src = crate::shader_sources()
        .into_iter()
        .find(|s| s.label == "terrain")
        .expect("the terrain module");
    naga::front::wgsl::parse_str(&src.wgsl).expect("parses")
}

/// Every function an entry point reaches through calls, by name.
fn reached(module: &naga::Module, entry: &str) -> std::collections::BTreeSet<String> {
    fn walk(
        module: &naga::Module,
        body: &naga::Block,
        out: &mut std::collections::BTreeSet<String>,
    ) {
        for st in body.iter() {
            match st {
                naga::Statement::Call { function, .. } => {
                    let f = &module.functions[*function];
                    if out.insert(f.name.clone().unwrap_or_default()) {
                        walk(module, &f.body, out);
                    }
                }
                naga::Statement::Block(b) => walk(module, b, out),
                naga::Statement::If { accept, reject, .. } => {
                    walk(module, accept, out);
                    walk(module, reject, out);
                }
                naga::Statement::Switch { cases, .. } => {
                    for c in cases {
                        walk(module, &c.body, out);
                    }
                }
                naga::Statement::Loop {
                    body, continuing, ..
                } => {
                    walk(module, body, out);
                    walk(module, continuing, out);
                }
                _ => {}
            }
        }
    }
    let e = module
        .entry_points
        .iter()
        .find(|e| e.name == entry)
        .unwrap_or_else(|| panic!("{entry}"));
    let mut out = Default::default();
    walk(module, &e.function.body, &mut out);
    out
}

/// The prepass writes the roughness the lit pass shades with (one
/// function of the layers' weights, `blended_roughness`, in both)
/// without reading a normal map or blending a colour: before, it ran
/// the whole `terrain_look`, every layer's normal taps included.
#[test]
fn the_prepass_reads_roughness_alone() {
    let module = lit_module();
    let prepass = reached(&module, "fs_prepass");
    let lit = reached(&module, "fs");
    for f in [
        "blended_roughness",
        "blend_weights",
        "pick_layers",
        "toksvig_roughness",
    ] {
        assert!(prepass.contains(f) && lit.contains(f), "{f}");
    }
    for f in [
        "layer_normal_map",
        "taps_normal",
        "terrain_look",
        "macro_tint",
    ] {
        assert!(!prepass.contains(f), "the prepass reaches {f}");
        assert!(lit.contains(f), "the lit pass reaches {f}");
    }
}

/// The layer records and the terrain's uniform have the layout the
/// shader reads (naga's offsets of the WGSL structs).
#[test]
fn the_terrain_uniform_matches_the_shader() {
    use std::mem::{offset_of, size_of};
    let module = lit_module();
    let mut layouter = naga::proc::Layouter::default();
    layouter.update(module.to_ctx()).expect("lays out");
    let members = |name: &str| {
        let (_, ty) = module
            .types
            .iter()
            .find(|(_, t)| t.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("{name}"));
        let naga::TypeInner::Struct { members, span } = &ty.inner else {
            panic!("{name} is a struct");
        };
        (
            members
                .iter()
                .map(|m| (m.name.clone().unwrap_or_default(), m.offset as usize))
                .collect::<Vec<_>>(),
            *span as usize,
        )
    };
    macro_rules! fields {
            ($t:ty: $($f:ident),*) => { vec![$((stringify!($f).to_string(), offset_of!($t, $f))),*] };
        }
    assert_eq!(
        members("TerrainLayerGpu"),
        (
            fields!(LayerGpu: color, mapping, slope, altitude, offset, look, hex, hex_base, coarse),
            size_of::<LayerGpu>()
        )
    );
    assert_eq!(
        members("TerrainStampUniform"),
        (
            fields!(StampGpu: model, normal, to_terrain, params),
            size_of::<StampGpu>()
        )
    );
    let (list, span) = members("TerrainUniform");
    assert_eq!(span, size_of::<TerrainGpu>());
    assert_eq!(
        list,
        fields!(TerrainGpu: model, normal, origin, extent, params, tiles, displace, view_proj,
                prev_view_proj, pyramid, layers, grass)
    );
}

/// The hex-tiling lattice, as `plane_taps` places it from the values
/// [`hex_lattice`] gives, covers the same ground wherever the scene's
/// origin is: a world point falls in the same cell (the same hash, so
/// the same turn and shift) at the same uv, whole repeats aside. The
/// shader's arithmetic in f32 from the point relative to the origin.
#[test]
fn hexagons_stay_on_the_ground_when_the_origin_moves() {
    const S: f32 = HEX_SCALE as f32;
    let centre = |v: Vec2| Vec2::new(v.x + 0.5 * v.y, 0.866_025_4 * v.y) / S;
    // Per tap: its cell's ids and the uv of the point in its copy
    // before the turn and hash shift (both of which follow the ids).
    let taps = |world: Vec3, origin: Vec3, tile: f32| {
        let (hex, base) = hex_lattice(origin, tile);
        let k = 1.0 / tile;
        let rel = world - origin;
        let u = Vec2::new(rel.x, rel.z) * k;
        let hf = hex[0];
        let st = u * S;
        let sk =
            Vec2::new(st.x - 0.577_350_27 * st.y, 1.154_700_5 * st.y) + Vec2::new(hf[0], hf[1]);
        let id = sk.floor();
        let f = sk - id;
        let s = if 1.0 - f.x - f.y < 0.0 { 1.0 } else { 0.0 };
        let mf = centre(Vec2::new(hf[0], hf[1]));
        [
            id + Vec2::new(s, s),
            id + Vec2::new(s, 1.0 - s),
            id + Vec2::new(1.0 - s, s),
        ]
        .map(|v| {
            let cell = (
                (v.x as i32).wrapping_add(base[0][0]),
                (v.y as i32).wrapping_add(base[0][1]),
            );
            let cen = centre(v);
            let local = u + mf - cen;
            let at = Vec2::new(hf[2], hf[3]) + cen;
            (cell, local, at - at.floor())
        })
    };
    let tile = 3.7;
    for world in [
        Vec3::new(10.3, 2.0, -4.6),
        Vec3::new(20_001.13, 0.0, 15_000.9),
    ] {
        let a = taps(world, Vec3::ZERO, tile);
        for origin in [
            Vec3::new(world.x - 3.3, 0.0, world.z + 7.1),
            Vec3::new(world.x - 100.0, 50.0, world.z - 64.0),
        ] {
            let b = taps(world, origin, tile);
            for ((ca, la, aa), (cb, lb, ab)) in a.iter().zip(&b) {
                assert_eq!(ca, cb, "{world} from {origin}");
                assert!((*la - *lb).length() < 2e-3, "{la} {lb}");
                let d = (*aa - *ab).abs();
                let d = d.min(Vec2::ONE - d);
                assert!(d.length() < 2e-3, "{aa} {ab}");
            }
        }
    }
}

/// Each ring variant's blocks cover its range in order, every quad in
/// the block whose square holds it, and the hole's quads in none.
#[test]
fn a_rings_blocks_partition_its_quads_by_square() {
    let n = 64u32;
    let (idx, ranges) = ring_indices(n);
    let q = n / BLOCKS;
    for (v, r) in ranges.variants.iter().enumerate() {
        assert_eq!(r.blocks[0].start, r.all.start);
        assert_eq!(r.blocks.last().unwrap().end, r.all.end);
        for w in r.blocks.windows(2) {
            assert_eq!(w[0].end, w[1].start);
        }
        let quads = r.all.len() as u32 / 12;
        assert_eq!(quads, if v == 0 { n * n } else { n * n - n * n / 4 });
        for (b, br) in r.blocks.iter().enumerate() {
            let (bx, bz) = (b as u32 % BLOCKS, b as u32 / BLOCKS);
            // Every third index is a triangle's cell centre.
            for &i in idx[br.start as usize..br.end as usize]
                .iter()
                .skip(2)
                .step_by(3)
            {
                let cell = i - (n + 1) * (n + 1);
                let (x, z) = (cell % n, cell / n);
                assert!(
                    x / q == bx && z / q == bz,
                    "variant {v}: quad ({x}, {z}) in block {b}"
                );
            }
        }
    }
}

/// From the terrain's edge looking out over it from above, culling
/// each level's blocks draws less than half the triangles culling
/// whole levels does (every level holds the camera, so whole-level
/// culling draws them all).
#[test]
fn blocks_out_of_view_are_not_drawn() {
    let size = Vec2::splat(2048.0);
    let layout = TerrainLayout::new(size, 1025, 1025, 128, None).unwrap();
    let t =
        Terrain::new(crate::RenderScene::new().add_terrain(TerrainData::flat(size, 1025, 1025)));
    let eye = Vec3::new(1000.0, 40.0, 0.0);
    let camera = crate::Camera {
        position: eye,
        target: Vec3::new(1400.0, 0.0, 30.0),
        near: 0.3,
        far: 3000.0,
        ..Default::default()
    };
    let f = crate::frame::camera_frame(&camera, 16.0 / 9.0, Vec2::ZERO);
    let m = f.view_proj;
    let planes = [
        m.row(3) + m.row(0),
        m.row(3) - m.row(0),
        m.row(3) + m.row(1),
        m.row(3) - m.row(1),
        m.row(2),
        m.row(3) - m.row(2),
    ]
    .map(|p| p / p.truncate().length());
    let visible = |c: Vec3, r: f32| planes.iter().all(|p| p.truncate().dot(c) + p.w >= -r);
    let clipmap = t.clipmap(layout, &crate::TerrainView::of(&camera, 720), &[]);
    let (_, ranges) = ring_indices(clipmap.ring);
    let (mut levels, mut blocks) = (0usize, 0usize);
    for l in &clipmap.levels {
        let ring = match l.hole {
            None => &ranges.variants[0],
            Some((ox, oz)) => &ranges.variants[1 + (ox + 2 * oz) as usize],
        };
        let boxes = level_blocks(l, clipmap.ring, clipmap.unit, size, (0.0, 10.0));
        let lo = boxes.iter().fold(Vec3::MAX, |a, b| a.min(b.0));
        let hi = boxes.iter().fold(Vec3::MIN, |a, b| a.max(b.1));
        let (c, r) = renderer::world_sphere(t.transform, lo, hi);
        if !visible(c, r) {
            continue;
        }
        levels += ring.all.len() / 3;
        let world: Vec<_> = boxes
            .iter()
            .zip(&ring.blocks)
            .map(|((a, b), range)| {
                let (lo, hi) = renderer::world_box(t.transform, *a, *b);
                (range.clone(), lo, hi)
            })
            .collect();
        let inside = |lo: Vec3, hi: Vec3| {
            planes.iter().all(|p| {
                let n = p.truncate();
                n.dot(Vec3::select(n.cmpge(Vec3::ZERO), hi, lo)) + p.w >= 0.0
            })
        };
        blocks += visible_ranges(&world, inside)
            .iter()
            .map(|r| r.len() / 3)
            .sum::<usize>();
    }
    assert!(levels > 0 && blocks > 0, "{levels} {blocks}");
    assert!(
        blocks * 2 <= levels,
        "blocks draw {blocks} triangles, whole levels {levels}"
    );
}

/// Every level's hole (the ring variant its index range draws) is
/// exactly the finer level's square, so the levels tile the plane: no
/// quad is drawn twice and none is missing.
/// A level's morph band moves out past where the next coarser level's
/// height error shows, so a peak large on screen keeps its finer level;
/// with no error the bands stay where the lod's ring puts them.
#[test]
fn a_visible_height_error_keeps_the_finer_level_out_to_it() {
    let t = Terrain::new(crate::RenderScene::new().add_terrain(TerrainData::flat(
        Vec2::splat(64.0),
        2,
        2,
    )));
    let layout = TerrainLayout::new(Vec2::splat(4096.0), 4097, 4097, 128, None).unwrap();
    let view = crate::TerrainView::perspective(Vec3::new(0.0, 50.0, 0.0), 5000.0, 60.0, 720);
    let flat = t.clipmap(layout, &view, &[]);
    assert_eq!(flat.ring, 64);
    assert!(flat.levels.iter().all(|l| l.morph_start == 22.0));
    // Level 1's coarser neighbour (4 samples apart) errs as far as 150 m
    // out: level 1 (2 m quads) must stay unmorphed to 75 of its quads,
    // which a ring of 64 cannot hold.
    let mut reach = [0.0f32; 32];
    reach[2] = 150.0;
    let c = t.clipmap(layout, &view, &reach);
    assert_eq!(c.ring, AUTO_RING);
    assert_eq!(c.levels[1].morph_start, 54.0, "{:?}", c.levels);
    // The others keep the base band, past their inner edge (ring / 4 + 1).
    assert!(
        c.levels
            .iter()
            .enumerate()
            .all(|(l, v)| l == 1 || v.morph_start <= 33.0)
    );
    // Within 44 m (22 quads), ring 64's own band covers it.
    reach[2] = 40.0;
    let c = t.clipmap(layout, &view, &reach);
    assert_eq!((c.ring, c.levels[1].morph_start), (64, 22.0));
    // At 100 m the band starts at 50 quads, in a ring of 128.
    reach[2] = 100.0;
    let c = t.clipmap(layout, &view, &reach);
    assert_eq!((c.ring, c.levels[1].morph_start), (AUTO_RING, 50.0));
}

#[test]
fn levels_tile_the_plane_without_overlap_or_gaps() {
    let ring = 16u32;
    let n = ring as f64;
    for cam in [
        Vec2::new(500.3, 700.9),
        Vec2::new(3.0, 1.0),
        Vec2::new(1023.99, 0.01),
    ] {
        let list = levels(cam, Vec2::splat(1024.0), ring, 1e9);
        assert!(list.len() >= 4, "{list:?}");
        let outer = &list[list.len() - 1];
        let reach = n * 0.5 * outer.step;
        let (cx, cz) = (cam.x as i64, cam.y as i64);
        for z in (cz - 120)..(cz + 120) {
            for x in (cx - 120)..(cx + 120) {
                let p = [x as f64 + 0.5, z as f64 + 0.5];
                if !(0..2).all(|i| (p[i] - outer.center[i]).abs() < reach) {
                    continue;
                }
                let mut hits = 0;
                for l in &list {
                    // The level's quads, less its ring variant's hole.
                    let lo = [0, 1].map(|i| l.center[i] - n * 0.5 * l.step);
                    let q = [0, 1].map(|i| ((p[i] - lo[i]) / l.step).floor());
                    if q.iter().any(|&q| q < 0.0 || q >= n) {
                        continue;
                    }
                    if let Some((ox, oz)) = l.hole {
                        let o = [ox as f64, oz as f64];
                        if (0..2).all(|i| q[i] >= n / 4.0 + o[i] && q[i] < 3.0 * n / 4.0 + o[i]) {
                            continue;
                        }
                    }
                    hits += 1;
                }
                assert_eq!(hits, 1, "cell {x},{z} drawn {hits} times for camera {cam}");
            }
        }
    }
}
