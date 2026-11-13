use super::*;

/// water_sea.wgsl's `ocean_displace` with every wave kept, on the CPU.
fn drawn(swell: &[[f32; 4]], frame_xz: Vec2) -> Vec3 {
    let mut d = Vec3::ZERO;
    for w in swell.chunks(2) {
        let (a, b) = (w[0], w[1]);
        let th = a[0] * frame_xz.x + a[1] * frame_xz.y + a[3];
        d += Vec3::new(b[0] * th.cos(), a[2] * th.sin(), b[1] * th.cos());
    }
    d
}

/// A display frame between ticks draws the sea at the time its blended
/// transforms are drawn at, between the last two ticks (the code before
/// drew every such frame at the last tick: the sea stepped at the
/// tick's rate on a faster display); a frame at the tick draws the tick.
#[test]
fn the_sea_runs_at_the_display_rate_between_ticks() {
    let mut scene = crate::RenderScene::new();
    scene.frame.tick = 600;
    scene.frame.tick_rate = 60.0;
    assert_eq!(seconds(&scene), 10.0);
    scene.frame.interpolation = Some(0.25);
    assert!((seconds(&scene) - 599.25 / 60.0).abs() < 1e-12);
    scene.frame.interpolation = Some(1.0);
    assert_eq!(seconds(&scene), 10.0);
    // Two frames of one tick at different alphas draw different seas.
    let water = Water {
        size: Vec2::splat(2000.0),
        wind: Vec2::new(9.0, 3.0),
        ..Default::default()
    };
    let sea = ocean(&water).expect("a sea");
    let at = glam::DVec2::new(13.0, -7.0);
    let mut height = |a: f32| {
        scene.frame.interpolation = Some(a);
        sea.height(at, seconds(&scene))
    };
    assert!((height(0.2) - height(0.7)).abs() > 1e-3);
    // Monotonic across a tick: the frame after the tick's last blends on.
    let late = 599.95 / 60.0;
    let mut next = crate::RenderScene::new();
    next.frame.tick = 601;
    next.frame.tick_rate = 60.0;
    next.frame.interpolation = Some(0.05);
    assert!(seconds(&next) > late);
}

