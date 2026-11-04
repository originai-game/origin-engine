//! The stylisation's line, toon and paint (`MaterialStyle`, `Stylise`'s
//! paint): what each change must do on the GPU, by pixels measured against
//! the same scene drawn without it.
//!
//! - A metallic toon sphere bands as a matte one does (it was black but
//!   for its highlight: the toon diffuse was times 1 - metallic).
//! - A toon sphere's ambient light is one flat colour with
//!   `toon_ambient` 0 (it was the hemisphere's gradient, top to bottom).
//! - Outlines placed outside the silhouette ink the pixels round the
//!   object and leave its own (they were drawn inside only).
//! - The painterly filter keeps a material's detail with `paintDetail` 1
//!   while it paints the same texture at 0.

use glam::{Mat4, Vec3};
use og_render::{
    Antialias, Background, Camera, ColorSpace, DirectionalLight, Exposure, MaterialStyle, Mips,
    OutlinePlace, Paint, RenderDesc, RenderScene, RenderTarget, Renderer, StandardMaterial,
    TextureData, shapes,
};
use og_tests::scenes;

const W: u32 = 320;
const H: u32 = 180;

fn desc() -> RenderDesc {
    RenderDesc {
        width: W,
        height: H,
        antialias: Antialias::Off,
        ..scenes::base_desc()
    }
}

fn draw(renderer: &mut Renderer, scene: &RenderScene) -> Vec<u8> {
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("frame");
    renderer.read_offscreen().expect("capture").pixels
}

fn px(img: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * W + x) * 4) as usize;
    [img[i], img[i + 1], img[i + 2]]
}

fn luma(c: [u8; 3]) -> f32 {
    0.2126 * f32::from(c[0]) + 0.7152 * f32::from(c[1]) + 0.0722 * f32::from(c[2])
}

/// A sphere of `material` against black, the camera square on, the sun
/// from the upper left, no image-based light unless `env`.
fn sphere_scene(material: StandardMaterial, toon: bool, env: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let m = scene.add_standard_material(material);
    let sphere = scene.add_mesh(shapes::sphere(64, 48));
    scene.add_instance(sphere, m, Mat4::from_scale(Vec3::splat(2.0)));
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.6, 0.5, 0.65),
        intensity: 3.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.environment.intensity = env;
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 4.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.stylise.material = MaterialStyle {
        toon,
        ..Default::default()
    };
    scene
}

/// The pixels of the sphere's disc (inside 90 % of its radius on screen,
/// found from a white unlit draw of it).
fn disc(renderer: &mut Renderer) -> Vec<(u32, u32)> {
    let white = StandardMaterial {
        unlit: 1,
        ..Default::default()
    };
    let img = draw(renderer, &sphere_scene(white, false, 0.0));
    let on: Vec<(u32, u32)> = (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .filter(|&(x, y)| luma(px(&img, x, y)) > 128.0)
        .collect();
    assert!(on.len() > 1000, "the sphere covers the frame's middle");
    let (cx, cy) = on.iter().fold((0.0, 0.0), |(a, b), &(x, y)| {
        (
            a + x as f32 / on.len() as f32,
            b + y as f32 / on.len() as f32,
        )
    });
    let r = (on.len() as f32 / std::f32::consts::PI).sqrt() * 0.9;
    on.into_iter()
        .filter(|&(x, y)| (x as f32 - cx).hypot(y as f32 - cy) < r)
        .collect()
}

/// A metal bands under toon as a dielectric does, in its own colour: most
/// of its lit half is lit. Broken by the PBR diffuse's 1 - metallic in the
/// toon light: the sphere stayed black but for its highlight (under a
/// tenth of the disc lit).
#[test]
fn a_metallic_toon_sphere_bands() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let disc = disc(&mut renderer);
    let metal = StandardMaterial {
        color: Vec3::new(0.9, 0.7, 0.3),
        metalness: 1.0,
        roughness: 0.5,
        ..Default::default()
    };
    let img = draw(&mut renderer, &sphere_scene(metal, true, 0.0));
    let lit = disc
        .iter()
        .filter(|&&(x, y)| luma(px(&img, x, y)) > 20.0)
        .count() as f32
        / disc.len() as f32;
    println!("metal toon sphere: {:.3} of the disc lit", lit);
    assert!(lit > 0.35, "the metal bands: {lit:.3} of the disc lit");
}

