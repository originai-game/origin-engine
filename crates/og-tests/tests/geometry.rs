//! The depth wave's geometry (docs/depth.md §1): the reference frames of
//! the three.js primitives and a buffer-geometry height field
//! (`geometry-procedural`), lines (`lines`), point clouds (`points`),
//! wireframe materials (`wireframe`) and debug draw over the finished
//! frame (`debug-overlay`), and from frames: a wireframe draws edges, not
//! its faces; debug draw is not hidden by the geometry in front of it.

use og_tests::gfx;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-geometry")
}

fn check(scene: &str) {
    match gfx::check(scene, &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn geometry_procedural_matches_its_reference() {
    check("geometry-procedural");
}

#[test]
fn lines_match_their_reference() {
    check("lines");
}

#[test]
fn points_match_their_reference() {
    check("points");
}

#[test]
fn wireframe_matches_its_reference() {
    check("wireframe");
}

#[test]
fn debug_overlay_matches_its_reference() {
    check("debug-overlay");
}

fn px(img: &og_render::RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    [p[0], p[1], p[2]]
}

/// A wireframe sphere shows the floor between its edges: most of the
/// pixels inside its outline are not the wire's cyan, while the solid
/// sphere beside it covers its disc.
#[test]
fn a_wireframe_draws_its_edges_not_its_faces() {
    let frame = gfx::render("wireframe").expect("render").image;
    let cyan = |p: [u8; 3]| p[2] > 180 && p[1] > 180 && p[0] < 150;
    let (mut wire, mut total) = (0, 0);
    for y in 300..400 {
        for x in 470..570 {
            total += 1;
            if cyan(px(&frame, x, y)) {
                wire += 1;
            }
        }
    }
    assert!(wire > total / 20, "the wire is drawn ({wire} of {total})");
    assert!(wire < total / 2, "the faces are not ({wire} of {total})");
}
