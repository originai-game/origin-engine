//! The rendering fields of the Grand View Garden fork merge (PR #2), each
//! asserted on frames: the scene of `og_tests::fork_scenes` (registered in
//! `scenes::build`) rendered with and without its field, and the pixels
//! the field must change read where it changes them. Ways these could fail
//! are named on each test.

use glam::{Vec2, Vec3};
use og_render::{
    Antialias, FrameStats, Projection, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
};
use og_tests::fork_scenes as fs;
use og_tests::{gfx, scenes};
use std::path::PathBuf;

/// Frames are rendered at half the reference size (the same 16:9 view).
const W: u32 = 640;
const H: u32 = 360;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-fork-fields")
}

fn save(name: &str, img: &RgbaImage) {
    let _ = gfx::save_png(&out_dir().join(format!("{name}.png")), img);
}

/// A registered scene and its desc at the tests' size.
fn built(name: &str) -> (RenderDesc, RenderScene) {
    let g = scenes::build(name).expect(name);
    (
        RenderDesc {
            width: W,
            height: H,
            ..g.desc
        },
        g.scene,
    )
}

fn desc() -> RenderDesc {
    RenderDesc {
        width: W,
        height: H,
        ..scenes::base_desc()
    }
}

/// Renders `frames` frames of `scene` and reads the last back.
fn render_n(desc: &RenderDesc, scene: &RenderScene, frames: u32) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for _ in 0..frames.max(1) {
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    renderer.finish_stats(&mut stats);
    (renderer.read_offscreen().expect("capture"), stats)
}

fn render(desc: &RenderDesc, scene: &RenderScene) -> (RgbaImage, FrameStats) {
    render_n(desc, scene, 1)
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    let i = ((y.min(img.height - 1) * img.width + x.min(img.width - 1)) * 4) as usize;
    [
        img.pixels[i] as f32,
        img.pixels[i + 1] as f32,
        img.pixels[i + 2] as f32,
    ]
}

fn luma(p: [f32; 3]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

/// Every pixel of a rectangle (x0, y0, x1, y1), exclusive ends.
fn pixels(img: &RgbaImage, r: (u32, u32, u32, u32)) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for y in r.1..r.3 {
        for x in r.0..r.2 {
            out.push(px(img, x, y));
        }
    }
    out
}

/// The mean colour over a rectangle.
fn mean(img: &RgbaImage, r: (u32, u32, u32, u32)) -> [f32; 3] {
    let p = pixels(img, r);
    let n = p.len() as f32;
    let s = p
        .iter()
        .fold([0.0; 3], |a, c| [a[0] + c[0], a[1] + c[1], a[2] + c[2]]);
    [s[0] / n, s[1] / n, s[2] / n]
}

/// The luma's standard deviation over a rectangle.
fn spread(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let l: Vec<f32> = pixels(img, r).into_iter().map(luma).collect();
    let m = l.iter().sum::<f32>() / l.len() as f32;
    (l.iter().map(|v| (v - m).powi(2)).sum::<f32>() / l.len() as f32).sqrt()
}

/// Local contrast: the mean squared luma step between horizontal and
/// vertical neighbours over a rectangle.
fn sharpness(img: &RgbaImage, r: (u32, u32, u32, u32)) -> f32 {
    let (mut sum, mut n) = (0.0, 0.0);
    for y in r.1..r.3 - 1 {
        for x in r.0..r.2 - 1 {
            let c = luma(px(img, x, y));
            sum += (c - luma(px(img, x + 1, y))).powi(2) + (c - luma(px(img, x, y + 1))).powi(2);
            n += 2.0;
        }
    }
    sum / n
}

/// A square of `r` pixels either side of a point.
fn around(p: (u32, u32), r: u32) -> (u32, u32, u32, u32) {
    (
        p.0.saturating_sub(r),
        p.1.saturating_sub(r),
        p.0 + r + 1,
        p.1 + r + 1,
    )
}

/// Pixels with any channel more than `by` apart.
fn differing(a: &RgbaImage, b: &RgbaImage, by: u8) -> usize {
    a.pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.pixels.as_chunks::<4>().0.iter())
        .filter(|(p, q)| (0..3).any(|i| p[i].abs_diff(q[i]) > by))
        .count()
}

/// The pixel a world point projects to in a W x H frame of `scene`'s camera.
fn project(scene: &RenderScene, p: Vec3) -> (u32, u32) {
    let cam = &scene.camera;
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let proj = match cam.projection {
        Projection::Perspective { fov_y_degrees } => glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            W as f32 / H as f32,
            cam.near,
            cam.far,
        ),
        Projection::Orthographic { height } => {
            let hh = height * 0.5;
            let hw = hh * W as f32 / H as f32;
            glam::camera::rh::proj::directx::orthographic(-hw, hw, -hh, hh, cam.near, cam.far)
        }
    };
    let c = proj * view * p.extend(1.0);
    let ndc = c.truncate() / c.w;
    (
        ((ndc.x * 0.5 + 0.5) * W as f32) as u32,
        ((0.5 - ndc.y * 0.5) * H as f32) as u32,
    )
}

fn fmt(p: [f32; 3]) -> String {
    format!("({:.0}, {:.0}, {:.0})", p[0], p[1], p[2])
}

// ---------------------------------------------------------------------------
// Water.
// ---------------------------------------------------------------------------

