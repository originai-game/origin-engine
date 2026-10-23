//! The depth wave's light (docs/depth.md §7): the reference frames of rect
//! lights shaded with LTC (`rect-lights`), spot and point light cookies
//! (`light-cookies`), volumetric fog lit by the sun through a slotted wall
//! and by a spot (`volumetric-fog`), and natively a Cornell box baked by
//! og-bake with a light volume (`baked-cornell`); and from frames: the bake
//! bleeds red onto the white block beside the red wall and lights the
//! ceiling, which the unbaked box leaves dark.

use og_tests::gfx;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-light")
}

fn check(scene: &str) {
    match gfx::check(scene, &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn rect_lights_match_their_reference() {
    check("rect-lights");
}

#[test]
fn light_cookies_match_their_reference() {
    check("light-cookies");
}

#[test]
fn volumetric_fog_matches_its_reference() {
    check("volumetric-fog");
}

#[test]
fn baked_cornell_matches_its_reference() {
    check("baked-cornell");
}

fn mean(img: &og_render::RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> [f32; 3] {
    let mut s = [0.0f32; 3];
    for y in y0..y1 {
        for x in x0..x1 {
            let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
            for k in 0..3 {
                s[k] += p[k] as f32;
            }
        }
    }
    let n = ((x1 - x0) * (y1 - y0)) as f32;
    s.map(|v| v / n)
}

/// The bake carries the bounces: the tall block's side facing the red wall
/// turns red and the ceiling, unlit by the spot under it, is lit; built
/// unbaked, neither.
#[test]
fn the_bake_bleeds_colour_and_lights_the_ceiling() {
    let baked = gfx::render("baked-cornell").expect("render").image;
    let mut built = og_tests::scenes::build("baked-cornell").expect("build");
    built.scene = og_tests::light_scenes::baked_cornell_scene(false).expect("unbaked");
    let unbaked = gfx::render_built(&mut built).expect("render").image;
    let side = mean(&baked, 472, 340, 488, 580);
    assert!(
        side[0] > 1.3 * side[1],
        "the block's side is red from the wall ({side:?})"
    );
    let side_off = mean(&unbaked, 472, 340, 488, 580);
    assert!(
        side_off[0] < 1.15 * side_off[1],
        "unbaked it is not ({side_off:?})"
    );
    let luma = |c: [f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let (on, off) = (
        luma(mean(&baked, 480, 16, 800, 40)),
        luma(mean(&unbaked, 480, 16, 800, 40)),
    );
    assert!(
        on > off + 40.0,
        "the ceiling is lit by bounces ({on:.0} against {off:.0})"
    );
}
