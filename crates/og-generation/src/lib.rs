//! `og.procgen`: deterministic procedural generation over the wire (noise,
//! gradients, curl, Worley, Poisson-disk, BSP dungeons, AC-4 WFC, eroded
//! heightmaps, procedural plants, and Voronoi mesh fracture). Each call is a
//! pure function of its desc (including its seed), so it needs no world.

use og_api::wire;
use og_core::glam::{DVec3, Vec2, Vec3};
use og_core::noise::{self, Fbm, NoiseKind};
use og_core::procgen::{self, BspDesc, Wfc, WfcTile};
use og_core::rng::Rng;

/// An FbmDesc read off the wire as og-core's generator.
fn fbm(d: &wire::FbmDesc) -> Fbm {
    Fbm {
        seed: d.seed,
        kind: match d.kind {
            wire::NoiseKind::Perlin => NoiseKind::Perlin,
            wire::NoiseKind::Simplex => NoiseKind::Simplex,
            wire::NoiseKind::Worley => NoiseKind::Worley,
            wire::NoiseKind::WorleyEdge => NoiseKind::WorleyEdge,
            wire::NoiseKind::Ridged => NoiseKind::Ridged,
            wire::NoiseKind::Billow => NoiseKind::Billow,
        },
        octaves: d.octaves.clamp(1, 16),
        frequency: d.frequency,
        lacunarity: d.lacunarity,
        gain: d.gain,
    }
}

/// Evaluates `FbmDesc` with optional multi-octave domain warping.
fn eval_fbm(d: &wire::FbmDesc, x: f32, y: f32, z: f32) -> f32 {
    let base = fbm(d);
    let flat = !d.three_d;
    if d.warp <= 0.0 || !d.warp.is_finite() {
        return base.at(x, y, z, flat);
    }
    let wf = d.frequency * d.warp_frequency.max(0.0);
    let warp_octaves = d.warp_octaves.clamp(1, 8);
    let warp_gen = |offset: u32| Fbm {
        seed: d.seed.wrapping_add(offset),
        kind: NoiseKind::Simplex,
        octaves: warp_octaves,
        frequency: wf,
        lacunarity: d.lacunarity,
        gain: d.gain,
    };
    let wx = warp_gen(0x517c_c1b7).at(x, y, z, flat);
    let wy = warp_gen(0x27d4_eb2d).at(x, y, z, flat);
    let wz = if flat {
        0.0
    } else {
        warp_gen(0x1656_67b1).at(x, y, z, false)
    };
    base.at(x + wx * d.warp, y + wy * d.warp, z + wz * d.warp, flat)
}

pub fn og_procgen_noise(desc: String, x: f64, y: f64, z: f64) -> f64 {
    match wire::arg::<wire::FbmDesc>(Some(&desc), "desc") {
        Ok(d) => f64::from(eval_fbm(&d, x as f32, y as f32, z as f32)),
        Err(_) => f64::NAN,
    }
}

pub fn og_procgen_noise_grad(desc: String, x: f64, y: f64, z: f64) -> wire::NoiseGrad {
    let Ok(d) = wire::arg::<wire::FbmDesc>(Some(&desc), "desc") else {
        return wire::NoiseGrad {
            value: f32::NAN,
            dx: f32::NAN,
            dy: f32::NAN,
            dz: f32::NAN,
        };
    };
    let noise_gen = fbm(&d);
    if d.three_d {
        let g = noise_gen.deriv3(x as f32, y as f32, z as f32);
        wire::NoiseGrad {
            value: g[0],
            dx: g[1],
            dy: g[2],
            dz: g[3],
        }
    } else {
        let g = noise_gen.deriv2(x as f32, y as f32);
        wire::NoiseGrad {
            value: g[0],
            dx: g[1],
            dy: g[2],
            dz: 0.0,
        }
    }
}

pub fn og_procgen_curl_noise(desc: String, x: f64, y: f64, z: f64) -> Vec3 {
    let Ok(d) = wire::arg::<wire::FbmDesc>(Some(&desc), "desc") else {
        return Vec3::splat(f32::NAN);
    };
    let noise_gen = fbm(&d);
    if d.three_d {
        Vec3::from_array(noise_gen.curl3(x as f32, y as f32, z as f32))
    } else {
        let c = noise_gen.curl2(x as f32, y as f32);
        Vec3::new(c[0], c[1], 0.0)
    }
}

pub fn og_procgen_worley(seed: u32, x: f64, y: f64, z: f64, three_d: bool) -> Vec2 {
    let s = if three_d {
        noise::worley3(seed, x as f32, y as f32, z as f32)
    } else {
        noise::worley2(seed, x as f32, y as f32)
    };
    Vec2::new(s.f1, s.f2)
}

