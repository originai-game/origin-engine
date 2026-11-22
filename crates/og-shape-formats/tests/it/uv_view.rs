//! The UV view: outlines at seams, tiles past the unit square.

use og_shape::math::DVec3;
use og_shape::math::*;
use og_shape::mesh::Solid;
use og_shape::tools::export::Part;
use og_shape::tools::uvview::*;
use og_shape::tools::*;
use og_shape_formats::texture::CODEC;

#[test]
fn a_seam_is_an_outline_and_a_shared_edge_is_not() {
    // A square of two triangles in one chart: its diagonal is inside,
    // until the second triangle's UVs move off it (a seam).
    let mut s = Solid::default();
    for (x, y) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
        s.push_vertex(DVec3::new(x, y, 0.0));
    }
    let uv = |x: f64, y: f64| DVec2::new(x, y);
    s.push_triangle_uv(
        [0, 1, 2],
        0,
        &[],
        [uv(0.1, 0.1), uv(0.4, 0.1), uv(0.4, 0.4)],
    );
    s.push_triangle_uv(
        [0, 2, 3],
        0,
        &[],
        [uv(0.1, 0.1), uv(0.4, 0.4), uv(0.1, 0.4)],
    );
    let outline = outlines(&s.triangles, s.uvs.as_ref().unwrap());
    assert_eq!(outline.len(), 4);
    assert!(!outline.contains(&(uv(0.1, 0.1), uv(0.4, 0.4))));
    let mut seam = s.clone();
    seam.uvs.as_mut().unwrap()[1] = [uv(0.6, 0.1), uv(0.9, 0.4), uv(0.6, 0.4)];
    assert_eq!(
        outlines(&seam.triangles, seam.uvs.as_ref().unwrap()).len(),
        6
    );
    // UVs in 0..1: the unit square; out to 2.3: three tiles.
    let scene = Scene {
        name: "t".into(),
        parts: vec![Part::new("p", &s)],
    };
    let v = uv_view(&scene, &UvView { size: 64 }, &CODEC).unwrap();
    assert_eq!((v.len(), v[0].tiles, v[0].uv_min), (1, 1, [0.0, 0.0]));
    assert!(!v[0].textured);
    // The pixel at UV (0.25, 0.25) is covered (not dimmed), at (0.8,
    // 0.8) not.
    let at = |u: f64, w: f64| {
        let (x, y) = ((u * 64.0) as usize, ((1.0 - w) * 64.0) as usize);
        v[0].image.rgba[(y * 64 + x) * 4]
    };
    assert!(at(0.27, 0.23) >= DARK[0]);
    assert!(at(0.8, 0.8) < DARK[0]);
    s.uvs.as_mut().unwrap()[0][2] = uv(2.3, 0.4);
    let scene = Scene {
        name: "t".into(),
        parts: vec![Part::new("p", &s)],
    };
    assert_eq!(
        uv_view(&scene, &UvView { size: 64 }, &CODEC).unwrap()[0].tiles,
        3
    );
}