/// With `toon_ambient` 0 a toon sphere lit by the hemisphere alone (no
/// sun) is one flat colour: its top and bottom agree within a few levels.
/// Broken by the PBR ambient: the hemisphere's white sky over a black
/// ground lit the top and left the bottom dark.
#[test]
fn toon_ambient_is_flat() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let disc = disc(&mut renderer);
    let mut scene = sphere_scene(
        StandardMaterial {
            color: Vec3::splat(0.8),
            ..Default::default()
        },
        true,
        0.0,
    );
    scene.light.intensity = 0.0;
    scene.hemisphere = ([1.0; 3], [0.0; 3], 2.0);
    let img = draw(&mut renderer, &scene);
    let l: Vec<f32> = disc.iter().map(|&(x, y)| luma(px(&img, x, y))).collect();
    let (lo, hi) = l
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    println!("toon ambient: luma {lo:.1} to {hi:.1}");
    assert!(lo > 10.0, "the ambient reaches the sphere ({lo})");
    assert!(hi - lo < 4.0, "one flat colour: {lo:.1} to {hi:.1}");
    // The PBR ambient (toon_ambient 1) keeps the gradient: the control.
    scene.stylise.material.toon_ambient = 1.0;
    let img = draw(&mut renderer, &scene);
    let l: Vec<f32> = disc.iter().map(|&(x, y)| luma(px(&img, x, y))).collect();
    let (lo, hi) = l
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(
        hi - lo > 30.0,
        "the smooth ambient's gradient: {lo:.1} to {hi:.1}"
    );
}

/// A glowing white box (lit, so the style reaches it: an unlit material
/// keeps its look) on a grey sky, outlined `width` px in red.
fn box_scene(place: OutlinePlace, width: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let white = scene.add_standard_material(StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::ONE,
        emissiveIntensity: 4.0,
        ..Default::default()
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    scene.add_instance(
        cube,
        white,
        Mat4::from_rotation_y(0.5) * Mat4::from_rotation_x(0.3),
    );
    scene.environment.background = Background::Color(Vec3::splat(0.2));
    scene.environment.intensity = 0.0;
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 4.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.stylise.material = MaterialStyle {
        outline_width: width,
        outline_color: [1.0, 0.0, 0.0],
        outline_place: place,
        ..Default::default()
    };
    scene
}

fn red(c: [u8; 3]) -> bool {
    c[0] > 150 && c[1] < 90 && c[2] < 90
}

/// `OutlinePlace::Outside`: the line inks the background round the box
/// (round at its corners, the jump flood's) and leaves the box's own
/// pixels white; inside does the opposite; the line outside has
/// intermediate coverage at its rim. Broken when the place was ignored:
/// the line was inside only (no red pixel off the box).
#[test]
fn outlines_outside_ink_round_the_silhouette() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let plain = draw(&mut renderer, &box_scene(OutlinePlace::Inside, 0.0));
    let on_box = |x, y| luma(px(&plain, x, y)) > 200.0;
    let count = |img: &[u8], inside: bool| {
        (0..H)
            .flat_map(|y| (0..W).map(move |x| (x, y)))
            .filter(|&(x, y)| on_box(x, y) == inside && red(px(img, x, y)))
            .count()
    };
    let outside = draw(&mut renderer, &box_scene(OutlinePlace::Outside, 4.0));
    let inside = draw(&mut renderer, &box_scene(OutlinePlace::Inside, 4.0));
    let (out_off, out_on) = (count(&outside, false), count(&outside, true));
    let (in_off, in_on) = (count(&inside, false), count(&inside, true));
    println!("outside: {out_off} off the box, {out_on} on it; inside: {in_off} off, {in_on} on");
    assert!(out_off > 300, "outside inks round the box ({out_off})");
    assert!(out_on < out_off / 20, "and not the box ({out_on})");
    assert!(
        in_on > 300 && in_off < in_on / 20,
        "inside: {in_on} on, {in_off} off"
    );
    // Anti-aliased: some pixels off the box are between the ground (grey,
    // 149 in sRGB) and the line (red, green 0): green part way down, red
    // above it. A hard edge leaves none.
    let partial = (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let c = px(&outside, x, y);
            !on_box(x, y) && c[1] > 20 && c[1] < 130 && c[0] > c[1] + 15
        })
        .count();
    println!("outside: {partial} pixels part way round the rim");
    assert!(partial > 20, "the rim is covered by degrees ({partial})");
}

