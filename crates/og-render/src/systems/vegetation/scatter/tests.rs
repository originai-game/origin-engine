use super::*;
use std::sync::Arc;

/// A 40 m terrain of 81 × 81 samples whose height rises along x from
/// -10 m to +10 m.
fn ramp() -> (Terrain, TerrainData) {
    let mut data = TerrainData::flat(Vec2::splat(40.0), 81, 81);
    data.heights = Arc::from(
        (0..81 * 81)
            .map(|i| (i % 81) as f32 / 80.0 * 20.0 - 10.0)
            .collect::<Vec<_>>(),
    );
    let mut scene = crate::RenderScene::new();
    let h = scene.add_terrain(data.clone());
    (Terrain::new(h), data)
}

fn horizontal(m: &Mat4) -> Vec2 {
    Vec2::new(m.w_axis.x, m.w_axis.z)
}

/// A scatter placed after another keeps every copy at least the larger
/// footprint away from the other's copies (Horizon's placement order);
/// without footprints copies of the two come closer than that.
#[test]
fn footprints_keep_the_later_scatter_clear_of_the_earlier() {
    let (t, data) = ramp();
    let rocks = TerrainScatter {
        density: 0.05,
        distance: 30.0,
        footprint: 1.5,
        priority: 1,
        seed: 3,
        ..Default::default()
    };
    let flowers = TerrainScatter {
        density: 4.0,
        distance: 30.0,
        footprint: 0.25,
        seed: 9,
        ..Default::default()
    };
    let all = [(0, flowers), (0, rocks)];
    let ctx = ScatterContext::of(&all, 0, 1, []);
    assert_eq!(ctx.before, vec![rocks], "the higher priority places first");
    let placed_rocks = t.scatter_among(
        &data,
        &rocks,
        &ScatterContext::of(&all, 1, 1, []),
        Vec3::ZERO,
    );
    let placed = t.scatter_among(&data, &flowers, &ctx, Vec3::ZERO);
    let free = t.scatter(&data, &flowers, Vec3::ZERO);
    let closest = |copies: &[(u32, Mat4)]| {
        copies
            .iter()
            .flat_map(|f| {
                placed_rocks
                    .iter()
                    .map(move |r| (horizontal(&f.1) - horizontal(&r.1)).length())
            })
            .fold(f32::MAX, f32::min)
    };
    assert!(placed_rocks.len() > 40, "{} rocks", placed_rocks.len());
    assert!(placed.len() > 1000, "{} flowers", placed.len());
    assert!(
        closest(&free) < 0.5,
        "without footprints a flower stands {} from a rock",
        closest(&free)
    );
    assert!(
        closest(&placed) >= 1.5,
        "a flower stands {} from a rock",
        closest(&placed)
    );
    // Only the flowers near rocks go: most of the field stays.
    assert!(
        placed.len() * 2 > free.len(),
        "{} of {} flowers",
        placed.len(),
        free.len()
    );
}

/// A tree scattered whole is two draws, its wood and its leaves: the
/// leaves take the wood's place in the placing order, so its
/// footprint does not clear them, and what places after the tree
/// keeps clear of it once.
#[test]
fn a_models_further_part_is_placed_as_its_first() {
    let mut scene = crate::RenderScene::new();
    let mesh = scene.add_mesh(crate::shapes::plane(1.0, 1.0));
    let material = scene.add_standard_material(Default::default());
    let tree = TerrainScatter {
        density: 0.05,
        footprint: 3.0,
        priority: 2,
        seed: 3,
        ..Default::default()
    };
    let flowers = TerrainScatter {
        density: 4.0,
        footprint: 0.25,
        seed: 9,
        ..Default::default()
    };
    let draw = |scatter, with| ScatterDraw {
        terrain: 0,
        mesh,
        material,
        scatter,
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
        with,
        lods: [None; 2],
    };
    let all = [
        draw(flowers, None),
        draw(tree, None),
        draw(tree, Some(1)),
        // A part of a part is no part: it places its own copies.
        draw(flowers, Some(2)),
    ];
    let (placing, slots) = ScatterDraw::placing(&all);
    assert_eq!(placing, vec![(0, flowers), (0, tree), (0, flowers)]);
    assert_eq!(slots, vec![0, 1, 1, 2]);
    assert!(
        ScatterContext::of(&placing, slots[2], 1, [])
            .before
            .is_empty(),
        "the leaves keep clear of nothing, as the wood"
    );
    assert_eq!(
        ScatterContext::of(&placing, slots[0], 1, []).before,
        vec![tree],
        "the flowers keep clear of the tree once"
    );
}

