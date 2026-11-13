use super::*;

/// [`pack_normal`]'s direction and length back.
fn unpack_normal(t: u32) -> (Vec3, f32) {
    let f = |s: u32| ((t >> s) & 1023) as f32 / 1023.0;
    let (x, z) = (f(0) * 2.0 - 1.0, f(10) * 2.0 - 1.0);
    let y = (1.0 - x * x - z * z).max(0.0).sqrt();
    (Vec3::new(x, y, z), f(20))
}

/// A 257 × 257 field over 256 m in tiles of 16: five levels.
fn layout() -> TerrainLayout {
    TerrainLayout::new(Vec2::splat(256.0), 257, 257, 16, None).expect("layout")
}

#[test]
fn a_coarse_level_holds_its_finer_levels_mean_not_one_sample() {
    let l = layout();
    // A checker of ±1 one sample wide: decimation keeps +1 at every
    // even sample; the mean under each coarse sample is 0.
    let checker = |c: u32, r: u32| if (c + r).is_multiple_of(2) { 1.0 } else { -1.0 };
    let p = HeightPyramid::new(&l, checker);
    let (sx, sz, h) = &p.levels[0];
    assert_eq!((*sx, *sz), l.samples(1));
    for z in 1..sz - 1 {
        for x in 1..sx - 1 {
            assert_eq!(h[(z * sx + x) as usize], 0.0, "level 1 at ({x}, {z})");
        }
    }
    // A ramp keeps its height everywhere off the edges, at every level.
    let ramp = |c: u32, r: u32| 0.5 * c as f32 - 0.25 * r as f32;
    let p = HeightPyramid::new(&l, ramp);
    for k in 1..l.levels {
        let (sx, sz, h) = &p.levels[k as usize - 1];
        let step = l.step(k);
        for z in 1..sz - 1 {
            for x in 1..sx - 1 {
                let want = ramp(x * step, z * step);
                let got = h[(z * sx + x) as usize];
                assert!(
                    (got - want).abs() < 1e-4,
                    "level {k} ({x}, {z}): {got}, not {want}"
                );
            }
        }
    }
}

#[test]
fn a_coarse_tile_reads_its_levels_samples_with_their_border() {
    let l = layout();
    let field = |c: u32, r: u32| ((c * 7 + r * 13) % 11) as f32;
    let p = HeightPyramid::new(&l, field);
    let (sx, _, h1) = &p.levels[0];
    // Level 1's tile (1, 0): its samples from x 16 and z 0, its border
    // one sample further out (clamped at z -1).
    let tile = p.tile_heights(&l, l.index(1, 1, 0), field);
    let side = l.heights_side();
    for j in 0..side {
        for i in 0..side {
            let (gx, gz) = (16 + i as i64 - 1, (j as i64 - 1).max(0));
            let want = h1[(gz as u32 * sx + gx as u32) as usize];
            assert_eq!(tile[(j * side + i) as usize], want, "({i}, {j})");
        }
    }
    // Level 0 is the field's own.
    let t0 = p.tile_heights(&l, l.index(0, 2, 3), field);
    assert_eq!(t0[side as usize + 1], field(32, 48));
}

