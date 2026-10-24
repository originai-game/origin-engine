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
    TerrainLayer, TerrainScatter, Tonemap, Water, shapes,
};
use og_tests::gfx;
use og_tests::scenes;
use og_tests::terrain_scenes as ts;
use std::path::PathBuf;
use std::sync::Arc;

fn check(scene: &str) -> gfx::GfxReport {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    match gfx::check(scene, &out) {
        Ok(report) => {
            println!("{report}");
            report
        }
        Err(e) => panic!("{e}"),
    }
}

fn render(scene: &RenderScene) -> (RgbaImage, FrameStats) {
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let stats = r.render(scene, &RenderTarget::Offscreen).expect("render");
    (r.read_offscreen().expect("capture"), stats)
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

#[test]
fn terrain_scenes_match_their_references() {
    for scene in ["terrain-near", "terrain-mid", "terrain-far"] {
        let r = check(scene);
        assert!(r.stats.terrain_draws >= 3, "{scene}: {:?}", r.stats);
        assert!(
            r.stats.terrain_shadow_draws > 0,
            "{scene}: the terrain casts"
        );
    }
}

#[test]
fn water_phase_2_scenes_match_their_references() {
    let planar = check("water-planar");
    assert!(planar.stats.reflection_draws > 0, "the reflection pass ran");
    let shadows = check("water-shadows");
    assert_eq!(
        shadows.stats.reflection_draws, 0,
        "no planar reflection asked"
    );
    check("water-underwater");
}

/// The island seen from 300 m over its mountains against a magenta
/// background, with no sky and no fog: wherever the camera stands
/// (across the levels' snapping steps), not one pixel of the background
/// shows through the terrain's surface.
#[test]
fn the_clipmap_leaves_no_cracks() {
    let mut scene = ts::island_scene();
    scene.fog = None;
    scene.environment.background = Background::Color(Vec3::new(1.0, 0.0, 1.0));
    scene.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    for step in 0..6 {
        let x = -150.0 + step as f32 * 3.7;
        let z = -150.0 + step as f32 * 2.9;
        scene.camera = Camera {
            position: Vec3::new(x, 300.0, z + 260.0),
            target: Vec3::new(x, 40.0, z - 120.0),
            near: 0.5,
            far: 6000.0,
            exposure: Exposure::linear(1.0),
            ..Default::default()
        };
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let img = renderer.read_offscreen().expect("capture");
        // Rows below the island's horizon (the terrain covers them).
        let mut holes = 0;
        for y in 260..720 {
            for x in 0..1280 {
                let p = px(&img, x, y);
                if p[0] > 220.0 && p[1] < 40.0 && p[2] > 220.0 {
                    holes += 1;
                }
            }
        }
        assert_eq!(holes, 0, "step {step}: background through the terrain");
    }
}

/// Close to the ground the finest levels are drawn; from 4 km above the
/// island its whole extent fits in a few coarse levels.
#[test]
fn levels_follow_the_camera() {
    let near = ts::island_view("near");
    let (_, s_near) = render(&near);
    let mut high = ts::island_view("far");
    high.camera.position = Vec3::new(0.0, 4000.0, 1.0);
    high.camera.target = Vec3::ZERO;
    let (_, s_high) = render(&high);
    assert!(
        s_near.terrain_draws > s_high.terrain_draws,
        "near {} levels, high {}",
        s_near.terrain_draws,
        s_high.terrain_draws
    );
    assert!(s_near.terrain_triangles > 0 && s_high.terrain_triangles > 0);
    // Past the far plane nothing is drawn.
    let mut away = ts::island_view("far");
    away.camera.position = Vec3::new(0.0, 50.0, 30000.0);
    away.camera.target = Vec3::new(0.0, 50.0, 20000.0);
    away.camera.far = 2000.0;
    let (_, s_away) = render(&away);
    assert_eq!(s_away.terrain_draws, 0, "{s_away:?}");
}

/// A low sun behind the mountains: their shadows darken the valley in
/// front of them, and turning the terrain's shadows off lights it.
/// The boulders' shadows fall on the meadow (the terrain receives).
#[test]
fn terrain_casts_and_receives_shadows() {
    let mut scene = ts::island_view("mid");
    scene.fog = None;
    let sun = Vec3::new(0.2, 0.12, -1.0);
    scene.light.direction_to_light = sun;
    scene.light.max_shadow_distance = 1200.0;
    let (lit, _) = render(&scene);
    for t in &mut scene.terrains {
        t.cast_shadows = false;
    }
    let (open, _) = render(&scene);
    // The valley floor below the ridge (lower middle of the frame).
    let a = mean(&lit, 300, 420, 980, 700);
    let b = mean(&open, 300, 420, 980, 700);
    assert!(a < b * 0.85, "the ridge's shadow: {a:.1} vs {b:.1} without");

    // The boulders on the meadow, seen from the near camera, under a sun
    // from -x: the ground just east of one is in its shadow.
    let sun = Vec3::new(-0.6, 0.5, 0.0);
    let mut near = ts::island_view("near");
    near.light.direction_to_light = sun;
    let (with, _) = render(&near);
    keep("boulders", &with);
    let mut no_shadow = ts::island_view("near");
    no_shadow.light.direction_to_light = sun;
    no_shadow.light.cast_shadows = false;
    let (without, _) = render(&no_shadow);
    let (bx, bz) = (-88.0f32, 133.0f32);
    let spot = Vec3::new(bx + 3.0, ts::island_height(bx + 3.0, bz) + 0.05, bz);
    let at = project(&near.camera, spot);
    let (a, b) = (luma(color_at(&with, at)), luma(color_at(&without, at)));
    assert!(
        a < b * 0.75,
        "a boulder's shadow on the terrain at {at:?}: {a:.1} vs {b:.1}"
    );
}

/// A cone of plain-coloured layers: red on the flat, blue on slopes past
/// 30 degrees, white above 40 m; a control map that paints the fourth
/// (green) over the -x half's flat ground.
#[test]
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

/// Raising a mound in the data (a runtime edit) changes the next frame of
/// the same renderer where it stands and nowhere far from it; the CPU's
/// height answers the edit.
#[test]
fn an_edit_changes_the_drawn_surface() {
    let mut scene = ts::island_view("near");
    scene.fog = None;
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut frame = |scene: &RenderScene| {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
        renderer.read_offscreen().expect("capture")
    };
    let before = frame(&scene);
    let handle = scene.terrains[0].terrain;
    let mut data = scene.terrain(handle).expect("data").clone();
    let mut heights = data.heights.to_vec();
    let cell = data.cell();
    let centre = Vec2::new(-110.0, 100.0);
    for r in 0..data.rows {
        for c in 0..data.columns {
            let p = Vec2::new(c as f32, r as f32) * cell - data.size * 0.5;
            let d = (p - centre).length();
            if d < 20.0 {
                heights[(r * data.columns + c) as usize] += 15.0 * (1.0 - d / 20.0);
            }
        }
    }
    let ground = data.height(centre);
    data.heights = Arc::from(heights);
    assert!((data.height(centre) - (ground + 15.0)).abs() < 0.2);
    assert!(scene.set_terrain(handle, data));
    let after = frame(&scene);
    let (mx, my) = project(&scene.camera, Vec3::new(centre.x, ground + 8.0, centre.y));
    let changed = |x0: u32, y0: u32, x1: u32, y1: u32| {
        (mean(&before, x0, y0, x1, y1) - mean(&after, x0, y0, x1, y1)).abs()
    };
    assert!(
        changed(mx - 20, my - 10, mx + 20, my + 10) > 5.0,
        "the mound at ({mx}, {my}) shows"
    );
    assert!(
        changed(1100, 600, 1270, 710) < 0.5,
        "the near ground far from it is unchanged"
    );
}

/// A tall red post stands in a lake whose top is above the frame: the
/// planar reflection mirrors it in the water below; screen-space
/// reflection (the default) cannot, the post's top being off screen. A
/// block under the water between the mirrored camera and the post is cut
/// away by the reflection's clip plane.
#[test]
fn the_planar_reflection_shows_what_the_screen_does_not() {
    let make = |planar: f32| {
        let mut scene = RenderScene::new();
        let post = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 30.0, 1.0)));
        let red = scene.add_material(Material {
            base_color: [0.9, 0.05, 0.05, 1.0],
            roughness: 0.6,
            ..Default::default()
        });
        scene.add_instance(
            post,
            red,
            Mat4::from_translation(Vec3::new(0.0, 15.0, -30.0)),
        );
        // A block under the water, on the mirrored camera's line of sight
        // to the post: the reflection's clip plane leaves it out.
        let block = scene.add_mesh(shapes::cuboid(Vec3::new(3.0, 2.0, 3.0)));
        let yellow = scene.add_material(Material {
            base_color: [0.9, 0.8, 0.1, 1.0],
            ..Default::default()
        });
        scene.add_instance(
            block,
            yellow,
            Mat4::from_translation(Vec3::new(0.0, -3.2, 2.5)),
        );
        scene.water.push(Water {
            size: Vec2::splat(200.0),
            segments: 1,
            wave_strength: 0.0,
            reflectivity: 2.0,
            planar_reflection: planar,
            ..Default::default()
        });
        scene.camera = Camera {
            position: Vec3::new(0.0, 6.0, 10.0),
            target: Vec3::new(0.0, -6.0, -30.0),
            exposure: Exposure::linear(1.0),
            ..Default::default()
        };
        scene
    };
    let (planar, stats) = render(&make(0.5));
    keep("planar", &planar);
    assert!(stats.reflection_draws > 0);
    let (screen, stats) = render(&make(0.0));
    assert_eq!(stats.reflection_draws, 0);
    // The post's mirror image 10 m below the surface (its 10 m mark seen in the water).
    let scene = make(0.5);
    let at = project(&scene.camera, Vec3::new(0.0, -10.0, -30.0));
    let (p, s) = (color_at(&planar, at), color_at(&screen, at));
    let redness = |c: [f32; 3]| c[0] - (c[1] + c[2]) * 0.5;
    assert!(
        redness(p) > 12.0 && redness(p) > redness(s) + 25.0,
        "reflected post: planar {p:?}, screen-space {s:?}"
    );
}