/// `avoid_water 2` grows nothing whose ground stands less than 2 m
/// above a water surface over it, and everything above that; a copy
/// beside the water's rectangle is not affected.
#[test]
fn avoid_water_keeps_copies_above_the_water() {
    let (t, data) = ramp();
    let s = TerrainScatter {
        density: 2.0,
        distance: 30.0,
        avoid_water: 2.0,
        ..Default::default()
    };
    // A pool at height 0 over the -z half (z from -20 to 0).
    let pool = Mat4::from_translation(Vec3::new(0.0, 0.0, -10.0));
    let ctx = ScatterContext::of(&[(0, s)], 0, 1, [(pool, Vec2::new(40.0, 20.0), false)]);
    let placed = t.scatter_among(&data, &s, &ctx, Vec3::ZERO);
    let free = t.scatter(&data, &s, Vec3::ZERO);
    let low = |c: &&(u32, Mat4)| c.1.w_axis.y < 2.0 && c.1.w_axis.z < 0.0;
    assert!(
        free.iter().filter(low).count() > 100,
        "the low ground grows copies without the rule"
    );
    assert_eq!(
        placed.iter().filter(low).count(),
        0,
        "copies within 2 m of the pool"
    );
    let kept = |copies: &[(u32, Mat4)], f: &dyn Fn(&Mat4) -> bool| {
        copies.iter().filter(|c| f(&c.1)).count()
    };
    let above = |m: &Mat4| m.w_axis.y >= 2.0 || m.w_axis.z >= 0.0;
    assert_eq!(
        kept(&placed, &above),
        kept(&free, &above),
        "copies away from the pool"
    );
    // An endless sea at height 0 clears both halves' low ground.
    let sea = ScatterContext::of(&[(0, s)], 0, 1, [(Mat4::IDENTITY, Vec2::ONE, true)]);
    let placed = t.scatter_among(&data, &s, &sea, Vec3::ZERO);
    assert!(placed.iter().all(|c| c.1.w_axis.y >= 2.0));
}

/// A noise threshold gathers the copies into patches: at 0.5 it keeps
/// part of the field, the same part every time, and a copy's
/// neighbours are kept far more often than the field's average.
#[test]
fn a_noise_threshold_gathers_copies_into_patches() {
    let (t, data) = ramp();
    let s = TerrainScatter {
        density: 4.0,
        distance: 30.0,
        noise_scale: 6.0,
        noise_threshold: 0.5,
        ..Default::default()
    };
    let free = t.scatter(
        &data,
        &TerrainScatter {
            noise_threshold: 0.0,
            ..s
        },
        Vec3::ZERO,
    );
    let placed = t.scatter(&data, &s, Vec3::ZERO);
    assert_eq!(placed, t.scatter(&data, &s, Vec3::ZERO));
    let share = placed.len() as f32 / free.len() as f32;
    assert!((0.15..0.85).contains(&share), "keeps {share} of the field");
    // Kept copies within 1 m of a kept copy, against the share.
    let near = placed
        .iter()
        .map(|a| {
            let around = |c: &[(u32, Mat4)]| {
                c.iter()
                    .filter(|b| b.0 != a.0 && (horizontal(&a.1) - horizontal(&b.1)).length() < 1.0)
                    .count() as f32
            };
            around(&placed) / around(&free).max(1.0)
        })
        .sum::<f32>()
        / placed.len() as f32;
    assert!(
        near > share + 0.25,
        "a kept copy's neighbours are kept {near} of the time, the field {share}"
    );
}

/// Chunk keys compare by what the chunk was built from, not by the
/// shared copy a frame made of it: a key from another, equal copy finds
/// the kept chunk (a scatter changed and changed back, or placed at
/// another index), and any difference finds none.
#[test]
fn chunk_keys_compare_by_value() {
    let scatter = |params: Vec<u32>| ScatterKey {
        data: 0,
        revision: 3,
        params,
        transform: [0; 16],
        origin: [0; 3],
        layers: vec![1, 2],
        mesh: 4,
    };
    let key = |s: &SharedKey| ChunkKey {
        scatter: s.clone(),
        under: Vec::new(),
        chunk: (1, 2),
        gpu: false,
    };
    let a = SharedKey::new(scatter(vec![5, 6]));
    let mut chunks = HashMap::new();
    chunks.insert(key(&a), ());
    let b = SharedKey::new(scatter(vec![5, 6]));
    assert!(
        chunks.contains_key(&key(&b)),
        "an equal key of another copy"
    );
    let c = SharedKey::new(scatter(vec![5, 7]));
    assert!(!chunks.contains_key(&key(&c)), "other parameters");
    let gpu = ChunkKey {
        gpu: true,
        ..key(&a)
    };
    assert!(!chunks.contains_key(&gpu), "placed the other way");
}
