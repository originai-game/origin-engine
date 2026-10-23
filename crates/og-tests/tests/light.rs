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

/// Two boxes each baked on its own, as two baked scenes loaded together
/// (the right one's red and green walls swapped): each box draws its own
/// pages (lightmap.rs binds a batch's own at group 3), so the left box's
/// short block and the right box's tall block are green on the sides
/// facing their green walls. Drawn with one set of pages for both, as the
/// old renderer did, the right box samples the left's texels: red there.
#[test]
fn two_baked_scenes_draw_their_own_pages() {
    let mut built = og_tests::scenes::build("baked-cornell").expect("build");
    built.scene = og_tests::light_scenes::baked_cornell_pair_scene().expect("pair");
    let img = gfx::render_built(&mut built).expect("render").image;
    let left = mean(&img, 522, 440, 552, 530);
    let right = mean(&img, 724, 340, 750, 500);
    println!("left block side {left:?}, right block side {right:?}");
    for (c, which) in [
        (left, "the left box's short block"),
        (right, "the right box's tall block"),
    ] {
        assert!(
            c[1] > 1.3 * c[0] && c[1] > 1.3 * c[2],
            "{which} takes green from the wall beside it ({c:?})"
        );
    }
}

#[test]
fn baked_batches_matches_its_reference() {
    check("baked-batches");
}

/// Every kind of lit batch draws its lightmap (debt render-light #2): the
/// box drawn with a physical ceiling, a compact back wall, an array-map tall
/// block and a material-shader short block matches the all-standard
/// `baked-cornell` over each of those parts, its bake drawn. A batch that
/// ignores its lightmap takes the probes or the sky instead: the ceiling and
/// the back wall, outside the light volume, the sky's blue-grey or near
/// black; the blocks the probes' coarser light, the tall block's side
/// with a third less red from the wall. On lavapipe (Mesa 22.3) every
/// physical pipeline crashes the driver's draw thread, main's physical
/// tests too, so this one is read on Metal.
#[test]
fn every_batch_kind_draws_its_lightmap() {
    let standard = gfx::render("baked-cornell").expect("render").image;
    let batches = gfx::render("baked-batches").expect("render").image;
    for (part, [x0, y0, x1, y1]) in [
        ("the physical ceiling", [480, 16, 800, 40]),
        ("the compact back wall", [660, 150, 840, 280]),
        ("the array-map tall block's red side", [472, 340, 488, 580]),
        (
            "the material-shader short block's front",
            [650, 480, 820, 640],
        ),
    ] {
        let (a, b) = (
            mean(&standard, x0, y0, x1, y1),
            mean(&batches, x0, y0, x1, y1),
        );
        let off = (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0, f32::max);
        println!("{part}: standard {a:?}, batches {b:?}");
        assert!(
            off < 3.0,
            "{part} draws its lightmap: {b:?} against the standard batch's {a:?}"
        );
    }
}

/// The renderer's state restore keeps baked data (debt render-light #7): a
/// baked frame, a renderer snapshot, frames drawn unbaked (the pages and
/// uvs let go) and from elsewhere, the restore: the baked frame again, byte
/// for byte.
#[test]
fn a_renderer_restore_keeps_the_baked_frame() {
    use og_render::{RenderTarget, Renderer};
    let built = og_tests::scenes::build("baked-cornell").expect("build");
    let mut r = Renderer::new_blocking(&built.desc).expect("renderer");
    let frames = built.frames.max(1);
    let mut scene = built.scene;
    let camera = scene.camera;
    let draw = |r: &mut Renderer, s: &og_render::RenderScene| {
        for _ in 0..frames {
            r.render(s, &RenderTarget::Offscreen).expect("render");
        }
        r.read_offscreen().expect("capture")
    };
    let first = draw(&mut r, &scene);
    let saved = r.save_state();
    let baked = std::mem::take(&mut scene.baked);
    let unbaked = draw(&mut r, &scene);
    assert_ne!(unbaked.pixels, first.pixels, "unbaked draws otherwise");
    scene.camera.position.x += 0.4;
    draw(&mut r, &scene);
    scene.camera = camera;
    scene.baked = baked;
    r.restore_state(saved);
    let again = draw(&mut r, &scene);
    assert!(
        again.pixels == first.pixels,
        "the restored baked frame differs"
    );
}