/// `Water::mesh`: the surface is drawn over the mesh (a 3 m stream) instead
/// of the 24 m grid. A pixel over the stream is water-coloured; a pixel on
/// the floor beside it, which the grid covers, shows the bare floor as a
/// frame without water does. Fails if the mesh is ignored (the grid draws
/// over the floor), or if nothing is drawn over the mesh.
#[test]
fn water_over_a_mesh_draws_on_the_mesh_and_not_where_the_grid_would() {
    let (desc, stream) = built("water-mesh");
    let mut bare = fs::water_mesh_scene(true);
    bare.water.clear();
    let grid = fs::water_mesh_scene(false);
    let (with_mesh, _) = render(&desc, &stream);
    let (no_water, _) = render(&desc, &bare);
    let (with_grid, _) = render(&desc, &grid);
    save("water-mesh", &with_mesh);
    save("water-mesh-grid", &with_grid);
    let over = around(project(&stream, Vec3::new(-2.0, fs::WATER_LEVEL, -2.5)), 3);
    let beside = around(project(&stream, Vec3::new(-2.0, 0.0, 2.0)), 3);
    let (o_mesh, o_bare) = (mean(&with_mesh, over), mean(&no_water, over));
    let (b_mesh, b_bare, b_grid) = (
        mean(&with_mesh, beside),
        mean(&no_water, beside),
        mean(&with_grid, beside),
    );
    println!(
        "over the stream: mesh {} bare {}; beside it: mesh {} bare {} grid {}",
        fmt(o_mesh),
        fmt(o_bare),
        fmt(b_mesh),
        fmt(b_bare),
        fmt(b_grid)
    );
    // The sand floor is warm (red over blue); water over it is blue-green.
    let blueness = |p: [f32; 3]| p[2] - p[0];
    assert!(blueness(o_bare) < -20.0, "the bare floor is sand");
    assert!(
        blueness(o_mesh) > blueness(o_bare) + 30.0,
        "the stream is water-coloured"
    );
    assert!(
        (0..3).all(|i| (b_mesh[i] - b_bare[i]).abs() < 3.0),
        "beside the stream the floor is bare"
    );
    assert!(
        blueness(b_grid) > blueness(b_bare) + 30.0,
        "the grid alone covers it (the control)"
    );
}

/// `Water::normal_map`, `normal_scale` and `flow` (three.js Water2): a
/// ripple map breaks the surface's shading up from pixel to pixel, which a
/// flat map leaves smooth;
/// more repeats a metre put more ripples across the same pixels (their
/// neighbour-to-neighbour contrast rises); and with a flow the pattern moves
/// between two ticks, where the lake's layers with no wave speed stand
/// still. Fails if the map is not bound or not read, if the scale is
/// ignored, or if the flow does not move the layers.
#[test]
fn a_water_normal_map_ripples_scales_and_flows() {
    let (desc, rippled) = built("water-ripples");
    let surface = (40, 200, 600, 340);
    let mut flat = fs::water_ripples_scene();
    let flat_map = fs::flat_normal_map(&mut flat);
    flat.water[0].normal_map = Some(flat_map);
    let (a, _) = render(&desc, &rippled);
    let (b, _) = render(&desc, &flat);
    save("water-ripples", &a);
    save("water-ripples-flat", &b);
    let (s_ripple, s_flat) = (sharpness(&a, surface), sharpness(&b, surface));
    println!("neighbour contrast: ripple map {s_ripple:.2}, flat map {s_flat:.2}");
    assert!(
        s_ripple > 5.0 * s_flat + 1.0,
        "the ripples break up the shading"
    );

    let mut fine = fs::water_ripples_scene();
    fine.water[0].normal_scale = 1.0;
    let (c, _) = render(&desc, &fine);
    save("water-ripples-fine", &c);
    let (k_coarse, k_fine) = (sharpness(&a, surface), sharpness(&c, surface));
    println!("neighbour contrast: 0.25 repeats/m {k_coarse:.2}, 1 repeat/m {k_fine:.2}");
    assert!(k_fine > 2.0 * k_coarse, "more repeats, finer ripples");

    let later = |mut s: RenderScene| {
        s.tick += 15;
        s
    };
    let (d, _) = render(&desc, &later(fs::water_ripples_scene()));
    let moved = differing(&a, &d, 6);
    let mut lake = fs::water_ripples_scene();
    lake.water[0].flow = Vec2::ZERO;
    lake.water[0].wave_speed = 0.0;
    lake.water[0].normal_scale = 40.0;
    let (e, _) = render(&desc, &lake);
    let (f, _) = render(&desc, &later(lake));
    save("water-ripples-lake", &e);
    let still = differing(&e, &f, 6);
    println!("pixels moved over 15 ticks: flow {moved}, the lake without wave speed {still}");
    assert!(moved > 5000, "the flow moves the ripples");
    assert!(
        still < 50,
        "the lake's layers with no wave speed stand still"
    );
    assert!(
        sharpness(&e, surface) > 2.0 * s_flat,
        "the lake's layers ripple too"
    );
}

/// `Water::f0` and `Water::reflection_bias` (three.js Water.js: 0.3 and
/// 0.1). Looking straight down at dark, still water under a bright sky, the
/// reflection at normal incidence is F0: at 0.3 the surface there turns
/// towards the sky, at 0.02 it stays the water's dark colour. With the sky
/// turned off (nothing to reflect), the bias alone is the reflected colour:
/// 0.4 lifts the surface, 0 leaves it dark. Fails if either field does not
/// reach the shader's Fresnel term or reflected colour.
#[test]
fn water_f0_and_reflection_bias_raise_the_reflection() {
    let (desc, scene) = built("water-fresnel");
    let below = around(project(&scene, Vec3::new(0.0, 4.0, 3.6)), 4);
    let with = |f: &dyn Fn(&mut og_render::Water), dark: bool| {
        let mut s = fs::water_fresnel_scene();
        f(&mut s.water[0]);
        if dark {
            s.environment.intensity = 0.0;
        }
        let (img, _) = render(&desc, &s);
        img
    };
    let (three, _) = render(&desc, &scene);
    save("water-fresnel", &three);
    let clean = with(&|w| w.f0 = 0.02, false);
    save("water-fresnel-f0-0.02", &clean);
    let (l3, l0) = (luma(mean(&three, below)), luma(mean(&clean, below)));
    println!("straight down: f0 0.3 {l3:.1}, f0 0.02 {l0:.1}");
    assert!(l3 > l0 + 25.0, "F0 0.3 reflects the sky straight down");
    let unbiased = with(&|w| w.reflection_bias = 0.0, true);
    let biased = with(&|w| w.reflection_bias = 0.4, true);
    save("water-fresnel-bias-dark", &biased);
    let (b4, b0) = (luma(mean(&biased, below)), luma(mean(&unbiased, below)));
    println!("no sky, straight down: bias 0.4 {b4:.1}, bias 0 {b0:.1}");
    assert!(b4 > b0 + 15.0, "the bias lifts the reflected colour");
}

