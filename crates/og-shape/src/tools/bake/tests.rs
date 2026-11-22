use super::*;
use crate::tools::{Atlas, Unwrap, unwrap_atlas};

/// A bump on a 40 mm square: z = h (1 − r²/R²)² inside R.
fn height(x: f64, y: f64) -> (f64, DVec3) {
    let (h, r) = (4.0, 12.0);
    let (dx, dy) = (x - 20.0, y - 20.0);
    let q = 1.0 - (dx * dx + dy * dy) / (r * r);
    if q <= 0.0 {
        return (0.0, DVec3::Z);
    }
    // ∂z/∂x = h · 2q · (−2 dx / R²).
    let gx = h * 2.0 * q * (-2.0 * dx / (r * r));
    let gy = h * 2.0 * q * (-2.0 * dy / (r * r));
    (h * q * q, unit(DVec3::new(-gx, -gy, 1.0)))
}

/// The dense source: an n × n grid over the square with the bump,
/// UVs over the unit square, a colour per corner from x, one surface.
fn source(n: usize) -> Solid {
    let mut s = Solid::default();
    for j in 0..=n {
        for i in 0..=n {
            let (x, y) = (40.0 * i as f64 / n as f64, 40.0 * j as f64 / n as f64);
            s.positions.push(DVec3::new(x, y, height(x, y).0));
        }
    }
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    let mut uvs = Vec::new();
    let mut cols = Vec::new();
    for j in 0..n {
        for i in 0..n {
            for t in [
                [at(i, j), at(i + 1, j), at(i + 1, j + 1)],
                [at(i, j), at(i + 1, j + 1), at(i, j + 1)],
            ] {
                s.push_triangle(t, 0, &[]);
                let p = t.map(|v| s.positions[v as usize]);
                uvs.push(p.map(|q| DVec2::new(q.x / 40.0, q.y / 40.0)));
                cols.push(p.map(|q| crate::mesh::stored([q.x / 40.0, 1.0, 1.0, 1.0])));
            }
        }
    }
    s.uvs = Some(uvs);
    s.colors = Some(cols);
    s
}

/// A 4 × 4 checker of 16-texel squares, black and white, as PNG (row
/// 0 at the top).
fn checker() -> Texture {
    let rgba = (0..64 * 64)
        .flat_map(|k| {
            let (x, y) = ((k % 64) / 16, (k / 64) / 16);
            let v = if (x + y) % 2 == 0 { 0 } else { 255 };
            [v, v, v, 255]
        })
        .collect();
    Texture {
        name: "checker.png".into(),
        mime: "image/png".into(),
        bytes: crate::tools::png::png(&Image {
            width: 64,
            height: 64,
            rgba,
        }),
    }
}

fn low() -> Solid {
    let mut s = Solid::default();
    for (x, y) in [(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)] {
        s.positions.push(DVec3::new(x, y, 0.0));
    }
    s.push_triangle([0, 1, 2], 0, &[]);
    s.push_triangle([0, 2, 3], 0, &[]);
    s
}

