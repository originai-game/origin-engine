//! Phase 2's terrain and water (`crates/og-render/README.md` §Terrain,
//! §Water): each scene matches its reference for this backend tier, and
//! behaviour read from rendered frames. The clipmap leaves no cracks
//! wherever the camera stands and draws fewer levels from far away; the
//! terrain casts and receives shadows; layers follow their slope and
//! altitude rules and the control map; an edit changes what is drawn. The
//! water's planar reflection shows what the screen does not; its shadows
//! darken only where something stands between it and the sun; below the
//! surface the water's fog closes in and caustics play on the bed.

use glam::{Mat4, Vec2, Vec3, Vec4};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, FrameStats, Material, Projection,
    RenderScene, RenderTarget, Renderer, RgbaImage, Terrain, TerrainControl, TerrainData,
    TerrainLayer, TerrainScatter, Tonemap, Water, Wind, shapes,
};
use og_tests::gfx;
use og_tests::scenes;
use og_tests::terrain_scenes as ts;
use std::path::PathBuf;
use std::sync::Arc;

#[path = "terrain/displacement.rs"]
mod displacement;
#[path = "terrain/grass.rs"]
mod grass;
#[path = "terrain/impostors.rs"]
mod impostors;
#[path = "terrain/material.rs"]
mod material;
#[path = "terrain/sampling_scatter.rs"]
mod sampling_scatter;
#[path = "terrain/stamps.rs"]
mod stamps;
#[path = "terrain/streaming_wind.rs"]
mod streaming_wind;
#[path = "terrain/wind_models.rs"]
mod wind_models;

fn render(scene: &RenderScene) -> (RgbaImage, FrameStats) {
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let stats = r.render(scene, &RenderTarget::Offscreen).expect("render");
    (r.read_offscreen().expect("capture"), stats)
}