/// `Water::glint` (three.js Water's sun glint): with it the light's mirror
/// image on still water is a bright spot that the surface 2.5 m to its side
/// does not have, growing with the glint (2 against 1 and 0.01, all three
/// with three.js Water's body), and it is the reflected sky's colour times
/// the light, so with nothing to reflect (no sky) the spot is gone. Fails
/// if the glint is not drawn, does not scale, or is drawn from the light
/// alone like the default highlight (which stays with no sky).
#[test]
fn water_glint_mirrors_the_sun_on_the_reflected_image() {
    let (desc, scene) = built("water-glint");
    let spot = around(project(&scene, fs::glint_point()), 2);
    let side = around(
        project(&scene, fs::glint_point() + Vec3::new(2.5, 0.0, 0.0)),
        2,
    );
    let peak = |img: &RgbaImage| luma(mean(img, spot)) - luma(mean(img, side));
    let (glint, _) = render(&desc, &scene);
    save("water-glint", &glint);
    let mut dark = fs::water_glint_scene();
    dark.environment.intensity = 0.0;
    let (glint_dark, _) = render(&desc, &dark);
    let mut plain = fs::water_glint_scene();
    plain.water[0].glint = 0.0;
    plain.environment.intensity = 0.0;
    let (highlight_dark, _) = render(&desc, &plain);
    save("water-glint-dark", &glint_dark);
    save("water-highlight-dark", &highlight_dark);
    let glint_at = |k: f32| {
        let mut t = fs::water_glint_scene();
        t.water[0].glint = k;
        peak(&render(&desc, &t).0)
    };
    let (faint, half) = (glint_at(0.01), glint_at(1.0));
    let (g, gd, hd) = (peak(&glint), peak(&glint_dark), peak(&highlight_dark));
    println!(
        "spot over its side: glint 2 {g:.1}, 1 {half:.1}, 0.01 {faint:.1}; glint 2 with no sky {gd:.1}, default highlight with no sky {hd:.1}"
    );
    assert!(
        g > faint + 35.0,
        "the glint is a bright spot at the light's mirror image"
    );
    assert!(g > 1.6 * half, "it grows with the glint");
    assert!(gd < 5.0, "the glint is on the reflected image");
    assert!(
        hd > 40.0,
        "the default highlight is the light's (the control)"
    );
}

// ---------------------------------------------------------------------------
// The directional light's shadow box.
// ---------------------------------------------------------------------------

/// The ground in a post's shadow (with the sun 31 degrees up from -z it
/// falls 0 to 5 m towards +z), and open ground 2.5 m to its side.
fn shadow_of(post: Vec3) -> (Vec3, Vec3) {
    let at = Vec3::new(post.x, 0.0, post.z + 3.0);
    (at, at + Vec3::X * 2.5)
}

/// `DirectionalLight::shadow_box`, `shadow_focus` and `shadow_focus_drop`:
/// one shadow map over an 8 m box focused 12 m ahead of the camera on the
/// ground 6 m under it. The post inside the box shadows the ground; the
/// post 42 m ahead stands outside it, so its ground is lit, where the
/// cascades (box 0) shadow both. Focusing the box 42 m ahead moves the
/// shadow to the far post; dropping its centre 22 m (to 16 m under the
/// ground: along the 31 degree light its footprint on the ground moves 27 m
/// away, onto the far post) does the same. Fails if the box is not
/// used, if outside it reads as shadowed (the clamped border), or if the
/// focus or the drop is ignored.
#[test]
fn the_shadow_box_shadows_inside_it_and_lights_outside_it() {
    let (desc, scene) = built("shadow-box");
    let (near, open_near) = shadow_of(fs::BOX_NEAR_POST);
    let (far, open_far) = shadow_of(fs::BOX_FAR_POST);
    let near = around(project(&scene, near), 2);
    let open_near = around(project(&scene, open_near), 2);
    let far = around(project(&scene, far), 1);
    let open_far = around(project(&scene, open_far), 1);
    let with = |f: &dyn Fn(&mut og_render::DirectionalLight)| {
        let mut s = fs::shadow_box_scene();
        f(&mut s.light);
        render(&desc, &s)
    };
    let (boxed, stats) = render(&desc, &scene);
    save("shadow-box", &boxed);
    let (cascades, cstats) = with(&|l| l.shadow_box = 0.0);
    save("shadow-box-cascades", &cascades);
    let (ahead, _) = with(&|l| l.shadow_focus = 42.0);
    save("shadow-box-focus-42", &ahead);
    let (dropped, _) = with(&|l| l.shadow_focus_drop = 22.0);
    save("shadow-box-drop-22", &dropped);
    println!(
        "passes: box {}, cascades {}",
        stats.shadow_passes, cstats.shadow_passes
    );
    // Shadowed: darker than the open ground at about the same distance.
    let dark = |img: &RgbaImage, at, open| luma(mean(img, at)) / luma(mean(img, open));
    let rows = [
        ("box", &boxed),
        ("cascades", &cascades),
        ("focus 42", &ahead),
        ("drop 22", &dropped),
    ];
    for (name, img) in rows {
        println!(
            "{name}: near post's shadow {:.2} of open ground, far post's {:.2}",
            dark(img, near, open_near),
            dark(img, far, open_far)
        );
    }
    assert_eq!(stats.shadow_passes, 1, "one map, not the cascades");
    assert!(cstats.shadow_passes > 1);
    assert!(
        dark(&boxed, near, open_near) < 0.75,
        "inside the box: shadowed"
    );
    assert!(dark(&boxed, far, open_far) > 0.95, "outside the box: lit");
    assert!(
        dark(&cascades, far, open_far) < 0.75,
        "the cascades shadow the far post"
    );
    assert!(
        dark(&ahead, far, open_far) < 0.75,
        "focused 42 m ahead: the far post"
    );
    assert!(
        dark(&ahead, near, open_near) > 0.95,
        "and the near post is outside"
    );
    assert!(
        dark(&dropped, near, open_near) > 0.95,
        "dropped 22 m: the near post is outside"
    );
    assert!(
        dark(&dropped, far, open_far) < 0.75,
        "and the far post inside"
    );
}

