//! The UI pass's scene that cooks (`ui-vector`; og-tests' tests/ui.rs has
//! the others): against its reference frame, and its fill read from it.

use og_tests_cooked::gfx::{self, Rendered};
use og_tests_cooked::scenes::{self, GfxScene};

fn render(scene: &str) -> Rendered {
    let mut built: GfxScene = scenes::build(scene).expect("scene");
    gfx::render_built(&mut built).expect("render")
}

/// The sRGB bytes of pixel (x, y) (the overlay is 1280×720 CSS px on a
/// 1280×720 frame: one CSS px a pixel).
fn px(r: &Rendered, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * r.image.width + x) * 4) as usize;
    let p = &r.image.pixels;
    [f32::from(p[i]), f32::from(p[i + 1]), f32::from(p[i + 2])]
}

/// The cooked emblem's even-odd ring keeps its hole.
#[test]
#[ignore = "requires GPU"]
fn ui_vector_fills_by_its_rule() {
    let r = render("ui-vector");
    // The 640 px emblem: view box 120 at 640/120 px a unit from (600, 40).
    let k = 640.0 / 120.0;
    let at = |ux: f32, uy: f32| px(&r, (600.0 + ux * k) as u32, (40.0 + uy * k) as u32);
    let ring = at(60.0, 56.0 - 26.0);
    let hole = at(60.0 + 16.0, 56.0 + 6.0);
    println!("ring {ring:?} hole {hole:?}");
    assert!(ring[2] > ring[0] + 40.0, "the ring is not blue: {ring:?}");
    assert!(hole[0] > hole[2] + 60.0, "the hole is filled: {hole:?}");
}
