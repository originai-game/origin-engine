//! The E5 round's rendering (`crates/og-render/README.md` §Atmosphere,
//! §Water, §Material shaders): each scene matches its reference for this
//! backend tier, and behaviour read from rendered frames: fog hides the
//! distance, not the foreground; the day-night look darkens from noon to
//! dusk to night and draws stars only at night; water darkens with depth,
//! foams where it meets geometry, ripples with its height field and
//! reflects; the 2D water line follows its surface and draws in the sprite
//! order; metaballs merge particles within reach and keep apart those that
//! are not; a dissolve shader cuts holes by its amount and the grass
//! shader moves only the grass with the clock.

use glam::{Mat4, Vec2, Vec3, Vec4};
use og_render::{Camera, RenderScene, RenderTarget, Renderer, RgbaImage, Sprite, Water2d};
use og_tests::atmo_scenes as atmo;
use og_tests::gfx;
use og_tests::scenes;
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

fn frame(scene: &RenderScene) -> RgbaImage {
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    r.render(scene, &RenderTarget::Offscreen).expect("render");
    r.read_offscreen().expect("capture")
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

/// Mean colour over a rectangle.
fn rgb(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    for y in y0..y1 {
        for x in x0..x1 {
            let p = px(img, x, y);
            for c in 0..3 {
                sum[c] += p[c];
            }
        }
    }
    sum.map(|s| s / ((x1 - x0) * (y1 - y0)) as f32)
}

/// Mean luma over a rectangle.
fn mean(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += luma(px(img, x, y));
        }
    }
    sum / ((x1 - x0) * (y1 - y0)) as f32
}

/// Mean absolute difference over a rectangle (all channels).
fn diff(a: &RgbaImage, b: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let (p, q) = (px(a, x, y), px(b, x, y));
            sum += (p[0] - q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs();
        }
    }
    sum / ((x1 - x0) * (y1 - y0) * 3) as f32
}

/// Luma's standard deviation over a rectangle (how much detail shows).
fn contrast(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let m = mean(img, x0, y0, x1, y1);
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += (luma(px(img, x, y)) - m).powi(2);
        }
    }
    (sum / ((x1 - x0) * (y1 - y0)) as f32).sqrt()
}

#[test]
fn atmosphere_scenes_match_their_references() {
    for scene in ["atmosphere-fog", "atmosphere-dusk", "atmosphere-night"] {
        check(scene);
    }
}

/// Rain and snow over the colonnade: their particles fill the view after
/// the warm-up, under the preset's fog and dimmed light. The presets: rain
/// dims more than snow, `dim` 0 and intensity 0 leave the light alone,
/// mist has no particles, and an orthographic camera gets 2D sprites.
#[test]
fn weather_presets_match_their_references() {
    for scene in ["weather-rain", "weather-snow"] {
        let r = check(scene);
        assert_eq!(
            r.stats.particle_systems, 1,
            "{scene}: the preset's particles"
        );
        assert!(
            r.stats.particle_capacity >= 6000,
            "{scene}: {}",
            r.stats.particle_capacity
        );
    }
    let w = |kind, intensity, dim, layer| {
        og_render::weather(kind, intensity, Vec3::X, 18.0, Vec3::ONE, dim, layer)
    };
    let (rain, snow) = (
        w(og_render::WEATHER_RAIN, 1.0, 1.0, None),
        w(og_render::WEATHER_SNOW, 1.0, 1.0, None),
    );
    assert!(rain.light < snow.light && snow.light < 1.0);
    assert_eq!(w(og_render::WEATHER_RAIN, 1.0, 0.0, None).light, 1.0);
    assert_eq!(w(og_render::WEATHER_RAIN, 0.0, 1.0, None).light, 1.0);
    let mist = w(og_render::WEATHER_MIST, 1.0, 1.0, None);
    assert!(mist.emitter.is_none() && mist.fog.is_some_and(|f| f.height_density > 0.0));
    let flat = w(og_render::WEATHER_SNOW, 1.0, 1.0, Some(7))
        .emitter
        .unwrap();
    assert_eq!(flat.render, og_render::ParticleRender::Sprite { layer: 7 });
}

#[test]
fn water_scenes_match_their_references() {
    let w = check("water-3d");
    assert!(w.stats.prepass_draws > 0, "water reads the depth prepass");
    check("water-2d");
    check("fluid-2d");
}

/// The pixel of world point (x, y) in the fluid-groups scene's camera
/// (orthographic, centred on (-0.4, 0.4), 7 units tall over 720 pixels).
fn group_px(img: &RgbaImage, x: f32, y: f32) -> [f32; 3] {
    let k = 720.0 / 7.0;
    px(
        img,
        (640.0 + (x + 0.4) * k) as u32,
        (360.0 - (y - 0.4) * k) as u32,
    )
}