/// `DirectionalLight::shadow_bias` and `shadow_normal_bias` with the box:
/// seen up close, a white card 2 cm behind a striped leaf card (both facing
/// the sun) is shadowed by the leaf's stripes with no bias (one texel,
/// about 8 mm), and lit with 4 cm of depth bias, or with 12 cm of normal
/// bias (3 cm along the normal facing the light). On a single card no bias
/// shows acne here: 0 is already one texel of each. Fails if either bias
/// is ignored.
#[test]
fn the_shadow_box_bias_keeps_close_leaf_cards_from_shadowing_each_other() {
    let l = fs::BOX_SUN.normalize();
    let look = |bias: f32, normal: f32| {
        let mut s = fs::shadow_box_scene();
        s.light.shadow_bias = bias;
        s.light.shadow_normal_bias = normal;
        s.camera.position = fs::BOX_LEAF + glam::Quat::from_rotation_y(0.8) * l * 1.2;
        s.camera.target = fs::BOX_LEAF;
        let (img, _) = render(&desc(), &s);
        let r = around(project(&s, fs::BOX_LEAF), 40);
        let (l, sp) = (luma(mean(&img, r)), spread(&img, r));
        (img, l, sp)
    };
    let (none, l0, s0) = look(0.0, 0.0);
    let (depth, l1, s1) = look(0.04, 0.0);
    let (normal, l2, s2) = look(0.0, 0.12);
    save("leaf-bias-0", &none);
    save("leaf-bias-4cm", &depth);
    save("leaf-normal-bias-12cm", &normal);
    println!(
        "the cards' mean luma (spread): no bias {l0:.1} ({s0:.1}), 4 cm bias {l1:.1} ({s1:.1}), 12 cm normal bias {l2:.1} ({s2:.1})"
    );
    assert!(
        s0 > 25.0 && l0 < l1 - 25.0,
        "no bias: the stripes shadow the card behind"
    );
    assert!(s1 < 15.0, "4 cm of bias: the card behind is lit");
    assert!(
        s2 < 15.0 && l2 > l0 + 25.0,
        "12 cm of normal bias: lit as well"
    );
}

// ---------------------------------------------------------------------------
// The hemisphere light.
// ---------------------------------------------------------------------------

/// `RenderScene::hemisphere` (Environment hemiSky, hemiGround,
/// hemiIntensity): with no sun, a red sky colour and a blue ground colour
/// at intensity 2, the floor (facing up) gains red and no blue, the ceiling
/// (facing down) gains blue and no red, against the same frame with the
/// light off; a mirror-metal tile on the floor (no diffuse term) does not
/// change, and intensity 0 is byte for byte the frame without the light.
/// With the environment at intensity 0 the light still reaches the floor:
/// it has its own strength. Fails if the normal's y does not pick the
/// colour, if the light reaches the reflections, if 0 is not off, or if
/// the light is scaled by the environment's intensity.
#[test]
fn the_hemisphere_light_lights_by_facing_and_only_diffusely() {
    let (desc, scene) = built("hemisphere-light");
    let off = |i: f32| {
        let mut s = fs::hemisphere_scene();
        s.hemisphere.2 = i;
        s
    };
    let mut none = fs::hemisphere_scene();
    none.hemisphere = ([0.0; 3], [0.0; 3], 0.0);
    let (lit, _) = render(&desc, &scene);
    let (dark, _) = render(&desc, &none);
    let (zero, _) = render(&desc, &off(0.0));
    save("hemisphere-light", &lit);
    save("hemisphere-light-off", &dark);
    let floor = around(project(&scene, Vec3::new(-1.5, 0.0, 2.0)), 3);
    let ceiling = around(project(&scene, Vec3::new(-1.5, fs::HEMI_CEILING, -8.0)), 3);
    let metal = around(project(&scene, fs::HEMI_METAL), 2);
    let gain = |r| {
        let (a, b) = (mean(&lit, r), mean(&dark, r));
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    };
    let (f, c, m) = (gain(floor), gain(ceiling), gain(metal));
    println!(
        "gained: floor {} (of {}), ceiling {} (of {}), metal tile {} (of {})",
        fmt(f),
        fmt(mean(&dark, floor)),
        fmt(c),
        fmt(mean(&dark, ceiling)),
        fmt(m),
        fmt(mean(&dark, metal))
    );
    assert!(
        f[0] > 30.0 && f[2].abs() < 3.0,
        "the floor takes the sky's red"
    );
    assert!(
        c[2] > 30.0 && c[0].abs() < 3.0,
        "the ceiling takes the ground's blue"
    );
    assert!(
        m.iter().all(|v| v.abs() < 3.0),
        "the metal reflects, it is not lit diffusely"
    );
    assert_eq!(zero.pixels, dark.pixels, "intensity 0 is off");
    // The light has its own strength, not the environment's: with the
    // environment at intensity 0 the floor is black without it and red with it.
    let mut unlit = fs::hemisphere_scene();
    unlit.environment.intensity = 0.0;
    let (alone, _) = render(&desc, &unlit);
    unlit.hemisphere = ([0.0; 3], [0.0; 3], 0.0);
    let (black, _) = render(&desc, &unlit);
    println!(
        "environment at 0: floor {} with the light, {} without",
        fmt(mean(&alone, floor)),
        fmt(mean(&black, floor))
    );
    assert!(
        mean(&alone, floor)[0] - mean(&black, floor)[0] > 30.0,
        "the hemisphere light does not need the environment's intensity"
    );
}

// ---------------------------------------------------------------------------
// Post effects.
// ---------------------------------------------------------------------------

