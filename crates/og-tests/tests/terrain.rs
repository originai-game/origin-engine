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
    TerrainLayer, Tonemap, Water, shapes,
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