/// A deck over the lake under a high sun: with `shadows` the water under
/// it darkens; open water away from it does not change.
#[test]
fn water_receives_shadows() {
    let make = |shadows: f32| {
        let mut scene = RenderScene::new();
        let deck = scene.add_mesh(shapes::cuboid(Vec3::new(8.0, 0.3, 8.0)));
        let wood = scene.add_material(Material::default());
        scene.add_instance(
            deck,
            wood,
            Mat4::from_translation(Vec3::new(0.0, 4.0, -20.0)),
        );
        let bed = scene.add_mesh(shapes::cuboid(Vec3::new(200.0, 1.0, 200.0)));
        scene.add_instance(bed, wood, Mat4::from_translation(Vec3::new(0.0, -4.0, 0.0)));
        scene.water.push(Water {
            size: Vec2::splat(120.0),
            segments: 1,
            wave_strength: 0.0,
            deep: Vec3::new(0.1, 0.3, 0.3),
            opacity: 1.0,
            shadows,
            ..Default::default()
        });
        scene.light = DirectionalLight {
            direction_to_light: Vec3::new(0.0, 1.0, -0.6),
            intensity: 3.0,
            ..Default::default()
        };
        scene.camera = Camera {
            position: Vec3::new(0.0, 25.0, 5.0),
            target: Vec3::new(0.0, 0.0, -20.0),
            exposure: Exposure::linear(1.0),
            ..Default::default()
        };
        scene
    };
    let (shadowed, _) = render(&make(1.0));
    let (plain, _) = render(&make(0.0));
    keep("water-shadowed", &shadowed);
    let scene = make(1.0);
    // Water in front of the deck, in its shadow (the sun high behind it).
    let under = project(&scene.camera, Vec3::new(0.0, 0.0, -14.5));
    let open = project(&scene.camera, Vec3::new(12.0, 0.0, -20.0));
    let (a, b) = (
        luma(color_at(&shadowed, under)),
        luma(color_at(&plain, under)),
    );
    assert!(a < b * 0.8, "in the deck's shadow: {a:.1} vs {b:.1}");
    let (c, d) = (
        luma(color_at(&shadowed, open)),
        luma(color_at(&plain, open)),
    );
    assert!((c - d).abs() < 2.0, "open water: {c:.1} vs {d:.1}");
}

