//! The physical material lobes (`og-render/src/physical.rs`): the
//! `materials-physical` reference board, and from frames: each lobe does
//! what it is for and its map steers it, transmission shows the scene
//! behind (blurred by roughness, tinted by absorption), a physical
//! material with its lobes off shades as the standard one, and skinned
//! physical materials draw.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, RenderScene, RenderTarget,
    Renderer, RgbaImage, StandardMaterial, shapes,
};
use og_tests::physical_scenes::stripes;
use og_tests::scenes;

fn frame(scene: &RenderScene) -> RgbaImage {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.read_offscreen().expect("capture")
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    [p[0] as f32, p[1] as f32, p[2] as f32]
}

/// The pixels of a box (x0..x1, y0..y1).
fn area(x0: u32, x1: u32, y0: u32, y1: u32) -> impl Iterator<Item = (u32, u32)> {
    (y0..y1).flat_map(move |y| (x0..x1).map(move |x| (x, y)))
}

fn mean(img: &RgbaImage, pixels: impl Iterator<Item = (u32, u32)>) -> [f32; 3] {
    let (mut sum, mut n) = ([0.0f32; 3], 0.0);
    for (x, y) in pixels {
        let p = px(img, x, y);
        for c in 0..3 {
            sum[c] += p[c];
        }
        n += 1.0;
    }
    sum.map(|s| s / n)
}

