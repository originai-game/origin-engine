use super::*;

/// The area under a curve, against the trapezoids by hand: before the
/// first key it holds the first value, after the last the last.
#[test]
fn a_curve_integrates_its_trapezoids_and_holds_its_ends() {
    let c = Curve::new(&[(0.2, 1.0), (0.6, 3.0), (1.0, 0.0)]);
    // 0..0.2 at 1: 0.2; 0.2..0.6 from 1 to 3: 0.8; 0.6..1 from 3 to 0: 0.6.
    for (t, want) in [
        (0.1, 0.1),
        (0.2, 0.2),
        (0.4, 0.2 + 0.2 * 1.5),
        (0.6, 1.0),
        (1.0, 1.6),
        (1.5, 1.6),
    ] {
        assert!(
            (c.integral(t) - want).abs() < 1e-5,
            "t {t}: {} against {want}",
            c.integral(t)
        );
    }
    assert!((Curve::constant(2.0).integral(0.3) - 0.6).abs() < 1e-6);
}

fn live(e: &ParticleEmitter, sys: &mut ParticleSystem, ticks: u64) -> Vec<ParticleState> {
    let mut cpu = CpuParticles::default();
    let points = shape_points(&e.shape, None);
    for t in 0..ticks {
        sys.tick = t;
        cpu.advance(e, sys, 60.0, &points, &[], &[], None, None);
    }
    cpu.slots.into_iter().filter(|p| p.alive()).collect()
}

/// A rate curve shapes the emission: half the cycle at 0 and half at
/// 2 spawns what the flat rate does over the cycle, none of it in the
/// first half.
#[test]
fn a_rate_curve_shapes_when_particles_are_born() {
    let e = ParticleEmitter {
        rate: 60.0,
        duration: 2.0,
        looping: false,
        lifetime: [10.0, 10.0],
        rate_over_time: Some(Curve::new(&[(0.5, 0.0), (0.5, 2.0)])),
        max_particles: 1024,
        ..Default::default()
    };
    let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 1, 0);
    assert_eq!(
        live(&e, &mut sys, 60).len(),
        0,
        "the curve's first half is 0"
    );
    let n = live(&e, &mut sys, 130).len();
    assert!(
        (118..=122).contains(&n),
        "{n} born over the cycle (120 flat)"
    );
}

/// Emission thins an emitter by the share it names, and changing it
/// replaces nothing in the air: every particle living before the change
/// is the same particle after it.
#[test]
fn emission_thins_an_emitter_and_keeps_what_is_in_the_air() {
    let full = ParticleEmitter {
        rate: 600.0,
        lifetime: [1.0, 1.0],
        max_particles: 1024,
        ..Default::default()
    };
    let half = ParticleEmitter {
        emission: 0.5,
        ..full.clone()
    };
    let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 5, 0);
    let all = live(&full, &mut sys, 60).len();
    assert!((590..=600).contains(&all), "{all} alive of 600 a second");
    let n = live(&half, &mut sys, 60).len();
    // Half of 600, within four standard deviations (12 each).
    assert!((252..=348).contains(&n), "{n} alive at half the emission");

    let mut cpu = CpuParticles::default();
    let points = shape_points(&full.shape, None);
    let mut step = |e: &ParticleEmitter, cpu: &mut CpuParticles, t: u64| {
        sys.tick = t;
        cpu.advance(e, &sys, 60.0, &points, &[], &[], None, None);
    };
    for t in 0..30 {
        step(&full, &mut cpu, t);
    }
    let before = cpu.slots.clone();
    let thin = ParticleEmitter {
        emission: 0.25,
        ..full.clone()
    };
    step(&thin, &mut cpu, 30);
    let kept = before
        .iter()
        .zip(&cpu.slots)
        .filter(|(b, a)| b.alive() && a.alive() && a.misc == b.misc)
        .count();
    let living = before.iter().filter(|p| p.alive()).count();
    assert!(living >= 290, "{living} in the air before the change");
    assert_eq!(kept, living, "every particle in the air stayed");
}