#[test]
fn ten_fluids_draw_in_one_frame_and_a_group_blends_its_colours() {
    check("fluid-groups");
    let img = frame(&atmo::fluid_groups_scene());
    // Six lone pools, each its own colour: more than the four of one field.
    // Red, amber, green, blue, purple, pink: (a brighter, a darker channel).
    let pools = [[0, 1], [0, 2], [1, 2], [2, 0], [2, 1], [0, 1]];
    for (i, [hi, lo]) in pools.iter().enumerate() {
        let c = group_px(&img, -5.0 + i as f32 * 2.0, 2.4);
        assert!(
            c[*hi] > c[*lo] + 60.0,
            "pool {i} is drawn in its colour: {c:?}"
        );
    }
    // The blended group: water blue on the left, oil orange on the right,
    // and between them a blend of the two, not one or the other.
    let (water, oil, mid) = (
        group_px(&img, -4.8, -1.0),
        group_px(&img, 0.2, -1.0),
        group_px(&img, -2.5, -1.0),
    );
    assert!(
        water[2] > water[0] + 60.0 && oil[0] > oil[2] + 60.0,
        "water {water:?}, oil {oil:?}"
    );
    for k in [0, 2] {
        let (lo, hi) = (water[k].min(oil[k]), water[k].max(oil[k]));
        assert!(
            mid[k] > lo + 30.0 && mid[k] < hi - 30.0,
            "the band blends channel {k}: {mid:?} between {water:?} and {oil:?}"
        );
    }
    // The layered group: purple under green, one surface with a boundary.
    let (heavy, light) = (group_px(&img, 3.6, -1.35), group_px(&img, 3.6, -0.55));
    assert!(
        heavy[2] > heavy[1] + 40.0 && light[1] > light[2] + 40.0,
        "heavy {heavy:?}, light {light:?}"
    );
}

#[test]
fn shader_hooks_match_their_reference() {
    check("shader-hooks");
}

/// Fog takes the detail out of the far colonnade and leaves the near
/// ground almost as it was.
#[test]
fn fog_hides_the_distance_not_the_foreground() {
    // Distance fog only (height fog covers the near ground on purpose).
    let mut fogged = atmo::fog_scene();
    if let Some(f) = &mut fogged.fog {
        f.height_density = 0.0;
    }
    let fogged = frame(&fogged);
    let mut clear = atmo::fog_scene();
    clear.fog = None;
    let clear = frame(&clear);
    // The vanishing point of the colonnade, and the ground at the bottom.
    let far = (560, 300, 720, 400);
    let near = (300, 660, 980, 720);
    let far_clear = contrast(&clear, far.0, far.1, far.2, far.3);
    let far_fogged = contrast(&fogged, far.0, far.1, far.2, far.3);
    assert!(
        far_fogged < far_clear * 0.5,
        "far detail {far_clear:.1} -> {far_fogged:.1} with fog"
    );
    let d_near = diff(&clear, &fogged, near.0, near.1, near.2, near.3);
    let d_far = diff(&clear, &fogged, far.0, far.1, far.2, far.3);
    assert!(
        d_far > d_near * 2.0,
        "fog changes the distance ({d_far:.1}) more than the foreground ({d_near:.1})"
    );
}

/// The same place at noon, dusk and night: the light falls, the sky turns,
/// and only the night sky has stars (isolated bright pixels).
#[test]
fn day_night_darkens_towards_night_with_stars() {
    let noon = frame(&atmo::day_night_scene(12.0, false));
    let dusk = frame(&atmo::day_night_scene(18.2, false));
    let night = frame(&atmo::day_night_scene(23.0, false));
    let all = |img: &RgbaImage| mean(img, 0, 0, 1280, 720);
    let (a, b, c) = (all(&noon), all(&dusk), all(&night));
    assert!(a > b && b > c, "noon {a:.1} > dusk {b:.1} > night {c:.1}");
    // The light itself: the sun weakens towards dusk; at night the moon,
    // opposite the sun and dim, lights the scene.
    let (l12, l18, l23) = (
        og_render::day_night(12.0, 25.0, -20.0),
        og_render::day_night(18.2, 25.0, -20.0),
        og_render::day_night(23.0, 25.0, -20.0),
    );
    assert!(
        l12.light_intensity > 2.0 * l18.light_intensity,
        "{} vs {}",
        l12.light_intensity,
        l18.light_intensity
    );
    assert!(
        l23.light_direction.dot(l23.sun_direction) < -0.99,
        "the moon is opposite the sun"
    );
    assert!(l23.light_intensity > 0.0 && l23.light_intensity < 0.2 * l12.light_intensity);
    // Stars: sky pixels much brighter than their neighbours, between the
    // two nearest pillars (x 420..840, y 20..300).
    let stars = |img: &RgbaImage| {
        let mut n = 0;
        for y in 21..300 {
            for x in 421..840 {
                let l = luma(px(img, x, y));
                let around = (luma(px(img, x - 1, y))
                    + luma(px(img, x + 1, y))
                    + luma(px(img, x, y - 1))
                    + luma(px(img, x, y + 1)))
                    / 4.0;
                if l > around + 40.0 {
                    n += 1;
                }
            }
        }
        n
    };
    assert!(stars(&night) > 20, "night has stars ({})", stars(&night));
    assert_eq!(stars(&noon), 0, "noon has none");
}

