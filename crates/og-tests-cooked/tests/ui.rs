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

/// `ui-rive`: converted Rive artboards draw where their numbers put them.
/// Each artboard is contained in a 280 px box from its cell's corner.
#[test]
#[ignore = "requires GPU"]
fn ui_rive_draws_its_artboards_where_the_file_puts_them() {
    let r = render("ui-rive");
    let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 3.0);
    let grey = |v: f32| [v; 3];
    // shapetest: 400 × 200 at 0.7 from (40, 60 + 70); the circle of radius
    // 65.94 about (205.14, 97.67), Rive's default grey 0x74 on 0x31.
    let (cx, cy) = (40.0 + 205.14 * 0.7, 130.0 + 97.67 * 0.7);
    let c = px(&r, cx as u32, cy as u32);
    assert!(near(c, grey(116.0)), "the circle: {c:?}");
    let out = px(&r, (cx + 65.94 * 0.7 + 6.0) as u32, cy as u32);
    assert!(near(out, grey(49.0)), "past the circle: {out:?}");
    // nested_artboard_opacity: the nested red artboard at 0.3275 over the
    // parent's blue, 125..375 of 500 at 0.56 from (340, 60).
    let inside = px(&r, 480, 200);
    let outside = px(&r, 360, 80);
    assert!(
        inside[0] > 100.0 && outside[0] < 30.0,
        "{inside:?} {outside:?}"
    );
    // cubic_value_test at frame 24 (third cell of the second row): the
    // key's own x, 333.98, of a 159.8 px rectangle at 0.56 from (640, 380).
    let x = 640.0 + 333.98 * 0.56;
    let y = 380.0 + 143.47 * 0.56;
    for dx in [-40.0, 40.0] {
        let p = px(&r, (x + dx) as u32, y as u32);
        assert!(
            near(p, grey(116.0)),
            "inside the keyed rectangle at {dx}: {p:?}"
        );
    }
    for dx in [-50.0, 50.0] {
        let p = px(&r, (x + dx) as u32, y as u32);
        assert!(
            near(p, grey(49.0)),
            "outside the keyed rectangle at {dx}: {p:?}"
        );
    }
}