/// The vertices draw the sea the tick floats bodies on: water_sea.wgsl's
/// `ocean_displace` (every wave of og_core, each left out only where
/// the clipmap's vertex spacing cannot hold it, the rest point found as
/// the tick finds it) is [`og_env::ocean::Ocean::height`] to a
/// centimetre plus the amplitudes of the waves that spacing leaves out:
/// nothing in a gale to past 10 m, and at any distance, were they all
/// to crest at one point, under a hundredth of it (half a degree from
/// the eye; their RMS a fifth of that, a pixel of a 720p frame). The waves the cascades add start
/// past the sea's shortest wave, so none is drawn twice, and are under
/// 2 % of its height in RMS. With a grid of the tick's own under both
/// (r5-fft), the cascades redrew what the grid faded and the two were
/// 0.2 m RMS apart in a gale at the middle of the surface and 0.8 m at
/// its edge.
#[test]
fn the_vertices_draw_the_sea_the_tick_floats_bodies_on() {
    let eye = Vec3::new(0.0, 10.0, 0.0);
    let (segments, cascades) = sea_tier(&crate::RenderQuality::default());
    let levels = clipmap(eye, Vec3::ZERO, segments);
    // water_sea.wgsl sea_vertex's spacing d metres from the clipmap's centre: the first level that holds it, its cell
    // growing to twice at its edge (the geomorph).
    let spacing = |d: f32| {
        let (cell, half) = levels
            .iter()
            .map(|l| (l[2], l[2] * segments as f32 * 0.5))
            .find(|(_, half)| *half >= d)
            .expect("inside the clipmap");
        let m = ((d / half - 0.7) / 0.25).clamp(0.0, 1.0);
        cell * (1.0 + m * m * (3.0 - 2.0 * m))
    };
    for (wind, choppiness) in [
        (Vec2::new(-3.0, -4.0), 0.6),
        (Vec2::new(-3.0, -9.5), 0.6),
        (Vec2::new(7.0, -18.7), 1.0),
    ] {
        let water = Water {
            size: Vec2::splat(6000.0),
            wind,
            choppiness,
            ..Default::default()
        };
        let sea = ocean(&water).expect("a sea");
        let hs = sea.significant_height();
        // The amplitudes of the waves a spacing leaves out.
        let left = |cell: f32| -> f32 {
            sea.waves
                .iter()
                .map(|w| w.amplitude * (1.0 - sea_keep(w.k, cell)))
                .sum()
        };
        let mut whole = 0;
        for seconds in [0.0, 3.0, 4.5, 61.25] {
            let (_, swell) = ocean_gpu(&water, seconds, Vec3::ZERO);
            let drawn = |p: Vec2| {
                let cell = spacing(p.distance(Vec2::new(eye.x, eye.z)));
                let mut d = Vec3::ZERO;
                for w in swell.chunks(2) {
                    let (a, b) = (w[0], w[1]);
                    let k = Vec2::new(a[0], a[1]);
                    let th = k.dot(p) + a[3];
                    let f = sea_keep(k.length(), cell);
                    d += Vec3::new(b[0] * th.cos(), a[2] * th.sin(), b[1] * th.cos()) * f;
                }
                d
            };
            for i in 0..48 {
                let r = 1.5 * 1.1f32.powi(i);
                let at = Vec2::from_angle(i as f32 * 2.4) * r;
                let mut rest = at;
                for _ in 0..8 {
                    let d = drawn(rest);
                    rest = at - Vec2::new(d.x, d.z);
                }
                let got = drawn(rest).y;
                let tick = sea.height(at.as_dvec2(), seconds);
                // The rest point lies within the crests' lean of `at`.
                let out = left(spacing(r + 0.5 * hs));
                whole += usize::from(out == 0.0);
                assert!(
                    (got - tick).abs() < 0.01 + out,
                    "{wind} at {seconds} s, {at}: drawn {got} m, the tick's {tick} m ({out} m left out)"
                );
                assert!(
                    out < 0.01 * r.max(2.0),
                    "{wind}: {r} m from the eye the vertices leave {out} m of a {hs} m sea out"
                );
            }
        }
        println!("{wind}: Hs {hs} m, every wave drawn at {whole} of 192 points");
        assert!(wind.length() < 15.0 || whole >= 100, "{wind}: {whole}");
        let built = sea_spectrum(&water, &sea, cascades);
        let shortest = og_env::ocean::shortest_wavenumber(wind.length());
        let mut variance = 0.0;
        for c in &built.cascades {
            assert!(
                c.band.lo >= f64::from(shortest) * 0.999,
                "{wind}: a cascade from {} rad/m over the sea's waves to {shortest}",
                c.band.lo
            );
            variance += c.variance;
        }
        println!("{wind}: the cascades {} m RMS", variance.sqrt());
        assert!(
            variance.sqrt() < 0.02 * f64::from(hs),
            "{wind}: the cascades draw {} m RMS over a {hs} m sea",
            variance.sqrt()
        );
    }
}

