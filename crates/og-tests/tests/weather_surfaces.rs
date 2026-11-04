//! Weather on surfaces and cloth (Wave B, weather_scenes.rs): each frame
//! matches its reference, and from frames: rain runs down walls in
//! streaks, the ground dries in patches, dust settles on what faces up,
//! a material's weathering 0 keeps it as it is, and puddles hold to the
//! low parts of a surface's relief.

use glam::{Mat4, Vec3};
use og_render::{
    Camera, ColorSpace, DirectionalLight, Exposure, Mips, RenderScene, RenderTarget, Renderer,
    RgbaImage, StandardMaterial, SurfaceWeather, TextureData, shapes,
};
use og_tests::weather_scenes::WEATHER_SCENES;
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

#[test]
fn weather_and_cloth_scenes_match_their_references() {
    for scene in WEATHER_SCENES {
        match gfx::check(scene, &out_dir()) {
            Ok(r) => println!("{r}"),
            Err(e) => panic!("{e}"),
        }
    }
}

/// The mean luminance of each 16 px block of a box (x0..x1, y0..y1).
fn blocks(img: &RgbaImage, x0: u32, x1: u32, y0: u32, y1: u32) -> Vec<f32> {
    let mut out = Vec::new();
    for by in (y0..y1).step_by(16) {
        for bx in (x0..x1).step_by(16) {
            let mut sum = 0.0;
            for y in by..(by + 16).min(y1) {
                for x in bx..(bx + 16).min(x1) {
                    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
                    sum += 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32;
                }
            }
            out.push(sum / 256.0);
        }
    }
    out
}

/// Coefficient of variation: the spread of `v` over its mean.
fn spread(v: &[f32]) -> f32 {
    let n = v.len() as f32;
    let m = v.iter().sum::<f32>() / n;
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f32>() / n).sqrt() / m.max(1e-3)
}

fn mean(v: &[f32]) -> f32 {
    v.iter().sum::<f32>() / v.len() as f32
}

/// A flat, untextured ground (or a wall) of `m`, lit by a high sun and the
/// default sky, seen from above at a slant (`wall`: a wall 6 m ahead
/// filling the frame).
fn plain(m: StandardMaterial, wall: bool, weather: Option<SurfaceWeather>) -> RgbaImage {
    let mut scene = RenderScene::new();
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.3, 0.8, 0.5).normalize(),
        intensity: 2.0,
        cast_shadows: false,
        ..Default::default()
    };
    let h = scene.add_standard_material(m);
    if wall {
        let mesh = scene.add_mesh(shapes::cuboid(Vec3::new(30.0, 20.0, 1.0)));
        scene.add_instance(mesh, h, Mat4::from_translation(Vec3::new(0.0, 8.0, -6.5)));
        scene.camera = Camera {
            position: Vec3::new(0.0, 2.0, 0.0),
            target: Vec3::new(0.0, 2.0, -6.0),
            exposure: Exposure::linear(1.0),
            ..Default::default()
        };
    } else {
        let mesh = scene.add_mesh(shapes::plane(60.0, 6.0));
        scene.add_instance(mesh, h, Mat4::IDENTITY);
        scene.camera = Camera {
            position: Vec3::new(0.0, 6.0, 4.0),
            target: Vec3::new(0.0, 0.0, -4.0),
            exposure: Exposure::linear(1.0),
            ..Default::default()
        };
    }
    scene.surface_weather = weather.map(|w| SurfaceWeather { shelter: 0.0, ..w });
    frame(&scene)
}

fn concrete() -> StandardMaterial {
    StandardMaterial {
        color: Vec3::new(0.45, 0.4, 0.36),
        roughness: 0.9,
        ..Default::default()
    }
}

/// Rain runs down a wall in streaks: the soaked wall is darker than the
/// dry one and its columns differ (rivulets beside drier stone), where
/// the dry wall is even. (Wetness alone darkened a wall evenly.)
#[test]
fn rain_runs_down_a_wall_in_streaks() {
    let dry = plain(concrete(), true, None);
    let wet = plain(
        concrete(),
        true,
        Some(SurfaceWeather {
            wetness: 1.0,
            rain: 1.0,
            ..SurfaceWeather::default()
        }),
    );
    let (dry_across, _) = grain(&dry);
    let (across, down) = grain(&wet);
    println!(
        "wall: dry {} across {dry_across}, wet {} across {across} down {down}",
        mean(&blocks(&dry, 160, 1120, 100, 620)),
        mean(&blocks(&wet, 160, 1120, 100, 620)),
    );
    assert!(
        mean(&blocks(&wet, 160, 1120, 100, 620)) < 0.9 * mean(&blocks(&dry, 160, 1120, 100, 620)),
        "a soaked wall is darker"
    );
    assert!(across > 4.0 * dry_across.max(0.05), "and streaked");
    assert!(across > 3.0 * down, "the streaks run down the wall");
}

/// How much the luminance changes from a pixel to its neighbour across the
/// frame and down it (the mean absolute step, over the middle of the frame).
fn grain(img: &RgbaImage) -> (f32, f32) {
    let l = |x: u32, y: u32| {
        let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
        0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
    };
    let (mut across, mut down, mut n) = (0.0, 0.0, 0.0);
    for y in 100..620 {
        for x in 160..1120 {
            across += (l(x + 1, y) - l(x, y)).abs();
            down += (l(x, y + 1) - l(x, y)).abs();
            n += 1.0;
        }
    }
    (across / n, down / n)
}

