//! `trails-beams`: a ward on the shrine gate at blue hour. A scrolling
//! beam bars the torii between its posts and a lightning arc crackles
//! under it; three wisps loop through the gate trailing ribbons of light
//! (trails), each throwing its colour on the flags, and a spiral ribbon
//! winds up round a stone lantern.

use super::*;

/// The wisps' colours.
const WISPS: [[f32; 3]; 3] = [[1.0, 0.45, 0.15], [0.25, 0.8, 1.0], [0.7, 1.0, 0.35]];

/// Where wisp `i` is at `tick`: loops through the gate, tilted so each
/// passes in front of a post and behind the other.
fn wisp(i: usize, tick: u64) -> Vec3 {
    let t = tick as f32 / 60.0;
    let a = t * (1.5 + i as f32 * 0.3) + i as f32 * 2.1;
    let r = 1.3 + i as f32 * 0.35;
    Vec3::new(
        a.cos() * r,
        1.5 + (t * 1.7 + i as f32 * 1.3).sin() * 0.6 + a.sin() * 0.25,
        -2.0 + a.sin() * r * 0.8,
    )
}

pub fn trails_stage_scene() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    crate::shadow_scenes::court(&mut scene);
    kit::prop(&mut scene, "torii").place(&mut scene, Vec3::new(0.0, 0.0, -2.0), 0.0, 4.2);
    let toro = kit::prop(&mut scene, "toro");
    let fire = crate::shadow_scenes::toro(&mut scene, &toro, Vec3::new(-3.7, 0.0, -0.4), 0.5, 1.7);
    // Blue hour: the sun under 1° the horizon, ahead of the gate.
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(-0.25, -0.012, -0.97));
    let beam_tex = beam_texture(&mut scene);
    // The ward: a scrolling beam from post to post at shoulder height.
    scene.effects.beams.push(Beam {
        start: Vec3::new(-1.9, 2.3, -2.0),
        end: Vec3::new(1.9, 2.3, -2.0),
        width: 0.34,
        color: [1.2, 3.2, 6.4, 1.0],
        texture: Some(beam_tex),
        uv: RibbonUv::Tile { length: 1.0 / 1.4 },
        scroll: 1.6,
        ..Default::default()
    });
    // A lightning arc crackling between the posts' feet.
    scene.effects.beams.push(Beam {
        start: Vec3::new(-1.9, 0.75, -2.0),
        end: Vec3::new(1.9, 0.75, -2.0),
        width: 0.05,
        color: [8.0, 7.0, 14.0, 1.0],
        segments: 28,
        jitter: 0.22,
        seed: 9,
        ..Default::default()
    });
    // The ward's glow on the posts and the flags.
    scene.lighting.lights.push(LightDesc {
        position: Vec3::new(0.0, 2.3, -1.8),
        color: Vec3::new(0.35, 0.7, 1.0),
        intensity: 14.0,
        range: 7.0,
        ..Default::default()
    });
    // The spiral ribbon, rising and widening round the lantern.
    let points = (0..72)
        .map(|k| {
            let f = k as f32 / 71.0;
            let a = f * std::f32::consts::TAU * 2.5;
            let r = 0.55 + 0.15 * f;
            RibbonPoint {
                position: fire.with_y(0.15 + f * 2.9) + Vec3::new(a.cos() * r, 0.0, a.sin() * r),
                width: 0.04 + 0.16 * f,
                color: [
                    2.4 - 1.4 * f,
                    1.1 - 0.3 * f,
                    0.3 + 1.2 * f,
                    0.9 * (1.0 - f * f),
                ],
            }
        })
        .collect();
    scene.effects.ribbons.push(Ribbon {
        points,
        texture: None,
        uv: RibbonUv::Stretch,
        blend: ParticleBlend::Additive,
        effect: None,
    });
    // The lantern's own flame.
    scene.lighting.lights.push(LightDesc {
        position: fire,
        color: Vec3::new(1.0, 0.6, 0.3),
        intensity: 6.0,
        range: 5.0,
        ..Default::default()
    });
    // The wisps: a trail, an orb and a light each. The orbs are the last
    // instances and their lights the last lights (move_ward_wisps).
    let orb = scene.add_mesh(shapes::sphere(16, 12));
    for (i, c) in WISPS.iter().enumerate() {
        let style = scene.add_trail_style(TrailStyle {
            lifetime: 1.1,
            min_distance: 0.02,
            width: Curve::new(&[(0.0, 0.2), (1.0, 0.0)]),
            color: ColorCurve::new(&[
                (0.0, [c[0] * 4.0, c[1] * 4.0, c[2] * 4.0, 1.0]),
                (1.0, [c[0], c[1], c[2], 0.0]),
            ]),
            ..Default::default()
        });
        scene.effects.trails.push(Trail {
            id: i as u64,
            style,
            position: wisp(i, 0),
            tick: 0,
        });
    }
    for (i, c) in WISPS.iter().enumerate() {
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: [c[0] * 8.0, c[1] * 8.0, c[2] * 8.0],
            ..Default::default()
        });
        scene.add_instance(orb, m, Mat4::IDENTITY);
        scene.lighting.lights.push(LightDesc {
            position: wisp(i, 0),
            color: Vec3::from(*c),
            intensity: 5.0,
            range: 4.5,
            ..Default::default()
        });
    }
    move_ward_wisps(&mut scene, 0);
    scene.look.post.bloom = Some(Bloom::default());
    scene.frame.camera = Camera {
        position: Vec3::new(1.4, 1.3, 6.2),
        target: Vec3::new(-0.3, 1.8, -2.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.9 },
        ..Default::default()
    };
    scene
}

/// Moves the ward's wisps, their orbs and their lights to the frame's tick.
pub fn move_ward_wisps(scene: &mut RenderScene, frame: u32) {
    let tick = frame as u64;
    tick_all(scene, tick);
    let n = WISPS.len();
    let (orbs, lights) = (
        scene.objects.instances.len() - n,
        scene.lighting.lights.len() - n,
    );
    for i in 0..n {
        let p = wisp(i, tick);
        scene.effects.trails[i].position = p;
        scene.objects.instances[orbs + i].transform =
            Mat4::from_translation(p) * Mat4::from_scale(Vec3::splat(0.16));
        scene.lighting.lights[lights + i].position = p;
    }
}