/// `AmbientOcclusion::direct` (PostEffects aoDirect). With no sky light
/// (environment 0), the AO scene's corner, where the ground meets the lit
/// wall, takes only direct light: at `direct` 0 the occlusion leaves it
/// as it is without AO, at 1 it darkens. Open ground stays within a few
/// levels. Fails if the occlusion never reaches the direct light, or reaches
/// it at 0.
#[test]
fn ao_direct_darkens_occluded_direct_light_only_above_zero() {
    let (desc, scene) = built("ao-direct");
    let at = |direct: Option<f32>| {
        let mut s = fs::ao_direct_scene();
        s.environment.intensity = 0.0;
        match direct {
            Some(d) => s.post.ao.as_mut().expect("ao").direct = d,
            None => s.post.ao = None,
        }
        render(&desc, &s).0
    };
    let (full, _) = render(&desc, &scene);
    save("ao-direct", &full);
    let (none, zero, one) = (at(None), at(Some(0.0)), at(Some(1.0)));
    save("ao-direct-no-sky", &one);
    let corner = around(project(&scene, Vec3::new(1.0, 0.03, -2.75)), 2);
    let open = around(project(&scene, Vec3::new(1.5, 0.0, 2.0)), 2);
    let l = |img: &RgbaImage, r| luma(mean(img, r));
    println!(
        "corner: no AO {:.1}, direct 0 {:.1}, direct 1 {:.1}; open: {:.1} {:.1} {:.1}",
        l(&none, corner),
        l(&zero, corner),
        l(&one, corner),
        l(&none, open),
        l(&zero, open),
        l(&one, open)
    );
    assert!(
        (l(&zero, corner) - l(&none, corner)).abs() < 2.0,
        "direct 0: the direct light is untouched"
    );
    assert!(
        l(&one, corner) < l(&none, corner) - 12.0,
        "direct 1: occluded direct light darkens"
    );
    assert!(
        (l(&one, open) - l(&none, open)).abs() < 0.05 * l(&none, open) + 3.0,
        "open ground"
    );
}

/// `AmbientOcclusion::radius_scale` (aoRadiusScale): the radius becomes
/// `min(aoRadius, scale × distance)`. In the AO scene (SSAO, 0.7 m), about
/// 5 m from the camera, a scale of 0.02 shrinks the radius to 0.1 m: the
/// ground 0.8 m in front of the wall, which the 0.7 m radius occludes, is
/// left as it is without AO, and open ground stays unchanged. A scale of 1
/// (5 m, more than 0.7 m everywhere in view) is byte for byte the fixed
/// radius. Fails if the scale is ignored or the min is not taken.
#[test]
fn ao_radius_scale_shrinks_the_radius_up_close() {
    let desc = desc();
    let frame = |scale: Option<f32>| {
        let mut s = og_tests::fx_scenes::ao_scene(og_render::AoMethod::Ssao);
        match scale {
            Some(k) => s.post.ao.as_mut().expect("ao").radius_scale = k,
            None => s.post.ao = None,
        }
        render(&desc, &s).0
    };
    let scene = og_tests::fx_scenes::ao_scene(og_render::AoMethod::Ssao);
    let (fixed, scaled, wide, off) = (
        frame(Some(0.0)),
        frame(Some(0.02)),
        frame(Some(1.0)),
        frame(None),
    );
    save("ao-radius-scale", &scaled);
    let l = |img: &RgbaImage, p: Vec3| luma(mean(img, around(project(&scene, p), 2)));
    let wall = Vec3::new(1.0, 0.0, -2.0);
    let open = Vec3::new(1.5, 0.0, 2.0);
    println!(
        "0.8 m from the wall: no AO {:.1}, scale 0 {:.1}, scale 0.02 {:.1}; open: {:.1} {:.1} {:.1}",
        l(&off, wall),
        l(&fixed, wall),
        l(&scaled, wall),
        l(&off, open),
        l(&fixed, open),
        l(&scaled, open)
    );
    assert!(
        l(&fixed, wall) < l(&off, wall) - 8.0,
        "the 0.7 m radius reaches the wall"
    );
    assert!(
        (l(&scaled, wall) - l(&off, wall)).abs() < 2.0,
        "the scaled 0.1 m does not"
    );
    assert!(
        (l(&scaled, open) - l(&off, open)).abs() < 2.0,
        "open ground"
    );
    assert_eq!(
        fixed.pixels, wide.pixels,
        "min(aoRadius, 1 x distance) is aoRadius"
    );
}

/// `AoMethod::Screen` with water: the water's surface is in the AO's depth
/// (its `fs_ao_depth` pass after the prepass). A 0.5 m block sunk 0.5 m
/// under the surface: with the surface in the AO's depth, the water over
/// the block's foot darkens little more than open water does (a flat
/// surface there: under half the foot's extra occlusion of the control);
/// with the surface below the bed instead (the bed and the block in the
/// AO's depth) the block's foot darkens much more than the open bed.
/// Fails if the water is left out of the AO's depth (the foot's contact
/// shows through), or the AO is not applied at all.
#[test]
fn the_screen_ao_sees_the_water_surface() {
    let (desc, scene) = built("ao-screen-water");
    let pair = |water: bool| {
        let on = fs::ao_screen_water_scene(water);
        let mut off = fs::ao_screen_water_scene(water);
        off.post.ao = None;
        (render(&desc, &on).0, render(&desc, &off).0)
    };
    let (on, plain) = pair(true);
    let (bed_on, bed_plain) = pair(false);
    save("ao-screen-water", &on);
    save("ao-screen-water-bed", &bed_on);
    let foot = around(
        project(&scene, fs::POOL_BLOCK + Vec3::new(-0.3, -0.25, 0.0)),
        1,
    );
    let open = around(project(&scene, Vec3::new(1.6, 0.0, 1.5)), 2);
    // How much the AO keeps there (1: nothing occluded).
    let kept = |a: &RgbaImage, b: &RgbaImage, r| luma(mean(a, r)) / luma(mean(b, r));
    let (wf, wo) = (kept(&on, &plain, foot), kept(&on, &plain, open));
    let (bf, bo) = (
        kept(&bed_on, &bed_plain, foot),
        kept(&bed_on, &bed_plain, open),
    );
    println!(
        "kept by the AO: water: block's foot {wf:.3}, open {wo:.3}; no water: foot {bf:.3}, open {bo:.3}"
    );
    assert!(
        bf < bo - 0.08,
        "without water the sunk block's contact is occluded (the control)"
    );
    assert!(
        wo - wf < 0.5 * (bo - bf),
        "with water the AO sees the flat surface over the foot, not its contact"
    );
    assert!(wo < 0.97, "and the AO applies to the water");
}

