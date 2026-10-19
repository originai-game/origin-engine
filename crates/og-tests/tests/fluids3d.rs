//! 3D particle fluids drawn as a screen-space surface (`crates/og-render/README.md`
//! §3D fluids): the fluid-3d scene matches its reference for this backend
//! tier, and behaviour read from rendered frames against the same scene
//! with something removed: the pool tints what lies behind it blue while
//! the tiles still show through (more of them as it clears) and are bent by
//! its refraction; the drop is drawn in its particles' own colours; the
//! smoothing melts the spheres into one surface; the clustered local light
//! lights the fluid; the basin's wall hides the fluid behind it.

use og_render::{RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::{gfx, p2_scenes, scenes};
use std::path::PathBuf;

fn frame(edit: impl FnOnce(&mut RenderScene)) -> RgbaImage {
    let mut scene = p2_scenes::fluid_3d_scene();
    edit(&mut scene);
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    r.render(&scene, &RenderTarget::Offscreen).expect("render");
    r.read_offscreen().expect("capture")
}

/// A rectangle of the 1280x720 frame: x0, y0, x1, y1.
type Rect = (u32, u32, u32, u32);
/// The pool's middle, over the tiles.
const POOL: Rect = (480, 360, 900, 450);
/// The drop's middle.
const DROP: Rect = (670, 160, 750, 250);

fn pixels(img: &RgbaImage, (x0, y0, x1, y1): Rect) -> impl Iterator<Item = [f32; 3]> + '_ {
    (y0..y1).flat_map(move |y| {
        (x0..x1).map(move |x| {
            let i = ((y * img.width + x) * 4) as usize;
            [0, 1, 2].map(|k| img.pixels[i + k] as f32)
        })
    })
}

fn area(r: Rect) -> f32 {
    ((r.2 - r.0) * (r.3 - r.1)) as f32
}

/// The mean colour over a rectangle.
fn mean(img: &RgbaImage, r: Rect) -> [f32; 3] {
    let sum = pixels(img, r).fold([0.0; 3], |s, p| [s[0] + p[0], s[1] + p[1], s[2] + p[2]]);
    sum.map(|c| c / area(r))
}

fn luma(p: [f32; 3]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

/// Luma's standard deviation over a rectangle (how much detail shows).
fn contrast(img: &RgbaImage, r: Rect) -> f32 {
    let m = luma(mean(img, r));
    (pixels(img, r).map(|p| (luma(p) - m).powi(2)).sum::<f32>() / area(r)).sqrt()
}

/// Mean absolute difference over a rectangle (all channels).
fn diff(a: &RgbaImage, b: &RgbaImage, r: Rect) -> f32 {
    pixels(a, r)
        .zip(pixels(b, r))
        .map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).sum::<f32>())
        .sum::<f32>()
        / (area(r) * 3.0)
}

/// The mean absolute Laplacian of luma over a rectangle: how bumpy it is
/// from pixel to pixel.
fn roughness(img: &RgbaImage, (x0, y0, x1, y1): Rect) -> f32 {
    let l = |x: u32, y: u32| luma(pixels(img, (x, y, x + 1, y + 1)).next().unwrap());
    let mut sum = 0.0;
    for y in y0 + 1..y1 - 1 {
        for x in x0 + 1..x1 - 1 {
            sum += (4.0 * l(x, y) - l(x - 1, y) - l(x + 1, y) - l(x, y - 1) - l(x, y + 1)).abs();
        }
    }
    sum / ((x1 - x0 - 2) * (y1 - y0 - 2)) as f32
}

#[test]
fn fluid_3d_matches_its_reference() {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    match gfx::check("fluid-3d", &out) {
        Ok(report) => println!("{report}"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn the_pool_tints_shows_through_and_refracts_what_lies_behind_it() {
    let wet = frame(|_| {});
    let dry = frame(|s| s.fluids_3d.clear());
    let (w, d) = (mean(&wet, POOL), mean(&dry, POOL));
    assert!(
        w[2] - w[0] > d[2] - d[0] + 40.0,
        "the pool tints the tiles blue: {w:?} over {d:?}"
    );
    // The tiles show through, and more of them when the fluid is clearer.
    let shown = contrast(&wet, POOL);
    assert!(
        shown > 0.3 * contrast(&dry, POOL),
        "the tiles show through the pool: contrast {shown} of {}",
        contrast(&dry, POOL)
    );
    let clearer = contrast(&frame(|s| s.fluids_3d[0].opacity = 0.3), POOL);
    assert!(
        clearer > shown * 1.2,
        "a clearer pool shows more: {clearer} vs {shown}"
    );
    let straight = frame(|s| s.fluids_3d[0].refraction = 0.0);
    let bent = diff(&wet, &straight, POOL);
    assert!(bent > 1.5, "refraction bends the tiles: {bent}");
}

#[test]
fn the_drop_is_drawn_in_its_particles_colours() {
    let wet = frame(|_| {});
    let dry = frame(|s| s.fluids_3d.clear());
    let (w, d) = (mean(&wet, DROP), mean(&dry, DROP));
    assert!(
        w[0] > w[2] + 80.0 && (d[0] - d[2]).abs() < 30.0,
        "an orange drop over the grey wall: {w:?} over {d:?}"
    );
    // Its particles shade from orange at the top to red at the bottom.
    let (top, bottom) = (
        mean(&wet, (680, 165, 740, 185)),
        mean(&wet, (680, 235, 740, 255)),
    );
    assert!(
        top[1] > bottom[1] + 30.0,
        "orange above red: {top:?} over {bottom:?}"
    );
}

#[test]
fn smoothing_melts_the_spheres_into_one_surface() {
    let smooth = roughness(&frame(|_| {}), POOL);
    let beads = roughness(&frame(|s| s.fluids_3d[0].smoothing = 0.0), POOL);
    assert!(
        beads > smooth * 2.0,
        "unsmoothed spheres are bumpy: {beads} vs {smooth}"
    );
}

#[test]
fn a_local_light_lights_the_fluid() {
    let lit = frame(|_| {});
    let unlit = frame(|s| s.lights.clear());
    let change = diff(&lit, &unlit, DROP);
    assert!(
        change > 3.0,
        "the orange point light reaches the drop: {change}"
    );
}

#[test]
fn the_basin_wall_hides_the_fluid_behind_it() {
    let wet = frame(|_| {});
    let dry = frame(|s| s.fluids_3d.clear());
    // The pool reaches down to the front wall's top edge (row 470) ...
    let above = diff(&wet, &dry, (300, 440, 1000, 468));
    assert!(above > 10.0, "the pool is drawn above the wall: {above}");
    // ... and the wall's top and face, in front of it, are untouched.
    let wall = diff(&wet, &dry, (300, 474, 1000, 540));
    assert!(wall < 0.5, "the wall hides the pool: {wall}");
}
