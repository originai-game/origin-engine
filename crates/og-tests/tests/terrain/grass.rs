//! R6's grass: a default og.Grass grows a meadow over the island's grass
//! layer, lit from behind it lets the sun through, it reaches the horizon
//! as the ground's colour, the wind bends it, and the same tick draws the
//! same blades.
//!
//! Ways this could fail: generation keeps nothing (a wrong density share,
//! an empty draw), the blades draw but black (unlit, a missing group),
//! translucency does nothing, the far tint never reaches the ground, or
//! the blades' order or bend depends on more than the tick (an atomic
//! append, a wall clock).

use super::*;

/// G3's meadow: red blades 0.6 m tall over a dark flat terrain to 2.5 m
/// from a camera at ankle height (0.12 m) looking along +x, against a
/// black sky, so every red pixel above the frame's middle row is a blade
/// part above the ankle.
fn trample_scene() -> RenderScene {
    let data = TerrainData::flat(Vec2::splat(64.0), 129, 129);
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(data);
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.02),
            ..Default::default()
        }],
        cast_shadows: false,
        fade_in: false,
        ..Terrain::new(h)
    });
    scene.world.grass.push(og_render::GrassDraw {
        terrain: 0,
        grass: og_render::Grass {
            distance: 2.5,
            height: 0.6,
            height_variation: 0.0,
            lean: 0.0,
            clump_lean: 0.0,
            base_color: Vec3::new(0.9, 0.05, 0.05),
            tip_color: Vec3::new(0.9, 0.05, 0.05),
            far_tint: false,
            cast_shadows: false,
            recovery: 1.0,
            ..Default::default()
        },
    });
    scene.atmosphere.fog = None;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.light.cast_shadows = false;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.12, 0.0),
        target: Vec3::new(1.0, 0.12, 0.0),
        near: 0.02,
        far: 50.0,
        projection: Projection::Perspective {
            fov_y_degrees: 60.0,
        },
        ..Default::default()
    };
    scene
}

/// Red pixels above the frame's middle row: blade parts above the ankle.
fn blade_pixels_above(img: &RgbaImage) -> usize {
    (0..img.height / 2 - 2)
        .flat_map(|y| (0..img.width).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px_f32(img, x, y);
            p[0] > 30.0 && p[0] > 2.5 * p[1]
        })
        .count()
}