/// An attractor gathers particles, a speed curve of 0 holds them where
/// they are, a spline sheds them along its curve and a moving emitter
/// lends them its velocity.
#[test]
fn attractors_speed_splines_and_inherited_velocity_move_particles() {
    let base = ParticleEmitter {
        shape: EmitterShape::Sphere {
            radius: 2.0,
            surface: true,
        },
        speed: [0.0, 0.0],
        gravity: Vec3::ZERO,
        lifetime: [5.0, 5.0],
        rate: 0.0,
        bursts: vec![Burst::once(0.0, 100)],
        ..Default::default()
    };
    let mean_r = |ps: &[ParticleState]| {
        ps.iter()
            .map(|p| Vec3::from_slice(&p.pos_age[..3]).length())
            .sum::<f32>()
            / ps.len() as f32
    };
    let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 3, 0);
    let still = mean_r(&live(&base, &mut sys, 60));
    let pulled = ParticleEmitter {
        attractors: vec![Attractor {
            position: Vec3::ZERO,
            strength: 4.0,
            radius: 0.0,
        }],
        ..base.clone()
    };
    let gathered = mean_r(&live(&pulled, &mut sys, 60));
    assert!(
        (still - 2.0).abs() < 1e-3 && gathered < 1.7,
        "radius {still} still, {gathered} pulled"
    );
    let held = ParticleEmitter {
        speed_over_life: Some(Curve::constant(0.0)),
        ..pulled
    };
    let h = live(&held, &mut sys, 60);
    assert!((mean_r(&h) - 2.0).abs() < 1e-3, "held at {}", mean_r(&h));
    // The pull at 2 m is 4 / (1 + 4) = 0.8 m/s², for a second.
    let speeds: Vec<f32> = h
        .iter()
        .map(|p| Vec3::from_slice(&p.vel_life[..3]).length())
        .collect();
    assert!(
        speeds.iter().all(|v| (0.7..0.9).contains(v)),
        "the pull still builds velocity: {:?}",
        &speeds[..4]
    );

    // A spline from (0,0,0) to (4,0,0): every particle is born on the x axis, spread along it.
    let mut points = [Vec3::ZERO; MAX_SPLINE_POINTS];
    points[1] = Vec3::new(4.0, 0.0, 0.0);
    let line = ParticleEmitter {
        shape: EmitterShape::Spline {
            points,
            count: 2,
            closed: false,
        },
        ..base.clone()
    };
    let born = live(&line, &mut sys, 1);
    assert!(
        born.iter()
            .all(|p| p.pos_age[1].abs() < 1e-4 && p.pos_age[2].abs() < 1e-4)
    );
    let xs: Vec<f32> = born.iter().map(|p| p.pos_age[0]).collect();
    let (lo, hi) = xs
        .iter()
        .fold((f32::MAX, f32::MIN), |a, &x| (a.0.min(x), a.1.max(x)));
    assert!(lo < 0.5 && hi > 3.5, "spread from {lo} to {hi}");

    // Moving at 6 m/s along x: half of it inherited.
    let thrown = ParticleEmitter {
        shape: EmitterShape::Point,
        rate: 60.0,
        bursts: Vec::new(),
        inherit_velocity: 0.5,
        ..base
    };
    let mut cpu = CpuParticles::default();
    for t in 0..30u64 {
        sys.tick = t;
        sys.transform = Mat4::from_translation(Vec3::new(t as f32 * 0.1, 0.0, 0.0));
        cpu.advance(
            &thrown,
            &sys,
            60.0,
            &[[0.0; 4], [0.0, 1.0, 0.0, 0.0]],
            &[],
            &[],
            None,
            None,
        );
    }
    let vx: Vec<f32> = cpu
        .slots
        .iter()
        .filter(|p| p.alive())
        .map(|p| p.vel_life[0])
        .collect();
    assert!(
        vx.len() > 20 && vx.iter().skip(2).all(|v| (v - 3.0).abs() < 1e-3),
        "{vx:?}"
    );
}