#[test]
fn a_normal_mip_keeps_the_mean_direction_and_the_spread() {
    // Level 0's tile 0 over ridges 8 m apart (slopes to 57°) on a
    // gentle rise towards +x.
    let l = layout();
    let field = |c: u32, _: u32| -> f32 {
        let x = c as f32;
        2.0 * (x * std::f32::consts::FRAC_PI_4).sin() + 0.2 * x
    };
    let p = HeightPyramid::new(&l, field);
    let tile = TerrainTile::build(
        &l,
        0,
        p.tile_heights(&l, 0, field),
        vec![0; l.horizon_words() as usize],
        None,
    );
    let ns = l.normals_side() as usize;
    let (base, top) = (&tile.normals[..ns * ns], *tile.normals.last().unwrap());
    // The mean of level 0's unit normals the halvings reach (129 halves
    // to 64 over the first 128), straight from them.
    let m = ns - 1;
    let mean = (0..m * m)
        .map(|k| unpack_normal(base[(k / m) * ns + k % m]).0)
        .sum::<Vec3>()
        / (m * m) as f32;
    let (dir, len) = unpack_normal(top);
    assert!(
        dir.angle_between(mean.normalize()) < 0.01,
        "{dir} against {}",
        mean.normalize()
    );
    assert!(
        (len - mean.length()).abs() < 0.01,
        "length {len}, the mean's {}",
        mean.length()
    );
    assert!(len < 0.95, "the ridges' spread shows: {len}");
    // A plane's mips are its normal, at full length.
    let plane = |c: u32, r: u32| 0.3 * c as f32 + 0.1 * r as f32;
    let p = HeightPyramid::new(&l, plane);
    let flat = TerrainTile::build(
        &l,
        0,
        p.tile_heights(&l, 0, plane),
        vec![0; l.horizon_words() as usize],
        None,
    );
    let (d, len) = unpack_normal(*flat.normals.last().unwrap());
    assert!(len > 0.998 && d.angle_between(Vec3::new(-0.3, 1.0, -0.1).normalize()) < 0.01);
}

/// Behind a ridge 40 m high the horizon towards it (+x) stands at the
/// ridge's elevation seen from the texel or a little over it, on level
/// 0 and on a coarse level (read from the maxima, not the mean, which
/// would flatten a one-sample ridge); away from it (-x)
/// the flat ground's horizon is level.
#[test]
fn a_ridge_raises_the_horizon_towards_it() {
    let l = layout();
    let ridge = 200u32;
    let field = |c: u32, _: u32| if c == ridge { 40.0 } else { 0.0 };
    let p = HeightPyramid::new(&l, field);
    let angle = |words: &[u32], texel: usize, a: usize| {
        let top = (1u32 << HORIZON_BITS) - 1;
        let w = words[texel * HORIZON_WORDS as usize + a / 2];
        let b = (w >> (HORIZON_BITS * (a as u32 % 2))) & top;
        (b as f32 / top as f32 * 2.0 - 1.0) * 90.0
    };
    let step: f32 = 180.0 / 4095.0;
    for k in [0u32, 2] {
        let tile = l.index(k, 0, 0);
        let h = p.horizon(&l, tile, field);
        assert_eq!(h.len(), l.horizon_words() as usize);
        let side = l.horizon_side() as usize;
        // Texel (2, 2): field column 16 · 2^k (metres too: 1 m cells).
        let x = (2 * HORIZON_SPACING * l.step(k)) as f32;
        let texel = 2 * side + 2;
        let d = ridge as f32 - x;
        let want = (40.0 / d).atan().to_degrees();
        // Conservative: never under it, and over it by no more than
        // the ridge an eighth nearer.
        let most = (40.0 / (d * 0.875)).atan().to_degrees();
        let step = step.max(0.05);
        let got = angle(&h, texel, 0);
        assert!(
            got >= want - step && got <= most + step,
            "level {k}: towards the ridge {got}°, not {want}° to {most}°"
        );
        assert!(angle(&h, texel, 4).abs() <= step, "level {k}: away");
    }
}

