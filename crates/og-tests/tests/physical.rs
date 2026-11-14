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
fn ball_falloff_in(scene: RenderScene, m: StandardMaterial, radius: f32) -> Vec<(f32, [f32; 3])> {
    let l = scene.lighting.light.direction_to_light.normalize();
    ball_normals(scene, m, radius)
        .into_iter()
        .map(|(n, c)| (n.dot(l), c))
        .collect()
}

/// A ball of radius `radius` at the frame's centre under `m` in `scene`:
/// per pixel on it, its normal there and its colour. The ball's poles
/// are on y, its uv's v running from the top one down.
fn ball_normals(mut scene: RenderScene, m: StandardMaterial, radius: f32) -> Vec<(Vec3, [f32; 3])> {
    let sphere = scene.add_mesh(shapes::sphere(96, 64));
    let h = scene.add_standard_material(m);
    scene.add_instance(sphere, h, Mat4::from_scale(Vec3::splat(2.0 * radius)));
    let img = frame(&scene);
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
            out.push((n, px(&img, x, y)));
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

/// The linear light of an 8-bit channel.
fn linear(c: f32) -> f32 {
    (c / 255.0).powf(2.2)
}

/// Hair (`StandardMaterial::hair`, hair.wgsl) on a ball whose strands run
/// down its meridians, lit from 40 degrees above the camera:
/// - black hair shows one highlight, white, in a band across the strands
///   where a glossy ball shows a spot, and nearer the roots than the
///   mirror's (the cuticle's tilt);
/// - coloured hair shows a second one nearer the tips, in its colour.
#[test]
#[ignore = "requires GPU"]
fn hair_shines_in_two_bands_across_its_strands() {
    let ball = |m: StandardMaterial| {
        let mut scene = studio();
        scene.lighting.environment.intensity = 0.0;
        scene.lighting.light.intensity = 1.5;
        let up = 40f32.to_radians();
        scene.lighting.light.direction_to_light = Vec3::new(0.0, up.sin(), up.cos());
        ball_normals(scene, m, 1.0)
    };
    let strands = |color: Vec3, hair: f32| StandardMaterial {
        color,
        roughness: 0.2,
        hair,
        ..Default::default()
    };
    // The colour down the meridian at longitude `lon` (degrees; 0 faces
    // the camera), per degree of latitude from -10 to 50.
    let meridian = |f: &[(Vec3, [f32; 3])], lon: f32| -> Vec<(f32, [f32; 3])> {
        (-10..=50)
            .map(|lat| {
                let lat = lat as f32;
                let (mut sum, mut n) = ([0.0f32; 3], 0.0f32);
                for (p, c) in f {
                    let (la, lo) = (p.y.asin().to_degrees(), p.x.atan2(p.z).to_degrees());
                    if (la - lat).abs() < 0.75 && (lo - lon).abs() < 1.5 {
                        for i in 0..3 {
                            sum[i] += c[i];
                        }
                        n += 1.0;
                    }
                }
                assert!(n > 3.0, "{n} pixels at {lat}, {lon}");
                (lat, sum.map(|s| s / n))
            })
            .collect()
    };
    // Where a profile peaks (the mean latitude weighted by what is over
    // half its peak) and how high.
    let peak = |profile: &[(f32, f32)]| {
        let top = profile.iter().map(|p| p.1).fold(0.0, f32::max);
        let (mut at, mut w) = (0.0, 0.0);
        for (lat, v) in profile.iter().filter(|p| p.1 > 0.5 * top) {
            at += lat * v;
            w += v;
        }
        (at / w.max(1e-6), top)
    };
    let red = |m: &[(f32, [f32; 3])]| -> Vec<(f32, f32)> {
        m.iter().map(|(lat, c)| (*lat, linear(c[0]))).collect()
    };
    let black = ball(strands(Vec3::ZERO, 1.0));
    let glossy = ball(strands(Vec3::ZERO, 0.0));
    let (hair_front, glossy_front) = (meridian(&black, 0.0), meridian(&glossy, 0.0));
    let (hair_at, hair_top) = peak(&red(&hair_front));
    let (mirror_at, mirror_top) = peak(&red(&glossy_front));
    println!(
        "black hair peaks {hair_top:.3} at {hair_at:.1}, the glossy ball {mirror_top:.3} at {mirror_at:.1}"
    );
    // The cuticle's tilt, 0.035 rad, leans the first highlight twice it
    // towards the root: 4 degrees.
    assert!(
        (2.5..5.5).contains(&(hair_at - mirror_at)),
        "{hair_at} against {mirror_at}"
    );
    // White: the strand's surface has no colour.
    let worst = hair_front
        .iter()
        .map(|(_, c)| (c[0] - c[1]).abs().max((c[0] - c[2]).abs()))
        .fold(0.0, f32::max);
    assert!(hair_top > 0.05 && worst < 1.5, "{hair_top}, {worst}");
    // A band across the strands: 30 degrees round the ball it is as
    // bright; the glossy ball's spot is gone.
    for lon in [-30.0, 30.0] {
        let (_, side) = peak(&red(&meridian(&black, lon)));
        let (_, spot) = peak(&red(&meridian(&glossy, lon)));
        println!("at {lon}: hair {side:.3}, glossy {spot:.4}");
        assert!(side > 0.6 * hair_top, "{side} of {hair_top} at {lon}");
        assert!(spot < 0.02 * mirror_top, "{spot} of {mirror_top} at {lon}");
    }
    // Coloured hair: what it adds over black hair is the light between
    // the strands, which varies slowly, and a second highlight.
    let color = Vec3::new(0.6, 0.3, 0.1);
    let brown = meridian(&ball(strands(color, 1.0)), 0.0);
    let over = |ch: usize| -> Vec<(f32, f32)> {
        let add: Vec<(f32, f32)> = brown
            .iter()
            .zip(&hair_front)
            .map(|(a, b)| (a.0, linear(a.1[ch]) - linear(b.1[ch])))
            .collect();
        // Less the slow part: the line through its two ends.
        let (first, last) = (add[0], add[add.len() - 1]);
        add.iter()
            .map(|(lat, v)| {
                let t = (lat - first.0) / (last.0 - first.0);
                (*lat, v - (first.1 + (last.1 - first.1) * t))
            })
            .collect()
    };
    let (second_at, second_red) = peak(&over(0));
    let (_, second_blue) = peak(&over(2));
    println!("the second highlight: {second_red:.3} red, {second_blue:.3} blue at {second_at:.1}");
    // Four times the tilt towards the tip, split between the two angles: 4 degrees.
    assert!(
        (2.5..6.5).contains(&(mirror_at - second_at)),
        "{second_at} against {mirror_at}"
    );
    // In the hair's colour: twice through the strand.
    assert!(
        second_red > 0.1 && second_red > 2.5 * second_blue,
        "{second_red}, {second_blue}"
    );
}

/// Light behind hair `thickness` deep comes through it in its colour,
/// once through every strand on its way (hair.wgsl `hair_path`), and not
/// without a `thickness`:
/// - a head-sized ball, a closed surface, glows at its rim, where the
///   light's chord stays in the hair, and not across its middle, where
///   the chord dips under it;
/// - under a sun that casts shadows the rim glows as much, the ball not
///   shadowing what comes through itself, and a wall behind it puts it out;
/// - a sheet drawn from both sides glows all over, less the deeper it is.
#[test]
#[ignore = "requires GPU"]
fn light_behind_hair_comes_through_it() {
    let blond = StandardMaterial {
        color: Vec3::new(0.8, 0.6, 0.3),
        roughness: 0.5,
        hair: 1.0,
        ..Default::default()
    };
    // The ball's rim against the light and its middle. (`thickness` is in
    // the mesh's units: the ball is the unit sphere at a fifth.)
    let ball = |m: StandardMaterial, shadows: bool, wall: bool| {
        let mut scene = studio();
        scene.lighting.environment.intensity = 0.0;
        scene.lighting.light.intensity = 1.0;
        scene.lighting.light.direction_to_light = Vec3::new(0.5, 0.1, -1.0);
        scene.lighting.light.cast_shadows = shadows;
        scene.frame.camera.position = Vec3::new(0.0, 0.0, 0.4);
        if wall {
            let slab = scene.add_mesh(shapes::cuboid(Vec3::new(4.0, 4.0, 0.1)));
            let grey = scene.add_standard_material(StandardMaterial::basic(Vec3::splat(0.5)));
            scene.add_instance(
                slab,
                grey,
                Mat4::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            );
        }
        let f = ball_falloff_in(scene, m, 0.1);
        (falloff_mean(&f, -0.15, -0.03), falloff_mean(&f, -1.0, -0.8))
    };
    let deep = StandardMaterial {
        thickness: 0.15,
        ..blond
    };
    let (solid, _) = ball(blond, false, false);
    let (glow, middle) = ball(deep, false, false);
    let (shadowed, _) = ball(deep, true, false);
    let (walled, _) = ball(deep, true, true);
    let (matte, _) = ball(StandardMaterial { hair: 0.0, ..deep }, false, false);
    println!("the ball's rim: no thickness {solid:?}, 3 cm {glow:?}, its middle {middle:?}");
    println!(
        "under a sun that casts shadows {shadowed:?}, behind a wall {walled:?}; no hair {matte:?}"
    );
    let over = [0, 1, 2].map(|c| glow[c] - solid[c]);
    // The rim's chord is a strand or three: blond's blue goes first.
    assert!(
        over[0] > 30.0 && over[0] > over[1] + 5.0 && over[1] > over[2] + 15.0,
        "{over:?}"
    );
    assert!(middle[0] < 3.0, "{middle:?}");
    assert!(
        (shadowed[0] - glow[0]).abs() < 0.1 * glow[0],
        "{shadowed:?} of {glow:?}"
    );
    assert!(walled[0] < 3.0 && matte[0] < 3.0, "{walled:?}, {matte:?}");

    // A sheet drawn from both sides, its strands down it, lit from behind.
    let sheet = |thickness: f32, side: u32| {
        let mut scene = studio();
        scene.lighting.environment.intensity = 0.0;
        scene.lighting.light.intensity = 0.3;
        scene.lighting.light.direction_to_light = Vec3::new(0.2, 0.05, -1.0);
        let quad = scene.add_mesh(shapes::plane_xy(1.0, 1.0));
        let h = scene.add_standard_material(StandardMaterial {
            thickness,
            side,
            ..blond
        });
        scene.add_instance(quad, h, Mat4::IDENTITY);
        mean(&frame(&scene), area(600, 680, 320, 400))
    };
    let (none, lock, mane, closed) = (
        sheet(0.0, 2),
        sheet(0.01, 2),
        sheet(0.05, 2),
        sheet(0.01, 0),
    );
    println!(
        "a sheet lit from behind: no thickness {none:?}, 1 cm {lock:?}, 5 cm {mane:?}; one-sided {closed:?}"
    );
    // 1 cm is a strand or two, 5 cm six: each takes blue most.
    assert!(none[0] < 3.0 && closed[0] < 3.0, "{none:?}, {closed:?}");
    assert!(lock[0] > 100.0 && lock[0] > lock[2] + 10.0, "{lock:?}");
    assert!(
        mane[0] < 0.8 * lock[0] && mane[2] < 0.4 * lock[2] && mane[0] > 20.0,
        "{mane:?} of {lock:?}"
    );
}

/// An eye (`StandardMaterial::irisRadius`, eye.wgsl): the iris lies under
/// the cornea, so on a sheet turned 40 degrees from the camera the pupil
/// is seen where the bent ray meets it, towards the camera's side of the
/// sheet, along u and along v; face on it is where it is painted, and
/// the white outside the iris never moves.
#[test]
#[ignore = "requires GPU"]
fn the_iris_lies_under_the_cornea() {
    const RADIUS: f32 = 0.25;
    const DEPTH: f32 = 0.6;
    const IOR: f32 = 1.336;
    let shot = |eye: bool, turn: Mat4| {
        let mut scene = studio();
        scene.lighting.light.direction_to_light = Vec3::new(0.0, 0.0, 1.0);
        let quad = scene.add_mesh(shapes::plane_xy(1.0, 1.0));
        let map = scene.add_texture(og_tests::physical_scenes::eye(512, RADIUS, 0.06));
        let h = scene.add_standard_material(StandardMaterial {
            map: map.slot(),
            roughness: 1.0,
            ior: IOR,
            irisRadius: if eye { RADIUS } else { 0.0 },
            irisDepth: DEPTH,
            ..Default::default()
        });
        scene.add_instance(quad, h, turn);
        frame(&scene)
    };
    // The centre of the pupil (the pixels darker than the iris) and how many it is.
    let pupil = |img: &RgbaImage| {
        let (mut x, mut y, mut n) = (0.0f32, 0.0f32, 0.0f32);
        // Within the iris, which is blue.
        for (px_x, px_y) in area(604, 676, 324, 396) {
            let c = px(img, px_x, px_y);
            if c[2] < 20.0 {
                x += px_x as f32;
                y += px_y as f32;
                n += 1.0;
            }
        }
        assert!(n > 100.0, "{n} pupil pixels");
        (x / n, y / n)
    };
    // Where the pupil is seen, in iris radii from where it is painted,
    // on a sheet turned by `angle`: the ray bent into the cornea
    // (Snell) falls `DEPTH` radii, less where the dome is lower, so x
    // solves x = tan(bent) DEPTH (1 - x²).
    let bent = (40f32.to_radians().sin() / IOR).asin();
    let mut seen = 0.0f32;
    for _ in 0..20 {
        seen = bent.tan() * DEPTH * (1.0 - seen * seen);
    }
    // A metre at the sheet is 194 pixels (a unit sphere is a disc of
    // radius 97), foreshortened by the turn.
    let expected = seen * RADIUS * 40f32.to_radians().cos() * 194.0;
    let flat = pupil(&shot(false, Mat4::IDENTITY));
    let face_on = pupil(&shot(true, Mat4::IDENTITY));
    println!(
        "face on: painted {flat:?}, seen {face_on:?}; expected shift at 40 degrees {expected:.1} px"
    );
    assert!((face_on.0 - flat.0).abs() < 0.3 && (face_on.1 - flat.1).abs() < 0.3);
    // Turned about y, its normal towards +x: the camera is on its -u
    // side, left on the screen.
    let about_y = Mat4::from_rotation_y(40f32.to_radians());
    let (painted, eye) = (pupil(&shot(false, about_y)), pupil(&shot(true, about_y)));
    println!("turned about y: painted {painted:?}, seen {eye:?}");
    assert!((painted.0 - eye.0 - expected).abs() < 1.5 && (painted.1 - eye.1).abs() < 0.5);
    // Turned about x, its normal downwards: the camera is on its top
    // side (v 0), up on the screen.
    let about_x = Mat4::from_rotation_x(40f32.to_radians());
    let (painted, eye) = (pupil(&shot(false, about_x)), pupil(&shot(true, about_x)));
    println!("turned about x: painted {painted:?}, seen {eye:?}");
    assert!((painted.1 - eye.1 - expected).abs() < 1.5 && (painted.0 - eye.0).abs() < 0.5);
    // The white of the eye is the surface itself: no pixel of it moves
    // (but the iris's rim, filtered over both).
    let (a, b) = (shot(false, about_y), shot(true, about_y));
    let white = |img: &RgbaImage, x: u32, y: u32| px(img, x, y)[0] > 150.0;
    let (mut moved, mut whites) = (0, 0);
    for (x, y) in area(440, 840, 160, 560) {
        if white(&a, x, y) {
            whites += 1;
            if px(&a, x, y) != px(&b, x, y) {
                moved += 1;
            }
        }
    }
    println!("{moved} of {whites} white pixels moved");
    // The cornea is level with the white at the iris's rim: the iris
    // ends where it is painted, on the near side and the far one.
    let ends = |img: &RgbaImage| {
        let blue: Vec<u32> = (440..840)
            .filter(|&x| {
                let c = px(img, x, 360);
                c[0] < 100.0 && c[2] > 40.0
            })
            .collect();
        (blue[0] as f32, blue[blue.len() - 1] as f32)
    };
    let (painted, eye) = (ends(&a), ends(&b));
    println!("the iris's ends: painted {painted:?}, seen {eye:?}");
    assert!((painted.0 - eye.0).abs() <= 1.0 && (painted.1 - eye.1).abs() <= 1.0);
    assert!(
        whites > 10_000 && moved * 50 < whites,
        "{moved} of {whites}"
    );
}