/// Curl noise is divergence free: its divergence by central
/// differences is a small fraction of its size (value noise itself,
/// the turbulence `noise` adds, is not).
#[test]
fn curl_noise_has_no_divergence() {
    let h = 0.01;
    let (mut div, mut mag) = (0.0f32, 0.0f32);
    for i in 0..64 {
        let q = Vec3::new(
            i as f32 * 0.37,
            (i % 7) as f32 * 0.61,
            (i % 5) as f32 * 0.43 + 0.1,
        );
        let c = |p: Vec3| curl_noise(p, [0; 3], 9);
        let d = (c(q + Vec3::X * h).x - c(q - Vec3::X * h).x + c(q + Vec3::Y * h).y
            - c(q - Vec3::Y * h).y
            + c(q + Vec3::Z * h).z
            - c(q - Vec3::Z * h).z)
            / (2.0 * h);
        div += d.abs();
        mag += c(q).length();
    }
    assert!(mag > 1.0, "the field is {mag}");
    assert!(div < 0.15 * mag, "divergence {div} against size {mag}");
}

/// A fountain: a burst, then 60 a second, each living a second.
fn fountain() -> ParticleEmitter {
    ParticleEmitter {
        max_particles: 256,
        rate: 60.0,
        bursts: vec![Burst::once(0.0, 20)],
        lifetime: [1.0, 1.0],
        speed: [2.0, 3.0],
        ..Default::default()
    }
}

fn run(sys: &mut ParticleSystem, cpu: &mut CpuParticles, ticks: std::ops::RangeInclusive<u64>) {
    let e = fountain();
    for t in ticks {
        sys.tick = t;
        cpu.advance(&e, sys, 60.0, &[], &[], &[], None, None);
    }
}

fn alive(cpu: &CpuParticles) -> Vec<ParticleState> {
    cpu.slots
        .iter()
        .copied()
        .filter(ParticleState::alive)
        .collect()
}

/// At a quarter of the pace a system takes four ticks for one: after
/// 80 ticks it holds what a normal one holds after 20 (the same spawns,
/// ages and positions to float error), and at 0 it holds still.
#[test]
fn a_time_scale_slows_spawning_ageing_and_motion() {
    let mut normal = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 7, 0);
    let mut slow = normal;
    slow.time_scale = 0.25;
    let (mut a, mut b) = (CpuParticles::default(), CpuParticles::default());
    run(&mut normal, &mut a, 0..=19);
    run(&mut slow, &mut b, 0..=79);
    let (a1, b1) = (alive(&a), alive(&b));
    assert_eq!(a1.len(), 20 + 20, "the burst and a third of a second");
    assert_eq!(b1.len(), a1.len());
    // The same particles (by their random values), each spawned within
    // a tick of its twin: a slow step spawns within its quarter tick.
    for (p, q) in a1.iter().zip(&b1) {
        assert_eq!(p.misc, q.misc);
        assert!(
            (p.pos_age[3] - q.pos_age[3]).abs() <= 1.0 / 60.0,
            "{p:?} against {q:?}"
        );
        let d = Vec3::from_slice(&p.pos_age[..3]).distance(Vec3::from_slice(&q.pos_age[..3]));
        assert!(d < 3.0 / 60.0, "{d}: {p:?} against {q:?}");
    }
    // Frozen: nothing spawns, ages or moves.
    slow.time_scale = 0.0;
    run(&mut slow, &mut b, 80..=139);
    assert_eq!(alive(&b), b1);
    // Back to normal: it goes on from where it stood.
    slow.time_scale = 1.0;
    run(&mut slow, &mut b, 140..=159);
    run(&mut normal, &mut a, 20..=39);
    assert_eq!(alive(&b).len(), alive(&a).len());
}