/// A noise texture: each texel grey, 64 to 192, hashed.
fn noise_texture() -> TextureData {
    let (w, h) = (64u32, 64u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    let mut seed = 0x9e37_79b9u32;
    for _ in 0..w * h {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let v = 64 + (seed >> 25) as u8;
        rgba.extend_from_slice(&[v, v, v, 255]);
    }
    TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    }
}

/// Two noise-textured quads side by side, the left `paintDetail` 1, the
/// right 0, with `paint` (radius 5) or without.
fn quads(paint: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    let t = scene.add_texture(noise_texture());
    scene.set_texture_filter(t, og_render::TextureFilter::Nearest);
    let mat = |detail: f32| StandardMaterial {
        map: t.slot(),
        unlit: 1,
        paintDetail: detail,
        ..Default::default()
    };
    let kept = scene.add_standard_material(mat(1.0));
    let painted = scene.add_standard_material(mat(0.0));
    let quad = scene.add_mesh(shapes::plane_xy(1.6, 1.6));
    scene.add_instance(
        quad,
        kept,
        Mat4::from_translation(Vec3::new(-0.9, 0.0, 0.0)),
    );
    scene.add_instance(
        quad,
        painted,
        Mat4::from_translation(Vec3::new(0.9, 0.0, 0.0)),
    );
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    if paint {
        scene.stylise.paint = Some(Paint {
            radius: 5.0,
            anisotropy: 0.5,
            ..Paint::DEFAULT
        });
    }
    scene
}

/// The mean absolute difference over the middle of each quad between two
/// frames: (the kept quad's, the painted quad's).
fn quad_change(a: &[u8], b: &[u8]) -> (f32, f32) {
    let mean = |x0: u32, x1: u32| {
        let mut sum = 0.0;
        let mut n = 0.0;
        for y in H / 2 - 20..H / 2 + 20 {
            for x in x0..x1 {
                sum += (luma(px(a, x, y)) - luma(px(b, x, y))).abs();
                n += 1.0;
            }
        }
        sum / n
    };
    // At 3 m with the default 50° fov the quads' middles lie about 30 px
    // either side of the frame's quarter lines.
    (
        mean(W / 4 - 15, W / 4 + 15),
        mean(3 * W / 4 - 15, 3 * W / 4 + 15),
    )
}

/// `paintDetail` 1 keeps the material as drawn while the same texture at
/// 0 is painted into strokes. Broken by one radius for every pixel: both
/// quads were painted alike.
#[test]
fn paint_detail_keeps_a_material_drawn() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let plain = draw(&mut renderer, &quads(false));
    let painted = draw(&mut renderer, &quads(true));
    let (kept, smoothed) = quad_change(&plain, &painted);
    println!("mean change: detail 1 {kept:.2}, detail 0 {smoothed:.2}");
    assert!(
        smoothed > 8.0,
        "the noise is painted at detail 0 ({smoothed})"
    );
    assert!(
        kept < smoothed / 4.0,
        "detail 1 keeps it ({kept} against {smoothed})"
    );
}