/// An edit's pyramids, updated where it reaches, are the edited field's
/// own; every tile the edit changes is one [`edited_tiles`] names, and
/// those are a few per level, not the terrain. Edits inside the field,
/// on a tile edge and at its far corner (where the levels' last
/// samples clamp).
#[test]
fn an_edit_builds_again_only_the_tiles_it_reaches() {
    let (cols, rows) = (301, 237);
    let size = Vec2::new(300.0, 236.0);
    let control = |seed: u32| TerrainControl {
        width: 75,
        height: 59,
        rgba: (0..75 * 59 * 4)
            .map(|i| ((i as u32 * 37 + seed) % 251) as u8)
            .collect(),
    };
    let field = |c: u32, r: u32| ((c * 7 + r * 13) % 11) as f32 + 0.01 * c as f32;
    let before = TerrainData {
        size,
        columns: cols,
        rows,
        heights: (0..rows)
            .flat_map(|r| (0..cols).map(move |c| field(c, r)))
            .collect(),
        control: Some(control(0)),
        holes: Some(vec![0u8; ((cols - 1) * (rows - 1)) as usize].into()),
    };
    let l = before.layout(16).expect("layout");
    assert!(l.levels >= 5 && l.control > 0);
    let all = |d: &TerrainData| {
        let (h, c, o) = (
            d.height_pyramid(&l),
            ControlPyramid::new(&l, d.control.as_ref().unwrap()),
            d.hole_counts(),
        );
        // The surface only: a horizon sees the whole terrain, so an
        // edit moves every tile's a little ([`edited_tiles`]).
        (0..l.tile_count())
            .map(|t| TerrainTile {
                horizon: Arc::from([]),
                ..d.tile(&l, t, &h, c.as_ref(), o.as_ref())
            })
            .collect::<Vec<_>>()
    };
    let old = all(&before);
    let mut heights = before.height_pyramid(&l);
    let mut pyramid = ControlPyramid::new(&l, before.control.as_ref().unwrap()).unwrap();
    let mut data = before.clone();
    // A 3 m brush in the field, one on a level-0 tile edge (x 32 m)
    // and one at the far corner.
    for centre in [Vec2::new(-41.3, 17.8), Vec2::new(-118.0, -60.0), size * 0.5] {
        let (lo, hi) = (centre - 3.0, centre + 3.0);
        let ((c0, r0), (c1, r1)) = l.samples_in(lo, hi);
        let mut h = data.heights.to_vec();
        let mut holes = data.holes.as_ref().unwrap().to_vec();
        for r in r0..=r1 {
            for c in c0..=c1 {
                h[(r * cols + c) as usize] += 5.0;
                if c < cols - 1 && r < rows - 1 {
                    holes[(r * (cols - 1) + c) as usize] = 1;
                }
            }
        }
        let mut map = data.control.clone().unwrap();
        let mut px = map.rgba.to_vec();
        let to = |p: Vec2| ((p + size * 0.5) / size * Vec2::new(75.0, 59.0)).floor();
        let (a, b) = (to(lo).max(Vec2::ZERO), to(hi).min(Vec2::new(74.0, 58.0)));
        for y in a.y as usize..=b.y as usize {
            for x in a.x as usize..=b.x as usize {
                px[(y * 75 + x) * 4] = 255;
            }
        }
        map.rgba = px.into();
        data.heights = h.into();
        data.holes = Some(holes.into());
        data.control = Some(map);
        heights.update(
            &l,
            |c, r| data.heights[(r * cols + c) as usize],
            (c0, r0),
            (c1, r1),
        );
        pyramid.update(&l, data.control.as_ref().unwrap(), lo, hi);
        assert_eq!(heights, data.height_pyramid(&l), "the heights at {centre}");
        assert_eq!(
            pyramid,
            ControlPyramid::new(&l, data.control.as_ref().unwrap()).unwrap(),
            "the control map at {centre}"
        );
    }
    let new = all(&data);
    let mut named = std::collections::HashSet::new();
    for centre in [Vec2::new(-41.3, 17.8), Vec2::new(-118.0, -60.0), size * 0.5] {
        edited_tiles(&l, centre - 3.0, centre + 3.0, |t| {
            named.insert(t);
        });
    }
    let changed: Vec<u32> = (0..l.tile_count())
        .filter(|&t| old[t as usize] != new[t as usize])
        .collect();
    assert!(!changed.is_empty());
    for t in &changed {
        assert!(
            named.contains(t),
            "tile {t} ({:?}) changed unnamed",
            l.locate(*t)
        );
    }
    assert!(
        named.len() <= 3 * 4 * l.levels as usize && named.len() * 4 < l.tile_count() as usize,
        "{} of {} tiles ({} changed)",
        named.len(),
        l.tile_count(),
        changed.len()
    );
}