/// An emitter's `wind` is the share of the air its drag closes on: at 1
/// its particles end up moving with the air, in the world whichever way a
/// local-space emitter is turned; at 0.5 with half of it; at 0, or with no
/// drag, the air leaves them alone, bit for bit.
#[test]
fn drag_carries_particles_to_their_share_of_the_air() {
    let e = |wind: f32, drag: f32, space| ParticleEmitter {
        rate: 60.0,
        lifetime: [10.0, 10.0],
        speed: [1.0, 2.0],
        gravity: Vec3::ZERO,
        max_particles: 256,
        drag,
        wind,
        space,
        ..Default::default()
    };
    let air = Vec3::new(3.0, 0.0, -1.5);
    let turned = Mat4::from_rotation_y(1.1) * Mat4::from_rotation_x(0.4);
    // The oldest particles' velocity in the world after 3 s at drag 4.
    let settled = |wind: f32, space, transform: Mat4, air: Vec3| {
        let mut sys = ParticleSystem::new(1, EmitterHandle(0), transform, 3, 0);
        sys.air = air;
        let slots = live(&e(wind, 4.0, space), &mut sys, 180);
        let old: Vec<Vec3> = slots
            .iter()
            .filter(|p| p.pos_age[3] > 2.5)
            .map(|p| {
                let v = Vec3::from_slice(&p.vel_life[..3]);
                match space {
                    SimulationSpace::Local => transform.transform_vector3(v),
                    SimulationSpace::World => v,
                }
            })
            .collect();
        assert!(old.len() > 20, "{} old particles", old.len());
        old
    };
    for (space, transform) in [
        (SimulationSpace::World, Mat4::IDENTITY),
        (SimulationSpace::World, turned),
        (SimulationSpace::Local, turned),
    ] {
        for (wind, want) in [(1.0, air), (0.5, air * 0.5), (0.0, Vec3::ZERO)] {
            for v in settled(wind, space, transform, air) {
                assert!(
                    v.distance(want) < 0.01,
                    "{space:?} wind {wind}: {v} against {want}"
                );
            }
        }
    }
    // No wind share, or no drag: the same particles as in still air.
    let run = |wind: f32, drag: f32, air: Vec3| {
        let mut sys = ParticleSystem::new(1, EmitterHandle(0), turned, 3, 0);
        sys.air = air;
        live(&e(wind, drag, SimulationSpace::World), &mut sys, 120)
    };
    assert_eq!(run(0.0, 4.0, air), run(0.0, 4.0, Vec3::ZERO));
    assert_eq!(run(1.0, 0.0, air), run(0.0, 0.0, Vec3::ZERO));
}

/// A death sub-emitter fires `count` particles at each death, where it
/// happened, every run alike; the events report each death once.
#[test]
fn a_death_sub_emitter_fires_where_each_particle_dies() {
    let mut e = fountain();
    e.sub_emitters.push(SubEmitter {
        trigger: SubTrigger::Event(ParticleEventKind::Death),
        emitter: EmitterHandle(1),
        count: 3,
        inherit_velocity: 0.0,
    });
    let child = ParticleEmitter {
        shape: EmitterShape::Point,
        rate: 0.0,
        speed: [0.0, 0.0],
        lifetime: [5.0, 5.0],
        gravity: Vec3::ZERO,
        max_particles: 512,
        ..Default::default()
    };
    let go = || {
        let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 3, 0);
        let mut cpu = CpuParticles::default();
        let mut events = Vec::new();
        for t in 0..=89 {
            sys.tick = t;
            cpu.advance(
                &e,
                &sys,
                60.0,
                &[],
                &[],
                &[Some(&child)],
                Some(&mut events),
                None,
            );
        }
        (cpu, events)
    };
    let (a, events) = go();
    let deaths: Vec<&ParticleEvent> = events
        .iter()
        .filter(|ev| ev.kind == ParticleEventKind::Death)
        .collect();
    assert!(
        deaths.len() >= 20,
        "the burst dies after a second: {}",
        deaths.len()
    );
    assert_eq!(a.children[0].born, deaths.len() as u64 * 3);
    // Still children (speed 0, no gravity) sit where their parent died.
    let kids: Vec<Vec3> = a.children[0]
        .slots
        .iter()
        .filter(|s| s.alive())
        .map(|s| Vec3::from_slice(&s.pos_age[..3]))
        .collect();
    for d in &deaths {
        let near = kids
            .iter()
            .filter(|k| k.distance(d.position) < 1e-3)
            .count();
        assert_eq!(near, 3, "three children at {:?}", d.position);
    }
    let (b, again) = go();
    assert_eq!(events, again);
    assert_eq!(a.children[0].slots, b.children[0].slots);
}