/// `PostEffects::sharpen` (CAS): across the dark block's edges against the
/// ground and the checker's far squares, the neighbour-to-neighbour
/// contrast rises with sharpening at 0 and further at 1, while flat ground
/// keeps its mean. Fails if the pass does not run or ignores its strength.
#[test]
fn sharpen_raises_edge_contrast() {
    let (desc, scene) = built("post-sharpen");
    let at = |k: Option<f32>| {
        let mut s = fs::sharpen_scene();
        s.post.sharpen = k;
        render(&desc, &s).0
    };
    let (one, _) = render(&desc, &scene);
    let (zero, off) = (at(Some(0.0)), at(None));
    save("post-sharpen", &one);
    save("post-sharpen-off", &off);
    let block = project(&scene, Vec3::new(0.0, 0.75, 0.0));
    let edges = (block.0 - 70, block.1 - 70, block.0 + 70, block.1 + 70);
    let far = (0, H / 2 - 30, W, H / 2 + 10);
    let (s1, s0, so) = (
        sharpness(&one, edges),
        sharpness(&zero, edges),
        sharpness(&off, edges),
    );
    let (f1, fo) = (sharpness(&one, far), sharpness(&off, far));
    println!(
        "edge contrast: off {so:.1}, 0 {s0:.1}, 1 {s1:.1}; far checker: off {fo:.1}, 1 {f1:.1}"
    );
    assert!(s0 > 1.05 * so, "CAS at 0 sharpens");
    assert!(s1 > 1.1 * s0, "CAS at 1 sharpens more");
    assert!(f1 > 1.2 * fo, "the far checker is crisper");
    let flat = (20, H - 40, 120, H - 10);
    assert!(
        (luma(mean(&one, flat)) - luma(mean(&off, flat))).abs() < 2.0,
        "flat ground keeps its level"
    );
}

/// `PostEffects::bloom_unreal` and `bloom_contrast` (RenderSettings
/// bloomRadius, bloomContrast): `None` (bloomRadius below 0) is Origin's
/// bloom, byte for byte the `bloom` scene; UnrealBloomPass draws another
/// halo; its radius 1 weights the wide mips up, so the halo far from a
/// sphere grows against the halo near it; contrast 2 lifts the bright
/// spheres before the high pass, so their halo grows. Fails if the radius
/// does not switch the path, does not reach the mip weights, or the
/// contrast is ignored.
#[test]
fn unreal_bloom_switches_path_and_its_radius_widens_the_halo() {
    let (desc, scene) = built("bloom-unreal");
    let (origin_desc, origin) = built("bloom");
    let (unreal, _) = render(&desc, &scene);
    let (classic, _) = render(&origin_desc, &origin);
    let mut none = fs::bloom_unreal(scenes::build("bloom").expect("bloom").scene);
    none.post.bloom_unreal = None;
    none.post.bloom.as_mut().expect("bloom").intensity = 0.8;
    let (classic_again, _) = render(&desc, &none);
    assert_eq!(
        classic.pixels, classic_again.pixels,
        "None is Origin's bloom"
    );
    let with = |f: &dyn Fn(&mut RenderScene)| {
        let mut s = fs::bloom_unreal(scenes::build("bloom").expect("bloom").scene);
        f(&mut s);
        render(&desc, &s).0
    };
    let r0 = with(&|s| s.post.bloom_unreal = Some(0.0));
    let r1 = with(&|s| s.post.bloom_unreal = Some(1.0));
    let c2 = with(&|s| s.post.bloom_contrast = 2.0);
    let mut dark = fs::bloom_unreal(scenes::build("bloom").expect("bloom").scene);
    dark.post.bloom = None;
    let (unbloomed, _) = render(&desc, &dark);
    save("bloom-unreal", &unreal);
    save("bloom-unreal-r0", &r0);
    save("bloom-unreal-r1", &r1);
    save("bloom-unreal-c2", &c2);
    assert!(
        differing(&unreal, &classic, 4) > 1000,
        "the Unreal path draws another halo"
    );
    // The white sphere (index 3, emissive 12) at x 2, y 0.6, z -2.1: the
    // halo 50 px above it and 165 px above it, in the dark.
    let c = project(&scene, Vec3::new(2.0, 0.6, -2.1));
    let near = around((c.0, c.1 - 50), 1);
    let far = around((c.0, c.1 - 165), 1);
    let halo = |img: &RgbaImage, r| luma(mean(img, r)) - luma(mean(&unbloomed, r));
    for (name, img) in [("r0", &r0), ("r0.5", &unreal), ("r1", &r1), ("c2", &c2)] {
        println!(
            "{name}: halo near {:.1}, far {:.1}",
            halo(img, near),
            halo(img, far)
        );
    }
    let ratio = |img: &RgbaImage| halo(img, far) / halo(img, near).max(0.5);
    assert!(halo(&r0, near) > 30.0, "the halo is there");
    assert!(ratio(&r1) > 1.5 * ratio(&r0), "radius 1 widens it");
    assert!(
        halo(&c2, far) > 3.0 * halo(&unreal, far),
        "contrast 2 lifts the bloom"
    );
}

// ---------------------------------------------------------------------------
// Materials and hooks.
// ---------------------------------------------------------------------------

/// Pixels whose green is strictly between `lo` and `hi` (partly covered).
fn partial(img: &RgbaImage, lo: f32, hi: f32) -> usize {
    pixels(img, (0, 0, W, H))
        .iter()
        .filter(|p| p[1] > lo && p[1] < hi)
        .count()
}