/// The pool seen from above: the shallow near end is lighter than the
/// deep far end; foam rings the posts; the height field moves the surface;
/// reflectivity changes what the surface shows.
#[test]
fn water_darkens_with_depth_foams_ripples_and_reflects() {
    let above = |f: &dyn Fn(&mut og_render::Water)| {
        let mut s = atmo::water_scene();
        s.camera = Camera {
            position: Vec3::new(0.0, 14.0, -5.0),
            target: Vec3::new(0.0, 0.0, -5.01),
            ..s.camera
        };
        // A steeper bed: 0.2 m deep at the image's bottom edge, 3.5 m at its top.
        s.instances[0].transform =
            Mat4::from_translation(Vec3::new(0.0, -5.55, -22.0)) * Mat4::from_rotation_x(-0.206);
        f(&mut s.water[0]);
        frame(&s)
    };
    let base = above(&|_| {});
    // Near end (image bottom, under 1 m deep) against the far end (top, over 3 m).
    let shallow = mean(&base, 400, 620, 880, 700);
    let deep = mean(&base, 400, 20, 880, 100);
    assert!(
        shallow > deep + 15.0,
        "shallow {shallow:.1} vs deep {deep:.1}"
    );
    let no_foam = above(&|w| w.foam_width = 0.0);
    let changed = (0..720)
        .flat_map(|y| (0..1280).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let (p, q) = (px(&base, x, y), px(&no_foam, x, y));
            (luma(p) - luma(q)).abs() > 20.0
        })
        .count();
    assert!(
        changed > 200 && changed < 1280 * 720 / 10,
        "foam lights a band where water meets geometry ({changed} pixels)"
    );
    let matte = above(&|w| w.reflectivity = 0.0);
    let side = frame(&atmo::water_scene());
    let mut flat = atmo::water_scene();
    flat.water[0].heights = None;
    let flat = frame(&flat);
    let moved = diff(&side, &flat, 0, 230, 1280, 720);
    assert!(
        moved > 1.0,
        "the height field moves the surface ({moved:.2})"
    );
    let mut side_matte = atmo::water_scene();
    side_matte.water[0].reflectivity = 0.0;
    let side_matte = frame(&side_matte);
    assert!(
        diff(&side, &side_matte, 0, 300, 1280, 720) > diff(&base, &matte, 0, 0, 1280, 720),
        "reflections matter more at a grazing view than from above"
    );
}

/// The topmost row in column `x` whose colour is the water lip's (bright
/// blue-white) between rows `y0..y1`.
fn lip_row(img: &RgbaImage, x: u32) -> Option<u32> {
    (0..720).find(|&y| {
        let p = px(img, x, y);
        p[2] > 180.0 && p[1] > 160.0 && p[0] > 120.0
    })
}

/// The 2D water line follows its surface: raising one end raises the lip
/// there and nowhere else; it draws between the sprites below and above
/// its layer.
#[test]
fn water_2d_follows_its_surface_in_the_sprite_order() {
    let body = |surface: Vec<f32>| {
        let mut s = RenderScene::new();
        s.camera = Camera::orthographic_2d(Vec2::ZERO, 12.0);
        s.environment.background = og_render::Background::Color(Vec3::ZERO);
        s.post.tonemap = og_render::Tonemap::None;
        s.water_2d.push(Water2d {
            origin: Vec2::new(-10.0, -1.0),
            width: 20.0,
            depth: 4.0,
            surface,
            lip_width: 0.12,
            layer: 2,
            ..Default::default()
        });
        s
    };
    let flat = frame(&body(vec![0.0; 11]));
    let mut raised = vec![0.0; 11];
    raised[2] = 2.0;
    let raised_img = frame(&body(raised));
    // x = -6 world is pixel 640 - 6 * 60 = 280; 1 unit is 60 pixels.
    let (a, b) = (
        lip_row(&flat, 280).unwrap(),
        lip_row(&raised_img, 280).unwrap(),
    );
    assert!(
        a - b > 100 && a - b < 140,
        "the lip rose by 2 units: {a} -> {b}"
    );
    let (c, d) = (
        lip_row(&flat, 1000).unwrap(),
        lip_row(&raised_img, 1000).unwrap(),
    );
    assert!(c.abs_diff(d) <= 1, "the far end stays: {c} -> {d}");
    // A red sprite below the water's layer is tinted by it; one above is not.
    let mut s = body(vec![0.0; 11]);
    for (x, layer) in [(-3.0, 1), (3.0, 3)] {
        s.add_sprite(Sprite {
            position: Vec3::new(x, -3.0, 0.0),
            size: Vec2::splat(1.0),
            color: [1.0, 0.0, 0.0, 1.0],
            layer,
            ..Default::default()
        });
    }
    let img = frame(&s);
    let below = px(&img, 640 - 180, 360 + 180);
    let above = px(&img, 640 + 180, 360 + 180);
    assert!(
        above[0] > 250.0 && above[2] < 5.0,
        "above the water: pure red {above:?}"
    );
    assert!(
        below[0] < 200.0 && below[2] > 40.0,
        "under the water: tinted {below:?}"
    );
}

