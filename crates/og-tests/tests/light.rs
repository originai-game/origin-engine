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
    let camera = scene.frame.camera;
    let draw = |r: &mut Renderer, s: &og_render::RenderScene| {
        for _ in 0..frames {
            r.render(s, &RenderTarget::Offscreen).expect("render");
        }
        r.read_offscreen().expect("capture")
    };
    let first = draw(&mut r, &scene);
    let saved = r.save_state();
    let baked = std::mem::take(&mut scene.lighting.baked);
    let unbaked = draw(&mut r, &scene);
    assert_ne!(unbaked.pixels, first.pixels, "unbaked draws otherwise");
    scene.frame.camera.position.x += 0.4;
    draw(&mut r, &scene);
    scene.frame.camera = camera;
    scene.lighting.baked = baked;
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
            let (x, y) = project(&built.scene.frame.camera, p);
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

/// Fog inside a region (og.FogVolume, og-render `FogVolume`) lies where its
/// og.Volume puts it and nowhere else. Seen from straight above, a box 10 m
/// long, turned 30° about y over a grey floor in a plain fog: black smoke
/// (scattering 0) darkens the floor under the turned box's end and leaves
/// the floor where the box's end would lie turned the other way as it was,
/// plain fog and all; a white mist is brighter than the smoke there; and a
/// sphere's fog fades over its fade. The shape is og_env's
/// `Volume::weight`, which the tick's regions read: the test places its
/// points with it.
///
/// Ways this could fail: the shader turns the box by the entity's
/// quaternion instead of its inverse (the two ends swap); the volumes'
/// medium replaces the scene's plain fog (the floor outside brightens);
/// the fog ignores its scattering; a fade is a hard edge.
#[test]
#[ignore = "requires GPU"]
fn fog_in_a_region_lies_where_its_volume_puts_it() {
    use glam::{Mat4, Quat, Vec3};
    use og_env::volume::{Placement, Volume};
    use og_render::{Camera, Fog, FogVolume, RenderScene, RenderTarget, Renderer, shapes};
    let camera = Camera {
        position: Vec3::new(0.0, 24.0, 0.0),
        target: Vec3::ZERO,
        up: Vec3::NEG_Z,
        ..Default::default()
    };
    let draw = |volumes: &[FogVolume]| {
        let mut scene = RenderScene::new();
        let floor = scene.add_mesh(shapes::plane(60.0, 1.0));
        let grey = scene.add_standard_material(og_render::StandardMaterial {
            color: Vec3::splat(0.6),
            roughness: 0.9,
            ..Default::default()
        });
        scene.add_instance(floor, grey, Mat4::IDENTITY);
        scene.atmosphere.fog = Some(Fog {
            density: 0.01,
            ..Default::default()
        });
        scene.atmosphere.fog_volumes = volumes.to_vec();
        scene.frame.camera = camera;
        let mut renderer =
            Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
        renderer
            .render_capture(&scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("capture")
    };
    let luma = |img: &og_render::RgbaImage, p: Vec3| {
        let (x, y) = project(&camera, p);
        let c = mean(img, x - 6, y - 6, x + 6, y + 6);
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    };
    let at = Placement {
        position: Vec3::new(0.0, 1.0, 0.0),
        quaternion: Quat::from_rotation_y(30f32.to_radians()),
        scale: Vec3::ONE,
    };
    let bar = Volume {
        size: Vec3::new(10.0, 2.0, 2.0),
        radius: 0.0,
        fade: 0.0,
    };
    let fog = |scattering: f32| FogVolume {
        at,
        volume: Some(bar),
        color: Vec3::ONE,
        density: 0.6,
        scattering,
    };
    // The bar's end, and where it would lie turned the other way.
    let end = at.quaternion * Vec3::new(4.0, 0.0, 0.0);
    let other = at.quaternion.inverse() * Vec3::new(4.0, 0.0, 0.0);
    assert_eq!(bar.weight(&at, end + at.position), 1.0);
    assert_eq!(bar.weight(&at, other + at.position), 0.0);
    let (clear, smoke, mist) = (draw(&[]), draw(&[fog(0.0)]), draw(&[fog(1.0)]));
    let (clear_end, smoke_end, mist_end) = (luma(&clear, end), luma(&smoke, end), luma(&mist, end));
    let (clear_other, smoke_other) = (luma(&clear, other), luma(&smoke, other));
    println!(
        "under the bar's end: clear {clear_end:.1}, smoke {smoke_end:.1}, mist {mist_end:.1}; beside it: clear {clear_other:.1}, smoke {smoke_other:.1}"
    );
    assert!(
        smoke_end < 0.75 * clear_end,
        "no smoke under the bar's end ({smoke_end:.1} against {clear_end:.1})"
    );
    assert!(
        (smoke_other - clear_other).abs() < 1.5,
        "the smoke changed the floor outside it ({smoke_other:.1} against {clear_other:.1})"
    );
    assert!(
        mist_end > smoke_end + 8.0,
        "a white mist is no brighter than black smoke ({mist_end:.1} against {smoke_end:.1})"
    );
    // A ball of 3 m fading over 4: halfway through the fade the floor is
    // darkened, less than under the ball and more than past the fade.
    let ball_at = Placement::at(Vec3::new(0.0, 0.0, 0.0));
    let ball = Volume {
        size: Vec3::ONE,
        radius: 3.0,
        fade: 4.0,
    };
    let faded = draw(&[FogVolume {
        at: ball_at,
        volume: Some(ball),
        color: Vec3::ONE,
        density: 0.6,
        scattering: 0.0,
    }]);
    let [inside, half, past] = [0.0, 5.0, 9.0].map(|x| {
        let p = Vec3::new(x, 0.0, 0.0);
        luma(&faded, p) / luma(&clear, p)
    });
    println!(
        "a fading ball: {inside:.2} of the clear floor inside, {half:.2} halfway through its fade, {past:.2} past it"
    );
    assert!(
        inside < half - 0.1 && half < past - 0.1 && past > 0.97,
        "the ball's fog does not fade over its fade ({inside:.2}, {half:.2}, {past:.2})"
    );
}