/// From below the surface: the water's fog swallows a post 30 m away that
/// is clear from above, the surface overhead shows the sky inside Snell's
/// window, and caustics light the bed in a pattern only when asked.
#[test]
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

/// The CPU's height and normal (what physics and `og.terrain.sample`
/// answer) agree with the field on a plane of known slope, between
/// samples and clamped at the edge.
#[test]
fn the_cpu_height_and_normal_follow_the_field() {
    let n = 33u32;
    let size = Vec2::new(64.0, 32.0);
    let mut heights = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let x = -size.x * 0.5 + c as f32 * size.x / (n - 1) as f32;
            let z = -size.y * 0.5 + r as f32 * size.y / (n - 1) as f32;
            heights.push(0.5 * x - 0.25 * z + 3.0);
        }
    }
    let data = TerrainData {
        size,
        columns: n,
        rows: n,
        heights: Arc::from(heights),
        control: None,
    };
    for p in [
        Vec2::new(0.3, 0.7),
        Vec2::new(-20.1, 9.9),
        Vec2::new(31.0, -15.5),
    ] {
        let h = data.height(p);
        assert!(
            (h - (0.5 * p.x - 0.25 * p.y + 3.0)).abs() < 1e-3,
            "{p}: {h}"
        );
        let nrm = data.normal(p);
        let expect = Vec3::new(-0.5, 1.0, 0.25).normalize();
        assert!(nrm.dot(expect) > 0.9999, "{p}: {nrm}");
    }
    // Beyond the edge: the edge's height.
    assert!((data.height(Vec2::new(100.0, 0.0)) - data.height(Vec2::new(32.0, 0.0))).abs() < 1e-4);
    assert!(data.check().is_ok());
    let bad = TerrainData {
        heights: Arc::from(vec![0.0; 3]),
        ..data
    };
    assert!(bad.check().is_err());
}

#[test]
fn the_grass_scene_matches_its_reference() {
    let r = check("terrain-grass");
    assert!(
        r.stats.scatter_copies > 5_000,
        "{} copies drawn",
        r.stats.scatter_copies
    );
    assert!(r.stats.scatter_draws > 0 && r.stats.scatter_built > 0);
}

