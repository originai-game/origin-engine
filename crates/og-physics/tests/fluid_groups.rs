//! og.Fluid2D groups: two fluids of one group in a tank, the heavier one
//! (density 3) poured in above the lighter one (density 1). With `mixing`
//! 0 they trade places and settle in layers, the heavy one under the light
//! one (a few light particles stay trapped on the floor under it, as in a
//! particle fluid they have nowhere to rise from); with `mixing` 1 their
//! mix evens out and they end as one blend.
//!
//! Ways this could fail, each caught below: the fluids ignore each other
//! (each settles as if alone: the heavy one would sit on top of the light
//! one, where it was poured); the masses are not used (both weigh the same:
//! no layering); mixing at 0 (the fluids must stay pure); mixing does nothing, or does not conserve the amount of
//! each fluid, or leaves the weights unchanged (no blend); a fluid in no
//! group carries a mix; the result depends on more than the inputs.

use crate::support;

use og_core::glam::{Vec2, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

fn tank(mixing: f32) -> (World, PhysicsSystem, Entity, Entity) {
    let (mut w, p) = world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    );
    spawn_static(
        &mut w,
        at(0.0, -0.25, 0.0),
        Collider::cuboid(Vec3::new(5.0, 0.5, 1.0)),
    );
    spawn_static(
        &mut w,
        at(-2.25, 3.0, 0.0),
        Collider::cuboid(Vec3::new(0.5, 7.0, 1.0)),
    );
    spawn_static(
        &mut w,
        at(2.25, 3.0, 0.0),
        Collider::cuboid(Vec3::new(0.5, 7.0, 1.0)),
    );
    let fluid = |density: f32, seed: u32| Fluid2D {
        count: 400,
        size: Vec2::new(3.6, 1.1),
        group: 1,
        density,
        mixing,
        seed,
        ..Fluid2D::default()
    };
    let heavy = w.spawn((at(0.0, 2.2, 0.0), fluid(3.0, 1))).unwrap();
    let light = w.spawn((at(0.0, 0.6, 0.0), fluid(1.0, 2))).unwrap();
    (w, p, heavy, light)
}

fn mean_y(p: &PhysicsSystem, e: Entity) -> f32 {
    let pts = p.fluid_particles(e).unwrap();
    pts.iter().map(|q| q.y).sum::<f32>() / pts.len() as f32
}

#[test]
fn a_heavy_fluid_poured_on_a_light_one_sinks_under_it_and_they_layer() {
    let (mut w, p, heavy, light) = tank(0.0);
    run(&mut w, 1);
    assert!(mean_y(&p, heavy) > mean_y(&p, light) + 1.0, "poured on top");
    run(&mut w, 599);
    let (h, l) = (mean_y(&p, heavy), mean_y(&p, light));
    eprintln!("mixing 0 after 10 s: heavy at {h:.3} m, light at {l:.3} m");
    assert!(
        h + 0.3 < l,
        "the heavy fluid sank under the light one (heavy {h}, light {l})"
    );
    // Layered: nine tenths of the heavy fluid lie below the light fluid's
    // median height (a few light particles stay trapped on the floor).
    let mut light_y: Vec<f32> = p
        .fluid_particles(light)
        .unwrap()
        .iter()
        .map(|q| q.y)
        .collect();
    light_y.sort_by(f32::total_cmp);
    let median = light_y[light_y.len() / 2];
    let under = p
        .fluid_particles(heavy)
        .unwrap()
        .iter()
        .filter(|q| q.y < median)
        .count();
    eprintln!("{under} of 400 heavy particles below the light fluid's median ({median:.3} m)");
    assert!(
        under * 10 >= 400 * 9,
        "{under} of 400 heavy particles under the light layer"
    );
    // Unmixed: each particle is still only its own fluid.
    let (slot, mix) = p.fluid_mix(heavy).unwrap();
    assert_eq!(slot, 0, "the heavy fluid has the lower entity id");
    assert!(
        mix.iter().all(|m| m[0] == 1.0 && m[1] == 0.0),
        "mixing 0 keeps the fluids apart"
    );
    for q in p
        .fluid_particles(heavy)
        .unwrap()
        .iter()
        .chain(p.fluid_particles(light).unwrap().iter())
    {
        assert!(
            q.is_finite() && q.x > -2.01 && q.x < 2.01 && q.y > -0.01,
            "leaked: {q}"
        );
    }
}

#[test]
fn with_mixing_the_two_fluids_blend_into_one() {
    let (mut w, p, heavy, light) = tank(1.0);
    run(&mut w, 600);
    let (_, hm) = p.fluid_mix(heavy).unwrap();
    let (slot, lm) = p.fluid_mix(light).unwrap();
    assert_eq!(slot, 1);
    let all: Vec<[f32; 4]> = hm.iter().chain(lm.iter()).copied().collect();
    // The amount of each fluid is kept: half the particles' worth each.
    let heavy_part: f32 = all.iter().map(|m| m[0]).sum::<f32>() / all.len() as f32;
    assert!(
        (heavy_part - 0.5).abs() < 1e-3,
        "the heavy fluid's share is {heavy_part}"
    );
    for m in &all {
        assert!(
            (m[0] + m[1] - 1.0).abs() < 1e-4 && m[2] == 0.0 && m[3] == 0.0,
            "{m:?}"
        );
    }
    // Blended: every particle is mostly a mix, none still pure.
    let spread =
        (all.iter().map(|m| (m[0] - 0.5) * (m[0] - 0.5)).sum::<f32>() / all.len() as f32).sqrt();
    let pure = all.iter().filter(|m| m[0] > 0.95 || m[0] < 0.05).count();
    eprintln!("mixing 1 after 10 s: the heavy share spreads ±{spread:.3}, {pure} pure particles");
    assert!(spread < 0.15, "the mix evened out (±{spread})");
    assert!(pure < 40, "{pure} particles never mixed");
    // And the blend no longer layers by the two densities.
    let (h, l) = (mean_y(&p, heavy), mean_y(&p, light));
    assert!(
        (h - l).abs() < 0.3,
        "one blend, not layers (heavy {h}, light {l})"
    );
}

#[test]
fn a_fluid_alone_carries_no_mix_and_groups_replay() {
    let (mut w, p, heavy, light) = tank(0.5);
    let alone = w
        .spawn((
            at(30.0, 1.0, 0.0),
            Fluid2D {
                count: 50,
                size: Vec2::splat(0.5),
                ..Fluid2D::default()
            },
        ))
        .unwrap();
    run(&mut w, 120);
    assert_eq!(p.fluid_mix(alone), None);
    let snapshot = w.snapshot();
    run(&mut w, 120);
    let (mut w2, p2, heavy2, light2) = tank(0.5);
    w2.spawn((
        at(30.0, 1.0, 0.0),
        Fluid2D {
            count: 50,
            size: Vec2::splat(0.5),
            ..Fluid2D::default()
        },
    ))
    .unwrap();
    w2.restore(&snapshot).unwrap();
    run(&mut w2, 120);
    assert_eq!(w.world_hash(), w2.world_hash());
    assert_eq!(p.fluid_particles(heavy), p2.fluid_particles(heavy2));
    assert_eq!(p.fluid_mix(light), p2.fluid_mix(light2));
}
