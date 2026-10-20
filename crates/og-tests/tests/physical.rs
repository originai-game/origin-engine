//! The physical material lobes (`og-render/src/physical.rs`): the
//! `materials-physical` reference board, and from frames: each lobe does
//! what it is for and its map steers it, transmission shows the scene
//! behind (blurred by roughness, tinted by absorption), a physical
//! material with its lobes off shades as the standard one, and skinned
//! physical materials draw.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, RenderScene, RenderTarget,
    Renderer, RgbaImage, StandardMaterial, TextureData, shapes,
};
use og_tests::compare::{Tolerance, compare};
use og_tests::physical_scenes::{ramp, stripes};
use og_tests::{gfx, scenes};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

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

fn luma(img: &RgbaImage, x: u32, y: u32) -> f32 {
    let [r, g, b] = px(img, x, y);
    0.2126 * r + 0.7152 * g + 0.0722 * b
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

/// A sphere of `m` in the studio, its maps from `textures`.
fn ball(m: impl FnOnce(&mut RenderScene) -> StandardMaterial) -> RgbaImage {
    let mut scene = studio();
    let sphere = scene.add_mesh(shapes::sphere(64, 48));
    let m = m(&mut scene);
    let h = scene.add_standard_material(m);
    scene.add_instance(sphere, h, Mat4::IDENTITY);
    frame(&scene)
}

fn peak(img: &RgbaImage) -> f32 {
    area(440, 840, 160, 560)
        .map(|(x, y)| luma(img, x, y))
        .fold(0.0, f32::max)
}

#[test]
fn materials_physical_matches_its_reference() {
    match gfx::check("materials-physical", &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

/// With every lobe off (a material physical only by a map it does not
/// use), the physical shader draws the standard one's frame.
#[test]
fn a_physical_material_with_its_lobes_off_shades_as_the_standard_one() {
    let base = StandardMaterial {
        color: Vec3::new(0.7, 0.3, 0.1),
        metalness: 0.3,
        roughness: 0.35,
        clearcoat: 1.0,
        clearcoatRoughness: 0.1,
        ..Default::default()
    };
    let standard = ball(|_| base);
    let physical = ball(|s| StandardMaterial {
        iridescenceMap: s.add_texture(ramp(16)).slot(),
        ..base
    });
    let c = compare(
        &standard.pixels,
        &physical.pixels,
        standard.width,
        standard.height,
        &Tolerance::default(),
    )
    .expect("same size");
    println!(
        "physical against standard: rmse {:.5}, {} outliers",
        c.rmse, c.outliers
    );
    assert!(c.rmse < 0.002 && c.outliers == 0, "rmse {}", c.rmse);
}

/// Sheen brightens the rim of a dark rough ball in its colour; a sheen
/// map steers it.
#[test]
fn sheen_lights_the_rim_in_its_colour() {
    let velvet = StandardMaterial {
        color: Vec3::new(0.01, 0.01, 0.06),
        roughness: 0.9,
        ..Default::default()
    };
    let pink = Vec3::new(1.0, 0.2, 0.5);
    let bare = ball(|_| velvet);
    let sheen = ball(|_| StandardMaterial {
        sheen: 1.0,
        sheenColor: pink,
        sheenRoughness: 0.8,
        ..velvet
    });
    // A band inside the lit upper-left rim.
    let rim = || {
        area(440, 840, 160, 560).filter(|&(x, y)| {
            let d = Vec2::new(x as f32 - 640.0, y as f32 - 360.0);
            let r = d.length();
            (75.0..95.0).contains(&r) && d.x < 0.0 && d.y < 0.0
        })
    };
    let (b, s) = (mean(&bare, rim()), mean(&sheen, rim()));
    println!("rim: bare {b:?}, sheen {s:?}");
    assert!(
        s[0] > b[0] + 25.0,
        "the rim turns pink (red {} to {})",
        b[0],
        s[0]
    );
    assert!(s[0] - s[1] > b[0] - b[1] + 15.0, "in the sheen's colour");
    // A black sheen colour map takes the sheen away.
    let black = ball(|s| StandardMaterial {
        sheen: 1.0,
        sheenColor: pink,
        sheenRoughness: 0.8,
        sheenColorMap: s
            .add_texture(solid([0, 0, 0, 255], og_render::ColorSpace::Srgb))
            .slot(),
        ..velvet
    });
    let k = mean(&black, rim());
    assert!(
        (k[0] - b[0]).abs() < 3.0,
        "no sheen under a black map: {k:?} against {b:?}"
    );
}

fn solid(rgba: [u8; 4], color_space: og_render::ColorSpace) -> TextureData {
    TextureData {
        width: 4,
        height: 4,
        rgba: rgba.repeat(16),
        color_space,
        mips: og_render::Mips::None,
    }
}

/// clearcoatMap masks the coat: a black map takes its highlight away, a
/// white one keeps it, a coat normal map breaks it up.
#[test]
fn clearcoat_maps_mask_and_bend_the_coat() {
    let red = StandardMaterial {
        color: Vec3::new(0.7, 0.06, 0.05),
        roughness: 0.8,
        clearcoat: 1.0,
        clearcoatRoughness: 0.03,
        ..Default::default()
    };
    let white = ball(|s| StandardMaterial {
        clearcoatMap: s
            .add_texture(solid([255; 4], og_render::ColorSpace::Linear))
            .slot(),
        ..red
    });
    let black = ball(|s| StandardMaterial {
        clearcoatMap: s
            .add_texture(solid([0, 0, 0, 255], og_render::ColorSpace::Linear))
            .slot(),
        ..red
    });
    let bare = ball(|_| StandardMaterial {
        clearcoat: 0.0,
        ..red
    });
    let (pw, pk, pb) = (peak(&white), peak(&black), peak(&bare));
    println!("highlight: white map {pw:.0}, black map {pk:.0}, no coat {pb:.0}");
    assert!(pw > pb + 60.0, "a white map keeps the coat's highlight");
    assert!((pk - pb).abs() < 4.0, "a black map takes it away");
    let bumpy = ball(|s| StandardMaterial {
        clearcoatMap: s
            .add_texture(solid([255; 4], og_render::ColorSpace::Linear))
            .slot(),
        clearcoatNormalMap: s
            .add_texture(og_tests::p2_scenes::bump_normal_map(256, 8.0))
            .slot(),
        ..red
    });
    let busy = |img: &RgbaImage| {
        area(460, 820, 180, 540)
            .map(|(x, y)| (luma(img, x + 1, y) - luma(img, x, y)).abs())
            .sum::<f32>()
    };
    let (bw, bb) = (busy(&white), busy(&bumpy));
    println!("shading busyness: smooth coat {bw:.0}, bumped coat {bb:.0}");
    assert!(
        bb > 1.5 * bw,
        "the coat's normal map breaks its reflections up"
    );
}

/// Iridescence colours a black ball's reflections; a thin film is not
/// grey.
#[test]
fn iridescence_colours_the_reflections() {
    let black = StandardMaterial {
        color: Vec3::splat(0.02),
        roughness: 0.2,
        ..Default::default()
    };
    // The chroma of the highlight: the brightest pixels' channel spread.
    let chroma = |img: &RgbaImage| {
        let top = peak(img);
        let lit: Vec<f32> = area(440, 840, 160, 560)
            .filter(|&(x, y)| luma(img, x, y) > 0.5 * top)
            .map(|(x, y)| {
                let [r, g, b] = px(img, x, y);
                r.max(g).max(b) - r.min(g).min(b)
            })
            .collect();
        lit.iter().sum::<f32>() / lit.len() as f32
    };
    let plain = ball(|_| black);
    let film = ball(|_| StandardMaterial {
        iridescence: 1.0,
        iridescenceIOR: 1.3,
        iridescenceThicknessRange: Vec2::new(100.0, 700.0),
        ..black
    });
    let (cp, cf) = (chroma(&plain), chroma(&film));
    println!("highlight chroma: plain {cp:.1}, iridescent {cf:.1}");
    assert!(cf > 2.0 * cp + 20.0, "the film colours the highlight");
    // The thickness map changes the colours.
    let mapped = ball(|s| StandardMaterial {
        iridescence: 1.0,
        iridescenceThicknessRange: Vec2::new(100.0, 700.0),
        iridescenceThicknessMap: s.add_texture(ramp(64)).slot(),
        ..black
    });
    let c = compare(
        &film.pixels,
        &mapped.pixels,
        film.width,
        film.height,
        &Tolerance::default(),
    )
    .expect("same size");
    assert!(
        c.rmse > 0.005,
        "the thickness map moves the colours (rmse {})",
        c.rmse
    );
}

/// Anisotropy stretches the highlight along the tangent (around the
/// sphere, across the image), and its rotation turns the stretch.
#[test]
fn anisotropy_stretches_the_highlight() {
    let metal = StandardMaterial {
        color: Vec3::splat(0.9),
        metalness: 1.0,
        roughness: 0.3,
        ..Default::default()
    };
    // The bright pixels' spread across and down the image.
    let spread = |img: &RgbaImage| {
        let top = peak(img);
        let lit: Vec<(f32, f32)> = area(440, 840, 160, 560)
            .filter(|&(x, y)| luma(img, x, y) > top * 0.6)
            .map(|(x, y)| (x as f32, y as f32))
            .collect();
        let n = lit.len() as f32;
        let (mx, my) = lit
            .iter()
            .fold((0.0, 0.0), |a, p| (a.0 + p.0 / n, a.1 + p.1 / n));
        let (vx, vy) = lit.iter().fold((0.0, 0.0), |a, p| {
            (a.0 + (p.0 - mx).powi(2) / n, a.1 + (p.1 - my).powi(2) / n)
        });
        (vx.sqrt(), vy.sqrt())
    };
    let round = spread(&ball(|_| metal));
    let along = spread(&ball(|_| StandardMaterial {
        anisotropy: 0.9,
        ..metal
    }));
    let turned = spread(&ball(|_| StandardMaterial {
        anisotropy: 0.9,
        anisotropyRotation: std::f32::consts::FRAC_PI_2,
        ..metal
    }));
    println!("highlight spread (x, y): isotropic {round:?}, along u {along:?}, turned {turned:?}");
    assert!(along.0 > 1.5 * along.1, "stretched across the image");
    assert!(
        turned.1 > 1.5 * turned.0,
        "a quarter turn stretches it down the image"
    );
    assert!(along.0 > 1.5 * round.0);
}

/// A glass ball (radius 200 pixels) in front of a backdrop split red over
/// green: the backdrop shows through it, bent by its thickness, a rough
/// glass blurs it, and absorption tints it; a white opaque ball shows
/// none of it.
#[test]
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

/// Skinned meshes draw with the physical pipelines too (their own
/// layouts), shadows and all.
#[test]
fn skinned_physical_materials_draw() {
    let mut scene = og_tests::fx_scenes::skinning_scene();
    let before = frame(&scene);
    for h in scene.material_handles().collect::<Vec<_>>() {
        let m = *scene.standard_material(h).unwrap();
        if m.roughness == 0.45 {
            scene.set_standard_material(
                h,
                StandardMaterial {
                    sheen: 1.0,
                    sheenColor: Vec3::new(0.2, 1.0, 0.4),
                    sheenRoughness: 0.4,
                    anisotropy: 0.5,
                    ..m
                },
            );
        }
    }
    let after = frame(&scene);
    let c = compare(
        &before.pixels,
        &after.pixels,
        before.width,
        before.height,
        &Tolerance::default(),
    )
    .expect("same size");
    println!(
        "skinned sheen against plain: rmse {:.5}, {} outliers",
        c.rmse, c.outliers
    );
    assert!(c.outliers > 1000, "the skinned cylinders take the sheen");
}

/// A material shader runs on a physical material too (the physical
/// shader is built with its hooks): the ripple shader's bands colour a
/// sheen ball.
#[test]
fn material_shaders_run_on_physical_materials() {
    let draw = |hooked: bool| {
        let mut scene = studio();
        let sphere = scene.add_mesh(shapes::sphere(64, 48));
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::new(0.1, 0.1, 0.4),
            roughness: 0.8,
            sheen: 1.0,
            sheenColor: Vec3::ONE,
            ..Default::default()
        });
        if hooked {
            let shader = scene.add_shader(og_tests::p2_scenes::ripple_shader());
            assert!(scene.set_material_shader(m, Some(shader)));
        }
        scene.add_instance(sphere, m, Mat4::IDENTITY);
        frame(&scene)
    };
    let (plain, hooked) = (draw(false), draw(true));
    // The bands are the stripe colour: redder than the blue ball.
    let warm = |img: &RgbaImage| {
        area(560, 720, 280, 440)
            .filter(|&(x, y)| {
                let [r, _, b] = px(img, x, y);
                r > b + 10.0
            })
            .count()
    };
    let (p, h) = (warm(&plain), warm(&hooked));
    println!("warm pixels: plain {p}, hooked {h}");
    assert!(p == 0 && h > 2000, "the hook's bands are drawn");
}