/// Two particles closer than their kernels' reach merge into one blob
/// (the midpoint is covered); two further apart stay two.
#[test]
fn metaballs_merge_within_reach() {
    let fluid = |gap: f32| {
        let mut s = RenderScene::new();
        s.camera = Camera::orthographic_2d(Vec2::ZERO, 12.0);
        s.environment.background = og_render::Background::Color(Vec3::ZERO);
        s.post.tonemap = og_render::Tonemap::None;
        s.metaballs.push(og_render::Metaballs {
            particles: Arc::from(vec![Vec2::new(-gap / 2.0, 0.0), Vec2::new(gap / 2.0, 0.0)]),
            radius: 1.0,
            threshold: 0.5,
            color: Vec4::new(0.0, 0.0, 1.0, 1.0),
            rim_width: 0.0,
            ..Default::default()
        });
        frame(&s)
    };
    // Each kernel is (1 - d²)²: at the midpoint of a gap g it adds
    // 2 (1 - g²/4)², over 0.5 for g < 1.53.
    let near = fluid(1.2);
    let far = fluid(1.8);
    let mid = |img: &RgbaImage| px(img, 640, 360)[2];
    assert!(mid(&near) > 200.0, "merged: {}", mid(&near));
    assert!(mid(&far) < 5.0, "apart: {}", mid(&far));
    // Each blob's own centre is covered either way.
    assert!(px(&far, 640 + 54, 360)[2] > 200.0);
}