#[test]
fn a_life_sub_emitter_runs_its_emission_from_every_living_parent() {
    // Two parents born at once, moving apart along ±x at 1 m/s for 2 s;
    // each runs the child's 10 per second for its life: a line of 20
    // still children behind each, one every 0.1 m.
    let mut e = ParticleEmitter {
        shape: EmitterShape::Point,
        rate: 0.0,
        bursts: vec![Burst {
            time: 0.0,
            count: 2,
            cycles: 1,
            interval: 0.0,
        }],
        duration: 1.0,
        looping: false,
        spread_degrees: 0.0,
        speed: [0.0, 0.0],
        lifetime: [2.0, 2.0],
        gravity: Vec3::ZERO,
        max_particles: 4,
        ..Default::default()
    };
    e.sub_emitters.push(SubEmitter {
        trigger: SubTrigger::Life,
        emitter: EmitterHandle(1),
        count: 0,
        inherit_velocity: 0.0,
    });
    let child = ParticleEmitter {
        shape: EmitterShape::Point,
        rate: 10.0,
        duration: 5.0,
        looping: false,
        speed: [0.0, 0.0],
        lifetime: [5.0, 5.0],
        gravity: Vec3::ZERO,
        max_particles: 64,
        ..Default::default()
    };
    let go = |ticks: u64| {
        let mut sys = ParticleSystem::new(1, EmitterHandle(0), Mat4::IDENTITY, 3, 0);
        let mut cpu = CpuParticles::default();
        for t in 0..ticks {
            sys.tick = t;
            // The parents' motion: set by hand, ±x by their slot.
            cpu.advance(&e, &sys, 60.0, &[], &[], &[Some(&child)], None, None);
            for (i, p) in cpu.slots.iter_mut().enumerate() {
                if t == 0 && p.alive() {
                    p.vel_life[0] = if i == 0 { 1.0 } else { -1.0 };
                }
            }
        }
        cpu
    };
    let a = go(150);
    assert_eq!(a.children[0].born, 40, "20 from each parent's two seconds");
    let kids: Vec<Vec3> = a.children[0]
        .slots
        .iter()
        .filter(|s| s.alive())
        .map(|s| Vec3::from_slice(&s.pos_age[..3]))
        .collect();
    assert_eq!(kids.len(), 40);
    // Born where each parent was: as far along x as it had gone.
    let right = kids.iter().filter(|k| k.x > 1e-3).count();
    let left = kids.iter().filter(|k| k.x < -1e-3).count();
    assert_eq!((left, right), (20, 20), "{kids:?}");
    // One every 0.1 m along each line (a tick's lag where the test
    // set the parents going).
    let mut xs: Vec<f32> = kids.iter().filter(|k| k.x > 0.0).map(|k| k.x).collect();
    xs.sort_by(f32::total_cmp);
    for w in xs.windows(2) {
        assert!((w[1] - w[0] - 0.1).abs() < 0.02, "{xs:?}");
    }
    assert!(kids.iter().all(|k| k.x.abs() <= 2.05 && k.y.abs() < 1e-4));
    assert_eq!(a.children[0].slots, go(150).children[0].slots);
}