/// G3: an og.GrassPusher standing over the meadow presses its blades away
/// and down: above ankle height fewer than 30 % of the blade pixels stay;
/// once it leaves they stand again within the field's recovery (1 s) and
/// half a second, to at least 80 % of the untouched field. Before G3
/// nothing read a pusher: the blades stood as tall under it.
#[test]
#[ignore = "requires GPU"]
fn a_grass_pusher_tramples_the_blades_and_they_recover() {
    let mut scene = trample_scene();
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |scene: &mut RenderScene, tick: u64| {
        scene.frame.tick = tick;
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    let untouched = blade_pixels_above(&draw(&mut scene, 0));
    // The pusher covers every drawn blade near its centre (radius 6 m,
    // the field 2.5 m: pressed at least 97 %).
    scene.atmosphere.grass_pushers = vec![og_render::GrassPusher {
        position: Vec3::new(0.0, 0.0, 0.0),
        radius: 6.0,
        height: 1.8,
        strength: 1.0,
    }];
    let mut pressed = draw(&mut scene, 1);
    for tick in 2..=10 {
        pressed = draw(&mut scene, tick);
    }
    save("og-terrain", "grass-trample", &pressed);
    let under = blade_pixels_above(&pressed);
    scene.atmosphere.grass_pushers.clear();
    // 1.5 s later (the recovery and half a second), drawn every 4 ticks.
    let mut after = pressed.clone();
    for tick in (14..=100).step_by(4) {
        after = draw(&mut scene, tick);
    }
    save("og-terrain", "grass-trample-recovered", &after);
    let back = blade_pixels_above(&after);
    println!(
        "blade pixels above the ankle: untouched {untouched}, pressed {under}, recovered {back}"
    );
    assert!(
        untouched > 2000,
        "the meadow shows {untouched} blade pixels"
    );
    assert!(
        (under as f32) < 0.3 * untouched as f32,
        "pressed: {under} of {untouched} still stand"
    );
    assert!(
        back as f32 > 0.8 * untouched as f32,
        "recovered: {back} of {untouched}"
    );
}

thread_local! {
    static TICKETS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static COMPILED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A web host whose compute pipelines arrive late (og-render gpu/ready.rs):
/// while the grass's generate pipelines still compile, no blade is drawn
/// (nothing reads the blade buffers nothing generated), and once they are
/// made the meadow is drawn as with pipelines made at once. Fails if the
/// field were generated or drawn before its pipelines were ready, or left
/// out once they were.
#[test]
#[ignore = "requires GPU"]
fn grass_whose_generate_pipelines_arrive_late_draws_no_blade_until_they_do() {
    // Every pipeline made is a ticket of its own, compiled once COMPILED.
    fn count() -> u32 {
        TICKETS.with(|t| {
            t.set(t.get() + 1);
            t.get()
        })
    }
    fn ready(_: u32) -> bool {
        COMPILED.with(|c| c.get())
    }
    fn compiling() -> bool {
        !COMPILED.with(|c| c.get())
    }
    let mut scene = trample_scene();
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |scene: &mut RenderScene, tick: u64| {
        scene.frame.tick = tick;
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    og_render::set_pipeline_hooks(None, Some((count, ready, compiling)));
    let waiting = blade_pixels_above(&draw(&mut scene, 0));
    let still = blade_pixels_above(&draw(&mut scene, 1));
    COMPILED.with(|c| c.set(true));
    let arrived = blade_pixels_above(&draw(&mut scene, 2));
    og_render::set_pipeline_hooks(None, None);
    let at_once = blade_pixels_above(&draw(&mut scene, 2));
    println!("blade pixels: waiting {waiting}, {still}; arrived {arrived}; made at once {at_once}");
    assert_eq!(
        (waiting, still),
        (0, 0),
        "blades drawn while their pipelines compile"
    );
    assert!(at_once > 2000, "the meadow shows {at_once} blade pixels");
    assert!(
        arrived as f32 > 0.9 * at_once as f32,
        "the meadow once its pipelines arrived: {arrived} of {at_once}"
    );
}

/// A meadow and a scatter placed on the GPU make their pipelines the first
/// frame they are in the scene, out of the camera's reach, not in the
/// frame the camera reaches them: a cold driver compiled the grass's for
/// 1.5 s, the placement's for 0.7 s and the cull's for 0.14 s there, one
/// 2.4 s frame when a camera jumped into a wood (NVIDIA L4). Fails if the
/// grass's or the placement's are made when its first blades or chunk come
/// in reach, or if those frames wait for the cull's compile. Reads the
/// process's pipeline counts (gpu/traffic.rs), as the gate runs each test
/// alone.
#[test]
#[ignore = "requires GPU"]
fn grass_and_gpu_scatter_make_their_pipelines_before_the_camera_reaches_them() {
    use og_render::{Grass, GrassDraw, ScatterDraw};
    const MADE_AHEAD: [&str; 4] = ["grass", "grass prepass", "grass shadow", "scatter place"];
    let made = || {
        let mut n = [0u64; 4];
        og_render::traffic::each_site(|s| {
            if s.kind == og_render::traffic::Kind::Pipeline
                && let Some(i) = MADE_AHEAD.iter().position(|m| *m == s.name())
            {
                n[i] += s.count;
            }
        });
        n
    };
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData::flat(Vec2::splat(256.0), 65, 65));
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.2),
            ..Default::default()
        }],
        fade_in: false,
        ..Terrain::new(h)
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::new(0.5, 2.0, 0.5)));
    let white = scene.add_material(Material::default());
    scene.world.scatters.push(ScatterDraw {
        id: 0,
        terrain: 0,
        mesh: cube,
        material: white,
        scatter: TerrainScatter {
            density: 0.2,
            distance: 30.0,
            scale: [1.0, 1.0],
            seed: 7,
            gpu: true,
            ..TerrainScatter::default()
        },
        cast_shadows: true,
        impostor: None,
        impostor_distance: 0.0,
        with: None,
        lods: [None; 2],
        lod_materials: [None; 2],
    });
    scene.world.grass.push(GrassDraw {
        terrain: 0,
        grass: Grass {
            distance: 20.0,
            ..Default::default()
        },
    });
    scene.lighting.light.cast_shadows = true;
    // 1.5 km off, looking back at the terrain: out of both reaches.
    let far = Camera {
        position: Vec3::new(1500.0, 20.0, 0.0),
        target: Vec3::ZERO,
        far: 4000.0,
        ..Default::default()
    };
    scene.frame.camera = far;
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    og_render::traffic::set_clock(Some(|| 0.0));
    for tick in 0..3 {
        scene.frame.tick = tick;
        r.render(&scene, &RenderTarget::Offscreen).expect("render");
    }
    let ahead = made();
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.0, 0.0),
        target: Vec3::new(30.0, 0.0, 0.0),
        ..far
    };
    let mut reached = FrameStats::default();
    let mut waited = 0;
    for tick in 3..6 {
        scene.frame.tick = tick;
        reached = r.render(&scene, &RenderTarget::Offscreen).expect("render");
        waited += reached.compile_wait_us;
    }
    let after = made();
    og_render::traffic::set_clock(None);
    println!("{MADE_AHEAD:?} made out of reach {ahead:?}, after the camera reached them {after:?}");
    assert!(
        reached.scatter_draws > 0,
        "the camera in reach draws the scatter's chunks"
    );
    assert!(
        ahead.iter().all(|&n| n > 0),
        "made before the camera reached them: {ahead:?} of {MADE_AHEAD:?}"
    );
    assert_eq!(
        after, ahead,
        "pipelines of {MADE_AHEAD:?} made once the camera reached them"
    );
    assert_eq!(
        waited, 0,
        "the frames reaching them waited {waited} µs for a compile"
    );
}

