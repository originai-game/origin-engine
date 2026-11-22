use super::*;
use crate::generators::solids::{CylinderDesc, cylinder, sphere};

fn check(s: &Solid) -> Uvs {
    check_with(s, &Unwrap::default())
}

fn check_with(s: &Solid, d: &Unwrap) -> Uvs {
    let mut s = s.clone();
    let uv = unwrap(&mut s, d).unwrap();
    assert_eq!(s.uvs.as_ref(), Some(&uv.corners));
    // Inside the square, wound as the faces, no two triangles over one
    // another.
    for (i, t) in uv.corners.iter().enumerate() {
        for q in t {
            assert!(q.x >= 0.0 && q.x <= 1.0 && q.y >= 0.0 && q.y <= 1.0);
        }
        assert_eq!(
            orient2d(t[0], t[1], t[2]),
            Ordering::Greater,
            "triangle {i}"
        );
    }
    for i in 0..uv.corners.len() {
        for j in i + 1..uv.corners.len() {
            assert!(!tris_overlap_2d(uv.corners[i], uv.corners[j]), "{i} {j}");
        }
    }
    uv
}

#[test]
fn charts_never_overlap() {
    let s = sphere(10.0).unwrap();
    let uv = check(&s);
    assert!(
        uv.chart_count >= 6 && uv.chart_count < 40,
        "{}",
        uv.chart_count
    );
    let c = cylinder(&CylinderDesc {
        radius: 5.0,
        height: 20.0,
        ..Default::default()
    })
    .unwrap();
    check(&c);
}

/// With shared directions a torus is projected along a few
/// directions chosen over all of it, and its charts are fewer than
/// when each starts from its own seed's normal; a box's six sides
/// are six charts either way.
#[test]
fn shared_directions_are_chosen_over_the_whole_solid() {
    let shared = Unwrap {
        shared: true,
        ..Default::default()
    };
    let t = crate::generators::solids::torus(20.0, 6.0).unwrap();
    assert_eq!(check(&t).chart_count, 14);
    assert_eq!(check_with(&t, &shared).chart_count, 10);
    let b = crate::generators::solids::cuboid(&crate::generators::solids::BoxDesc {
        width: 10.0,
        depth: 20.0,
        height: 30.0,
    })
    .unwrap();
    assert_eq!(check_with(&b, &shared).chart_count, 6);
}

/// Charts that bend are flattened keeping their angles: a dome is
/// three charts where projecting takes six, and a cylinder's side
/// unrolls at its true area (a projection loses a fifth of it), in
/// two strips beside its two caps.
#[test]
fn conformal_charts_keep_angles_and_area() {
    let conformal = Unwrap {
        conformal: true,
        ..Default::default()
    };
    let dome = crate::generators::solids::dome(10.0).unwrap();
    assert_eq!(check(&dome).chart_count, 6);
    assert_eq!(check_with(&dome, &conformal).chart_count, 3);
    let c = cylinder(&CylinderDesc {
        radius: 5.0,
        height: 20.0,
        ..Default::default()
    })
    .unwrap();
    // The area the UVs cover, in the surface's own units, over the
    // surface's.
    let kept = |uv: &Uvs| {
        let flat: f64 = uv
            .corners
            .iter()
            .map(|t| (t[1] - t[0]).perp_dot(t[2] - t[0]) / 2.0)
            .sum();
        flat / (uv.scale * uv.scale) / c.measure().area
    };
    let (projected, unrolled) = (check(&c), check_with(&c, &conformal));
    assert_eq!(unrolled.chart_count, 4);
    assert!(kept(&projected) < 0.9, "{}", kept(&projected));
    assert!((kept(&unrolled) - 1.0).abs() < 1e-6, "{}", kept(&unrolled));
}

#[test]
fn seams_open_closed_surfaces() {
    let seams = Unwrap {
        seams: true,
        ..Default::default()
    };
    let conformal = Unwrap {
        conformal: true,
        ..Default::default()
    };
    // Each triangle's area in the atlas over its own, least and most.
    let stretch = |s: &Solid, uv: &Uvs| {
        let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
        for (t, c) in s.triangles.iter().zip(&uv.corners) {
            let [a, b, d] = t.map(|v| s.positions[v as usize]);
            let own = (b - a).cross(d - a).length();
            let flat = (c[1] - c[0]).perp_dot(c[2] - c[0]) / (uv.scale * uv.scale);
            lo = lo.min(flat / own);
            hi = hi.max(flat / own);
        }
        (lo, hi)
    };
    let ball = crate::generators::solids::sphere_desc(&crate::generators::solids::SphereDesc {
        radius: 10.0,
        segments: 16,
        ..Default::default()
    })
    .unwrap();
    let ring = crate::generators::solids::torus_desc(&crate::generators::solids::TorusDesc {
        radius: 20.0,
        tube_radius: 5.0,
        segments: 24,
        tube_segments: 12,
        ..Default::default()
    })
    .unwrap();
    for (s, without, with) in [(&ball, 4, 1), (&ring, 8, 3)] {
        let (a, b) = (check_with(s, &conformal), check_with(s, &seams));
        let (lo, hi) = stretch(s, &b);
        assert_eq!((a.chart_count, b.chart_count), (without, with));
        assert!(lo >= 0.5 - 1e-9 && hi <= 2.0 + 1e-9, "{lo} {hi}");
    }
}