/// `Terrain::scatter` places instances only where its layer shows, about
/// as many as the density and the layer's weight ask for, never beyond its
/// distance, at full size even where they fade; each keeps its cell's place
/// and turn as the camera moves.
#[test]
fn scatter_follows_its_layer_distance_and_stays_put() {
    let scene = ts::grass_scene();
    let terrain = scene.terrains[0].clone();
    let data = ts::island();
    let eye = scene.camera.position;
    let s = ts::grass_scatter();
    let placed = terrain.scatter(&data, &s, eye);
    let flat = |p: Vec3| Vec2::new(p.x - eye.x, p.z - eye.z).length();
    let fade_from = s.fade_from();
    let mut expected = 0.0;
    // The layer's weight summed over a 1 m grid inside the distance.
    let r = s.distance as i32;
    for dz in -r..=r {
        for dx in -r..=r {
            let p = Vec2::new(eye.x + dx as f32, eye.z + dz as f32);
            if p.distance(Vec2::new(eye.x, eye.z)) <= s.distance {
                expected += terrain.layer_weights(&data, p)[0] * s.density;
            }
        }
    }
    let n = placed.len() as f32;
    assert!(
        (n - expected).abs() < expected * 0.08,
        "{n} instances for an expected {expected:.0}"
    );
    for (_, m) in &placed {
        let (scale, _, at) = m.to_scale_rotation_translation();
        let local = Vec2::new(at.x, at.z);
        let w = terrain.layer_weights(&data, local)[0];
        assert!(
            w > 0.0,
            "an instance at {at} where the grass layer's weight is 0"
        );
        assert!(flat(at) <= s.distance + 1e-3, "{at} beyond the distance");
        assert!(
            (at.y - data.height(local)).abs() < 1e-3,
            "{at} off the ground"
        );
        assert!(
            scale.x <= s.scale[1] + 1e-4 && scale.x >= s.scale[0] - 1e-4,
            "scale {} at {:.1} m",
            scale.x,
            flat(at)
        );
    }
    // Near the edge, inside the fade, at full size (the renderer fades them).
    let edge: Vec<f32> = placed
        .iter()
        .map(|(_, m)| m.to_scale_rotation_translation())
        .filter(|(_, _, at)| flat(*at) > s.distance * 0.97)
        .map(|(sc, _, _)| sc.x)
        .collect();
    assert!(
        !edge.is_empty() && edge.iter().all(|&x| x >= s.scale[0] - 1e-4),
        "{edge:?}"
    );
    // The camera moves 7 m: instances well inside both circles are the same.
    let moved = eye + Vec3::new(5.0, 0.0, -5.0);
    let again: std::collections::HashMap<u32, Mat4> =
        terrain.scatter(&data, &s, moved).into_iter().collect();
    let mut same = 0;
    for (key, m) in &placed {
        let at = m.w_axis.truncate();
        let near = |e: Vec3| Vec2::new(at.x - e.x, at.z - e.z).length() < fade_from - 8.0;
        if near(eye) && near(moved) {
            assert_eq!(again.get(key), Some(m), "cell {key} moved");
            same += 1;
        }
    }
    assert!(same > 1_000, "{same} instances compared");
    // A seed places them elsewhere; the rock layer's stones stand on slopes.
    let other = terrain.scatter(&data, &TerrainScatter { seed: 4, ..s }, eye);
    assert_ne!(other.first().map(|x| x.1), placed.first().map(|x| x.1));
    // Under the mountains (300 m north of the meadow), where rock shows.
    let stones = terrain.scatter(&data, &ts::stone_scatter(), Vec3::new(-82.0, 0.0, -150.0));
    assert!(!stones.is_empty());
    for (_, m) in &stones {
        let at = m.w_axis.truncate();
        let n = data.normal(Vec2::new(at.x, at.z));
        assert!(
            n.y.acos().to_degrees() > 30.0 - 6.0,
            "a stone on a {:.0} degree slope",
            n.y.acos().to_degrees()
        );
        // Leaning with the ground: the stone's up is the normal.
        let up = m.transform_vector3(Vec3::Y).normalize();
        assert!(up.dot(n) > 0.999, "up {up} normal {n}");
    }
    // The sand track the control map paints south of the lake grows none.
    let track = Vec2::new(
        ts::LAKE.x - 40.0 + (400.0 - ts::LAKE.y) * 0.25 + 4.0f32.sin() * 30.0,
        400.0,
    );
    assert!(terrain.layer_weights(&data, track)[0] < 0.05);
    let there = terrain.scatter(&data, &s, Vec3::new(track.x, 0.0, track.y));
    assert!(there.len() > 1_000, "grass beside the track");
    assert!(
        there
            .iter()
            .all(|(_, m)| (m.w_axis.x - track.x).abs() > 2.5 || (m.w_axis.z - track.y).abs() > 2.5),
        "grass on the track"
    );
    // A layer the terrain does not have, or no density, places nothing.
    assert!(
        terrain
            .scatter(&data, &TerrainScatter { layer: 7, ..s }, eye)
            .is_empty()
    );
    assert!(
        terrain
            .scatter(&data, &TerrainScatter { density: 0.0, ..s }, eye)
            .is_empty()
    );
}

