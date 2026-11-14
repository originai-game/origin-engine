//! The physical material lobes (`og-render/src/shading/physical.rs`): the
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
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.6, 0.7, 0.6),
        intensity: 3.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        intensity: 0.15,
        background: Background::Color(Vec3::ZERO),
        ..Default::default()
    };
    scene.frame.camera = Camera {
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
        scene.lighting.environment.intensity = 0.4;
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

/// A ball of radius `radius` at the frame's centre under `m`, seen by
/// `studio`'s camera: per pixel on it, the cosine between its normal and
/// the light and its colour.
fn ball_falloff(m: StandardMaterial, radius: f32) -> Vec<(f32, [f32; 3])> {
    ball_falloff_in(studio(), m, radius)
}

/// [`ball_falloff`] in `scene`.
fn ball_falloff_in(
    mut scene: RenderScene,
    m: StandardMaterial,
    radius: f32,
) -> Vec<(f32, [f32; 3])> {
    let sphere = scene.add_mesh(shapes::sphere(96, 64));
    let h = scene.add_standard_material(m);
    scene.add_instance(sphere, h, Mat4::from_scale(Vec3::splat(2.0 * radius)));
    let img = frame(&scene);
    let l = scene.lighting.light.direction_to_light.normalize();
    let eye = scene.frame.camera.position;
    let og_render::Projection::Perspective { fov_y_degrees } = scene.frame.camera.projection else {
        panic!("a perspective camera");
    };
    let half = (fov_y_degrees.to_radians() * 0.5).tan();
    let (w, h) = (img.width as f32, img.height as f32);
    let mut out = Vec::new();
    for y in 0..img.height {
        for x in 0..img.width {
            let ray = Vec3::new(
                ((x as f32 + 0.5) / w * 2.0 - 1.0) * half * w / h,
                (1.0 - (y as f32 + 0.5) / h * 2.0) * half,
                -1.0,
            )
            .normalize();
            // Where the ray meets the ball, short of its rim.
            let b = eye.dot(ray);
            let disc = b * b - (eye.length_squared() - radius * radius);
            if disc < 0.02 * radius * radius {
                continue;
            }
            let n = (eye + ray * (-b - disc.sqrt())) / radius;
            out.push((n.dot(l), px(&img, x, y)));
        }
    }
    out
}

/// The mean colour where the cosine to the light is within `x0..x1`.
fn falloff_mean(pixels: &[(f32, [f32; 3])], x0: f32, x1: f32) -> [f32; 3] {
    let (mut sum, mut n) = ([0.0f32; 3], 0.0f32);
    for (x, p) in pixels.iter().filter(|(x, _)| *x >= x0 && *x < x1) {
        let _ = x;
        for c in 0..3 {
            sum[c] += p[c];
        }
        n += 1.0;
    }
    assert!(n > 50.0, "{n} pixels between {x0} and {x1}");
    sum.map(|s| s / n)
}

/// Light spreads under a subsurface material (`StandardMaterial::subsurface`,
/// subsurface.wgsl), by the surface's curvature:
/// - past the terminator a white ball is lit where the plain one is dark,
///   most in the channel that travels farthest, and its brightest part is
///   dimmer by the light that went there, the same channel most;
/// - a ball of half the radius, twice as curved, bleeds farther;
/// - with one distance for every channel the bleed has no colour;
/// - no distance, or no `subsurface`, is the plain material.
#[test]
#[ignore = "requires GPU"]
fn light_spreads_under_a_subsurface_surface() {
    let white = StandardMaterial {
        color: Vec3::splat(0.8),
        roughness: 0.9,
        ..Default::default()
    };
    let skin = StandardMaterial {
        subsurface: 1.0,
        subsurfaceRadius: Vec3::new(0.12, 0.04, 0.012),
        ..white
    };
    let plain = ball_falloff(white, 0.5);
    let soft = ball_falloff(skin, 0.5);
    let small = ball_falloff(skin, 0.25);
    let grey = ball_falloff(
        StandardMaterial {
            subsurfaceRadius: Vec3::splat(0.12),
            ..skin
        },
        0.5,
    );
    let none = ball_falloff(
        StandardMaterial {
            subsurfaceRadius: Vec3::ZERO,
            ..skin
        },
        0.5,
    );
    let past = |f: &[(f32, [f32; 3])]| falloff_mean(f, -0.3, -0.1);
    let peak = |f: &[(f32, [f32; 3])]| falloff_mean(f, 0.9, 1.0);
    let (dark, lit) = (past(&plain), past(&soft));
    println!("past the terminator: plain {dark:?}, subsurface {lit:?}");
    // Red travels ten times as far as blue: it bleeds most, blue hardly.
    let bled = [0, 1, 2].map(|c| lit[c] - dark[c]);
    assert!(
        bled[0] > 50.0 && bled[0] > 3.0 * bled[1] && bled[1] > 3.0 * bled[2] && bled[2] > 0.5,
        "{bled:?}"
    );
    // What went past the terminator left the lit side.
    let (full, less) = (peak(&plain), peak(&soft));
    println!("facing the light: plain {full:?}, subsurface {less:?}");
    let lost = [0, 1, 2].map(|c| full[c] - less[c]);
    assert!(
        lost[0] > 8.0 && lost[0] > 3.0 * lost[1] && lost[1] > lost[2] && lost[2] >= 0.0,
        "{lost:?}"
    );
    // Twice the curvature: farther into the shadow, in every channel.
    let more = past(&small);
    println!("half the radius: {more:?}");
    assert!(
        more[0] > lit[0] + 15.0 && more[1] > lit[1] + 15.0 && more[2] > lit[2] + 2.0,
        "{more:?} against {lit:?}"
    );
    // One distance for every channel: red's bleed in each.
    let even = past(&grey);
    assert!((even[0] - lit[0]).abs() < 0.5, "{even:?}");
    assert!(even[1] > lit[0] && even[2] > even[1], "{even:?}");
    // Under a sun that casts shadows the ball's own shadow does not take
    // the bleed back: light entering beside a point reaches it.
    let mut shadowed = studio();
    shadowed.lighting.light.cast_shadows = true;
    let kept = past(&ball_falloff_in(shadowed, skin, 0.5));
    println!("with the sun's shadows: {kept:?}");
    assert!(kept[0] > dark[0] + 0.8 * bled[0], "{kept:?} of {lit:?}");
    // No distance: the plain material at every pixel, to the last bit.
    let worst = none
        .iter()
        .zip(&plain)
        .flat_map(|(a, b)| (0..3).map(move |c| (a.1[c] - b.1[c]).abs()))
        .fold(0.0, f32::max);
    assert!(worst <= 1.0, "scattering nowhere differs by {worst}");
}

/// With a `thickness`, light from behind comes through a subsurface
/// material: a sheet lit from its far side glows, red most, less the
/// thicker it is, and not at all without `subsurface`; lit from the front
/// a flat sheet, which does not curve, is shaded as the plain one.
#[test]
#[ignore = "requires GPU"]
fn light_comes_through_a_thin_subsurface_sheet() {
    let sheet = |m: StandardMaterial, to_light: Vec3| {
        let mut scene = studio();
        scene.lighting.light.direction_to_light = to_light;
        let quad = scene.add_mesh(shapes::plane_xy(1.0, 1.0));
        let h = scene.add_standard_material(m);
        scene.add_instance(quad, h, Mat4::IDENTITY);
        let img = frame(&scene);
        mean(&img, area(600, 680, 320, 400))
    };
    let white = StandardMaterial {
        color: Vec3::splat(0.8),
        roughness: 0.9,
        ..Default::default()
    };
    let thin = StandardMaterial {
        subsurface: 1.0,
        subsurfaceRadius: Vec3::new(0.02, 0.008, 0.003),
        thickness: 0.01,
        ..white
    };
    let thick = StandardMaterial {
        thickness: 0.05,
        ..thin
    };
    let solid = StandardMaterial {
        thickness: 0.0,
        ..thin
    };
    let behind = Vec3::new(0.2, 0.3, -1.0);
    let (dark, glow, dim, shut) = (
        sheet(white, behind),
        sheet(thin, behind),
        sheet(thick, behind),
        sheet(solid, behind),
    );
    println!("lit from behind: plain {dark:?}, 1 cm {glow:?}, 5 cm {dim:?}, no thickness {shut:?}");
    // 1 cm lets 0.79, 0.57 and 0.26 of red, green and blue through; 5 cm
    // 0.35, 0.09 and under 0.01.
    assert!(
        glow[0] > dark[0] + 60.0 && glow[0] > glow[1] + 10.0 && glow[1] > glow[2] + 10.0,
        "{glow:?}"
    );
    assert!(
        dim[0] > dark[0] + 20.0 && dim[0] < glow[0] - 20.0 && dim[2] < dark[2] + 3.0,
        "{dim:?}"
    );
    assert_eq!(shut, dark);
    let front = Vec3::new(0.2, 0.3, 1.0);
    let (a, b) = (sheet(white, front), sheet(thin, front));
    println!("lit from the front: plain {a:?}, subsurface {b:?}");
    assert!((0..3).all(|c| (a[c] - b[c]).abs() < 0.6), "{a:?} {b:?}");
}