/// G5: under a sun 10 degrees up, a dense meadow (one colour root to tip,
/// no sky light) is more than twice as bright at its tips as at its roots:
/// the canopy between a blade's foot and the sun, and the blades' own
/// cascade shadows (now cast by default), darken it from inside. Seen from
/// the side through an orthographic camera, so each screen row is one
/// height above the ground. Before G5 a blade was lit alike root to tip
/// (no canopy, no shadows by default): a ratio near 1.
#[test]
#[ignore = "requires GPU"]
fn a_low_sun_darkens_the_meadow_towards_its_roots() {
    let mut scene = trample_scene();
    let g = &mut scene.world.grass[0].grass;
    g.distance = 30.0;
    g.height = 0.5;
    g.height_variation = 0.0;
    g.density = 400.0;
    g.base_color = Vec3::new(0.3, 0.5, 0.15);
    g.tip_color = g.base_color;
    g.translucency = 0.0;
    g.cast_shadows = og_render::Grass::default().cast_shadows;
    scene.lighting.light.cast_shadows = true;
    let up = 10f32.to_radians();
    scene.lighting.light.direction_to_light = Vec3::new(0.0, up.sin(), up.cos());
    scene.frame.camera = Camera {
        position: Vec3::new(-20.0, 0.25, 0.0),
        target: Vec3::new(0.0, 0.25, 0.0),
        near: 0.1,
        far: 60.0,
        projection: Projection::Orthographic { height: 0.8 },
        ..Default::default()
    };
    let (img, _) = render(&scene);
    save("og-terrain", "grass-low-sun", &img);
    // Rows 0.8 m tall over the frame's height, centred 0.25 m up: the
    // band 0.02..0.1 m over the ground and the band 0.38..0.46 m.
    let row = |y: f32| ((0.25 + 0.4 - y) / 0.8 * img.height as f32) as u32;
    let band = |lo: f32, hi: f32| {
        let (mut sum, mut n) = (0.0, 0);
        for y in row(hi)..row(lo) {
            for x in 0..img.width {
                let c = px_f32(&img, x, y);
                if c[1] > 2.0 {
                    sum += luma_f32(c);
                    n += 1;
                }
            }
        }
        (sum / n.max(1) as f32, n)
    };
    let (roots, nr) = band(0.02, 0.1);
    let (tips, nt) = band(0.38, 0.46);
    println!("luma at the roots {roots:.2} ({nr} px), at the tips {tips:.2} ({nt} px)");
    assert!(nr > 1000 && nt > 1000, "the field covers both bands");
    assert!(
        tips > 2.0 * roots,
        "tips {tips:.2} against roots {roots:.2}"
    );
}