/// A 64 × 16 m strip with a 6 m bump at its centre, one red layer, drawn
/// against a magenta background without fog or shadows; looked at from
/// 120 m straight above, or from the side at the bump.
fn strip_scene(transform: Mat4, from_side: bool) -> RenderScene {
    let (cols, rows) = (65u32, 17u32);
    let size = Vec2::new(64.0, 16.0);
    let mut heights = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let x = -size.x * 0.5 + c as f32;
            let z = -size.y * 0.5 + r as f32;
            heights.push(6.0 * (-(x * x + z * z) / 8.0).exp());
        }
    }
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData {
        size,
        columns: cols,
        rows,
        heights: Arc::from(heights),
        control: None,
    });
    scene.terrains.push(Terrain {
        transform,
        layers: vec![TerrainLayer {
            color: Vec3::new(0.8, 0.05, 0.05),
            roughness: 1.0,
            ..Default::default()
        }],
        cast_shadows: false,
        ..Terrain::new(h)
    });
    scene.fog = None;
    scene.environment.background = Background::Color(Vec3::new(1.0, 0.0, 1.0));
    scene.post.tonemap = Tonemap::None;
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.3, 1.0, 0.2),
        cast_shadows: false,
        ..Default::default()
    };
    scene.camera = Camera {
        position: if from_side {
            Vec3::new(0.0, 6.0, 80.0)
        } else {
            Vec3::new(0.0, 120.0, 0.01)
        },
        target: if from_side {
            Vec3::new(0.0, 6.0, 0.0)
        } else {
            Vec3::ZERO
        },
        near: 0.5,
        far: 1000.0,
        projection: Projection::Perspective {
            fov_y_degrees: 55.0,
        },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

fn is_background(img: &RgbaImage, at: (u32, u32)) -> bool {
    let p = color_at(img, at);
    p[0] > 220.0 && p[1] < 40.0 && p[2] > 220.0
}

/// A terrain is drawn where its transform puts it: turned a quarter about
/// y its strip runs along world z, scaled twice along x it reaches twice as
/// far, scaled three times along y its bump stands three times as tall,
/// and a translation moves all of it; its flat part shades the same turned
/// or not (normals follow the transform). Scattered copies stand on the
/// transformed surface.
#[test]
fn a_terrain_honours_its_transform() {
    let quarter = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let cases: [(&str, Mat4, Vec3, bool); 7] = [
        (
            "identity along x",
            Mat4::IDENTITY,
            Vec3::new(24.0, 0.0, 0.0),
            true,
        ),
        (
            "identity not along z",
            Mat4::IDENTITY,
            Vec3::new(0.0, 0.0, 24.0),
            false,
        ),
        ("turned along z", quarter, Vec3::new(0.0, 0.0, 24.0), true),
        (
            "turned not along x",
            quarter,
            Vec3::new(24.0, 0.0, 0.0),
            false,
        ),
        (
            "scaled along x",
            Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0)),
            Vec3::new(48.0, 0.0, 0.0),
            true,
        ),
        (
            "unscaled not that far",
            Mat4::IDENTITY,
            Vec3::new(48.0, 0.0, 0.0),
            false,
        ),
        (
            "moved",
            Mat4::from_translation(Vec3::new(0.0, 0.0, 30.0)),
            Vec3::new(20.0, 0.0, 30.0),
            true,
        ),
    ];
    let mut flat = Vec::new();
    for (label, transform, at, covered) in cases {
        let scene = strip_scene(transform, false);
        let (img, _) = render(&scene);
        keep(&format!("transform-{}", label.replace(' ', "-")), &img);
        let pixel = project(&scene.camera, at);
        assert_eq!(
            !is_background(&img, pixel),
            covered,
            "{label}: {at} at {pixel:?} is {:?}",
            color_at(&img, pixel)
        );
        if covered && label != "moved" {
            flat.push((label, color_at(&img, pixel)));
        }
    }
    for (label, c) in &flat {
        let d = (0..3)
            .map(|i| (c[i] - flat[0].1[i]).abs())
            .fold(0.0, f32::max);
        assert!(
            d < 6.0,
            "{label}: the flat ground shades {c:?}, not {:?}",
            flat[0].1
        );
    }
    // From the side: the bump's top (6 m) is under 12 m, three times taller is over it.
    let probe = Vec3::new(0.0, 12.0, 0.0);
    let plain = strip_scene(Mat4::IDENTITY, true);
    let (img, _) = render(&plain);
    assert!(
        is_background(&img, project(&plain.camera, probe)),
        "6 m bump under 12 m"
    );
    let tall = strip_scene(Mat4::from_scale(Vec3::new(1.0, 3.0, 1.0)), true);
    let (img, _) = render(&tall);
    keep("transform-tall", &img);
    assert!(
        !is_background(&img, project(&tall.camera, probe)),
        "18 m bump over 12 m"
    );
    // Scattered copies on the turned, raised and doubled strip lie on it.
    let placed_transform = Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 1.5, 2.0),
        glam::Quat::from_rotation_y(0.7),
        Vec3::new(5.0, 3.0, -4.0),
    );
    let scene = strip_scene(placed_transform, false);
    let terrain = scene.terrains[0].clone();
    let data = scene.terrain(terrain.terrain).cloned().expect("data");
    let s = TerrainScatter {
        density: 1.0,
        distance: 300.0,
        ..TerrainScatter::default()
    };
    let placed = terrain.scatter(&data, &s, Vec3::new(5.0, 50.0, -4.0));
    assert!(placed.len() > 1_000, "{} copies", placed.len());
    let back = placed_transform.inverse();
    for (_, m) in &placed {
        let local = back.transform_point3(m.w_axis.truncate());
        assert!(
            local.x.abs() <= 32.0 + 1e-3 && local.z.abs() <= 8.0 + 1e-3,
            "{local}"
        );
        let ground = data.height(Vec2::new(local.x, local.z));
        assert!(
            (local.y - ground).abs() < 1e-3,
            "{local} off the ground {ground}"
        );
    }
}

/// A terrain whose layers take their maps from two texture arrays draws as
/// the same terrain with each map its own texture: terrain-near (four
/// albedos, one normal map) with its maps moved into arrays, within the
/// mip filtering that differs between them (an array's mips are made on
/// the CPU), and with two layer textures bound instead of eight.
#[test]
fn layer_arrays_draw_as_separate_layer_textures() {
    let separate = ts::island_view("near");
    let arrayed = ts::with_layer_arrays(ts::island_view("near"));
    assert!(arrayed.terrains[0].layer_arrays.any());
    assert!(
        arrayed.terrains[0]
            .layers
            .iter()
            .all(|l| l.albedo_layer.is_some())
    );
    let (a, sa) = render(&separate);
    let (b, sb) = render(&arrayed);
    keep("layer-arrays-separate", &a);
    keep("layer-arrays", &b);
    assert_eq!(sa.terrain_draws, sb.terrain_draws);
    let e = rmse(&a, &b);
    println!("layer arrays vs separate textures: rmse {e:.4}");
    assert!(e < 0.03, "rmse {e}");
}

/// The root-mean-square difference of two frames' colours, 0 to 1.
fn rmse(a: &RgbaImage, b: &RgbaImage) -> f32 {
    let mut sum = 0.0f64;
    for (x, y) in a.pixels.chunks(4).zip(b.pixels.chunks(4)) {
        for k in 0..3 {
            let d = (x[k] as f64 - y[k] as f64) / 255.0;
            sum += d * d;
        }
    }
    (sum / (a.pixels.len() / 4 * 3) as f64).sqrt() as f32
}

/// Counts magenta background pixels below row `from` (the terrain covers them).
fn holes(img: &RgbaImage, from: u32) -> u32 {
    let mut n = 0;
    for y in from..img.height {
        for x in 0..img.width {
            let p = px(img, x, y);
            if p[0] > 220.0 && p[1] < 40.0 && p[2] > 220.0 {
                n += 1;
            }
        }
    }
    n
}