/// The foam's levels follow the wind's whitecap coverage: a breeze's
/// crest whitens where the surface has bunched to some six tenths of
/// its area without folding, before the foam it lays down does, the
/// gale's steeper sea foams only where it bunches much further, and a
/// round swell of the same gale (no horizontal displacement, a Jacobian
/// of 1 everywhere) never.
#[test]
fn foam_follows_the_winds_whitecap_coverage() {
    let thresholds = |wind: Vec2, choppiness: f32| {
        let water = Water {
            size: Vec2::splat(6000.0),
            segments: 256,
            wind,
            choppiness,
            ..Default::default()
        };
        let sea = ocean(&water).expect("a sea");
        foam_thresholds(&water, &sea_spectrum(&water, &sea, 3))
    };
    let breeze = thresholds(Vec2::new(-3.0, -8.5), 0.6);
    let gale = thresholds(Vec2::new(7.0, -18.7), 1.0);
    println!("breeze {breeze:?}, gale {gale:?}");
    assert!(breeze.fresh > 0.4 && breeze.fresh < 0.75, "{breeze:?}");
    assert!(breeze.fresh < breeze.laid[0]);
    assert!(
        gale.laid[0] < breeze.laid[0] - 0.15 && gale.fresh < breeze.fresh - 0.15,
        "{gale:?}"
    );
    assert!(gale.fresh > 0.0, "{gale:?}");
    // The gale's foam is full by the median of what lies under each
    // level, some 0.1 under it: the least gain would leave it a fifth.
    assert!(gale.gains.iter().all(|&g| g > 6.0), "{gale:?}");
    let round = FoamLevels {
        laid: [0.9; 2],
        fresh: 0.9,
        gains: [sea::FOAM_GAIN, sea::FOAM_GAIN, sea::FRESH_GAIN],
    };
    assert_eq!(thresholds(Vec2::new(7.0, -18.7), 0.0), round);
}

/// Each persistent foam level lays foam on the whitecap coverage's half
/// of the surface as it resolves the Jacobian (ocean_foam.wgsl reads the
/// cascades from their mips at its texel): the gale's coarse level, 2 m
/// texels, is much smoother than its crests, and under the fine level's
/// threshold it laid only a small part of it (r5 at 88d021868: two or
/// three whitecaps in a gale's view).
#[test]
fn each_foam_level_lays_the_whitecap_coverage_it_resolves() {
    let water = Water {
        size: Vec2::splat(6000.0),
        segments: 256,
        wind: Vec2::new(7.0, -18.7),
        choppiness: 1.0,
        ..Default::default()
    };
    let sea = ocean(&water).expect("a sea");
    let built = sea_spectrum(&water, &sea, 3);
    let levels = foam_thresholds(&water, &built);
    let half_w = 3.84e-6 * f64::from(water.wind.length()).powf(3.41) * 0.5;
    let share = |jac: &[f64], under: f32| {
        jac.iter().filter(|&&j| j < f64::from(under)).count() as f64 / jac.len() as f64
    };
    for (l, &under) in levels.laid.iter().enumerate() {
        let ts =
            (crate::ocean_fft::foam::LEVEL_SIZE[l] / f64::from(crate::ocean_fft::foam::RES)) as f32;
        let jac = jacobians(&water, &built, ts);
        let got = share(&jac, under);
        let fine = share(&jac, levels.laid[0]);
        println!(
            "level {l} ({ts} m): threshold {under}, lays {got:.4} (W/2 {half_w:.4}); under level 0's {fine:.4}"
        );
        assert!(
            (got / half_w - 1.0).abs() < 0.05,
            "level {l}: {got} of the surface, not {half_w}"
        );
    }
    let coarse = jacobians(&water, &built, 2.0);
    assert!(
        levels.laid[1] > levels.laid[0] + 0.05 && share(&coarse, levels.laid[0]) < 0.5 * half_w,
        "{levels:?}"
    );
}