#[test]
#[ignore = "requires GPU"]
fn decals_project_on_terrain_and_reject_back_faces() {
    use og_render::{ColorSpace, Decal, Mips, TextureData};
    let desc = og_render::RenderDesc {
        width: 96,
        height: 96,
        ..scenes::base_desc()
    };
    let mut renderer = Renderer::new_blocking(&desc).unwrap();
    let mut scene = RenderScene::new();
    let terrain = scene.add_terrain(TerrainData {
        size: Vec2::splat(8.0),
        columns: 3,
        rows: 3,
        heights: vec![-0.4, 0.0, 0.4, -0.4, 0.0, 0.4, -0.4, 0.0, 0.4].into(),
        control: None,
        holes: None,
    });
    scene.terrains.push(Terrain::new(terrain));
    scene.camera = Camera {
        position: Vec3::new(0.0, 10.0, 0.0),
        target: Vec3::ZERO,
        up: Vec3::Z,
        projection: Projection::Orthographic { height: 8.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.post.tonemap = Tonemap::None;
    let mut capture = |s: &RenderScene| {
        let stats = renderer.render(s, &RenderTarget::Offscreen).unwrap();
        assert!(stats.terrain_draws > 0);
        renderer.read_offscreen().unwrap()
    };
    let bare = capture(&scene);
    let atlas = scene.add_texture(TextureData {
        width: 2,
        height: 2,
        rgba: [255, 0, 0, 255].repeat(4),
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    });
    scene.decal_atlas = Some(atlas);
    let decal = Decal {
        emissive: Vec3::X,
        ..Decal::on_surface(Vec3::ZERO, Vec3::Y, Vec2::new(3.0, 2.0), 1.0, 0.0)
    };
    scene.decals.push(decal);
    let marked = capture(&scene);
    keep("terrain-decal", &marked);
    assert!(px(&marked, 48, 48)[0] > px(&bare, 48, 48)[0] + 30.0);
    assert_eq!(px(&marked, 12, 12), px(&bare, 12, 12));
    scene.decals[0] = Decal {
        emissive: decal.emissive,
        ..Decal::on_surface(Vec3::ZERO, -Vec3::Y, Vec2::new(3.0, 2.0), 1.0, 0.0)
    };
    let backwards = capture(&scene);
    assert_eq!(px(&backwards, 48, 48), px(&bare, 48, 48));
}

/// Writes a frame a test reads to `<target tmp>/og-terrain/<name>.png`
/// (for looking at a failure).
fn keep(name: &str, img: &RgbaImage) {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-terrain")
        .join(format!("{name}.png"));
    let _ = gfx::save_png(&path, img);
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [
        img.pixels[i] as f32,
        img.pixels[i + 1] as f32,
        img.pixels[i + 2] as f32,
    ]
}

fn luma(p: [f32; 3]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

fn mean(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += luma(px(img, x, y));
        }
    }
    sum / ((x1 - x0) * (y1 - y0)) as f32
}

/// The mean colour over a small square around a pixel.
fn color_at(img: &RgbaImage, (x, y): (u32, u32)) -> [f32; 3] {
    assert!(
        (2..img.width - 2).contains(&x) && (2..img.height - 2).contains(&y),
        "({x}, {y}) is not in the frame"
    );
    let mut c = [0.0; 3];
    for yy in y - 2..=y + 2 {
        for xx in x - 2..=x + 2 {
            let p = px(img, xx, yy);
            for i in 0..3 {
                c[i] += p[i] / 25.0;
            }
        }
    }
    c
}

/// Where a world point lands on a 1280x720 frame of `camera`.
fn project(camera: &Camera, p: Vec3) -> (u32, u32) {
    let view = glam::camera::rh::view::look_at_mat4(camera.position, camera.target, camera.up);
    let fov = match camera.projection {
        Projection::Perspective { fov_y_degrees } => fov_y_degrees,
        Projection::Orthographic { .. } => unreachable!("perspective cameras only"),
    };
    let proj = glam::camera::rh::proj::directx::perspective(
        fov.to_radians(),
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

/// A cone of plain-coloured layers: red on the flat, blue on slopes past
/// 30 degrees, white above 40 m; a control map that paints the fourth
/// (green) over the -x half's flat ground.
#[test]
#[ignore = "requires GPU"]
fn layers_follow_slope_altitude_and_the_control_map() {
    let n = 257u32;
    let size = 256.0f32;
    let mut heights = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let x = -size * 0.5 + c as f32 * size / (n - 1) as f32;
            let z = -size * 0.5 + r as f32 * size / (n - 1) as f32;
            // A cone of slope 1 (45 degrees) to 60 m at the centre.
            heights.push((60.0 - Vec2::new(x, z).length()).max(0.0));
        }
    }
    let m = 64u32;
    let mut rgba = Vec::new();
    for _y in 0..m {
        for x in 0..m {
            // West: the fourth layer instead of the first.
            let west = x < m / 2;
            let (r, a) = if west { (0, 255) } else { (255, 0) };
            rgba.extend_from_slice(&[r, 255, 255, a]);
        }
    }
    let data = TerrainData {
        size: Vec2::splat(size),
        columns: n,
        rows: n,
        heights: Arc::from(heights),
        control: Some(TerrainControl {
            width: m,
            height: m,
            rgba: Arc::from(rgba),
        }),
        holes: None,
    };
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(data);
    let plain = |color: Vec3| TerrainLayer {
        color,
        roughness: 1.0,
        ..Default::default()
    };
    scene.terrains.push(Terrain {
        layers: vec![
            TerrainLayer {
                slope: [0.0, 20.0],
                slope_fade: 2.0,
                altitude: [-100.0, 40.0],
                altitude_fade: 1.0,
                ..plain(Vec3::new(0.8, 0.05, 0.05))
            },
            TerrainLayer {
                slope: [30.0, 90.0],
                slope_fade: 2.0,
                altitude: [-100.0, 40.0],
                altitude_fade: 1.0,
                ..plain(Vec3::new(0.05, 0.05, 0.8))
            },
            TerrainLayer {
                altitude: [40.0, 1000.0],
                altitude_fade: 1.0,
                ..plain(Vec3::new(0.9, 0.9, 0.9))
            },
            TerrainLayer {
                slope: [0.0, 20.0],
                slope_fade: 2.0,
                altitude: [-100.0, 40.0],
                altitude_fade: 1.0,
                ..plain(Vec3::new(0.05, 0.8, 0.05))
            },
        ],
        ..Terrain::new(h)
    });
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.0, 1.0, 0.3),
        intensity: 2.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment = Environment {
        intensity: 0.3,
        ..Default::default()
    };
    scene.post.tonemap = Tonemap::None;
    scene.camera = Camera {
        position: Vec3::new(0.0, 170.0, 150.0),
        target: Vec3::new(0.0, 10.0, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let (img, _) = render(&scene);
    keep("layers", &img);
    let at = |p: Vec3| color_at(&img, project(&scene.camera, p));
    let dominant = |c: [f32; 3]| {
        if c[0] > 100.0 && c[1] > 100.0 && c[2] > 100.0 {
            "white"
        } else if c[0] > c[1] * 2.0 && c[0] > c[2] * 2.0 {
            "red"
        } else if c[2] > c[0] * 2.0 && c[2] > c[1] * 2.0 {
            "blue"
        } else if c[1] > c[0] * 2.0 && c[1] > c[2] * 2.0 {
            "green"
        } else {
            "mixed"
        }
    };
    // Flat ground east of the cone: red; west (the control map): green.
    assert_eq!(dominant(at(Vec3::new(90.0, 0.0, 30.0))), "red");
    assert_eq!(dominant(at(Vec3::new(-90.0, 0.0, 30.0))), "green");
    // The cone's 45-degree flank below 40 m: blue, on both halves.
    assert_eq!(dominant(at(Vec3::new(10.0, 25.0, 33.0))), "blue");
    assert_eq!(dominant(at(Vec3::new(-10.0, 25.0, 33.0))), "blue");
    // Above 40 m: white.
    assert_eq!(dominant(at(Vec3::new(0.0, 52.0, 8.0))), "white");
}

/// From below the surface: the water's fog swallows a post 30 m away that
/// is clear from above, the surface overhead shows the sky inside Snell's
/// window, and caustics light the bed in a pattern only when asked.
#[test]
#[ignore = "requires GPU"]
fn the_underwater_view_fogs_shows_the_surface_and_caustics() {
    let make = |eye_y: f32, caustics: f32| {
        let mut scene = RenderScene::new();
        let post = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 12.0, 1.0)));
        let red = scene.add_material(Material {
            base_color: [0.9, 0.05, 0.05, 1.0],
            ..Default::default()
        });
        scene.add_instance(
            post,
            red,
            Mat4::from_translation(Vec3::new(0.0, -2.0, -30.0)),
        );
        let bed = scene.add_mesh(shapes::cuboid(Vec3::new(200.0, 1.0, 200.0)));
        let sand = scene.add_material(Material {
            base_color: [0.7, 0.62, 0.45, 1.0],
            ..Default::default()
        });
        scene.add_instance(bed, sand, Mat4::from_translation(Vec3::new(0.0, -8.5, 0.0)));
        scene.water.push(Water {
            size: Vec2::splat(150.0),
            segments: 1,
            caustics,
            ..Default::default()
        });
        scene.environment.background = Background::Sky;
        scene.camera = Camera {
            position: Vec3::new(0.0, eye_y, 0.0),
            target: Vec3::new(0.0, eye_y - 0.5, -30.0),
            exposure: Exposure::linear(1.0),
            ..Default::default()
        };
        scene.tick = 30;
        scene
    };
    let below = make(-3.0, 0.0);
    let (under, _) = render(&below);
    let (above, _) = render(&make(3.0, 0.0));
    let redness = |c: [f32; 3]| c[0] - (c[1] + c[2]) * 0.5;
    let p_under = color_at(&under, project(&below.camera, Vec3::new(0.0, -3.0, -30.0)));
    let above_scene = make(3.0, 0.0);
    let p_above = color_at(
        &above,
        project(&above_scene.camera, Vec3::new(0.0, 2.0, -30.0)),
    );
    assert!(redness(p_above) > 60.0, "the post from above: {p_above:?}");
    assert!(
        redness(p_under) < redness(p_above) * 0.5,
        "the post fogged from below: {p_under:?}"
    );
    // Looking straight up from below: the sky's light through the window,
    // brighter than the fog around.
    let mut up = make(-3.0, 0.0);
    up.camera.target = Vec3::new(0.0, 10.0, -0.01);
    let (up_img, _) = render(&up);
    let centre = mean(&up_img, 600, 320, 680, 400);
    let mut down = make(-3.0, 0.0);
    down.camera.target = Vec3::new(0.0, -10.0, -12.0);
    let (fog_img, _) = render(&down);
    let fogged = mean(&fog_img, 0, 0, 1280, 120);
    assert!(
        centre > fogged + 10.0,
        "Snell's window {centre:.1} over the fog {fogged:.1}"
    );
    // Caustics: the bed's light varies (a pattern) only with them.
    let mut bed = make(-3.0, 1.0);
    bed.camera.target = Vec3::new(0.0, -8.0, -6.0);
    let (lit, _) = render(&bed);
    let mut flat = make(-3.0, 0.0);
    flat.camera.target = Vec3::new(0.0, -8.0, -6.0);
    let (plain, _) = render(&flat);
    let spread = |img: &RgbaImage| {
        let m = mean(img, 440, 420, 840, 620);
        let mut v = 0.0;
        for y in 420..620 {
            for x in 440..840 {
                v += (luma(px(img, x, y)) - m).powi(2);
            }
        }
        (v / (400.0 * 200.0)).sqrt()
    };
    let (sc, sp) = (spread(&lit), spread(&plain));
    assert!(sc > sp + 3.0, "caustics pattern {sc:.2} vs {sp:.2}");
    let _ = Vec4::ZERO;
}