#[test]
fn bakes_colour_and_bump_into_the_low_frame() {
    let src = source(64);
    let mut lo = low();
    let atlas = Atlas {
        size: 64,
        padding: 2,
        no_turns: false,
    };
    unwrap_atlas(&mut lo, &Unwrap::default(), &atlas).unwrap();
    let m = Material {
        map: Some(checker()),
        ..Default::default()
    };
    let b = bake(
        &BakeSource {
            solid: &src,
            materials: core::slice::from_ref(&m),
            material_of: &[],
        },
        &lo,
        &Bake {
            size: 64,
            distance: 6.0,
            ..Default::default()
        },
    )
    .unwrap();
    let r = &b.report;
    assert!(r.colors_baked && !r.factors_baked, "{r:?}");
    assert_eq!(r.no_source, 0);
    assert_eq!(r.hits, r.covered, "{r:?}");
    assert_eq!(r.empty, 0);
    let base = crate::tools::texture::decode(&b.material.map.as_ref().unwrap().bytes).unwrap();
    let nrm =
        crate::tools::texture::decode(&b.material.normal_map.as_ref().unwrap().bytes).unwrap();
    let (owner, at) = rasterize(lo.uvs.as_ref().unwrap(), 64);
    let frames = corner_frames(&lo);
    let lin = linear_table();
    let mut checked = 0;
    for texel in 0..64 * 64 {
        let li = owner[texel];
        if li == u32::MAX || at[texel].1 {
            continue;
        }
        let w = at[texel].0;
        let p = tri(&lo, li);
        let p = p[0] * w[0] + p[1] * w[1] + p[2] * w[2];
        let px = ((63 - texel / 64) * 64 + texel % 64) * 4;
        // The base colour: the checker's square × the colour x / 40.
        let (cx, cy) = ((p.x / 10.0).floor() as i64, (p.y / 10.0).floor() as i64);
        let (fx, fy) = (p.x / 10.0 - cx as f64, p.y / 10.0 - cy as f64);
        if (0.2..0.8).contains(&fx) && (0.2..0.8).contains(&fy) {
            // Row 0 of the image is v = 1: square (cx, 3 − cy).
            let white = (cx + (3 - cy)) % 2 == 1;
            let r_lin = f64::from(lin.get(base.rgba[px])) / 65535.0;
            let want = if white { p.x / 40.0 } else { 0.0 };
            assert!((r_lin - want).abs() < 0.01, "{p} {r_lin} {want}");
            assert_eq!(base.rgba[px + 1] == 0, !white);
            checked += 1;
        }
        // The normal, decoded in the low frame, against the bump's.
        let f = frame_at(&frames[li as usize], w);
        let bt = f.normal.cross(f.tangent) * f.w;
        let c = [0, 1, 2].map(|k| f64::from(nrm.rgba[px + k]) * 2.0 / 255.0 - 1.0);
        let seen = unit(f.tangent * c[0] + bt * c[1] + f.normal * c[2]);
        let (_, want) = height(p.x, p.y);
        let ang = trig::to_degrees(trig::acos(seen.dot(want).clamp(-1.0, 1.0)));
        assert!(ang < 3.0, "{p} {seen} {want} {ang}");
    }
    assert!(checked > 500, "{checked}");
    let nm = r.normals.unwrap();
    assert!(nm.mean < 1.0 && nm.mean_without > 5.0, "{nm:?}");
}

#[test]
fn a_thin_wall_bakes_its_near_side() {
    // Two sheets 1 mm apart facing away from each other (a wall's
    // outside at z = 0.5 facing +z, its inside at z = −0.5 facing −z),
    // red outside, blue inside; the low sheet at z = 0 faces +z, so
    // every texel takes red though blue is as near.
    let mut src = Solid::default();
    for z in [0.5, -0.5] {
        for (x, y) in [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)] {
            src.positions.push(DVec3::new(x, y, z));
        }
    }
    src.push_triangle([0, 1, 2], 0, &[]);
    src.push_triangle([0, 2, 3], 0, &[]);
    src.push_triangle([4, 6, 5], 0, &[]);
    src.push_triangle([4, 7, 6], 0, &[]);
    let red = [1.0, 0.0, 0.0, 1.0];
    let blue = [0.0, 0.0, 1.0, 1.0];
    src.colors = Some(vec![[red; 3], [red; 3], [blue; 3], [blue; 3]]);
    let mut lo = Solid::default();
    for (x, y) in [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)] {
        lo.positions.push(DVec3::new(x, y, 0.0));
    }
    lo.push_triangle([0, 1, 2], 0, &[]);
    lo.push_triangle([0, 2, 3], 0, &[]);
    let atlas = Atlas {
        size: 32,
        ..Default::default()
    };
    unwrap_atlas(&mut lo, &Unwrap::default(), &atlas).unwrap();
    let b = bake(
        &BakeSource {
            solid: &src,
            materials: &[],
            material_of: &[],
        },
        &lo,
        &Bake {
            size: 32,
            distance: 2.0,
            no_normals: true,
            ..Default::default()
        },
    )
    .unwrap();
    let r = &b.report;
    assert_eq!(r.hits, r.covered);
    assert_eq!(r.ambiguous, 0);
    let img = crate::tools::texture::decode(&b.material.map.as_ref().unwrap().bytes).unwrap();
    assert!(img.rgba.chunks(4).all(|p| p == [255, 0, 0, 255]));
}