/// The sea the shader draws at a frame point is og_core's at the same
/// place in the absolute world (frame + the scene's origin) and the
/// same tick, wherever the origin has moved to: what the simulation's
/// height query floats a boat on.
#[test]
fn the_drawn_sea_is_the_simulations_wherever_the_origin_is() {
    let water = Water {
        size: Vec2::splat(800.0),
        wind: Vec2::new(3.0, -8.0),
        choppiness: 0.8,
        ..Default::default()
    };
    let sea = ocean(&water).expect("a sea");
    let seconds = 4321.25;
    for origin in [Vec3::ZERO, Vec3::new(51_200.0, 0.0, -25_600.0)] {
        let (head, swell) = ocean_gpu(&water, seconds, origin);
        assert_eq!(head[0], 1.0);
        for p in [
            Vec2::new(0.0, 0.0),
            Vec2::new(37.5, -120.0),
            Vec2::new(-210.0, 66.0),
        ] {
            let at = glam::DVec2::new(
                f64::from(origin.x) + f64::from(p.x),
                f64::from(origin.z) + f64::from(p.y),
            );
            let phases = sea.phases(seconds, at);
            let want = sea.displacement(Vec2::ZERO, &phases);
            let got = drawn(&swell, p);
            assert!(
                (got - want).length() < 2e-3,
                "{origin} {p}: {got} vs {want}"
            );
        }
    }
    // A surface over a mesh, or without a wind, has no sea.
    let mut calm = water.clone();
    calm.wind = Vec2::ZERO;
    assert_eq!(ocean_gpu(&calm, seconds, Vec3::ZERO).0, [0.0; 4]);
}

/// The shore a sea with a seabed wets (W1): the band its surf runs up,
/// half the sea's significant height times the trains' strength, over
/// its seabed's rectangle in the frame, which moves with the frame's
/// render origin as the field does; none without a seabed.
/// W5: the clock the water's bank foam drifts by keeps a millisecond
/// after 10⁶ s (twelve days) of tick time, and runs on smoothly across
/// its fold: f32 seconds, which the foam read before, step by 1/16 s
/// there, which drew the drifting patches jumping.
#[test]
fn the_waters_clock_keeps_its_precision_in_a_long_session() {
    let water = Water::default();
    let q = crate::RenderQuality::default();
    let at = |s: f64| spray_gpu(&water, false, s, &q)[3];
    let t: f64 = 1.0e6 + 0.123_456;
    let exact = t.rem_euclid(SPRAY_FOLD);
    assert!(
        (f64::from(at(t)) - exact).abs() < 1e-3,
        "{} against {exact}",
        at(t)
    );
    assert!(
        (f64::from(t as f32) - t).abs() > 1e-3,
        "f32 seconds would hold it too"
    );
    // Across the fold the bank foam's 8 s phase runs on.
    let phase = |s: f64| (f64::from(at(s)) / 8.0).fract();
    let before = phase(SPRAY_FOLD * 300.0 - 0.01);
    let after = phase(SPRAY_FOLD * 300.0 + 0.01);
    assert!(
        ((after - before).rem_euclid(1.0) - 0.0025).abs() < 1e-3,
        "{before} to {after}"
    );
}

#[test]
fn a_seabed_wets_the_shore_its_surf_runs_up() {
    let depth: Vec<f32> = (0..16 * 8).map(|i| 5.0 - (i % 16) as f32).collect();
    let field =
        Arc::new(og_env::ocean::Shore::new(Vec2::new(-10.0, 4.0), 2.0, 16, 8, &depth).unwrap());
    let sea = Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0)),
        wind: Vec2::new(10.0, 0.0),
        ..Default::default()
    };
    assert_eq!(
        caustics(std::slice::from_ref(&sea), Vec3::ZERO, Vec3::ZERO)[4],
        [0.0; 4]
    );
    let hs = ocean(&sea).unwrap().significant_height();
    let mut beach = sea;
    beach.shore = Some(WaterShore {
        trains: og_env::ocean::ShoreTrains::new(hs, 1.0, 0.0),
        origin: field.origin + Vec2::new(3.0, -1.0),
        field,
        foam: 1.0,
    });
    let rows = caustics(&[beach], Vec3::ZERO, Vec3::ZERO);
    assert_eq!(rows[4][0], 1.5);
    assert!((rows[4][1] - 0.5 * hs).abs() < 1e-4, "{:?}", rows[4]);
    assert!(rows[4][2] > 0.0);
    assert_eq!(rows[5], [-7.0, 3.0, 23.0, 17.0]);
}
