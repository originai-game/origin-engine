//! `particles-lit`: lit particles in the shrine court under a low
//! afternoon sun. Smoke rises from a smouldering offering pit, its sun
//! side bright and its far side in its own shade; stone shards burst from
//! a crater in the flags, tumble lit, bounce and kick a dust puff at each
//! bounce (a collision sub-emitter, so the CPU path); a heat haze over a
//! glowing brazier bends the torii behind it. The measured fixture is
//! [`super::particles_lit_scene`].

use super::*;

const PIT: Vec3 = Vec3::new(-2.8, 0.0, -0.4);
const CRATER: Vec3 = Vec3::new(0.2, 0.02, 0.6);
const BRAZIER: Vec3 = Vec3::new(2.6, 0.0, -0.6);

pub fn particles_lit_stage_scene() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let rubble = crate::shadow_scenes::court(&mut scene);
    // The torii the haze bends, behind the brazier.
    kit::prop(&mut scene, "torii").place(&mut scene, Vec3::new(1.6, 0.0, -3.4), 0.0, 4.2);
    let smoke_tex = smoke_texture(&mut scene);
    // The pit's stone kerb and the brazier's block.
    for (min, max) in [
        (
            PIT - Vec3::new(0.6, 0.0, 0.6),
            PIT + Vec3::new(0.6, 0.25, 0.6),
        ),
        (
            BRAZIER - Vec3::new(0.35, 0.0, 0.35),
            BRAZIER + Vec3::new(0.35, 0.5, 0.35),
        ),
    ] {
        let block = scene.add_mesh(kit::block(min, max, 1.0));
        scene.add_instance(block, rubble, Mat4::IDENTITY);
    }
    let coals = scene.add_material(Material {
        base_color: [0.08, 0.06, 0.05, 1.0],
        emissive: [3.0, 0.9, 0.2],
        roughness: 0.9,
        ..Default::default()
    });
    let bed = scene.add_mesh(shapes::cuboid(Vec3::new(0.6, 0.06, 0.6)));
    scene.add_instance(bed, coals, Mat4::from_translation(BRAZIER + Vec3::Y * 0.52));
    scene.lighting.lights.push(Light {
        position: BRAZIER + Vec3::Y * 0.9,
        color: Vec3::new(1.0, 0.5, 0.15),
        intensity: 4.0,
        range: 4.0,
        ..Default::default()
    });
    // Lit smoke: a grey column the sun shades from the right.
    let smoke = scene.add_emitter(ParticleEmitter {
        max_particles: 220,
        shape: EmitterShape::Cone {
            angle_degrees: 10.0,
            radius: 0.35,
        },
        rate: 32.0,
        lifetime: [3.6, 4.6],
        speed: [0.8, 1.1],
        size: [0.7, 1.0],
        size_over_life: Curve::new(&[(0.0, 0.5), (1.0, 1.9)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.0]),
            (0.15, [1.0, 1.0, 1.0, 0.6]),
            (1.0, [1.0, 1.0, 1.0, 0.0]),
        ]),
        color: [0.32, 0.31, 0.3, 1.0],
        rotation_degrees: [0.0, 360.0],
        angular_velocity_degrees: [-20.0, 20.0],
        gravity: Vec3::new(0.15, 0.3, 0.0),
        drag: 0.4,
        texture: Some(smoke_tex),
        blend: ParticleBlend::Alpha,
        lit: true,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        50,
        smoke,
        Mat4::from_translation(PIT + Vec3::Y * 0.2),
        11,
        0,
    ));
    // Shards and their dust.
    let dust = scene.add_emitter(ParticleEmitter {
        max_particles: 400,
        shape: EmitterShape::Point,
        spread_degrees: 80.0,
        rate: 0.0,
        lifetime: [0.6, 0.9],
        speed: [0.2, 0.5],
        size: [0.18, 0.3],
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.7]),
            (1.0, [1.0, 1.0, 1.0, 0.0]),
        ]),
        // Linear: the flags' pale dust.
        color: [0.2, 0.17, 0.13, 1.0],
        gravity: Vec3::new(0.0, 0.2, 0.0),
        texture: Some(smoke_tex),
        blend: ParticleBlend::Alpha,
        lit: true,
        ..Default::default()
    });
    let shard = scene.add_mesh(shapes::tetrahedron());
    let debris = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Point,
        spread_degrees: 35.0,
        rate: 0.0,
        bursts: vec![Burst::once(0.0, 40)],
        duration: 1.0,
        lifetime: [1.6, 2.0],
        speed: [3.5, 5.0],
        size: [0.12, 0.22],
        color_over_life: ColorCurve::new(&[(0.0, [1.0; 4]), (1.0, [1.0; 4])]),
        // Linear: the flags' stone.
        color: [0.1, 0.085, 0.07, 1.0],
        rotation_degrees: [0.0, 360.0],
        angular_velocity_degrees: [-540.0, 540.0],
        gravity: Vec3::new(0.0, -9.8, 0.0),
        collision: Some(ParticleCollision {
            bounce: 0.35,
            friction: 0.4,
            ..Default::default()
        }),
        render: ParticleRender::Mesh { mesh: shard },
        lit: true,
        sub_emitters: vec![og_render::SubEmitter {
            trigger: og_render::SubTrigger::Event(og_render::ParticleEventKind::Collision),
            emitter: dust,
            count: 2,
            inherit_velocity: 0.1,
        }],
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        51,
        debris,
        Mat4::from_translation(CRATER),
        23,
        0,
    ));
    scene.world.colliders.push(og_render::SceneCollider::Plane {
        normal: Vec3::Y,
        offset: 0.0,
    });
    // The haze over the brazier.
    let haze = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(0.25, 0.05, 0.25),
        },
        rate: 30.0,
        lifetime: [1.6, 2.0],
        speed: [0.6, 0.9],
        size: [0.6, 0.9],
        size_over_life: Curve::new(&[(0.0, 0.6), (1.0, 1.5)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.0]),
            (0.25, [1.0, 1.0, 1.0, 1.0]),
            (1.0, [1.0, 1.0, 1.0, 0.0]),
        ]),
        rotation_degrees: [0.0, 360.0],
        gravity: Vec3::ZERO,
        distortion: 0.008,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        52,
        haze,
        Mat4::from_translation(BRAZIER + Vec3::Y * 0.6),
        31,
        0,
    ));
    // A low sun from the right, 20° up.
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(0.85, 0.36, 0.38));
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(-0.2, 1.7, 6.6),
        target: Vec3::new(0.0, 1.3, -0.8),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}