/// Once the rain stops the ground dries in patches: at half its wetness it
/// is a mottle of dark damp and dry ground, where in the rain the same
/// wetness darkens it evenly. (Drying was even everywhere.)
#[test]
fn the_ground_dries_in_patches_after_the_rain() {
    let area =
        |w: Option<SurfaceWeather>| blocks(&plain(concrete(), false, w), 160, 1120, 200, 700);
    let dry = area(None);
    let at = |rain: f32| {
        area(Some(SurfaceWeather {
            wetness: 0.5,
            rain,
            ..SurfaceWeather::default()
        }))
    };
    let (raining, drying) = (at(0.0001), at(0.0));
    println!(
        "ground: dry {} ({}), raining {} ({}), drying {} ({})",
        mean(&dry),
        spread(&dry),
        mean(&raining),
        spread(&raining),
        mean(&drying),
        spread(&drying)
    );
    // Blocks of it as bright as the dry ground and blocks darker than the
    // evenly wet ground: it has dried in some places and not in others.
    // (Block by block: the light falls off across the frame.)
    let share = |f: &dyn Fn(usize) -> bool| {
        (0..dry.len()).filter(|&i| f(i)).count() as f32 / dry.len() as f32
    };
    let dried = share(&|i| drying[i] > 0.95 * dry[i]);
    let soaked = share(&|i| drying[i] < 0.9 * raining[i]);
    println!("dried {dried}, soaked {soaked}");
    assert!(dried > 0.15, "part of it has dried");
    assert!(soaked > 0.15, "part of it is still soaked");
    assert!(spread(&drying) > spread(&raining) + 0.05, "in patches");
}

/// A dust storm leaves the ground under its dust: dark asphalt turns pale
/// and sandy. A material with weathering 0 stays exactly as it is under
/// the dust and under deep snow, while the same material taking the
/// weather is buried.
#[test]
fn dust_settles_and_weathering_0_keeps_a_material_as_it_is() {
    let asphalt = StandardMaterial {
        color: Vec3::splat(0.06),
        roughness: 0.85,
        ..Default::default()
    };
    let px = |img: &RgbaImage| {
        let p = &img.pixels[((450 * img.width + 640) * 4) as usize..][..3];
        [p[0] as f32, p[1] as f32, p[2] as f32]
    };
    let dry = plain(asphalt, false, None);
    let dust = SurfaceWeather {
        dust: 1.0,
        ..SurfaceWeather::default()
    };
    let dusty = plain(asphalt, false, Some(dust));
    let (d, s) = (px(&dry), px(&dusty));
    println!("asphalt {d:?}, dusty {s:?}");
    assert!(s[1] > d[1] + 40.0, "dust is pale over dark ground");
    assert!(s[0] - s[2] > d[0] - d[2] + 15.0, "and sandy");
    let kept = StandardMaterial {
        weathering: 0.0,
        ..asphalt
    };
    let snow = SurfaceWeather {
        snow: 1.0,
        ..SurfaceWeather::default()
    };
    assert_eq!(
        plain(kept, false, Some(dust)).pixels,
        dry.pixels,
        "untouched by dust"
    );
    assert_eq!(
        plain(kept, false, Some(snow)).pixels,
        dry.pixels,
        "and by snow"
    );
    assert!(
        px(&plain(asphalt, false, Some(snow)))[1] > d[1] + 80.0,
        "which buries the rest"
    );
}

/// Puddles fill a surface's own hollows first: a ground whose AO map is
/// dark (a hollow) on its left and white (a crest) on its right, lit the
/// same (aoMapIntensity 0), takes more standing water on its left. Dry,
/// the two halves are alike.
#[test]
fn puddles_fill_the_low_parts_of_a_surface_first() {
    let relief = TextureData {
        width: 2,
        height: 1,
        rgba: vec![0, 0, 0, 255, 255, 255, 255, 255],
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    };
    // The map spans the whole 60 m plane: left of the frame's centre its
    // dark texel, right of it its white one.
    let halves = |w: Option<SurfaceWeather>| {
        let m = StandardMaterial {
            aoMapIntensity: 0.0,
            ..concrete()
        };
        let img = plain_with(m, relief.clone(), w);
        // (The far ground, where the 3 m pattern is small enough for
        // several puddles to fall in each half.)
        (
            spread(&blocks(&img, 0, 560, 0, 320)),
            spread(&blocks(&img, 720, 1280, 0, 320)),
        )
    };
    let dry = halves(None);
    let wet = halves(Some(SurfaceWeather {
        wetness: 1.0,
        puddles: 0.45,
        ..SurfaceWeather::default()
    }));
    println!("dry {dry:?}, puddled {wet:?}");
    assert!((dry.0 - dry.1).abs() < 0.1 * dry.0, "dry halves alike");
    // Standing water mirrors the sky over the dark wet ground: the half
    // that holds it is a mottle, the other an even wet grey.
    assert!(
        wet.0 > 0.05 && wet.0 > 1.8 * wet.1,
        "the hollow half holds more water: {wet:?}"
    );
}

/// [`plain`]'s ground with its `aoMap` the texture `relief` (the material's
/// own slot is replaced by the scene's handle).
fn plain_with(
    mut m: StandardMaterial,
    relief: TextureData,
    weather: Option<SurfaceWeather>,
) -> RgbaImage {
    let mut scene = RenderScene::new();
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.3, 0.8, 0.5).normalize(),
        intensity: 2.0,
        cast_shadows: false,
        ..Default::default()
    };
    m.aoMap = scene.add_texture(relief).slot();
    let h = scene.add_standard_material(m);
    let mesh = scene.add_mesh(shapes::plane(60.0, 1.0));
    scene.add_instance(mesh, h, Mat4::IDENTITY);
    scene.camera = Camera {
        position: Vec3::new(0.0, 6.0, 4.0),
        target: Vec3::new(0.0, 0.0, -4.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.surface_weather = weather.map(|w| SurfaceWeather { shelter: 0.0, ..w });
    frame(&scene)
}

// SCRATCH (removed before commit): the frames for reading.