/// A decal marks the grass that stands in it: hung at the blades' middle
/// height, clear of the ground, a red decal two metres square reddens
/// the meadow seen from above inside its square and nowhere else.
///
/// Ways this could fail: the grass never calls the decals (as before), or
/// judges them by a blade's own near-level normal, which a decal
/// projected down onto the ground never faces.
#[test]
#[ignore = "requires GPU"]
fn a_decal_marks_the_grass_that_stands_in_it() {
    use og_render::{ColorSpace, Decal, Mips, TextureData};
    let mut scene = trample_scene();
    let grass = &mut scene.world.grass[0].grass;
    grass.distance = 30.0;
    grass.base_color = Vec3::new(0.05, 0.5, 0.05);
    grass.tip_color = Vec3::new(0.05, 0.5, 0.05);
    scene.look.post.tonemap = Tonemap::None;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 10.0, 0.001),
        target: Vec3::ZERO,
        near: 1.0,
        far: 50.0,
        projection: Projection::Orthographic { height: 8.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut capture = |s: &RenderScene| {
        r.render(s, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    let bare = capture(&scene);
    let atlas = scene.add_texture(TextureData {
        width: 2,
        height: 2,
        rgba: [255, 0, 0, 255].repeat(4),
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    });
    scene.effects.decal_atlas = Some(atlas);
    scene.effects.decals.push(Decal {
        emissive: Vec3::X,
        ..Decal::on_surface(Vec3::Y * 0.35, Vec3::Y, Vec2::splat(2.0), 0.3, 0.0)
    });
    let marked = capture(&scene);
    save("og-terrain", "grass-decal", &marked);
    // The square is the frame's middle quarter (2 m of the 8 m height).
    let (w, h) = (bare.width, bare.height);
    let side = h / 4;
    let inside = |x: u32, y: u32| x.abs_diff(w / 2) < side / 2 && y.abs_diff(h / 2) < side / 2;
    let redder = |x: u32, y: u32| px_f32(&marked, x, y)[0] > px_f32(&bare, x, y)[0] + 30.0;
    let all = || (0..h).flat_map(|y| (0..w).map(move |x| (x, y)));
    let within = all().filter(|&(x, y)| inside(x, y) && redder(x, y)).count();
    let beyond = all()
        .filter(|&(x, y)| {
            x.abs_diff(w / 2) > side / 2 + 4 && y.abs_diff(h / 2) > side / 2 + 4 && redder(x, y)
        })
        .count();
    // Seen from straight above a blade is a few pixels wide, and the
    // decal's box holds only its middle.
    assert!(
        within as u32 > side * side / 20,
        "{within} of the square's {} pixels reddened",
        side * side
    );
    assert_eq!(beyond, 0, "pixels reddened outside the decal");
}
