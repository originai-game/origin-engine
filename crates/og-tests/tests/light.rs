//! The depth wave's light (docs/design.md §10.7): the reference frames of rect
//! lights shaded with LTC (`rect-lights`), spot and point light cookies
//! (`light-cookies`), volumetric fog lit by the sun through a slotted wall
//! and by a spot (`volumetric-fog`), and natively a Cornell box baked by
//! og-bake with a light volume (`baked-cornell`); and from frames: the bake
//! bleeds red onto the white block beside the red wall and lights the
//! ceiling, which the unbaked box leaves dark.

use og_tests::gfx;

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

/// Every kind of lit batch draws its lightmap (debt render-light #2): the
/// box drawn with a physical ceiling, a compact back wall, an array-map tall
/// block and a material-shader short block matches the all-standard
/// `baked-cornell` over each of those parts, its bake drawn. A batch that
/// ignores its lightmap takes the probes or the sky instead: the ceiling and
/// the back wall, outside the light volume, the sky's blue-grey or near
/// black; the blocks the probes' coarser light, the tall block's side
/// with a third less red from the wall. On lavapipe (Mesa 22.3) every
/// physical pipeline crashes the driver's draw thread, main's physical
/// tests too, so this one is read on Metal. Then the whole frame: within
/// `baked-batches`' reference tolerance of the standard box's, which
/// `baked_cornell_matches_its_reference` holds to its reference (the two
/// frames were the same pixels, so `baked-batches` has no reference of its
/// own any more).
#[test]
#[ignore = "requires GPU"]
fn every_batch_kind_draws_its_lightmap() {
    let standard = gfx::render("baked-cornell").expect("render").image;
    let batches = gfx::render("baked-batches").expect("render").image;
    let tolerance = og_tests::scenes::build("baked-batches")
        .expect("scene")
        .tolerance;
    let whole = og_tests::compare::compare(
        &batches.pixels,
        &standard.pixels,
        standard.width,
        standard.height,
        &tolerance,
    )
    .expect("same size");
    println!(
        "whole frame against the standard batch's: rmse {:.5}, outliers {}",
        whole.rmse, whole.outliers
    );
    whole
        .check(&tolerance)
        .unwrap_or_else(|e| panic!("baked-batches against baked-cornell's frame: {e}"));
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
#[ignore = "requires GPU"]
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

/// A full custom lit shader lights a rect light as the standard material
/// does: as an area (the floor under the panel within a few percent of
/// the standard floor's) with its soft shadow (the post's foot dark).
/// Before the debt round 2 `custom_lit.wgsl` lit it as a point at its
/// centre without its shadow.
#[test]
#[ignore = "requires GPU"]
fn a_full_custom_shader_lights_rect_lights_with_their_shadow() {
    let read = |custom: bool, cast: bool| {
        let (scene, points) = if custom {
            og_tests::light_scenes::rect_shadow_custom_scene(cast)
        } else {
            og_tests::light_scenes::rect_shadow_scene(cast)
        };
        let mut built = og_tests::scenes::build("baked-cornell").expect("build");
        built.scene = scene;
        let img = gfx::render_built(&mut built).expect("render").image;
        if let Ok(dir) = std::env::var("OG_RECT_SHADOW_OUT") {
            gfx::save_png(
                std::path::Path::new(&format!("{dir}/rect-shadow-custom-{custom}-{cast}.png")),
                &img,
            )
            .expect("save");
        }
        points.map(|p| {
            let (x, y) = project(&built.scene.camera, p);
            let c = mean(&img, x - 4, y - 3, x + 4, y + 3);
            0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
        })
    };
    let [foot_std, tip_std] = read(false, false);
    let [foot_open, tip_open] = read(true, false);
    let [foot, tip] = read(true, true);
    println!(
        "standard: foot {foot_std:.1}, tip {tip_std:.1}; custom unshadowed: foot {foot_open:.1}, tip {tip_open:.1}; shadowed: foot {foot:.1}, tip {tip:.1}"
    );
    for (c, s) in [(foot_open, foot_std), (tip_open, tip_std)] {
        assert!(
            (c - s).abs() <= 0.06 * s + 2.0,
            "the custom floor is lit as the standard one ({c:.1} against {s:.1})"
        );
    }
    assert!(
        foot < 0.35 * foot_open,
        "no shadow at the post's foot ({foot:.1} against {foot_open:.1} unshadowed)"
    );
    assert!(
        tip / tip_open > foot / foot_open + 0.2,
        "the shadow does not soften toward its tip"
    );
}