/// The dissolve shader cuts holes by its amount: a white quad over a
/// black background keeps every pixel at amount 0 and loses a share at 0.6
/// (with no edge glow, a hole is background); unknown parameters are
/// refused. A parameter set between two frames of one renderer draws as
/// one set before the first (the renderer writes it into the material in
/// place). The grass shader moves the grass with the clock and nothing
/// else.
#[test]
fn shader_hooks_dissolve_and_sway() {
    let scene = |amount: f32| {
        let mut s = RenderScene::new();
        let shader = s.add_shader(atmo::dissolve_shader());
        let quad = s.add_mesh(og_render::shapes::cuboid(Vec3::new(4.0, 4.0, 0.1)));
        let m = s.add_standard_material(og_render::StandardMaterial {
            unlit: 1,
            alphaTest: 0.5,
            ..Default::default()
        });
        assert!(s.set_material_shader(m, Some(shader)));
        assert!(s.set_shader_param(m, "amount", &[amount]));
        assert!(s.set_shader_param(m, "edgeColor", &[0.0, 0.0, 0.0]));
        assert!(
            !s.set_shader_param(m, "nothing", &[1.0]),
            "unknown parameter"
        );
        s.add_instance(quad, m, Mat4::IDENTITY);
        s.environment.background = og_render::Background::Color(Vec3::ZERO);
        s.camera = Camera {
            position: Vec3::new(0.0, 0.0, 6.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        (s, m)
    };
    let with_amount = |amount: f32| frame(&scene(amount).0);
    let lit = |img: &RgbaImage| {
        (260..460)
            .flat_map(|y| (540..740).map(move |x| (x, y)))
            .filter(|&(x, y)| luma(px(img, x, y)) > 30.0)
            .count()
    };
    let (whole, cut) = (lit(&with_amount(0.0)), lit(&with_amount(0.6)));
    assert_eq!(whole, 200 * 200, "amount 0 keeps the whole quad");
    assert!(
        cut < whole * 3 / 4 && cut > whole / 20,
        "amount 0.6 cuts holes: {whole} -> {cut}"
    );
    let (mut s, m) = scene(0.0);
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |s: &RenderScene| {
        r.render(s, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    assert_eq!(lit(&draw(&s)), whole, "the first frame at amount 0");
    assert!(s.set_shader_param(m, "amount", &[0.6]));
    assert_eq!(lit(&draw(&s)), cut, "amount 0.6 set after the first frame");
    let early = frame(&atmo::hooks_scene(0));
    let late = frame(&atmo::hooks_scene(40));
    let grass = diff(&early, &late, 0, 500, 1280, 720);
    let objects = diff(&early, &late, 250, 200, 1000, 380);
    assert!(grass > 1.0, "the grass moved ({grass:.2})");
    assert!(objects < grass * 0.1, "the rest did not ({objects:.2})");
}

/// A mesh replaced with RenderScene::set_mesh (cloth) draws its new shape
/// on the next frame, in place when the sizes match and in new buffers when
/// they do not; a skinned mesh is refused.
#[test]
fn a_replaced_mesh_draws_its_new_shape() {
    let mut s = RenderScene::new();
    let quad = |x: f32, w: f32| og_render::MeshData {
        vertices: [(x - w, -1.0), (x + w, -1.0), (x + w, 1.0), (x - w, 1.0)]
            .iter()
            .map(|&(px, py)| og_render::Vertex {
                tangent: [0.0; 4],
                position: [px, py, 0.0],
                normal: [0.0, 0.0, 1.0],
                ..Default::default()
            })
            .collect(),
        indices: vec![0, 1, 2, 0, 2, 3],
    };
    let mesh = s.add_mesh(quad(-2.0, 1.0));
    let white = s.add_standard_material(og_render::StandardMaterial {
        unlit: 1,
        side: 2,
        ..Default::default()
    });
    s.add_instance(mesh, white, Mat4::IDENTITY);
    s.environment.background = og_render::Background::Color(Vec3::ZERO);
    s.camera = Camera {
        position: Vec3::new(0.0, 0.0, 8.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |s: &RenderScene| {
        r.render(s, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    let lit = |img: &RgbaImage, x: u32| luma(px(img, x, 360)) > 100.0;
    let a = draw(&s);
    assert!(lit(&a, 420) && !lit(&a, 860), "the quad on the left");
    assert!(s.set_mesh(mesh, quad(2.0, 1.0)));
    let b = draw(&s);
    assert!(!lit(&b, 420) && lit(&b, 860), "moved in place to the right");
    let mut wide = quad(0.0, 3.0);
    wide.vertices.push(wide.vertices[0]);
    assert!(s.set_mesh(mesh, wide));
    let c = draw(&s);
    assert!(lit(&c, 420) && lit(&c, 860), "a bigger mesh in new buffers");
    let skinned = s.add_skinned_mesh(
        quad(0.0, 1.0),
        og_render::SkinData {
            joints: vec![[0; 4]; 4],
            weights: vec![[1.0, 0.0, 0.0, 0.0]; 4],
        },
    );
    assert!(
        !s.set_mesh(skinned, quad(0.0, 1.0)),
        "skinned meshes are refused"
    );
    // Draw refinement changes the GPU geometry, never the submitted solve.
    // TAA exercises the surface-only prepass and its previous-position stream.
    let mut desc = scenes::base_desc();
    desc.antialias = og_render::Antialias::Taa;
    let mut r = Renderer::new_blocking(&desc).expect("surface renderer");
    for (level, x, triangles) in [(0, -2.0, 8), (3, 2.0, 18), (1, -2.0, 2)] {
        let coarse = quad(x, 1.0);
        s.cloth_subdivisions = level;
        assert!(s.set_simulation_surface(
            mesh,
            coarse.clone(),
            og_render::SimulationSurface::SoftBody
        ));
        let stats = r
            .render(&s, &RenderTarget::Offscreen)
            .expect("surface render");
        assert_eq!(stats.triangles, triangles);
        assert!(
            stats.prepass_draws > 0,
            "vertex-only motion needs TAA vectors"
        );
        assert_eq!(s.mesh(mesh), Some(&coarse));
        let image = r.read_offscreen().expect("surface capture");
        assert!(lit(&image, if x < 0.0 { 420 } else { 860 }));
        // Hold the subdivision level while moving the coarse solve, so the
        // second frame updates existing buffers rather than recreating them.
        assert!(s.set_simulation_surface(
            mesh,
            quad(-x, 1.0),
            og_render::SimulationSurface::SoftBody
        ));
        let stats = r
            .render(&s, &RenderTarget::Offscreen)
            .expect("surface moved");
        assert_eq!(stats.triangles, triangles);
        let image = r.read_offscreen().expect("moved surface capture");
        assert!(lit(&image, if x < 0.0 { 860 } else { 420 }));
    }
}

/// R4's physical sky over the colonnade: each frame matches its
/// reference, and reads as the sky it is. At noon the band above the
/// pillars is blue (blue over red); looking into the setting sun the
/// horizon below it is warm (red over blue); in twilight the sky is far
/// darker than at noon (half as bright with the camera opened 3.5 stops)
/// yet still blue overhead (ozone); a hazy afternoon
/// is paler (less blue for its brightness) than a clear noon.
#[test]
fn physical_sky_scenes_match_their_references_and_read_as_sky() {
    use og_tests::sky_scenes::{SKY_SCENES, sky_scene};
    for scene in SKY_SCENES.iter().filter(|s| s.starts_with("sky-")) {
        check(scene);
    }
    let noon = frame(&sky_scene("sky-physical-noon"));
    let top = rgb(&noon, 560, 0, 720, 40);
    assert!(top[2] > top[0] + 20.0, "noon is blue overhead: {top:?}");
    let sunset = frame(&sky_scene("sky-physical-sunset"));
    let low = rgb(&sunset, 400, 250, 880, 330);
    assert!(low[0] > low[2] + 20.0, "the sunset is warm: {low:?}");
    let twilight = frame(&sky_scene("sky-physical-twilight"));
    let dusk_top = rgb(&twilight, 560, 0, 720, 40);
    assert!(
        luma(dusk_top) * 2.0 < luma(top),
        "twilight is darker: {dusk_top:?} vs {top:?}"
    );
    assert!(
        dusk_top[2] >= dusk_top[0],
        "and blue overhead: {dusk_top:?}"
    );
    // The land keeps a little of the twilight sky's light (the art
    // director's 3–6 % of noon), not the tone curve's black.
    let dusk_land = mean(&twilight, 200, 640, 1080, 720);
    assert!(
        dusk_land > 6.0,
        "the twilight land is not black: {dusk_land}"
    );
    let hazy = frame(&sky_scene("sky-physical-hazy"));
    let h = rgb(&hazy, 560, 0, 720, 40);
    let blueness = |c: [f32; 3]| (c[2] - c[0]) / luma(c).max(1.0);
    assert!(
        blueness(h) < blueness(top),
        "haze pales the sky: {h:?} vs {top:?}"
    );
}

/// R4's clouds: each frame matches its reference; fair-weather cumulus
/// break up the blue (much more variation across the upper sky than the
/// clear noon sky has); an overcast sky is grey (little blue over red) and
/// darker on the ground than the fair one; the clouds over the sunset are
/// lit warm.
#[test]
fn cloud_scenes_match_their_references_and_read_as_clouds() {
    use og_tests::sky_scenes::{SKY_SCENES, sky_scene};
    for scene in SKY_SCENES.iter().filter(|s| s.starts_with("clouds-")) {
        check(scene);
    }
    let clear = frame(&sky_scene("clouds-fair-clear"));
    let fair = frame(&sky_scene("clouds-fair"));
    // The upper sky (y 0–320) against the same frame without clouds,
    // over the pixels that are blue sky there (the pillars left out, their
    // edges would outweigh any cloud): flat-based cumulus sit in its lower
    // part, at 15–35° up. Metal measured 1.8× (19.0 vs 10.5, the clear
    // band's spread being its own zenith-to-horizon gradient); 1.5× holds
    // that with margin and still fails a clear sky.
    let sky: Vec<(u32, u32)> = (0..320)
        .flat_map(|y| (0..1280).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(&clear, x, y);
            p[2] > p[0] + 30.0
        })
        .collect();
    assert!(sky.len() > 100_000, "the band is mostly sky: {}", sky.len());
    let spread = |img: &RgbaImage| {
        let l: Vec<f32> = sky.iter().map(|&(x, y)| luma(px(img, x, y))).collect();
        let m = l.iter().sum::<f32>() / l.len() as f32;
        (l.iter().map(|v| (v - m).powi(2)).sum::<f32>() / l.len() as f32).sqrt()
    };
    assert!(
        spread(&fair) > 1.5 * spread(&clear),
        "cumulus break up the sky: {} vs {}",
        spread(&fair),
        spread(&clear)
    );
    let overcast = frame(&sky_scene("clouds-overcast"));
    let top = px(&overcast, 640, 30);
    assert!(top[2] - top[0] < 25.0, "overcast is grey: {top:?}");
    // Darker on the ground at the same exposure (the overcast's camera
    // opens up for the hidden sun, as a meter would).
    let level = {
        let mut s = sky_scene("clouds-overcast");
        s.camera.exposure = sky_scene("clouds-fair").camera.exposure;
        frame(&s)
    };
    assert!(
        mean(&level, 400, 560, 880, 700) < mean(&fair, 400, 560, 880, 700),
        "and darker on the ground"
    );
    let sunset = frame(&sky_scene("clouds-sunset"));
    // The cumulus band over the horizon (the cirrus above is faint, the
    // sky between its streaks blue).
    let warm = rgb(&sunset, 400, 480, 880, 600);
    assert!(warm[0] > warm[2], "the sunset clouds are warm: {warm:?}");
}

/// The aurora: each frame matches its reference; the arcs turn the night
/// sky over the colonnade green (green gained well over blue against the
/// same night without them) and break it up into curtains (more variation
/// across the upper sky).
#[test]
fn aurora_scenes_match_their_references_and_read_as_aurora() {
    use og_tests::sky_scenes::{SKY_SCENES, sky_scene};
    for scene in SKY_SCENES.iter().filter(|s| s.starts_with("aurora-")) {
        check(scene);
    }
    let clear = frame(&sky_scene("aurora-arcs-clear"));
    let arcs = frame(&sky_scene("aurora-arcs"));
    let (sky_clear, sky_arcs) = (rgb(&clear, 0, 0, 1280, 300), rgb(&arcs, 0, 0, 1280, 300));
    let green = sky_arcs[1] - sky_clear[1];
    let blue = sky_arcs[2] - sky_clear[2];
    assert!(
        green > 4.0 && green > 2.0 * blue,
        "the arcs are green: {sky_clear:?} -> {sky_arcs:?}"
    );
    let spread = |img: &RgbaImage| {
        let l: Vec<f32> = (0..300)
            .flat_map(|y| (0..1280).step_by(2).map(move |x| (x, y)))
            .map(|(x, y)| luma(px(img, x, y)))
            .collect();
        let m = l.iter().sum::<f32>() / l.len() as f32;
        (l.iter().map(|v| (v - m).powi(2)).sum::<f32>() / l.len() as f32).sqrt()
    };
    assert!(
        spread(&arcs) > 1.5 * spread(&clear),
        "curtains break up the sky: {} vs {}",
        spread(&arcs),
        spread(&clear)
    );
}

/// R4's weather on surfaces: each frame matches its reference. The wet
/// ground is darker than the same ground dry; snow makes it much brighter
/// and whiter; the lightning frame is brighter than the same night
/// without the strike, the flash lighting the ground too, with the bolt
/// (a bright, blue-white column) in view.
#[test]
fn weather_surface_scenes_match_their_references_and_read_as_weather() {
    use og_tests::sky_scenes::{SKY_SCENES, sky_scene};
    for scene in SKY_SCENES.iter().filter(|s| s.starts_with("weather-")) {
        check(scene);
    }
    let dry = {
        let mut s = sky_scene("weather-wet");
        s.surface_weather = None;
        frame(&s)
    };
    // Wet ground darkens; the puddles are mirrors, so they are left out
    // (they show the sky, which may be brighter than the ground).
    let damp = {
        let mut s = sky_scene("weather-wet");
        if let Some(w) = s.surface_weather.as_mut() {
            w.puddles = 0.0;
            w.rain = 0.0;
        }
        frame(&s)
    };
    let ground = |img: &RgbaImage| mean(img, 200, 560, 1080, 720);
    assert!(
        ground(&damp) < 0.85 * ground(&dry),
        "wet ground is darker: {} vs {}",
        ground(&damp),
        ground(&dry)
    );
    let snow = frame(&sky_scene("weather-snow-cover"));
    assert!(ground(&snow) > 1.4 * ground(&dry), "snow is bright");
    let p = px(&snow, 640, 650);
    assert!((p[0] - p[2]).abs() < 30.0, "and white: {p:?}");
    let storm = frame(&sky_scene("weather-lightning"));
    let calm = frame(&og_tests::sky_scenes::storm(false));
    assert!(mean(&storm, 0, 0, 1280, 720) > 1.3 * mean(&calm, 0, 0, 1280, 720));
    assert!(
        mean(&storm, 0, 560, 1280, 720) > 1.5 * mean(&calm, 0, 560, 1280, 720),
        "the flash lights the ground"
    );
}

/// R4's shelter: the ground under the canopy stays free of snow and dry,
/// whether the canopy is plain meshes or clustered ones (the meshlet path).
/// With the weather occlusion map turned off (shelter 0) the snow lies
/// under it too, so the frame's lower half is brighter; with it the ground
/// under the canopy is darker than the open snow; and in the rain (its
/// puddles and ripples and the sun's shadows off) the ground under the
/// canopy with the map looks as it does with no weather, and without it
/// unlike it: dry, not soaked.
#[test]
fn a_canopy_keeps_the_snow_and_the_rain_off_the_ground_under_it() {
    use og_tests::sky_scenes::sky_scene;
    let open = |name: &str| {
        let mut s = sky_scene(name);
        if let Some(w) = s.surface_weather.as_mut() {
            w.shelter = 0.0;
        }
        frame(&s)
    };
    let lower = |img: &RgbaImage| mean(img, 0, 400, 1280, 720);
    let snow = frame(&sky_scene("weather-shelter-snow"));
    let snowed_under = open("weather-shelter-snow");
    assert!(
        lower(&snow) < 0.95 * lower(&snowed_under),
        "the canopy keeps the snow off: {} vs {}",
        lower(&snow),
        lower(&snowed_under)
    );
    // A canopy of clustered meshes (the meshlet path) shelters the same:
    // the snow is kept off as without the map it is not, and the ground
    // under it matches the plain canopy's to a few percent.
    let clustered = frame(&sky_scene("weather-shelter-meshlet"));
    assert!(
        lower(&clustered) < 0.95 * lower(&open("weather-shelter-meshlet")),
        "a clustered canopy keeps the snow off"
    );
    let under = |img: &RgbaImage| mean(img, 300, 435, 900, 495);
    assert!(
        (under(&clustered) - under(&snow)).abs() < 0.05 * under(&snow),
        "as the plain canopy does: {} vs {}",
        under(&clustered),
        under(&snow)
    );
    // The ground under the canopy, with and without the map, the puddles
    // and ripples left out of both (mirrors of the sky, brighter than any
    // ground) and the sun's shadows off in both (the weather map is drawn
    // all the same), against the same ground with no weather at all: with
    // the map it is dry, as the ground with no weather is, and without it
    // soaked, far from it. (Which way the wet film moves the ground's
    // light depends on what lights it: under the overcast deck, which hides
    // the sun, a film's gloss mirrors the bright horizon at this grazing
    // angle, where in the sun's light it darkened the ground; dryness is
    // the ground looking as it does dry.)
    let damp = |weather: Option<bool>| {
        let mut s = sky_scene("weather-shelter-rain");
        s.light.cast_shadows = false;
        match weather {
            None => s.surface_weather = None,
            Some(shelter) => {
                if let Some(w) = s.surface_weather.as_mut() {
                    w.puddles = 0.0;
                    w.rain = 0.0;
                    if !shelter {
                        w.shelter = 0.0;
                    }
                }
            }
        }
        frame(&s)
    };
    let (dry, soaked, bare) = (damp(Some(true)), damp(Some(false)), damp(None));
    let (dry, soaked, bare) = (under(&dry), under(&soaked), under(&bare));
    println!("under the canopy: sheltered {dry}, soaked {soaked}, no weather {bare}");
    assert!(
        (soaked - bare).abs() > 0.1 * bare && (dry - bare).abs() < 0.3 * (soaked - bare).abs(),
        "the canopy keeps the ground under it dry: sheltered {dry}, soaked {soaked}, no weather {bare}"
    );
}

/// The live sky (R3): the image-based light follows the per-pixel sky,
/// clouds included. live-sky matches its reference; its chrome sphere's
/// upper half, which reflects the sky out of the frame over the camera,
/// turns rose under the rose cloud deck (red gains on blue), which before
/// the live capture it never did (the light was the cloudless sky's). On
/// one renderer, a sky that changes under it reaches the reflection
/// within two captures (sixteen frames, a capture being spread over
/// eight) and then matches a renderer that drew the clouds from the start.
#[test]
fn the_live_sky_brings_the_clouds_into_reflections() {
    use og_tests::sky_scenes::sky_scene;
    check("live-sky");
    let (clear, clouded) = (sky_scene("live-sky-clear"), sky_scene("live-sky"));
    let run = |scenes: &[(&RenderScene, usize)]| {
        let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        for &(scene, frames) in scenes {
            for _ in 0..frames {
                r.render(scene, &RenderTarget::Offscreen).expect("render");
            }
        }
        r.read_offscreen().expect("capture")
    };
    // The sphere fills the middle of the frame (its centre the camera's
    // target, about 340 px across at 1280 × 720): a band of its upper
    // half, whose normals tilt up and back towards the camera.
    let band = (610, 210, 670, 270);
    let warmth = |img: &RgbaImage| {
        let c = rgb(img, band.0, band.1, band.2, band.3);
        c[0] - c[2]
    };
    let (before, after) = (run(&[(&clear, 1)]), run(&[(&clouded, 1)]));
    println!(
        "band (r - b): clear {}, clouded {}",
        warmth(&before),
        warmth(&after)
    );
    assert!(
        warmth(&after) > warmth(&before) + 20.0,
        "the sphere reflects the rose clouds: {} vs {}",
        warmth(&after),
        warmth(&before)
    );
    let switched = run(&[(&clear, 3), (&clouded, 16)]);
    let fresh = run(&[(&clouded, 19)]);
    let d = diff(&switched, &fresh, band.0, band.1, band.2, band.3);
    println!("switched against fresh: {d}");
    assert!(d < 4.0, "the clouds reached the reflection: {d}");
}