/// The island over magenta, unfogged, from 300 m over its mountains.
fn crack_view(mut scene: RenderScene) -> RenderScene {
    scene.fog = None;
    scene.environment.background = Background::Color(Vec3::new(1.0, 0.0, 1.0));
    scene.post.tonemap = Tonemap::None;
    scene.camera = Camera {
        position: Vec3::new(-150.0, 300.0, 110.0),
        target: Vec3::new(-150.0, 40.0, -270.0),
        near: 0.5,
        far: 6000.0,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// The tiles a frame wants come best first and every one after its parent;
/// the top level is always first; the capacity caps them; a camera far
/// above wants fewer than one on the ground; the detail distance brings
/// the finest tiles around the camera whatever their size on screen.
#[test]
fn wanted_tiles_are_ranked_and_closed_under_their_parents() {
    let data = ts::island();
    let layout = data.layout(128).expect("layout");
    assert_eq!(
        layout.levels, 4,
        "1024 quads in tiles of 128: 8, 4, 2, 1 per side"
    );
    let scene = ts::island_view("near");
    let t = &scene.terrains[0];
    let view = og_render::TerrainView::of(&scene.camera, 720);
    let near = og_render::terrain_tiles_wanted(&layout, t, data.range(), &view, 1000);
    let top = layout.index(layout.levels - 1, 0, 0);
    assert_eq!(near[0], top);
    for (i, tile) in near.iter().enumerate() {
        if let Some(p) = layout.parent(*tile) {
            let at = near.iter().position(|x| *x == p).expect("parent wanted");
            assert!(at < i, "tile {tile} before its parent {p}");
        }
    }
    let unique: std::collections::HashSet<_> = near.iter().collect();
    assert_eq!(unique.len(), near.len(), "a tile twice");
    // The camera's own level-0 tile is wanted.
    let eye = Vec2::new(scene.camera.position.x, scene.camera.position.z) + layout.size * 0.5;
    let own = (eye / (layout.cell() * 128.0)).floor();
    assert!(near.contains(&layout.index(0, own.x as u32, own.y as u32)));
    let capped = og_render::terrain_tiles_wanted(&layout, t, data.range(), &view, 6);
    assert_eq!(capped.len(), 6);
    assert_eq!(&capped[..], &near[..6], "the cap keeps the best");
    let mut high = view;
    high.camera = Vec3::new(0.0, 4000.0, 0.0);
    let from_high = og_render::terrain_tiles_wanted(&layout, t, data.range(), &high, 1000);
    assert!(
        from_high.len() < near.len(),
        "{} from 4 km, {} on the ground",
        from_high.len(),
        near.len()
    );
    // 300 m of detail from 4 km up: the level-0 tiles under the camera come too.
    high.detail = 300.0;
    let detailed = og_render::terrain_tiles_wanted(&layout, t, data.range(), &high, 1000);
    let level0 = |l: &[u32]| {
        l.iter()
            .filter(|i| layout.locate(**i).unwrap().0 == 0)
            .count()
    };
    assert!(level0(&detailed) >= 4 && level0(&detailed) > level0(&from_high));
}

/// The renderer keeps the tiles a frame wants within its terrain's budget,
/// and with fewer than it wants (eight slots) the terrain still covers the
/// view without a crack: what has no tile reads its parent's.
#[test]
fn a_tile_budget_keeps_the_terrain_whole() {
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let full = crack_view(ts::island_scene());
    let s_full = r.render(&full, &RenderTarget::Offscreen).expect("render");
    let img_full = r.read_offscreen().expect("capture");
    assert_eq!(holes(&img_full, 260), 0);
    let mut tight = crack_view(ts::island_scene());
    tight.terrains[0].lod.budget = 8 * ts::island().layout(128).unwrap().slot_bytes();
    let mut r2 = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let s_tight = r2.render(&tight, &RenderTarget::Offscreen).expect("render");
    let img_tight = r2.read_offscreen().expect("capture");
    keep("budget-8", &img_tight);
    assert_eq!(s_tight.terrain_tiles, 8, "{s_tight:?}");
    assert!(s_full.terrain_tiles > 8, "{s_full:?}");
    assert_eq!(
        holes(&img_tight, 260),
        0,
        "background through the terrain on 8 tiles"
    );
    // Moving on, tiles are swapped in place: never more than the budget.
    for step in 1..5 {
        tight.camera.position.x += 60.0;
        tight.camera.target.x += 60.0;
        let s = r2.render(&tight, &RenderTarget::Offscreen).expect("render");
        assert_eq!(s.terrain_tiles, 8);
        assert_eq!(holes(&r2.read_offscreen().unwrap(), 260), 0, "step {step}");
    }
}

/// A streamed terrain draws what has arrived: with only its top tile it
/// covers the view at the coarse level; tiles handed over later fade in
/// over a dozen frames and then draw exactly what the field in memory
/// draws; a tile dropped falls back to its parent. Half the tiles at random
/// (each with its parent) leave no crack.
#[test]
fn a_streamed_terrain_draws_what_has_arrived() {
    let data = ts::island();
    let layout = data.layout(128).expect("layout");
    let control = og_render::ControlPyramid::new(&layout, data.control.as_ref().unwrap());
    let tile = |i: u32| Arc::new(data.tile(&layout, i, control.as_ref()));
    let memory = crack_view(ts::island_scene());
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    r.render(&memory, &RenderTarget::Offscreen).expect("render");
    let reference = r.read_offscreen().expect("capture");

    let mut streamed = crack_view(ts::island_scene());
    let h = streamed.add_terrain_stream(layout, data.range());
    streamed.terrains[0].terrain = h;
    let top = layout.index(layout.levels - 1, 0, 0);
    assert!(streamed.set_terrain_tile(h, top, Some(tile(top))));
    let mut rs = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let s = rs
        .render(&streamed, &RenderTarget::Offscreen)
        .expect("render");
    let coarse = rs.read_offscreen().expect("capture");
    keep("stream-top-only", &coarse);
    assert_eq!(s.terrain_tiles, 1);
    assert_eq!(holes(&coarse, 260), 0, "the top tile alone covers the view");
    assert!(
        rmse(&coarse, &reference) > 0.005,
        "the top tile alone is coarser"
    );
    // Everything it wants arrives: a few frames in it is fading.
    let view = og_render::TerrainView::of(&streamed.camera, 720);
    let wanted =
        og_render::terrain_tiles_wanted(&layout, &streamed.terrains[0], data.range(), &view, 1000);
    for &i in wanted.iter().filter(|i| **i != top) {
        streamed.set_terrain_tile(h, i, Some(tile(i)));
    }
    let s = rs
        .render(&streamed, &RenderTarget::Offscreen)
        .expect("render");
    assert_eq!(
        s.terrain_uploads as usize,
        wanted.len() - 1,
        "one upload per new tile"
    );
    for _ in 0..3 {
        rs.render(&streamed, &RenderTarget::Offscreen)
            .expect("render");
    }
    let fading = rs.read_offscreen().expect("capture");
    assert_eq!(holes(&fading, 260), 0, "no crack while fading");
    for _ in 0..60 {
        let s = rs
            .render(&streamed, &RenderTarget::Offscreen)
            .expect("render");
        assert_eq!(s.terrain_uploads, 0, "nothing uploaded twice");
    }
    let arrived = rs.read_offscreen().expect("capture");
    keep("stream-arrived", &arrived);
    let e = rmse(&arrived, &reference);
    assert!(e < 1e-4, "streamed {e} against the field in memory");
    assert!(rmse(&fading, &reference) > rmse(&arrived, &reference));
    // Half the tiles at random, each with its parent: no crack.
    let mut kept = std::collections::HashSet::new();
    let mut seed = 7u32;
    for &i in &wanted {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let parent_in = layout.parent(i).is_none_or(|p| kept.contains(&p));
        if parent_in && (seed >> 16).is_multiple_of(2) || i == top {
            kept.insert(i);
        } else {
            streamed.set_terrain_tile(h, i, None);
        }
    }
    for _ in 0..2 {
        rs.render(&streamed, &RenderTarget::Offscreen)
            .expect("render");
    }
    let half = rs.read_offscreen().expect("capture");
    keep("stream-half", &half);
    assert_eq!(holes(&half, 260), 0, "background through half the tiles");
}

/// A streamed tile whose arrays are not the lengths its layout uploads is
/// refused (`set_terrain_tile` answers false and the terrain does not hold
/// it), so the frame after draws what the terrain holds instead of failing
/// in the upload.
#[test]
fn a_streamed_tile_of_the_wrong_size_is_refused() {
    let data = ts::island();
    let layout = data.layout(128).expect("layout");
    let control = og_render::ControlPyramid::new(&layout, data.control.as_ref().unwrap());
    let top = layout.index(layout.levels - 1, 0, 0);
    let good = data.tile(&layout, top, control.as_ref());
    let mut scene = crack_view(ts::island_scene());
    let h = scene.add_terrain_stream(layout, data.range());
    scene.terrains[0].terrain = h;
    let edit = |f: &dyn Fn(&mut og_render::TerrainTile)| {
        let mut t = good.clone();
        f(&mut t);
        Arc::new(t)
    };
    for (what, tile) in [
        ("heights", edit(&|t| t.heights = Arc::from(&t.heights[1..]))),
        ("normals", edit(&|t| t.normals = Arc::from(&t.normals[2..]))),
        (
            "control",
            edit(&|t| {
                let c = t.control.clone().expect("the island has a control map");
                t.control = Some(Arc::from(&c[4..]));
            }),
        ),
    ] {
        assert!(
            !scene.set_terrain_tile(h, top, Some(tile)),
            "a tile with too few {what} is refused"
        );
        assert!(
            scene.terrain_tiles(h).expect("streamed").is_empty(),
            "{what}"
        );
    }
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let s = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(s.terrain_tiles, 0, "nothing held, nothing drawn");
    assert!(scene.set_terrain_tile(h, top, Some(Arc::new(good))));
    let s = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(s.terrain_tiles, 1);
}

/// The renderer draws scattered copies from cached chunks: the copies a
/// frame draws are the CPU's placement (each 4 m cube where
/// `Terrain::scatter` puts it, at full size), none beyond the distance, and
/// over the fade a copy thins out by a dither across its whole footprint
/// rather than shrinking. A second frame builds no chunk; moving on builds
/// only the chunks that came into range.
#[test]
fn scattered_copies_are_drawn_from_chunks_and_fade_by_dither() {
    let n = 257u32;
    let size = 1024.0f32;
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData::flat(Vec2::splat(size), n, n));
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.05),
            ..Default::default()
        }],
        cast_shadows: false,
        ..Terrain::new(h)
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let red = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [8.0, 0.0, 0.0],
        ..Default::default()
    });
    let s = TerrainScatter {
        density: 0.03,
        distance: 90.0,
        fade: 0.5,
        scale: [3.0, 3.0],
        seed: 5,
        ..TerrainScatter::default()
    };
    scene.scatters.push(og_render::ScatterDraw {
        terrain: 0,
        mesh: cube,
        material: red,
        scatter: s,
    });
    scene.fog = None;
    scene.post.tonemap = Tonemap::None;
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 100.0, 0.001),
        target: Vec3::ZERO,
        near: 1.0,
        far: 500.0,
        projection: Projection::Orthographic { height: 200.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let first = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    let img = r.read_offscreen().expect("capture");
    keep("scatter-fade", &img);
    assert!(first.scatter_built >= 4, "{first:?}");
    let again = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert_eq!(again.scatter_built, 0, "a second frame builds nothing");
    assert_eq!(again.scatter_copies, first.scatter_copies);
    // Where each copy is: 3.6 pixels a metre (720 px over 200 m).
    let px_of = |p: Vec3| (640.0 + p.x * 720.0 / 200.0, 360.0 + p.z * 720.0 / 200.0);
    let red_at = |x: f32, y: f32| {
        let (x, y) = (x.round() as i64, y.round() as i64);
        if x < 0 || y < 0 || x >= 1280 || y >= 720 {
            return false;
        }
        let p = px(&img, x as u32, y as u32);
        p[0] > 150.0 && p[1] < 60.0
    };
    let placed = scene.terrains[0].scatter(
        &TerrainData::flat(Vec2::splat(size), n, n),
        &s,
        scene.camera.position,
    );
    let (mut solid, mut thinned, mut gone) = (0, 0, 0);
    let at_of = |m: &Mat4| m.w_axis.truncate();
    for (_, m) in &placed {
        let at = at_of(m);
        // Copies with a neighbour within 4 m would mix their pixels: skip them.
        if placed
            .iter()
            .any(|(_, o)| o != m && at_of(o).distance(at) < 4.0)
        {
            continue;
        }
        let d = Vec2::new(at.x, at.z).length();
        let (cx, cy) = px_of(at);
        // A disc of 4 px inside the 3 m (10.8 px) cube's top, and its outer ring from 2.5 px.
        let (mut inside, mut hit, mut ring) = (0, 0, 0);
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let r2 = (dx * dx + dy * dy) as f32;
                if r2 > 16.0 {
                    continue;
                }
                inside += 1;
                if red_at(cx + dx as f32, cy + dy as f32) {
                    hit += 1;
                    if r2 >= 6.25 {
                        ring += 1;
                    }
                }
            }
        }
        let fill = hit as f32 / inside as f32;
        if d < s.fade_from() - 3.0 {
            assert!(fill > 0.97, "a copy {d:.1} m out is filled {fill:.2}");
            solid += 1;
        } else if d > 60.0 && d < 74.0 {
            // Fading (about 0.25 to 0.6 kept): thinned across the footprint, its rim still drawn.
            assert!(
                fill > 0.05 && fill < 0.85,
                "a copy {d:.1} m out is filled {fill:.2}"
            );
            assert!(
                ring > 0,
                "a copy {d:.1} m out has lost its rim: shrunk, not dithered"
            );
            thinned += 1;
        }
    }
    assert!(solid > 5 && thinned > 3, "{solid} solid, {thinned} thinned");
    // Nothing past the distance: no red in the corners' ring beyond 92 m.
    for (x, y) in [(0u32, 0u32), (1279, 0), (0, 719), (1279, 719)] {
        let p = px(&img, x, y);
        assert!(p[0] < 100.0, "red at the corner {x},{y}");
    }
    for (_, m) in &placed {
        let at = m.w_axis.truncate();
        if Vec2::new(at.x, at.z).length() > 92.0 {
            gone += 1;
        }
    }
    assert_eq!(gone, 0, "the CPU placement keeps to the distance");
    // Moving 150 m east builds only the chunks that came into range.
    scene.camera.position.x += 150.0;
    scene.camera.target.x += 150.0;
    let moved = r.render(&scene, &RenderTarget::Offscreen).expect("render");
    assert!(
        moved.scatter_built > 0 && moved.scatter_built < first.scatter_built,
        "{} chunks built moving, {} at first",
        moved.scatter_built,
        first.scatter_built
    );
}