/// `StandardMaterial::alphaToCoverage` with MSAA 4: a green card whose
/// alpha falls from 1 at its centre to 0 at its edge, cut at 0.5. With alpha
/// to coverage the circle where alpha crosses 0.5 is an antialiased edge
/// (pixels of intermediate green); with the hard cut it is green or black
/// (MSAA alone does not smooth a discard inside a triangle). Without MSAA,
/// alpha to coverage is a hard cut within a pixel of alphaTest's (not
/// byte for byte: see the test). Fails if the material
/// flag never reaches the pipeline or the shader's coverage output.
#[test]
fn alpha_to_coverage_antialiases_an_alpha_tested_edge() {
    let (desc, scene) = built("alpha-to-coverage");
    assert_eq!(desc.antialias, Antialias::Msaa4);
    let (a2c, _) = render(&desc, &scene);
    let (cut, _) = render(&desc, &fs::alpha_to_coverage_scene(false));
    save("alpha-to-coverage", &a2c);
    save("alpha-to-coverage-cut", &cut);
    let (pa, pc) = (partial(&a2c, 20.0, 235.0), partial(&cut, 20.0, 235.0));
    println!("partly covered pixels: alpha to coverage {pa}, cut {pc}");
    assert!(pa > 300, "the circle's edge is antialiased");
    assert!(pc < pa / 5, "the hard cut is not");
    let single = RenderDesc {
        antialias: Antialias::Off,
        ..desc
    };
    let (a, _) = render(&single, &scene);
    let (b, _) = render(&single, &fs::alpha_to_coverage_scene(false));
    // Not byte for byte: the shader discards where the coverage reaches 0,
    // half a pixel's change of alpha below alphaTest, so the edge sits about
    // half a pixel further out (a ring of at most one pixel).
    let green = |img: &RgbaImage| {
        pixels(img, (0, 0, W, H))
            .iter()
            .filter(|p| p[1] > 128.0)
            .count()
    };
    let (ga, gb) = (green(&a), green(&b));
    println!(
        "no MSAA: green {ga} against the cut's {gb}, partly covered {}",
        partial(&a, 20.0, 235.0)
    );
    assert_eq!(partial(&a, 20.0, 235.0), 0, "without MSAA it is a hard cut");
    // The circle's circumference at the 0.5 alpha ring (radius 1 m: about 78 px here).
    assert!(
        ga >= gb && ga - gb < 500,
        "within a pixel of the alphaTest cut"
    );
}

/// `Hook.eye` and `Hook.front`: the shader paints red where `h.eye` equals
/// the camera's position (passed as a parameter), green on front faces and
/// blue on back faces. The left card, facing the camera, is yellow (red and
/// green); the right one, turned away, is magenta (red and blue). Fails if
/// `h.eye` is not the camera (no red) or `h.front` does not follow the face.
#[test]
fn hooks_see_the_eye_and_the_facing() {
    let (desc, scene) = built("hooks-eye-front");
    let (img, _) = render(&desc, &scene);
    save("hooks-eye-front", &img);
    let front = mean(&img, around(project(&scene, Vec3::new(-1.0, 1.0, 0.0)), 3));
    let back = mean(&img, around(project(&scene, Vec3::new(1.0, 1.0, 0.0)), 3));
    println!("front card {}, back card {}", fmt(front), fmt(back));
    assert!(
        front[0] > 240.0 && front[1] > 240.0 && front[2] < 15.0,
        "front: eye and front"
    );
    assert!(
        back[0] > 240.0 && back[1] < 15.0 && back[2] > 240.0,
        "back: eye and back"
    );
}

/// `planar_reflection(p, shift)` in a material shader: a card lying on the
/// water shows what the water's planar reflection sees there: at the red
/// block's mirror image, red; without the planar reflection the function
/// answers alpha 0 and the card is the shader's magenta. Fails if the
/// function is a stub where the slot exists, or reads the wrong texture.
#[test]
fn a_hook_reads_the_planar_reflection() {
    let (desc, scene) = built("hooks-planar");
    let (img, stats) = render(&desc, &scene);
    let (none, _) = render(&desc, &fs::hooks_planar_scene(false));
    save("hooks-planar", &img);
    save("hooks-planar-none", &none);
    let mirror = fs::PLANAR_BLOCK * Vec3::new(1.0, -1.0, 1.0) + Vec3::Y * 2.0;
    let under = around(project(&scene, mirror), 2);
    let (a, b) = (mean(&img, under), mean(&none, under));
    println!(
        "card at the block's mirror image: planar {}, none {}",
        fmt(a),
        fmt(b)
    );
    assert!(stats.reflection_draws > 0, "{stats:?}");
    assert!(
        a[0] > 120.0 && a[1] < 0.5 * a[0] && a[2] < 0.5 * a[0],
        "the mirrored block is red"
    );
    assert!(
        b[0] > 200.0 && b[2] > 200.0 && b[1] < 40.0,
        "no reflection: magenta"
    );
}

/// `screen_ao(p)` in a material shader: the ground's unlit shader paints
/// the frame's screen-space AO at its fragment in red (4 × the occlusion);
/// the ground at the block's foot is red, open ground black; with AO off
/// the function answers 1 everywhere (all black). Fails if the function is
/// a stub or reads the AO at the wrong place.
#[test]
fn a_hook_reads_the_screen_ao() {
    let (desc, scene) = built("hooks-screen-ao");
    let (img, _) = render(&desc, &scene);
    let mut off = fs::hooks_screen_ao_scene();
    off.post.ao = None;
    let (none, _) = render(&desc, &off);
    save("hooks-screen-ao", &img);
    let foot = around(project(&scene, Vec3::new(0.0, 0.0, 0.56)), 2);
    let open = around(project(&scene, Vec3::new(2.5, 0.0, 1.5)), 2);
    let (f, o, n) = (mean(&img, foot), mean(&img, open), mean(&none, foot));
    println!(
        "foot {}, open {}, foot with AO off {}",
        fmt(f),
        fmt(o),
        fmt(n)
    );
    assert!(f[0] > 80.0, "the block's foot is occluded");
    assert!(o[0] < f[0] / 3.0, "open ground much less");
    assert!(n[0] < 5.0, "AO off: 1 everywhere");
}

