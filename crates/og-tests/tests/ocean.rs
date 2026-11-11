//! R5's water (`crates/og-render/README.md` §Water): each scene matches its
//! reference for this backend tier, and behaviour read from the frames: a
//! wind raises a sea (much more light and dark across the water than the
//! same water becalmed) that moves with the tick; a gale's steep crests
//! foam white where a round swell of the same waves does not; the low sun
//! lays a glitter path on the sea under it; a flow map that holds one
//! direction everywhere draws the stream that flow draws without one.

use glam::{Vec2, Vec3};
use og_render::{RenderScene, RenderTarget, Renderer, RgbaImage};
use og_tests::scenes;

/// The mean absolute difference of two frames, per channel.
fn difference(a: &RgbaImage, b: &RgbaImage) -> f32 {
    let sum: f64 = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .map(|(x, y)| f64::from((*x as i32 - *y as i32).unsigned_abs()))
        .sum();
    (sum / a.pixels.len() as f64) as f32
}

/// The boats sample's floaters (the former test sample sit in the
/// sea the GPU draws: at their poses at 5 s and 6.5 s
/// (`boats/waterline.txt`, the run's dump), the drawn surface's height
/// under each, read back from the GPU (`Renderer::sea_heights`), has the
/// tick's waves drawn as og.physics.waterHeight floats them on (to 2 cm:
/// og_core::ocean at the tick's grid cell), every buoy crosses the drawn
/// surface, and no keel corner of a hull stands 5 cm clear of it: the FFT's
/// chop (3 to 14 m waves the tick does not float hulls on) is calmed under
/// each hull (its footprint in `Water::wakes`, as og.Buoyancy's step
/// records it), where it bared a corner by 6 cm. Prints each floater's
/// clearance against the drawn sea and the tick's.
#[test]
#[ignore = "requires GPU"]
fn the_floaters_sit_in_the_drawn_sea() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../og-render/tests/assets/waterline.txt"
    ))
    .expect("the floaters' poses");
    let water = og_render::Water {
        size: Vec2::splat(6000.0),
        segments: 256,
        wind: Vec2::new(7.0, -18.7),
        choppiness: 1.0,
        foam_width: 0.0,
        depth_fade: 30.0,
        wave_strength: 0.0,
        ..Default::default()
    };
    let sea = og_core::ocean::Ocean::new(water.wind, water.swell, water.choppiness).expect("a sea");
    // og-physics' WaterBody::surface: the grid the tick floats bodies on.
    let extent = water.size.max_element();
    let even = extent / og_core::ocean::grid_segments(water.segments, extent) as f32;
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut worst_keel = f32::MIN;
    // Every point is printed before anything is asserted, so one mismatch does not hide the rest.
    let mut failures: Vec<String> = Vec::new();
    for tick in [300u64, 390] {
        let mut scene = RenderScene::new();
        scene.tick = tick;
        scene.tick_rate = 60.0;
        let mut sea_now = water.clone();
        sea_now.wakes = Some(std::sync::Arc::new(og_render::WaterWakes {
            epoch: 1,
            dt: 1.0 / 60.0,
            ticks: vec![(tick, hull_cuts(&text, tick))],
            ..Default::default()
        }));
        scene.world.water.push(sea_now);
        scene.camera = og_render::Camera {
            position: Vec3::new(0.0, 10.0, 0.0),
            target: Vec3::new(-4.0, 0.0, -60.0),
            far: 3000.0,
            ..Default::default()
        };
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let seconds = tick as f64 / 60.0;
        // (name, point, kind): a buoy's centre, or a hull's keel corner.
        let mut points: Vec<(String, Vec3, bool)> = Vec::new();
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f[0].parse::<u64>() != Ok(tick) {
                continue;
            }
            let v: Vec<f32> = f[2..]
                .iter()
                .map(|x| x.parse().expect("a number"))
                .collect();
            let at = Vec3::new(v[0], v[1], v[2]);
            let q = glam::Quat::from_xyzw(v[3], v[4], v[5], v[6]);
            if f[1].starts_with("boat") {
                for (x, z) in [(-1.2, -3.0), (1.2, -3.0), (-1.2, 3.0), (1.2, 3.0)] {
                    points.push((f[1].to_string(), at + q * Vec3::new(x, -0.5, z), true));
                }
            } else {
                points.push((f[1].to_string(), at, false));
            }
        }
        let xz: Vec<Vec2> = points.iter().map(|(_, p, _)| Vec2::new(p.x, p.z)).collect();
        let drawn = renderer.sea_heights(&xz).expect("the sea's heights");
        assert_eq!(drawn.len(), points.len(), "a sea was drawn");
        for ((name, p, keel), [height, swell, cascades]) in points.iter().zip(&drawn) {
            let cell = og_core::ocean::grid_cell(Vec2::new(p.x, p.z), water.size * 0.5, even);
            let tick_height = sea.height_on_grid(
                glam::DVec2::new(f64::from(p.x), f64::from(p.z)),
                seconds,
                cell,
            );
            let what = if *keel { "keel corner" } else { "centre" };
            println!(
                "{seconds} s {name} {what}: {:+.3} m over the drawn sea ({:+.3} the tick's waves, {:+.3} the cascades), {:+.3} m over the tick's",
                p.y - height,
                swell,
                cascades,
                p.y - tick_height
            );
            if (swell - tick_height).abs() >= 0.02 {
                failures.push(format!(
                    "{seconds} s {name} {what}: the tick's waves drawn at {swell} m, og.physics.waterHeight {tick_height} m"
                ));
            }
            if *keel {
                worst_keel = worst_keel.max(p.y - height);
            } else if (p.y - height).abs() >= 0.6 {
                failures.push(format!(
                    "{seconds} s {name}: its centre {} m off the drawn sea (radius 0.6)",
                    p.y - height
                ));
            }
        }
    }
    println!("highest keel corner over the drawn sea: {worst_keel:+.3} m");
    if worst_keel >= 0.05 {
        failures.push(format!(
            "a keel corner stands {worst_keel} m clear of the drawn sea"
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The floaters' footprints at `tick` as og.Buoyancy's step records them
/// for a sea (og-physics ext.rs `hull_cuts`): each hull's box (a boat's
/// 2.4 × 1 × 6 m, a buoy's 1.2 m cube) turned by its pose, as the rectangle
/// its own x and z axes span over the water's xz.
fn hull_cuts(text: &str, tick: u64) -> Vec<og_render::WaterCut> {
    let mut cuts = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f[0].parse::<u64>() != Ok(tick) {
            continue;
        }
        let v: Vec<f32> = f[2..]
            .iter()
            .map(|x| x.parse().expect("a number"))
            .collect();
        let q = glam::Quat::from_xyzw(v[3], v[4], v[5], v[6]);
        let half = if f[1].starts_with("boat") {
            Vec3::new(1.2, 0.5, 3.0)
        } else {
            Vec3::splat(0.6)
        };
        let (ax, ay, az) = (q * Vec3::X, q * Vec3::Y, q * Vec3::Z);
        let flat = |a: Vec3| Vec2::new(a.x, a.z);
        let axis = flat(ax).try_normalize().unwrap_or(Vec2::X);
        let reach = |d: Vec2| {
            (flat(ax).dot(d) * half.x).abs()
                + (flat(ay).dot(d) * half.y).abs()
                + (flat(az).dot(d) * half.z).abs()
        };
        let centre = Vec2::new(v[0], v[2]);
        let extent = Vec2::new(reach(Vec2::X), reach(Vec2::Y));
        cuts.push(og_render::WaterCut {
            entity: cuts.len() as u64,
            min: centre - extent,
            max: centre + extent,
            centre,
            half: Vec2::new(reach(axis), reach(Vec2::new(-axis.y, axis.x))),
            axis,
            depth: 0.0,
            speed: 0.0,
        });
    }
    cuts
}

/// W8: the sea runs at the display's rate. Two frames of one tick drawn at
/// different interpolation alphas draw different seas (the code before drew
/// both at the tick, identically), and at either alpha the tick's waves are
/// drawn where og.physics.waterHeight puts them at that alpha's time, to
/// 2 cm, under the buoys floating there, so floaters blended between ticks
/// stay on them.
#[test]
#[ignore = "requires GPU"]
fn the_sea_moves_between_ticks_in_step_with_the_tick() {
    let water = og_render::Water {
        size: Vec2::splat(6000.0),
        segments: 256,
        wind: Vec2::new(7.0, -18.7),
        choppiness: 1.0,
        foam_width: 0.0,
        depth_fade: 30.0,
        wave_strength: 0.0,
        ..Default::default()
    };
    let sea = og_core::ocean::Ocean::new(water.wind, water.swell, water.choppiness).expect("a sea");
    let extent = water.size.max_element();
    let even = extent / og_core::ocean::grid_segments(water.segments, extent) as f32;
    let points: Vec<Vec2> = [(0.0, -20.0), (6.0, -35.0), (-9.0, -50.0), (3.0, -80.0)]
        .iter()
        .map(|&(x, z)| Vec2::new(x, z))
        .collect();
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut frames = Vec::new();
    // A buoy at each point, as a floater blended between ticks rides there: og.Buoyancy's hulls calm the FFT's chop
    // round them (water.wgsl hull_calm), whose sideways lean would carry the tick's waves off the point.
    let buoys: Vec<og_render::WaterCut> = points
        .iter()
        .enumerate()
        .map(|(i, &p)| og_render::WaterCut {
            entity: i as u64,
            min: p - Vec2::splat(0.6),
            max: p + Vec2::splat(0.6),
            centre: p,
            half: Vec2::splat(0.6),
            axis: Vec2::X,
            depth: 0.0,
            speed: 0.0,
        })
        .collect();
    for alpha in [0.1f32, 0.6] {
        let mut scene = RenderScene::new();
        scene.tick = 300;
        scene.tick_rate = 60.0;
        scene.interpolation = Some(alpha);
        scene.world.water.push(og_render::Water {
            wakes: Some(std::sync::Arc::new(og_render::WaterWakes {
                epoch: 1,
                dt: 1.0 / 60.0,
                ticks: vec![(300, buoys.clone())],
                ..Default::default()
            })),
            ..water.clone()
        });
        scene.camera = og_render::Camera {
            position: Vec3::new(0.0, 10.0, 0.0),
            target: Vec3::new(-4.0, 0.0, -60.0),
            far: 3000.0,
            ..Default::default()
        };
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        frames.push(renderer.read_offscreen().expect("capture"));
        let seconds = (299.0 + f64::from(alpha)) / 60.0;
        let drawn = renderer.sea_heights(&points).expect("the sea's heights");
        assert_eq!(drawn.len(), points.len(), "a sea was drawn");
        for (p, [_, swell, _]) in points.iter().zip(&drawn) {
            let cell = og_core::ocean::grid_cell(*p, water.size * 0.5, even);
            let tick_height = sea.height_on_grid(
                glam::DVec2::new(f64::from(p.x), f64::from(p.y)),
                seconds,
                cell,
            );
            assert!(
                (swell - tick_height).abs() < 0.02,
                "alpha {alpha} at {p}: the tick's waves drawn at {swell} m, waterHeight {tick_height} m"
            );
        }
    }
    let d = difference(&frames[0], &frames[1]);
    println!("one tick at alphas 0.1 and 0.6: mean difference {d}");
    assert!(d > 0.5, "the sea did not move between ticks ({d})");
}