/// One object on black, lit from the upper left, seen from +z (a unit
/// sphere is a disc of radius 97 pixels at the frame's centre).
fn studio() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.6, 0.7, 0.6),
        intensity: 3.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment = Environment {
        intensity: 0.15,
        background: Background::Color(Vec3::ZERO),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 4.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// A glass ball (radius 200 pixels) in front of a backdrop split red over
/// green: the backdrop shows through it, bent by its thickness, a rough
/// glass blurs it, and absorption tints it; a white opaque ball shows
/// none of it.
#[test]
#[ignore = "requires GPU"]
fn transmission_refracts_the_scene_behind() {
    // `bands`: the backdrop in 0.25 m bands instead of halves.
    let make_on = |m: StandardMaterial, bands: bool| {
        let mut scene = studio();
        scene.environment.intensity = 0.4;
        let (height, rows) = if bands { (0.25, 64) } else { (8.0, 2) };
        let wall = scene.add_mesh(shapes::cuboid(Vec3::new(8.0, height, 0.1)));
        let red = scene.add_standard_material(StandardMaterial::basic(Vec3::new(1.0, 0.05, 0.05)));
        let green =
            scene.add_standard_material(StandardMaterial::basic(Vec3::new(0.05, 1.0, 0.05)));
        for i in 0..rows {
            let y = (i as f32 - rows as f32 / 2.0 + 0.5) * height;
            let h = if i % 2 == 1 { red } else { green };
            scene.add_instance(wall, h, Mat4::from_translation(Vec3::new(0.0, y, -3.0)));
        }
        let sphere = scene.add_mesh(shapes::sphere(64, 48));
        let h = scene.add_standard_material(m);
        scene.add_instance(sphere, h, Mat4::from_scale(Vec3::splat(2.0)));
        frame(&scene)
    };
    let make = |m| make_on(m, false);
    let glass = StandardMaterial {
        roughness: 0.02,
        transmission: 1.0,
        thickness: 1.0,
        ..Default::default()
    };
    let clear = make(glass);
    let opaque = make(StandardMaterial {
        roughness: 0.02,
        ..Default::default()
    });
    let rough = make(StandardMaterial {
        roughness: 0.6,
        ..glass
    });
    let amber = make(StandardMaterial {
        attenuationColor: Vec3::new(1.0, 0.5, 0.05),
        attenuationDistance: 0.3,
        ..glass
    });

    let (upper, lower) = (|| area(610, 670, 220, 280), || area(610, 670, 440, 500));
    let (cu, cl) = (mean(&clear, upper()), mean(&clear, lower()));
    println!("clear glass: upper {cu:?}, lower {cl:?}");
    assert!(
        cu[0] > cu[1] + 40.0,
        "the upper half shows the red behind it"
    );
    assert!(cl[1] > cl[0] + 40.0, "the lower half shows the green");
    let (ou, ol) = (mean(&opaque, upper()), mean(&opaque, lower()));
    assert!(
        (ou[1] - ou[0]).abs() < 30.0 && (ol[0] - ol[1]).abs() < 30.0,
        "opaque: {ou:?} {ol:?}"
    );
    // The volume bends the view: behind bands, a thick ball shows other
    // bands than a thin wall does (where its surface tilts).
    let thin = make_on(
        StandardMaterial {
            thickness: 0.0,
            ..glass
        },
        true,
    );
    let thick = make_on(glass, true);
    let swapped = area(500, 780, 200, 520)
        .filter(|&(x, y)| {
            let (a, b) = (px(&thin, x, y), px(&thick, x, y));
            (a[0] - a[1]) * (b[0] - b[1]) < -400.0
        })
        .count();
    println!("pixels whose band flips with thickness: {swapped}");
    assert!(swapped > 5000, "thickness refracts");
    // Roughness blurs: the contrast between the halves drops.
    let (ru, rl) = (mean(&rough, upper()), mean(&rough, lower()));
    let contrast = |u: [f32; 3], l: [f32; 3]| (u[0] - u[1]) + (l[1] - l[0]);
    println!("rough glass: upper {ru:?}, lower {rl:?}");
    assert!(
        contrast(ru, rl) < 0.7 * contrast(cu, cl),
        "rough glass blurs"
    );
    // Absorption tints what is seen through the volume: blue and green go.
    let (a, c) = (mean(&amber, upper()), mean(&clear, upper()));
    println!("upper: clear {c:?}, amber {a:?}");
    assert!(a[1] < 0.7 * c[1] && a[2] < 0.7 * c[2], "the volume absorbs");
}

/// The transmission map: where it is black the glass is opaque.
#[test]
#[ignore = "requires GPU"]
fn a_transmission_map_masks_the_glass() {
    let make = |map: bool| {
        let mut scene = studio();
        let wall = scene.add_mesh(shapes::cuboid(Vec3::new(8.0, 8.0, 0.1)));
        let red = scene.add_standard_material(StandardMaterial::basic(Vec3::new(1.0, 0.05, 0.05)));
        scene.add_instance(wall, red, Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0)));
        let quad = scene.add_mesh(shapes::cuboid(Vec3::new(2.0, 2.0, 0.05)));
        let stripes = scene.add_texture(stripes(256, 2.0, false));
        let h = scene.add_standard_material(StandardMaterial {
            roughness: 0.3,
            transmission: 1.0,
            transmissionMap: if map { stripes.slot() } else { 0 },
            ..Default::default()
        });
        scene.add_instance(quad, h, Mat4::IDENTITY);
        frame(&scene)
    };
    let (plain, masked) = (make(false), make(true));
    // Across the quad's front: every column red without the map, white
    // and red stripes with it.
    let redness = |img: &RgbaImage, x: u32| {
        let [r, g, _] = mean(img, area(x, x + 4, 340, 380));
        r - g
    };
    let columns: Vec<u32> = (0..16).map(|i| 470 + i * 21).collect();
    let plain_red = columns
        .iter()
        .filter(|&&x| redness(&plain, x) > 60.0)
        .count();
    let masked_red = columns
        .iter()
        .filter(|&&x| redness(&masked, x) > 60.0)
        .count();
    println!(
        "red columns: no map {plain_red}, striped map {masked_red} of {}",
        columns.len()
    );
    assert!(plain_red >= 14, "without the map the glass shows the wall");
    assert!(
        (4..=12).contains(&masked_red),
        "the map's black stripes are opaque"
    );
}
