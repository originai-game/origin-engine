//! The rendering fields of the Grand View Garden fork merge (PR #2), each
//! asserted on frames: the scene of `og_tests::fork_scenes` (registered in
//! `scenes::build`) rendered with and without its field, and the pixels
//! the field must change read where it changes them. Ways these could fail
//! are named on each test.

use glam::Vec3;
use og_render::{FrameStats, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::fork_scenes as fs;
use og_tests::harness::{luma_f32, px_clamped, project, save};
use og_tests::scenes;

/// Frames are rendered at half the reference size (the same 16:9 view).
const W: u32 = 640;
const H: u32 = 360;

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

/// [`px_clamped`] as f32 (0 to 255, not 0 to 1).
fn px(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    px_clamped(img, x, y).map(f32::from)
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

/// A square of `r` pixels either side of a point.
fn around(p: (u32, u32), r: u32) -> (u32, u32, u32, u32) {
    (
        p.0.saturating_sub(r),
        p.1.saturating_sub(r),
        p.0 + r + 1,
        p.1 + r + 1,
    )
}

/// The pixel a world point projects to in a W x H frame of `scene`'s camera.
fn project_in(scene: &RenderScene, p: Vec3) -> (u32, u32) {
    project(&scene.frame.camera, W, H, p)
}

fn fmt(p: [f32; 3]) -> String {
    format!("({:.0}, {:.0}, {:.0})", p[0], p[1], p[2])
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
#[ignore = "requires GPU"]
fn the_shadow_box_shadows_inside_it_and_lights_outside_it() {
    // The registered scene is a shot of the feature; this measures its
    // plain fixture.
    let (desc, _) = built("shadow-box");
    let scene = fs::shadow_box_scene();
    let (near, open_near) = shadow_of(fs::BOX_NEAR_POST);
    let (far, open_far) = shadow_of(fs::BOX_FAR_POST);
    let near = around(project_in(&scene, near), 2);
    let open_near = around(project_in(&scene, open_near), 2);
    let far = around(project_in(&scene, far), 1);
    let open_far = around(project_in(&scene, open_far), 1);
    let with = |f: &dyn Fn(&mut og_render::DirectionalLight)| {
        let mut s = fs::shadow_box_scene();
        f(&mut s.lighting.light);
        render(&desc, &s)
    };
    let (boxed, stats) = render(&desc, &scene);
    save("og-fork-fields", "shadow-box", &boxed);
    let (cascades, cstats) = with(&|l| l.shadow_box = 0.0);
    save("og-fork-fields", "shadow-box-cascades", &cascades);
    let (ahead, _) = with(&|l| l.shadow_focus = 42.0);
    save("og-fork-fields", "shadow-box-focus-42", &ahead);
    let (dropped, _) = with(&|l| l.shadow_focus_drop = 22.0);
    save("og-fork-fields", "shadow-box-drop-22", &dropped);
    println!(
        "passes: box {}, cascades {}",
        stats.shadow_passes, cstats.shadow_passes
    );
    // Shadowed: darker than the open ground at about the same distance.
    let dark = |img: &RgbaImage, at, open| luma_f32(mean(img, at)) / luma_f32(mean(img, open));
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
#[ignore = "requires GPU"]
fn the_hemisphere_light_lights_by_facing_and_only_diffusely() {
    let (desc, _) = built("hemisphere-light");
    let scene = fs::hemisphere_scene();
    let off = |i: f32| {
        let mut s = fs::hemisphere_scene();
        s.lighting.hemisphere.2 = i;
        s
    };
    let mut none = fs::hemisphere_scene();
    none.lighting.hemisphere = ([0.0; 3], [0.0; 3], 0.0);
    let (lit, _) = render(&desc, &scene);
    let (dark, _) = render(&desc, &none);
    let (zero, _) = render(&desc, &off(0.0));
    save("og-fork-fields", "hemisphere-light", &lit);
    save("og-fork-fields", "hemisphere-light-off", &dark);
    let floor = around(project_in(&scene, Vec3::new(-1.5, 0.0, 2.0)), 3);
    let ceiling = around(project_in(&scene, Vec3::new(-1.5, fs::HEMI_CEILING, -8.0)), 3);
    let metal = around(project_in(&scene, fs::HEMI_METAL), 2);
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
    unlit.lighting.environment.intensity = 0.0;
    let (alone, _) = render(&desc, &unlit);
    unlit.lighting.hemisphere = ([0.0; 3], [0.0; 3], 0.0);
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