pub fn og_procgen_noise_grid(desc: Option<String>) -> Result<Vec<f32>, String> {
    let d: wire::NoiseGridDesc = wire::arg(desc.as_deref(), "desc")?;
    let (w, h) = (d.width, d.height);
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 1 << 24 {
        return Err(format!(
            "desc.width x desc.height is {w} x {h}: 1 to 16,777,216 samples"
        ));
    }
    if d.noise.warp <= 0.0 || !d.noise.warp.is_finite() {
        return Ok(fbm(&d.noise).grid(w, h, d.x, d.y, d.step, d.z, !d.noise.three_d));
    }
    let mut out = Vec::with_capacity((w as usize) * (h as usize));
    for j in 0..h {
        let py = d.y + j as f32 * d.step;
        for i in 0..w {
            let px = d.x + i as f32 * d.step;
            out.push(eval_fbm(&d.noise, px, py, d.z));
        }
    }
    Ok(out)
}

pub fn og_procgen_poisson_disk(desc: Option<String>) -> Result<Vec<Vec2>, String> {
    let d: wire::PoissonDesc = wire::arg(desc.as_deref(), "desc")?;
    let pts = procgen::poisson_disk_ex(
        d.seed,
        d.width,
        d.height,
        d.radius,
        d.attempts.clamp(1, 256),
        d.max_points.min(1 << 20) as usize,
        d.wrap,
    )
    .map_err(|e| e.reason().to_string())?;
    Ok(pts.into_iter().map(Vec2::from_array).collect::<Vec<_>>())
}

