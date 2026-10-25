//! Reflection probes beyond one room (`og_tests::probe_scenes`): their
//! reference frames, and from the frames and the captures read back: each
//! of two rooms sharing a sphere mesh reflects its own room; a capture
//! draws skinned meshes in their pose, terrain, grass and translucent
//! meshes; terrain and grass inside a probe's box are lit by it.

use og_render::{RenderDesc, RenderTarget, Renderer, RgbaImage};
use og_tests::gfx;
use og_tests::probe_scenes::{
    FLOOR_PROBE, Outdoor, SsrFloor, add_floor_fluid, add_floor_particles, add_floor_water,
    outdoor_scene, posed_figure_scene, probe_effects_scene, probe_floor_scene, probe_water_scene,
    ssr_floor_scene, two_rooms_scene,
};
use og_tests::scenes;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-probes")
}

fn check(scene: &str) {
    match gfx::check(scene, &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn probe_two_rooms_matches_its_reference() {
    check("probe-two-rooms");
}

#[test]
fn probe_outdoor_matches_its_reference() {
    check("probe-outdoor");
}

/// The frame of `scene` and, when `probe` is given, that probe's capture:
/// level 0's six faces (+x, -x, +y, -y, +z, -z) of linear RGB.
fn render(
    desc: &RenderDesc,
    scene: &og_render::RenderScene,
    probe: Option<u64>,
) -> (RgbaImage, Option<(u32, Vec<f32>)>) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    let capture = probe.map(|id| {
        let (size, mut levels) = renderer
            .read_reflection_probe(id)
            .expect("captured")
            .expect("read back");
        (size, levels.swap_remove(0))
    });
    (renderer.read_offscreen().expect("frame"), capture)
}

fn region_mean(img: &RgbaImage, x: std::ops::Range<u32>, y: std::ops::Range<u32>) -> [f32; 3] {
    let mut s = [0.0f32; 3];
    let n = (x.len() * y.len()) as f32;
    for py in y {
        for px in x.clone() {
            let p = &img.pixels[((py * img.width + px) * 4) as usize..][..3];
            for k in 0..3 {
                s[k] += p[k] as f32 / n;
            }
        }
    }
    s
}

/// The mean linear RGB of the middle half of cube face `face`.
fn face_mean(capture: &(u32, Vec<f32>), face: usize) -> [f32; 3] {
    let (n, data) = (capture.0 as usize, &capture.1);
    let mut s = [0.0f32; 3];
    let mut count = 0.0;
    for y in n / 4..3 * n / 4 {
        for x in n / 4..3 * n / 4 {
            let at = ((face * n + y) * n + x) * 3;
            for k in 0..3 {
                s[k] += data[at + k];
            }
            count += 1.0;
        }
    }
    s.map(|v| v / count)
}

/// Two rooms share one sphere mesh and material: each sphere's upper half
/// reflects its own room's ceiling and walls (red left, blue right). One
/// probe per batch, picked at the centre between the rooms, would give
/// both the environment's sky.
#[test]
fn each_of_two_rooms_reflects_its_own_room() {
    let desc = scenes::base_desc();
    let (with, _) = render(&desc, &two_rooms_scene(true), None);
    let (without, _) = render(&desc, &two_rooms_scene(false), None);
    let (left, right) = ((282..342, 322..344), (938..998, 322..344));
    let l = region_mean(&with, left.0.clone(), left.1.clone());
    let r = region_mean(&with, right.0.clone(), right.1.clone());
    let l0 = region_mean(&without, left.0, left.1);
    let r0 = region_mean(&without, right.0, right.1);
    println!("upper halves: left {l:?}, right {r:?}; without probes {l0:?}, {r0:?}");
    assert!(
        l[0] > l[2] + 40.0,
        "the left sphere shows the red room: {l:?}"
    );
    assert!(
        r[2] > r[0] + 40.0,
        "the right sphere shows the blue room: {r:?}"
    );
    assert!(
        (0..3).all(|k| (l0[k] - r0[k]).abs() < 8.0) && l0[2] > l0[0],
        "without probes both reflect the same sky: {l0:?} {r0:?}"
    );
}

/// A skinned figure takes the probe of the room its pose stands in: a
/// chrome cylinder whose instance stands in the red room and whose joints
/// carry it into the blue one reflects the blue room, as the same cylinder
/// placed there by its transform does. Picked by its bind pose's bounds,
/// as before the debt round 2, it reflected the red room it is not in.
#[test]
fn a_skinned_instance_takes_the_probe_of_its_pose() {
    let desc = scenes::base_desc();
    let frame = |posed: Option<bool>| {
        let scene = posed.map_or_else(|| two_rooms_scene(true), posed_figure_scene);
        let (img, _) = render(&desc, &scene, None);
        if let (Ok(dir), Some(posed)) = (std::env::var("OG_PROBE_OUT"), posed) {
            gfx::save_png(
                std::path::Path::new(&format!("{dir}/posed-figure-{posed}.png")),
                &img,
            )
            .expect("save");
        }
        img
    };
    let (bare, posed, placed) = (frame(None), frame(Some(true)), frame(Some(false)));
    // The cylinder's pixels: where the placed one changed the room.
    let px = |i: usize| &bare.pixels[i * 4..i * 4 + 3];
    let cylinder: Vec<usize> = (0..(bare.width * bare.height) as usize)
        .filter(|&i| {
            let q = &placed.pixels[i * 4..i * 4 + 3];
            px(i)
                .iter()
                .zip(q)
                .map(|(a, b)| a.abs_diff(*b) as u32)
                .sum::<u32>()
                > 60
        })
        .collect();
    let mean = |img: &RgbaImage| {
        let mut s = [0.0f32; 3];
        for &i in &cylinder {
            for (k, v) in s.iter_mut().enumerate() {
                *v += img.pixels[i * 4 + k] as f32 / cylinder.len() as f32;
            }
        }
        s
    };
    let (posed, placed) = (mean(&posed), mean(&placed));
    println!(
        "the cylinder ({} px): posed {posed:?}, placed {placed:?}",
        cylinder.len()
    );
    assert!(cylinder.len() > 2000, "the cylinder is in view");
    assert!(
        placed[2] > placed[0] + 6.0,
        "placed in the blue room its sides reflect blue: {placed:?}"
    );
    assert!(
        (0..3).all(|k| (posed[k] - placed[k]).abs() < 3.0),
        "posed into the blue room it reflects the blue room: {posed:?} against {placed:?}"
    );
}

/// The outdoor probe's capture has the skinned figures on its +x face, the
/// meadow (terrain and grass) on its -y face and the red pane over its -z
/// face; without the figures and the pane those faces are sky and meadow.
/// Before the debt was paid a capture drew only the opaque meshes and the
/// sky: +x and -z would match the control and -y would be the sky's ground.
#[test]
fn a_capture_draws_skinned_meshes_terrain_grass_and_glass() {
    let desc = scenes::base_desc();
    let probe = Some(1.0);
    let (_, full) = render(
        &desc,
        &outdoor_scene(Outdoor {
            content: true,
            probe,
        }),
        Some(7),
    );
    let (_, bare) = render(
        &desc,
        &outdoor_scene(Outdoor {
            content: false,
            probe,
        }),
        Some(7),
    );
    let (full, bare) = (full.unwrap(), bare.unwrap());
    let f = |c: &(u32, Vec<f32>), face| face_mean(c, face);
    println!(
        "+x {:?} / {:?}; -y {:?} / {:?}; -z {:?} / {:?}",
        f(&full, 0),
        f(&bare, 0),
        f(&full, 3),
        f(&bare, 3),
        f(&full, 5),
        f(&bare, 5)
    );
    let px = f(&full, 0);
    assert!(
        px[0] > px[1] * 1.5 && px[2] > px[1] * 1.3,
        "the magenta figures on +x: {px:?}"
    );
    let ny = f(&bare, 3);
    assert!(
        ny[1] > ny[0] && ny[1] > ny[2],
        "the meadow (green) on -y: {ny:?}"
    );
    let (nz, nz0) = (f(&full, 5), f(&bare, 5));
    assert!(
        nz[0] / nz[2].max(1e-3) > 1.5 * nz0[0] / nz0[2].max(1e-3),
        "the red pane over -z: {nz:?} against {nz0:?}"
    );
}

/// A capture draws what the main pass draws over its opaque scene: the
/// red world sprite on the probe's -z face, the yellow line loop on its +x
/// face and the green particles on its -x face. Before the debt round 2 a
/// capture drew none of them: each face would match the bare meadow's.
#[test]
fn a_capture_draws_particles_sprites_and_lines() {
    let desc = scenes::base_desc();
    let (_, with) = render(&desc, &probe_effects_scene(true), Some(7));
    let (_, bare) = render(&desc, &probe_effects_scene(false), Some(7));
    let (with, bare) = (with.unwrap(), bare.unwrap());
    let f = |c: &(u32, Vec<f32>), face| face_mean(c, face);
    let (nz, nz0) = (f(&with, 5), f(&bare, 5));
    let (px, px0) = (f(&with, 0), f(&bare, 0));
    let (nx, nx0) = (f(&with, 1), f(&bare, 1));
    println!("-z {nz:?} / {nz0:?}; +x {px:?} / {px0:?}; -x {nx:?} / {nx0:?}");
    let share = |c: [f32; 3], k: usize| c[k] / (c[0] + c[1] + c[2]).max(1e-4);
    let mut failures = Vec::new();
    if share(nz, 0) < share(nz0, 0) + 0.15 {
        failures.push(format!("the red sprite on -z: {nz:?} against {nz0:?}"));
    }
    if share(px, 2) > share(px0, 2) - 0.08 {
        failures.push(format!("the yellow lines on +x: {px:?} against {px0:?}"));
    }
    if share(nx, 1) < share(nx0, 1) + 0.08 {
        failures.push(format!("the green particles on -x: {nx:?} against {nx0:?}"));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A capture draws water: the -y face of the probe over a red-tinted pond
/// turns from the meadow's green to the water's red. Before the debt round
/// 2 the capture left water out and the face stayed the meadow's.
#[test]
fn a_capture_draws_water() {
    let desc = scenes::base_desc();
    let (_, with) = render(&desc, &probe_water_scene(true), Some(7));
    let (_, bare) = render(&desc, &probe_water_scene(false), Some(7));
    let (ny, ny0) = (face_mean(&with.unwrap(), 3), face_mean(&bare.unwrap(), 3));
    println!("-y {ny:?} / {ny0:?}");
    assert!(
        ny[0] > ny[1] * 1.5 && ny0[1] > ny0[0],
        "the pond on -y: {ny:?} against the meadow {ny0:?}"
    );
}

/// Water in a capture shows what lies below it: over the yellow floor,
/// 0.15 m of clear water leaves the -y face yellow, its red and green near
/// the bare floor's, as the frame's water does. Before the faces had a depth prepass
/// and a copy of themselves, the water there read the far plane below and
/// white through it, so the face took the water's dark blue deep colour.
#[test]
fn water_in_a_capture_shows_the_floor_below() {
    let desc = scenes::base_desc();
    let mut scene = probe_floor_scene();
    let (_, bare) = render(&desc, &scene, Some(FLOOR_PROBE.0));
    add_floor_water(&mut scene);
    let (_, with) = render(&desc, &scene, Some(FLOOR_PROBE.0));
    let (ny, ny0) = (face_mean(&with.unwrap(), 3), face_mean(&bare.unwrap(), 3));
    println!("-y {ny:?} / bare floor {ny0:?}");
    let yellow = |c: [f32; 3]| c[0] > 3.0 * c[2] && c[1] > 2.5 * c[2];
    assert!(yellow(ny0), "the bare floor on -y: {ny0:?}");
    assert!(
        yellow(ny) && (0..2).all(|k| ny[k] > 0.6 * ny0[k] && ny[k] < 1.3 * ny0[k]),
        "the floor through the water on -y: {ny:?} against the bare floor {ny0:?}"
    );
}

/// Soft particles in a capture fade against the face's depth: green
/// particles hovering 0.1 to 0.2 m over the floor, fading over 2 m, leave
/// the -y face near the bare floor's yellow, where the same particles drawn
/// hard turn it green. Before the faces had a depth prepass, a face's
/// soft particles read the far plane behind them and drew as hard ones.
#[test]
fn soft_particles_in_a_capture_fade_against_the_floor() {
    let desc = scenes::base_desc();
    let face = |particles: Option<bool>| {
        let mut scene = probe_floor_scene();
        if let Some(soft) = particles {
            add_floor_particles(&mut scene, soft);
        }
        face_mean(&render(&desc, &scene, Some(FLOOR_PROBE.0)).1.unwrap(), 3)
    };
    let (bare, hard, soft) = (face(None), face(Some(false)), face(Some(true)));
    let off = |c: [f32; 3]| (0..3).map(|k| (c[k] - bare[k]).abs()).sum::<f32>();
    println!(
        "-y bare {bare:?}, hard {hard:?} ({:.3} off), soft {soft:?} ({:.3} off)",
        off(hard),
        off(soft)
    );
    assert!(
        off(hard) > 0.3,
        "the hard particles on -y: {hard:?} against {bare:?}"
    );
    assert!(
        off(soft) < 0.4 * off(hard),
        "the soft particles on -y: {soft:?}, hard {hard:?}, bare floor {bare:?}"
    );
}

/// A capture draws 3D fluids: the +x face of the probe beside a red fluid
/// ball turns red where it showed the dark background and the yellow
/// floor (its green falls, red leads it more than twofold). Before, a
/// capture left 3D fluids out and the face stayed bare.
#[test]
fn a_capture_draws_3d_fluids() {
    let desc = scenes::base_desc();
    let mut scene = probe_floor_scene();
    let (_, bare) = render(&desc, &scene, Some(FLOOR_PROBE.0));
    add_floor_fluid(&mut scene);
    let (_, with) = render(&desc, &scene, Some(FLOOR_PROBE.0));
    let (px, px0) = (face_mean(&with.unwrap(), 0), face_mean(&bare.unwrap(), 0));
    println!("+x {px:?} / bare {px0:?}");
    assert!(
        px[0] > 2.0 * px[1] && px[0] > 2.0 * px[2] && px[1] < 0.6 * px0[1],
        "the red fluid on +x: {px:?} against the bare face {px0:?}"
    );
}

/// Terrain and grass inside the probe's box are lit by it: with the
/// probe's intensity raised the meadow under the sphere brightens, and the
/// mountains far outside the box do not change. Sampling the environment
/// directly, as terrain and scatter did, the two frames would be the same.
#[test]
fn terrain_and_grass_in_a_probes_box_take_its_light() {
    let desc = scenes::base_desc();
    let lit = |i| {
        outdoor_scene(Outdoor {
            content: false,
            probe: Some(i),
        })
    };
    let (one, _) = render(&desc, &lit(1.0), None);
    let (four, _) = render(&desc, &lit(4.0), None);
    // The meadow a few metres in front of the camera (inside the box) and
    // the far meadow by the horizon (outside it).
    let near = (300..500, 450..560);
    let far = (0..300, 255..275);
    let (a, b) = (
        region_mean(&one, near.0.clone(), near.1.clone()),
        region_mean(&four, near.0, near.1),
    );
    let (c, d) = (
        region_mean(&one, far.0.clone(), far.1.clone()),
        region_mean(&four, far.0, far.1),
    );
    println!("meadow in the box: x1 {a:?} x4 {b:?}; far meadow: {c:?} {d:?}");
    assert!(
        b[1] > a[1] + 6.0,
        "the meadow takes the probe: {a:?} -> {b:?}"
    );
    assert!(
        (0..3).all(|k| (c[k] - d[k]).abs() < 1.5),
        "the far meadow does not: {c:?} -> {d:?}"
    );
}

/// Screen-space reflections read the roughness the lit pass shades: a
/// floor of roughness factor 1 whose map makes it 0.04 mirrors the red
/// sphere, as one without the map does not; so does a terrain whose layer
/// is glossy. From the factors alone (and nothing for terrain), all three
/// patches would be the dark floor.
#[test]
fn ssr_reads_roughness_maps_and_terrain_layers() {
    let desc = scenes::base_desc();
    let tier = Renderer::new_blocking(&desc)
        .expect("a GPU adapter")
        .info()
        .tier;
    if tier != og_render::Tier::Full {
        return;
    }
    // Under the sphere, as `ssr_reflects_the_sphere_in_the_floor` reads it.
    let at = |floor, ssr| {
        let (img, _) = render(&desc, &ssr_floor_scene(floor, ssr), None);
        region_mean(&img, 460..530, 540..600)
    };
    let mapped = at(SsrFloor::Mapped(true), true);
    let factor = at(SsrFloor::Mapped(false), true);
    let terrain = at(SsrFloor::Terrain, true);
    let terrain_off = at(SsrFloor::Terrain, false);
    println!(
        "under the sphere: map {mapped:?}, factor only {factor:?}, terrain {terrain:?} (ssr off {terrain_off:?})"
    );
    assert!(
        mapped[0] > mapped[1] * 1.4 && (factor[0] - factor[1]).abs() < 3.0,
        "the mapped floor mirrors the red sphere, the rough one does not: {mapped:?} against {factor:?}"
    );
    assert!(
        terrain[0] > terrain_off[0] + 15.0 && terrain[0] > terrain[1] * 1.4,
        "the terrain mirrors it: {terrain:?} against {terrain_off:?}"
    );
}

/// Local lights reach terrain, grass and 3D fluids in a capture: a blue
/// point light under the outdoor probe turns the meadow on its -y face
/// blue, and one between the floor probe and the red fluid ball lights the
/// ball's near side on its +x face. A face's frame has no light clusters;
/// before the debt round 3 these shaders still read one (the main camera's
/// first), so the light was missing from the face as it is here bare.
#[test]
fn local_lights_reach_terrain_and_fluids_in_a_capture() {
    let desc = scenes::base_desc();
    let blue = |position| og_render::Light {
        position,
        color: glam::Vec3::new(0.1, 0.3, 1.0),
        intensity: 6.0,
        range: 4.0,
        ..Default::default()
    };
    let blue_share = |c: [f32; 3]| c[2] / (c[0] + c[1] + c[2]).max(1e-4);
    let mut failures = Vec::new();

    let mut meadow = outdoor_scene(Outdoor {
        content: false,
        probe: Some(1.0),
    });
    let ny0 = face_mean(&render(&desc, &meadow, Some(7)).1.unwrap(), 3);
    meadow.lights.push(blue(
        og_tests::probe_scenes::outdoor_probe() - glam::Vec3::Y * 1.2,
    ));
    let ny = face_mean(&render(&desc, &meadow, Some(7)).1.unwrap(), 3);
    println!("meadow -y: lit {ny:?}, bare {ny0:?}");
    if ny[2] < ny0[2] + 0.05 || blue_share(ny) < blue_share(ny0) + 0.1 {
        failures.push(format!("the meadow on -y: lit {ny:?}, bare {ny0:?}"));
    }

    let mut fluid = probe_floor_scene();
    add_floor_fluid(&mut fluid);
    let px0 = face_mean(&render(&desc, &fluid, Some(FLOOR_PROBE.0)).1.unwrap(), 0);
    fluid
        .lights
        .push(blue(FLOOR_PROBE.1 + glam::Vec3::new(0.6, 0.0, 0.0)));
    let px = face_mean(&render(&desc, &fluid, Some(FLOOR_PROBE.0)).1.unwrap(), 0);
    println!("fluid +x: lit {px:?}, bare {px0:?}");
    if px[2] < px0[2] + 0.03 {
        failures.push(format!("the fluid on +x: lit {px:?}, bare {px0:?}"));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