/// The second planar reflection slot: two pools at different heights (0.3
/// and 1.5 m) each get their own planar reflection, so each mirrors its own
/// slab, at that slab's mirror image in its plane: pool A red, pool B blue.
/// With only A asking for one, B falls back to its screen-space and sky
/// reflections (the slab's image is paler). Fails if the second surface reuses the first's
/// reflection (B would mirror in A's plane) or gets none.
#[test]
fn two_water_surfaces_reflect_in_their_own_planes() {
    let (desc, scene) = built("water-planar-two");
    let (img, stats) = render(&desc, &scene);
    let mut one = fs::water_planar_two_scene();
    one.water[1].planar_reflection = 0.0;
    let (single, single_stats) = render(&desc, &one);
    save("water-planar-two", &img);
    save("water-planar-one", &single);
    // Each slab's mirror image in its own pool's plane.
    let image = |slab: Vec3, level: f32| {
        around(
            project(&scene, Vec3::new(slab.x, 2.0 * level - slab.y, slab.z)),
            2,
        )
    };
    let (ra, rb) = (
        image(fs::SLAB_A, fs::POOL_A.y),
        image(fs::SLAB_B, fs::POOL_B.y),
    );
    let (a, b, b1) = (mean(&img, ra), mean(&img, rb), mean(&single, rb));
    println!(
        "pool A {}, pool B {}, pool B without its own {}; reflection draws {} vs {}",
        fmt(a),
        fmt(b),
        fmt(b1),
        stats.reflection_draws,
        single_stats.reflection_draws
    );
    assert!(a[0] > a[2] + 30.0, "pool A mirrors the red slab");
    assert!(b[2] > b[0] + 30.0, "pool B mirrors the blue slab");
    assert!(
        b[2] - b[0] > b1[2] - b1[0] + 30.0,
        "without its own reflection pool B's image of the slab is paler (screen-space)"
    );
    assert!(stats.reflection_draws > single_stats.reflection_draws);
}

// ---------------------------------------------------------------------------
// The terrain's visibility and its scatter's shadows.
// ---------------------------------------------------------------------------

fn is_magenta(p: [f32; 3]) -> bool {
    p[0] > 200.0 && p[2] > 200.0 && p[1] < 60.0
}

fn is_red(p: [f32; 3]) -> bool {
    p[0] > 60.0 && p[0] > 2.5 * p[1] && p[0] > 2.5 * p[2]
}

/// `Terrain::visible` false: the terrain is not drawn (nor in the shadow
/// passes) but its scattered posts still stand on it. Where the visible
/// terrain covers the frame's middle, the hidden one shows the magenta
/// background; the red posts are there in both frames, at the same pixels.
/// Fails if `visible` is ignored, or if hiding the terrain drops its
/// scatter too.
#[test]
fn a_hidden_terrain_keeps_its_scatter_standing() {
    let (desc, scene) = built("terrain-hidden");
    let (hidden, hs) = render_n(&desc, &scene, 2);
    let (shown, ss) = render_n(&desc, &fs::terrain_flags_scene(true, true), 2);
    save("terrain-hidden", &hidden);
    save("terrain-shown", &shown);
    let mid = (W / 4, H / 3, 3 * W / 4, 2 * H / 3);
    let count = |img: &RgbaImage, f: fn([f32; 3]) -> bool| {
        pixels(img, mid).into_iter().filter(|p| f(*p)).count()
    };
    let n = ((mid.2 - mid.0) * (mid.3 - mid.1)) as usize;
    let (hm, sm) = (count(&hidden, is_magenta), count(&shown, is_magenta));
    let (hr, sr) = (count(&hidden, is_red), count(&shown, is_red));
    let both = pixels(&hidden, mid)
        .into_iter()
        .zip(pixels(&shown, mid))
        .filter(|(a, b)| is_red(*a) && is_red(*b))
        .count();
    println!(
        "of {n} middle pixels: magenta hidden {hm} shown {sm}; red hidden {hr} shown {sr}, in both {both}; terrain draws {} vs {}, scatter copies {} vs {}",
        hs.terrain_draws, ss.terrain_draws, hs.scatter_copies, ss.scatter_copies
    );
    assert_eq!(hs.terrain_draws, 0, "the hidden terrain is not drawn");
    assert!(ss.terrain_draws > 0);
    assert!(
        sm == 0 && hm > n / 2,
        "the background shows where the terrain was"
    );
    assert!(sr > 500 && hr > 500, "the posts stand in both");
    assert!(both as f32 > 0.85 * sr.min(hr) as f32, "at the same pixels");
}

/// `ScatterDraw::cast_shadows` (TerrainScatter castShadows) false: the
/// posts cast no shadow on the terrain (its ground beside them is as lit as
/// with no posts' shadow at all), while the terrain still casts (its shadow
/// passes draw). Fails if the flag is ignored or turns the terrain's
/// shadows off too.
#[test]
fn unshadowed_scatter_copies_cast_no_shadow() {
    let (desc, scene) = built("terrain-scatter-unshadowed");
    let (off, os) = render_n(&desc, &scene, 2);
    let (on, ons) = render_n(&desc, &fs::terrain_flags_scene(true, true), 2);
    save("terrain-scatter-unshadowed", &off);
    save("terrain-scatter-shadowed", &on);
    // Ground (neither post nor background) in each frame, its luma sorted:
    // the darkest tenth against the median (shadows pull it down).
    let ground = |img: &RgbaImage| {
        let mut l: Vec<f32> = pixels(img, (W / 4, H / 3, 3 * W / 4, 2 * H / 3))
            .into_iter()
            .filter(|p| !is_red(*p) && !is_magenta(*p))
            .map(luma)
            .collect();
        l.sort_by(f32::total_cmp);
        (l[l.len() / 10], l[l.len() / 2])
    };
    let ((d_on, m_on), (d_off, m_off)) = (ground(&on), ground(&off));
    println!(
        "ground luma, darkest tenth / median: posts casting {d_on:.1} / {m_on:.1}, not casting {d_off:.1} / {m_off:.1}; terrain shadow draws {} / {}",
        os.terrain_shadow_draws, ons.terrain_shadow_draws
    );
    assert!(d_on < m_on - 25.0, "the posts shadow the ground");
    assert!(d_off > m_off - 5.0, "with castShadows false they do not");
    assert!(os.terrain_shadow_draws > 0, "the terrain still casts");
}