/// The froxel fog has no vertical streaks (debt render-light #6): over the
/// fog in front of the left pier, each column's mean luma (over 300 rows)
/// keeps within a fraction of a level of its neighbours' 9-column mean. A
/// froxel lit at a single depth inside a slice that a shaft crosses
/// (main's inject) flips between lit and shadowed column by column: the
/// streaks the old frame showed.
#[test]
fn the_volumetric_fog_has_no_froxel_streaks() {
    let img = gfx::render("volumetric-fog").expect("render").image;
    let (x0, x1, y0, y1) = (60u32, 440u32, 200u32, 500u32);
    let columns: Vec<f32> = (x0..x1)
        .map(|x| {
            let c = mean(&img, x, y0, x + 1, y1);
            0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
        })
        .collect();
    let residual = (4..columns.len() - 4)
        .map(|i| (columns[i] - columns[i - 4..=i + 4].iter().sum::<f32>() / 9.0).abs())
        .sum::<f32>()
        / (columns.len() - 8) as f32;
    println!("column residual {residual:.3}");
    assert!(
        residual < 0.1,
        "streaks: {residual:.3} levels between neighbouring columns"
    );
}

/// WebGL2's raymarch lights its fog with the point, spot and rect lights
/// too (debt render-light #5), drawn here on this device
/// (`RenderDesc::volumetric`): the orange spot's cone in the fog matches
/// the froxel grid's within a few levels. Lit by the sun alone, as main's
/// raymarch was, the cone is the sun's grey: 76,73,69 against 100,89,79.
#[test]
fn the_raymarch_lights_the_fog_with_the_spot() {
    let froxel = gfx::render("volumetric-fog").expect("render").image;
    let mut built = og_tests::scenes::build("volumetric-fog").expect("build");
    built.desc.volumetric = Some(og_render::VolumetricMethod::Raymarch);
    let rendered = gfx::render_built(&mut built).expect("render");
    assert_eq!(
        (rendered.info.volumetric, rendered.info.volumetric_lights),
        (
            og_render::VolumetricMethod::Raymarch,
            og_render::RAYMARCH_LIGHTS
        )
    );
    let (a, b) = (
        mean(&froxel, 1040, 120, 1090, 250),
        mean(&rendered.image, 1040, 120, 1090, 250),
    );
    println!("spot cone: froxel {a:?}, raymarch {b:?}");
    for k in 0..3 {
        assert!(
            (a[k] - b[k]).abs() < 6.0,
            "the raymarch's cone {b:?} against the froxel grid's {a:?}"
        );
    }
}

/// Where a world point lands on a 1280x720 frame of `camera`.
fn project(camera: &og_render::Camera, p: glam::Vec3) -> (u32, u32) {
    let view = glam::camera::rh::view::look_at_mat4(camera.position, camera.target, camera.up);
    let og_render::Projection::Perspective { fov_y_degrees } = camera.projection else {
        unreachable!("perspective cameras only")
    };
    let proj = glam::camera::rh::proj::directx::perspective(
        fov_y_degrees.to_radians(),
        1280.0 / 720.0,
        camera.near,
        camera.far,
    );
    let c = proj * view * p.extend(1.0);
    let ndc = c.truncate() / c.w;
    (
        ((ndc.x * 0.5 + 0.5) * 1280.0) as u32,
        ((0.5 - ndc.y * 0.5) * 720.0) as u32,
    )
}

/// Terrain and its scattered grass light a rect light as an area (debt
/// render-light #3): an upright one-sided panel facing left lights the
/// meadow to its left and leaves the meadow behind it, on its right,
/// dark. Lit as a point at its centre (main's terrain and scatter
/// shaders), both sides take the same light.
#[test]
fn terrain_and_scatter_light_rect_lights_as_areas() {
    for grass in [false, true] {
        let (scene, [left, right]) = og_tests::light_scenes::rect_meadow_scene(grass);
        let mut built = og_tests::scenes::build("baked-cornell").expect("build");
        built.scene = scene;
        let img = gfx::render_built(&mut built).expect("render").image;
        let luma = |p: glam::Vec3| {
            let (x, y) = project(&built.scene.camera, p);
            let c = mean(&img, x - 12, y - 6, x + 12, y + 6);
            0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
        };
        let (lit, behind) = (luma(left), luma(right));
        println!("grass {grass}: left of the panel {lit:.1}, behind it {behind:.1}");
        assert!(
            lit > 2.0 * behind + 4.0,
            "grass {grass}: the meadow behind the panel is lit ({behind:.1} against {lit:.1} in front)"
        );
    }
}