/// `terrain-generated`: a terrain the cook made (`generate`) draws as a
/// heightmap's does, its biomes' control map placing the layers and its
/// water at the cooked sea level: open sea at the frame's foot, a beach
/// strip at the island's shore (sand below 7 m, just above the 4 m sea),
/// meadows, rock on the slopes and snow on the top, then its reference.
#[test]
fn a_generated_terrain_draws_its_sea_shore_and_biomes() {
    let r = gfx::render("terrain-generated").expect("render");
    keep("terrain-generated", &r.image);
    assert!(r.stats.terrain_draws > 0, "{:?}", r.stats);
    let img = &r.image;
    for (x0, x1) in [(40, 240), (1040, 1240)] {
        let mut c = [0.0f32; 3];
        for y in 620..700 {
            for x in x0..x1 {
                let p = px(img, x, y);
                (0..3).for_each(|k| c[k] += p[k] / (200.0 * 80.0));
            }
        }
        assert!(c[2] > c[0] + 30.0, "the sea at x {x0}..{x1}: {c:?}");
    }
    let (mut sand, mut snow, mut grass) = (0, 0, 0);
    for y in 280..560 {
        for x in 320..960 {
            let [r, g, b] = px(img, x, y);
            if r.min(g).min(b) > 215.0 {
                snow += 1;
            } else if r >= g && g >= b && r - b >= 18.0 && r > 165.0 {
                sand += 1;
            } else if g > r + 8.0 && g > b + 8.0 {
                grass += 1;
            }
        }
    }
    // Read on lavapipe: 1,485 sand, 747 snow and 29,436 grass pixels.
    assert!(sand > 700, "the beach: {sand} pixels");
    assert!(snow > 300, "the snow: {snow} pixels");
    assert!(grass > 15000, "the meadows: {grass} pixels");
    check("terrain-generated");
}