fn rect(r: &procgen::CellRect) -> wire::CellRect {
    wire::CellRect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

pub fn og_procgen_bsp(desc: Option<String>) -> Result<wire::BspResult, String> {
    let d: wire::BspDesc = wire::arg(desc.as_deref(), "desc")?;
    let out = procgen::bsp_rooms(&BspDesc {
        seed: d.seed,
        width: d.width,
        height: d.height,
        min_room: d.min_room,
        max_room: d.max_room,
        depth: d.depth,
        padding: d.padding,
        corridor: d.corridor,
        loops: if d.loops.is_finite() {
            d.loops.clamp(0.0, 1.0)
        } else {
            0.0
        },
    })
    .map_err(|e| e.reason().to_string())?;
    Ok(wire::BspResult {
        rooms: out.rooms.iter().map(rect).collect(),
        corridors: out.corridors.iter().map(rect).collect(),
        edges: out.edges,
        cells: out.cells,
    })
}

pub fn og_procgen_wfc(desc: Option<String>) -> Result<wire::WfcResult, String> {
    let d: wire::WfcDesc = wire::arg(desc.as_deref(), "desc")?;
    let tiles = d
        .tiles
        .iter()
        .map(|t| WfcTile {
            weight: t.weight,
            sockets: [t.up, t.right, t.down, t.left],
            rotations: t.rotations,
        })
        .collect();
    let fixed = d
        .fixed
        .iter()
        .map(|c| (c.x, c.y, c.tile, c.quarter_turns))
        .collect();
    let out = procgen::wfc(&Wfc {
        seed: d.seed,
        width: d.width,
        height: d.height,
        tiles,
        fixed,
        wrap: d.wrap,
        attempts: d.attempts.clamp(1, 1000),
    })
    .map_err(|e| e.reason().to_string())?;
    Ok(wire::WfcResult {
        tiles: out.tiles,
        rotations: out.rotations.into_iter().map(|r| r as u8).collect(),
        attempt: out.attempt,
    })
}

pub fn og_procgen_heightmap(desc: Option<Vec<u8>>) -> Result<wire::HeightmapResult, String> {
    use og_render_data::terrain::generate::{
        Biome, Generate, Hydraulic, Mask, Moisture, Rivers, Thermal, generate, process,
    };
    let d: wire::HeightmapDesc = og_api::bin::arg(desc, "desc")?;
    if d.columns < 2 || d.rows < 2 || d.columns > 4097 || d.rows > 4097 {
        return Err(format!(
            "columns x rows is {} x {}: each must be 2 to 4097",
            d.columns, d.rows
        ));
    }
    if !(d.size.x.is_finite() && d.size.x > 0.0 && d.size.y.is_finite() && d.size.y > 0.0) {
        return Err("desc.size components must be finite and > 0".into());
    }
    if !(d.min_height.is_finite() && d.max_height.is_finite() && d.max_height >= d.min_height) {
        return Err(
            "desc.minHeight and desc.maxHeight must be finite with maxHeight >= minHeight".into(),
        );
    }
    if !(d.frequency.is_finite()
        && d.lacunarity.is_finite()
        && d.gain.is_finite()
        && d.warp.is_finite()
        && d.mask_strength.is_finite()
        && d.terrace_step.is_finite()
        && d.terrace_blend.is_finite()
        && d.shore.is_finite())
    {
        return Err("desc contains a non-finite parameter".into());
    }
    let mask = match d.mask {
        wire::HeightmapMask::None => Mask::None,
        wire::HeightmapMask::Island => Mask::Island(d.mask_strength),
        wire::HeightmapMask::Ridge => Mask::Ridge(d.mask_strength),
    };
    let erosion = d.erosion.map(|e| Hydraulic {
        drops: e.drops,
        lifetime: e.lifetime,
        radius: e.radius,
        inertia: e.inertia,
        capacity: e.capacity,
        deposition: e.deposition,
        erosion: e.erosion,
        evaporation: e.evaporation,
        gravity: e.gravity,
    });
    let thermal = d.thermal.map(|t| Thermal {
        talus: t.talus,
        iterations: t.iterations,
        rate: t.rate,
    });
    let rivers = d.rivers.map(|r| Rivers {
        threshold: r.threshold,
        depth: r.depth,
        width: r.width,
    });
    let terraces = (d.terrace_step != 0.0).then_some([d.terrace_step, d.terrace_blend]);
    let biomes: Vec<Biome> = d
        .biomes
        .iter()
        .map(|b| Biome {
            layer: b.layer,
            weight: b.weight,
            altitude: [b.altitude_min, b.altitude_max],
            altitude_fade: b.altitude_fade,
            slope: [b.slope_min, b.slope_max],
            slope_fade: b.slope_fade,
            moisture: [b.moisture_min, b.moisture_max],
            moisture_fade: b.moisture_fade,
            line: 0,
        })
        .collect();
    let spec = Generate {
        columns: d.columns,
        rows: d.rows,
        seed: d.seed,
        octaves: d.octaves,
        frequency: d.frequency,
        lacunarity: d.lacunarity,
        gain: d.gain,
        warp: d.warp,
        mask,
        erosion,
        thermal,
        terraces,
        sea: d.sea_level,
        shore: d.shore,
        rivers,
        moisture: Moisture {
            flow: d.moisture.flow,
            scale: d.moisture.scale,
            noise: d.moisture.noise,
            frequency: d.moisture.frequency,
        },
        biomes,
    };
    spec.check()?;
    let out = if let Some(heights) = d.heights {
        if heights.len() as u64 != u64::from(d.columns) * u64::from(d.rows)
            || heights.iter().any(|h| !h.is_finite())
        {
            return Err("desc.heights must contain columns × rows finite metre samples".into());
        }
        process(&spec, [d.size.x, d.size.y], None, heights)
    } else {
        generate(&spec, [d.size.x, d.size.y], [d.min_height, d.max_height])
    };
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for &h in &out.heights {
        if !h.is_finite() {
            return Err("terrain processing produced non-finite heights; reduce the input magnitude or stage parameters".into());
        }
        lo = lo.min(h);
        hi = hi.max(h);
    }
    Ok(wire::HeightmapResult {
        columns: d.columns,
        rows: d.rows,
        min_height: lo,
        max_height: hi,
        river_length: out.river_length,
        heights: out.heights,
        control: out.control,
    })
}

fn flatten_plant_part(mesh: &og_shape::generators::Mesh, wind: &[[f32; 8]]) -> wire::PlantMeshPart {
    let mut positions = Vec::with_capacity(mesh.positions.len() * 3);
    for p in &mesh.positions {
        positions.extend_from_slice(p);
    }
    let mut normals = Vec::with_capacity(mesh.normals.len() * 3);
    for n in &mesh.normals {
        normals.extend_from_slice(n);
    }
    let mut uvs = Vec::with_capacity(mesh.uvs.len() * 2);
    for uv in &mesh.uvs {
        uvs.extend_from_slice(uv);
    }
    let mut colors = Vec::with_capacity(mesh.colors.len() * 4);
    for c in &mesh.colors {
        colors.extend_from_slice(c);
    }
    let mut wind_pivot = Vec::with_capacity(wind.len() * 4);
    let mut wind_leaf = Vec::with_capacity(wind.len() * 4);
    for w in wind {
        wind_pivot.extend_from_slice(&w[0..4]);
        wind_leaf.extend_from_slice(&w[4..8]);
    }
    wire::PlantMeshPart {
        positions,
        normals,
        uvs,
        colors,
        indices: mesh.indices.clone(),
        wind_pivot,
        wind_leaf,
    }
}

pub fn og_procgen_plant(desc: Option<String>) -> Result<wire::PlantMeshResult, String> {
    use og_shape::generators::plant::{
        Flowers, Growth, LEVELS, LeafShape, Leaves, Level, Plant, plant,
    };
    let d: wire::PlantDesc = wire::arg(desc.as_deref(), "desc")?;
    let mut spec = Plant {
        seed: d.seed,
        growth: match d.growth {
            wire::PlantGrowth::Deciduous => Growth::Deciduous,
            wire::PlantGrowth::Evergreen => Growth::Evergreen,
        },
        levels: d.levels.min((LEVELS - 1) as u32),
        force_direction: DVec3::new(
            f64::from(d.force_direction.x),
            f64::from(d.force_direction.y),
            f64::from(d.force_direction.z),
        ),
        force_strength: f64::from(d.force_strength),
        leaves: d.leaves.map(|l| Leaves {
            shape: match l.shape {
                wire::PlantLeafShape::Blade => LeafShape::Blade,
                wire::PlantLeafShape::Card => LeafShape::Card,
                wire::PlantLeafShape::Cross => LeafShape::Cross,
            },
            count: l.count.min(256),
            start: f64::from(l.start).clamp(0.0, 1.0),
            end: f64::from(l.end).clamp(0.0, 1.0),
            angle: f64::from(l.angle).clamp(0.0, 175.0),
            facing: f64::from(l.facing).clamp(0.0, 1.0),
            rows: l.rows.min(16),
            size: f64::from(l.size).max(0.001),
            width: f64::from(l.width).clamp(0.05, 4.0),
            size_variance: f64::from(l.size_variance).clamp(0.0, 1.0),
            color: [l.color.r, l.color.g, l.color.b],
            color2: [l.color2.r, l.color2.g, l.color2.b],
            color_variation: l.color_variation.clamp(0.0, 1.0),
        }),
        flowers: d.flowers.map(|f| Flowers {
            share: f64::from(f.share).clamp(0.0, 1.0),
            count: f.count.min(64),
            petals: f.petals.clamp(3, 24),
            size: f64::from(f.size).max(0.001),
            width: f64::from(f.width).clamp(0.05, 4.0),
            cup: f64::from(f.cup).clamp(0.0, 85.0),
            color: [f.color.r, f.color.g, f.color.b],
            color2: [f.color2.r, f.color2.g, f.color2.b],
            center_color: [f.center_color.r, f.center_color.g, f.center_color.b],
        }),
        ..Plant::default()
    };
    for (i, l) in d.levels_spec.iter().take(LEVELS).enumerate() {
        spec.level[i] = Level {
            length: f64::from(l.length).max(0.0),
            radius: f64::from(l.radius).max(0.0),
            sections: l.sections.clamp(1, 32),
            segments: l.segments.clamp(3, 32),
            taper: f64::from(l.taper).clamp(0.0, 1.0),
            gnarliness: f64::from(l.gnarliness).clamp(0.0, 2.0),
            twist: f64::from(l.twist),
            children: l.children.min(64),
            angle: f64::from(l.angle).clamp(0.0, 175.0),
            start: f64::from(l.start).clamp(0.0, 1.0),
        };
    }
    let out = plant(&spec, 1.0)?;
    let leaves = if out.leaves.positions.is_empty() {
        None
    } else {
        Some(flatten_plant_part(&out.leaves, &out.leaf_wind))
    };
    Ok(wire::PlantMeshResult {
        wood: flatten_plant_part(&out.wood, &out.wood_wind),
        leaves,
        branches: out.branches as u32,
        leaf_count: out.leaf_count as u32,
        flower_count: out.flower_count as u32,
    })
}

pub fn og_procgen_fracture(desc: Option<Vec<u8>>) -> Result<wire::FractureResult, String> {
    use og_shape::boolean::fracture::{bounds, contains, voronoi};
    use og_shape::generators::Mesh;
    let d: wire::FractureDesc = og_api::bin::arg(desc, "desc")?;
    if d.positions.len() < 9 || !d.positions.len().is_multiple_of(3) {
        return Err("desc.positions must hold at least 3 vertices (x y z floats)".into());
    }
    if d.indices.len() < 3 || !d.indices.len().is_multiple_of(3) {
        return Err("desc.indices must hold at least one triangle (multiple of 3)".into());
    }
    if d.positions.iter().any(|x| !x.is_finite()) {
        return Err("desc.positions contains a non-finite coordinate".into());
    }
    let vert_count = d.positions.len() / 3;
    if d.indices.iter().any(|&i| i as usize >= vert_count) {
        return Err("desc.indices contains an out-of-range vertex index".into());
    }
    let chunks = d.chunks.clamp(2, 128) as usize;
    let metric = DVec3::new(
        f64::from(d.metric.x).abs().max(1e-4),
        f64::from(d.metric.y).abs().max(1e-4),
        f64::from(d.metric.z).abs().max(1e-4),
    );
    let uv_scale = if d.uv_scale.is_finite() {
        f64::from(d.uv_scale).max(1e-3)
    } else {
        1.0
    };
    let positions: Vec<[f32; 3]> = d.positions.as_chunks::<3>().0.to_vec();
    let mesh = Mesh {
        normals: vec![[0.0, 1.0, 0.0]; positions.len()],
        uvs: vec![[0.0, 0.0]; positions.len()],
        colors: Vec::new(),
        positions,
        indices: d.indices,
    };
    let (lo, hi) = bounds(&mesh);
    let extent = hi - lo;
    let diag = extent.length().max(1e-4);
    let mut rng = Rng::new(d.seed);
    let mut seeds = Vec::with_capacity(chunks);
    let impact = d
        .impact
        .map(|v| DVec3::new(f64::from(v.x), f64::from(v.y), f64::from(v.z)));
    let impact_r = if d.impact_radius.is_finite() && d.impact_radius > 0.0 {
        f64::from(d.impact_radius)
    } else {
        diag * 0.5
    };
    // Sample deterministic seed centres inside the closed mesh (clustering half
    // near `impact` when given), falling back to bounding-box samples if thin.
    let max_tries = chunks * 32;
    let mut tries = 0;
    while seeds.len() < chunks && tries < max_tries {
        tries += 1;
        let pt = if let Some(imp) = impact
            && seeds.len() * 2 < chunks
        {
            // Box-Muller-free deterministic cubic radial clustering near impact.
            let dx = rng.range(-1.0, 1.0);
            let dy = rng.range(-1.0, 1.0);
            let dz = rng.range(-1.0, 1.0);
            let dir = DVec3::new(dx, dy, dz);
            let len = dir.length();
            if len > 1e-6 {
                let r = impact_r * rng.next_f64() * rng.next_f64();
                imp + (dir / len) * r
            } else {
                imp
            }
        } else {
            DVec3::new(
                lo.x + rng.next_f64() * extent.x,
                lo.y + rng.next_f64() * extent.y,
                lo.z + rng.next_f64() * extent.z,
            )
        };
        if contains(&mesh, pt) {
            seeds.push(pt);
        }
    }
    while seeds.len() < chunks {
        seeds.push(DVec3::new(
            lo.x + rng.next_f64() * extent.x,
            lo.y + rng.next_f64() * extent.y,
            lo.z + rng.next_f64() * extent.z,
        ));
    }
    let material = vec![0u32; mesh.triangle_count()];
    let out = voronoi(&mesh, &material, &seeds, metric, uv_scale)?;
    let pieces = out
        .pieces
        .into_iter()
        .map(|p| {
            let mut pos = Vec::with_capacity(p.mesh.positions.len() * 3);
            for v in &p.mesh.positions {
                pos.extend_from_slice(v);
            }
            let mut nrm = Vec::with_capacity(p.mesh.normals.len() * 3);
            for v in &p.mesh.normals {
                nrm.extend_from_slice(v);
            }
            let mut uvs = Vec::with_capacity(p.mesh.uvs.len() * 2);
            for v in &p.mesh.uvs {
                uvs.extend_from_slice(v);
            }
            wire::FracturePiece {
                cell: p.cell,
                positions: pos,
                normals: nrm,
                uvs,
                indices: p.mesh.indices,
                volume: p.volume as f32,
                centroid: Vec3::new(
                    p.centroid.x as f32,
                    p.centroid.y as f32,
                    p.centroid.z as f32,
                ),
            }
        })
        .collect();
    let bonds = out
        .bonds
        .into_iter()
        .map(|b| wire::FractureBond {
            a: b.a,
            b: b.b,
            centroid: Vec3::new(
                b.centroid.x as f32,
                b.centroid.y as f32,
                b.centroid.z as f32,
            ),
            normal: Vec3::new(b.normal.x as f32, b.normal.y as f32, b.normal.z as f32),
            area: b.area as f32,
        })
        .collect();
    Ok(wire::FractureResult { pieces, bonds })
}
