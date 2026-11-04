//! R5's water (`crates/og-render/README.md` §Water): each scene matches its
//! reference for this backend tier, and behaviour read from the frames: a
//! wind raises a sea (much more light and dark across the water than the
//! same water becalmed) that moves with the tick; a gale's steep crests
//! foam white where a round swell of the same waves does not; the low sun
//! lays a glitter path on the sea under it; a flow map that holds one
//! direction everywhere draws the stream that flow draws without one.

use glam::{Vec2, Vec3};
use og_render::{RenderScene, RenderTarget, Renderer, RgbaImage, TextureData};
use og_tests::gfx;
use og_tests::ocean_scenes::{BEACH_LINE, OCEAN_SCENES, beach_height, flow_map_scene, ocean_scene};
use og_tests::scenes;
use std::path::PathBuf;

fn check(scene: &str) {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    match gfx::check(scene, &out) {
        Ok(report) => println!("{report}"),
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

/// The luma of every pixel of a rectangle.
fn lumas(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<f32> {
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| luma(px(img, x, y)))
        .collect()
}

fn mean(l: &[f32]) -> f32 {
    l.iter().sum::<f32>() / l.len() as f32
}

/// The mean luma step between neighbouring pixels over a rectangle: the
/// fine light and dark of waves, which a mirror's smooth reflection of the
/// sky has little of.
fn texture(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 - 1 {
        for x in x0..x1 - 1 {
            let l = luma(px(img, x, y));
            sum += (luma(px(img, x + 1, y)) - l).abs() + (luma(px(img, x, y + 1)) - l).abs();
        }
    }
    sum / ((x1 - x0 - 1) * (y1 - y0 - 1)) as f32
}

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

/// `name` with its sea's wind and choppiness replaced.
fn with_sea(name: &str, wind: Option<Vec2>, choppiness: Option<f32>) -> RenderScene {
    let mut s = ocean_scene(name);
    for w in &mut s.water {
        if let Some(v) = wind {
            w.wind = v;
        }
        if let Some(c) = choppiness {
            w.choppiness = c;
        }
    }
    s
}

#[test]
fn ocean_scenes_match_their_references_and_read_as_a_sea() {
    for scene in OCEAN_SCENES {
        check(scene);
    }
    // The breeze's sea against the same water becalmed (a mirror of the
    // sky): the waves' lit and shaded faces break its light up pixel to
    // pixel. (The spread of the luma alone, 1.42x on Metal, mostly measured
    // the sky's own gradient in both.)
    let sea = frame(&ocean_scene("water-ocean"));
    let calm = frame(&with_sea("water-ocean", Some(Vec2::ZERO), None));
    let water = |img: &RgbaImage| texture(img, 0, 420, 1280, 720);
    assert!(
        water(&sea) > 3.0 * water(&calm),
        "a wind raises a sea: {} vs {}",
        water(&sea),
        water(&calm)
    );
    // Half a second later the waves have moved on.
    let mut later = ocean_scene("water-ocean");
    later.tick += 30;
    let moved = difference(&frame(&later), &sea);
    assert!(moved > 1.0, "the sea moves with the tick: {moved}");

    // The gale's crests foam white; the same waves as a round swell
    // (choppiness 0) do not. Both without their reflection, so no mirror
    // of the bright overcast counts as foam.
    let matte = |mut s: RenderScene| {
        for w in &mut s.water {
            w.reflectivity = 0.0;
        }
        frame(&s)
    };
    let gale = matte(ocean_scene("water-ocean-storm"));
    let round = matte(with_sea("water-ocean-storm", None, Some(0.0)));
    let foam = |img: &RgbaImage| {
        let band = lumas(img, 0, 380, 1280, 720);
        let m = mean(&band);
        (380..720)
            .flat_map(|y| (0..1280).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let p = px(img, x, y);
                let lo = p[0].min(p[1]).min(p[2]);
                let hi = p[0].max(p[1]).max(p[2]);
                luma(p) > 1.5 * m && lo > 0.75 * hi
            })
            .count()
    };
    assert!(
        foam(&gale) > 4 * foam(&round) + 500,
        "whitecaps on the gale's crests: {} vs {}",
        foam(&gale),
        foam(&round)
    );
    // And enough of them to see: at least 1 % of the band's pixels, a
    // tenth of the 10 % Monahan's whitecap coverage gives this 20 m/s wind
    // (the count sees only the dense, white foam). 88d021868's storm passed
    // the comparison with two or three small whitecaps in view.
    let band = 1280 * (720 - 380);
    println!(
        "the gale's whitecaps: {:.2} % of the band",
        100.0 * foam(&gale) as f32 / band as f32
    );
    assert!(
        foam(&gale) * 100 >= band,
        "the gale's whitecaps cover {} of the band's {band} pixels, under 1 %",
        foam(&gale)
    );

    // The sun's glitter path: the column of sea under the low sun (the
    // camera faces it) is much brighter than the sea to the side.
    let sunset = frame(&ocean_scene("water-ocean-sunset"));
    let path = mean(&lumas(&sunset, 600, 400, 680, 620));
    let side = mean(&lumas(&sunset, 160, 400, 240, 620));
    assert!(
        path > 1.5 * side,
        "a glitter path under the sun: {path} vs {side}"
    );
}

/// The sea under a dim sky is no brighter than that sky: its body is lit by
/// the light that reaches it and its reflection mirrors the sky as drawn
/// (not lifted by the twilight's adaptation, which only the surfaces take).
/// Before, the sea glowed a daylight teal under a dusk storm and the night.
/// Under a dim or overcast sky the band of sea just under the horizon is
/// darker than the band of sky just over it, too: a rough sea mirrors its
/// sky at a grazing angle dimmed (Bruneton et al. 2010's mean Fresnel), and
/// a ray it reflects downwards meets the next wave, never the sky. The
/// whole sea's mean let r5-fft d0a5187fd's far sea through at 0.47 under a
/// sky of 0.27, and the band's mean let cca1cb296's hot foam through, so
/// no more than 1 % of the band may outshine the sky's brightest pixel.
#[test]
fn the_sea_darkens_with_the_sky() {
    for name in ["water-ocean-dusk-storm", "water-ocean-night"] {
        check(name);
        let img = frame(&ocean_scene(name));
        let sky = mean(&lumas(&img, 0, 60, 1280, 260));
        let sea = mean(&lumas(&img, 0, 440, 1280, 720));
        assert!(sea < sky, "{name}: the sea ({sea}) under its sky ({sky})");
    }
    for name in [
        "water-ocean-dusk-storm",
        "water-ocean-night",
        "water-ocean-storm",
    ] {
        let scene = ocean_scene(name);
        let img = frame(&scene);
        let h = horizon_row(&scene, img.height);
        let sky = mean(&lumas(&img, 0, h - 24, 1280, h - 4));
        let sea = mean(&lumas(&img, 0, h + 3, 1280, h + 23));
        println!("{name}: horizon at row {h}, sky band {sky}, sea band {sea}");
        assert!(
            sea < sky,
            "{name}: the sea just under the horizon ({sea}) outshines the sky just over it ({sky})"
        );
        // Nor does any patch of it: at most 1 % of the sea's first 60 rows
        // is brighter than the brightest of the sky band (the mean let
        // cca1cb296's storm through, its foam lit by the hidden sun 1.7
        // times the brightest sky on 10.6 % of those rows' pixels).
        let sky_max = lumas(&img, 0, h - 24, 1280, h - 4)
            .into_iter()
            .fold(0.0, f32::max);
        let near = lumas(&img, 0, h + 3, 1280, h + 63);
        let hot = near.iter().filter(|&&l| l > sky_max).count() as f32 / near.len() as f32;
        println!(
            "{name}: {:.2} % of the sea's band over the sky's brightest {sky_max}",
            hot * 100.0
        );
        assert!(
            hot <= 0.01,
            "{name}: {:.2} % of the sea under the horizon outshines the brightest sky over it ({sky_max})",
            hot * 100.0
        );
    }
}

/// The image row of the sea's horizon in `scene` (`height` rows): where the
/// endless sea's far edge (the camera's far plane less 5 %, at the sea's
/// level) lies straight ahead, through the camera's vertical field of view.
fn horizon_row(scene: &RenderScene, height: u32) -> u32 {
    let cam = &scene.camera;
    let fov = match cam.projection {
        og_render::Projection::Perspective { fov_y_degrees } => fov_y_degrees.to_radians(),
        _ => panic!("a perspective camera"),
    };
    let ahead = cam.target - cam.position;
    let pitch = ahead.y.atan2(Vec2::new(ahead.x, ahead.z).length());
    let edge = (-cam.position.y).atan2(cam.far * 0.95);
    let y = (edge - pitch).tan() / (fov * 0.5).tan();
    ((0.5 - 0.5 * y) * height as f32).round() as u32
}

/// The boats sample's floaters (tests/evidence/r5-water/boats) sit in the
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
fn the_floaters_sit_in_the_drawn_sea() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/evidence/r5-water/boats/waterline.txt"
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
        scene.water.push(sea_now);
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

/// A one-pixel flow map pointing all one way (rg `* 2 - 1`).
fn uniform_map(scene: &mut RenderScene, rg: [u8; 2]) -> og_render::TextureHandle {
    scene.add_texture(TextureData {
        width: 4,
        height: 4,
        rgba: [rg[0], rg[1], 128, 255].repeat(16),
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    })
}

#[test]
fn a_flow_map_turns_the_flow_and_one_way_is_the_flow_itself() {
    check("water-flow-map");
    // The stream without its map, flowing straight down at the far field's
    // speed, differs from the mapped one: the map turns the waves round the
    // boulder.
    let mapped = frame(&ocean_scene("water-flow-map"));
    let mut plain = RenderScene::new();
    flow_map_scene(&mut plain, false);
    let straight = frame(&plain);
    assert!(
        difference(&mapped, &straight) > 0.5,
        "the map turns the flow: {}",
        difference(&mapped, &straight)
    );
    // A map pointing down the stream everywhere (r 0.5, g 0: -z) at speed
    // 0.25 draws what the flow (0, -0.25) draws, with a normal map and
    // with the travelling waves (no normal map) alike.
    for normal_map in [true, false] {
        let make = |map: bool| {
            let mut s = RenderScene::new();
            flow_map_scene(&mut s, false);
            let handle = map.then(|| uniform_map(&mut s, [128, 0]));
            let ripples = normal_map.then(|| og_tests::fork_scenes::ripple_map(&mut s));
            let w = &mut s.water[0];
            w.normal_map = ripples;
            w.flow_map = handle;
            w.flow = Vec2::new(0.0, -0.25);
            s
        };
        let one_way = frame(&make(true));
        let flow = frame(&make(false));
        let d = difference(&one_way, &flow);
        assert!(
            d < 0.6,
            "a one-way map is the flow (normal map {normal_map}): {d}"
        );
        // And it is the flow's direction that matters: the same map turned
        // across the stream (+x) draws something else.
        let across = {
            let mut s = make(false);
            let h = uniform_map(&mut s, [255, 128]);
            s.water[0].flow_map = Some(h);
            frame(&s)
        };
        assert!(
            difference(&across, &flow) > 2.0 * d + 0.3,
            "a map across the stream is not the flow down it (normal map {normal_map})"
        );
    }
}

/// The image row (of `height`) where the level surface at world z `z`
/// (x = 0, y = 0) lies, seen through `scene`'s camera.
fn row_of(scene: &RenderScene, z: f32, height: u32) -> u32 {
    let cam = &scene.camera;
    let fov = match cam.projection {
        og_render::Projection::Perspective { fov_y_degrees } => fov_y_degrees.to_radians(),
        _ => panic!("a perspective camera"),
    };
    let ahead = cam.target - cam.position;
    let pitch = ahead.y.atan2(Vec2::new(ahead.x, ahead.z).length());
    let to = Vec3::new(0.0, 0.0, z) - cam.position;
    let angle = to.y.atan2(Vec2::new(to.x, to.z).length());
    let y = (angle - pitch).tan() / (fov * 0.5).tan();
    ((0.5 - 0.5 * y) * height as f32).round() as u32
}

/// The image row (of `height`) of the point (0, `y`, `z`) through `scene`'s camera.
fn row_of_height(scene: &RenderScene, z: f32, y: f32, height: u32) -> u32 {
    let mut lifted = ocean_scene("water-shore");
    lifted.camera = scene.camera;
    lifted.camera.position.y -= y;
    lifted.camera.target.y -= y;
    row_of(&lifted, z, height)
}

/// W1: a sea feels the beach under it. Along the waterline the shore
/// trains break into surf: over a tenth of the surf zone, the 40 m of water
/// seaward of the waterline (on the 1:20 beach the trains break in some
/// 2 m of water), is white foam, where the same sea without its surf
/// (shoreFoam 0) has next to none; and no swell reaches up the dry beach
/// (the sand there is never under the drawn sea: before, the open sea's
/// metres-high crests ran through the beach); the sand the surf runs up is
/// wet, darker than the dry beach above its run-up.
#[test]
fn the_surf_breaks_along_the_waterline() {
    for name in ["water-shore", "water-shore-storm"] {
        check(name);
        let scene = ocean_scene(name);
        let img = frame(&scene);
        // The waterline at x = 0 (the cove's middle) and the surf zone 40 m out from it.
        let line = BEACH_LINE + 15.0;
        assert_eq!(beach_height(0.0, line), 0.0);
        let (top, bottom) = (
            row_of(&scene, line, img.height),
            row_of(&scene, line + 40.0, img.height),
        );
        // Open water past the outer breakers (which lie some 20 rows further out).
        let open = mean(&lumas(&img, 440, bottom + 60, 840, bottom + 100));
        let white = |img: &RgbaImage| {
            let mut n = 0;
            for y in top..bottom {
                for x in 440..840 {
                    let p = px(img, x, y);
                    let (lo, hi) = (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2]));
                    // White: bright over the open water and grey, where the sea mirroring the sky is blue.
                    if luma(p) > 1.25 * open && lo > 0.9 * hi {
                        n += 1;
                    }
                }
            }
            n as f32 / ((bottom - top) * 400) as f32
        };
        let surf = white(&img);
        let mut calm = ocean_scene(name);
        for w in &mut calm.water {
            if let Some(s) = &mut w.shore {
                s.foam = 0.0;
            }
            w.foam_width = 0.0;
        }
        let none = white(&frame(&calm));
        println!(
            "{name}: rows {top}..{bottom}, {:.1} % surf, {:.1} % without",
            surf * 100.0,
            none * 100.0
        );
        assert!(
            surf > 0.1,
            "{name}: {:.1} % of the band foams",
            surf * 100.0
        );
        // The surf wets the sand just above the waterline, darker than the
        // dry beach further up (weather_surface.wgsl shore_wetness).
        // The wet sand: the darkest row of the beach within 8 m of the waterline (at this distance a metre of it is
        // under a row, and the surf's white edge lies a row or two below it), against the dry beach 80 m up, 4 m
        // over the sea (a storm's surf runs up some half its significant height: 3 m here, Stockdon et al. 2006).
        let row = |z: f32| row_of_height(&scene, z, beach_height(0.0, z), img.height);
        let wet = (row(line - 8.0)..=row(line - 0.5))
            .map(|y| mean(&lumas(&img, 560, y, 720, y + 1)))
            .fold(f32::MAX, f32::min);
        let dry = mean(&lumas(
            &img,
            560,
            row(line - 80.0) - 2,
            720,
            row(line - 80.0) + 2,
        ));
        println!("{name}: wet sand {wet}, dry {dry}");
        assert!(
            wet < 0.85 * dry,
            "{name}: the sand by the surf ({wet}) is not darker than up the beach ({dry})"
        );
        assert!(
            none < 0.25 * surf,
            "{name}: {:.1} % foams without its surf",
            none * 100.0
        );
    }
}

/// W1: floaters in the shallows ride the sea that is drawn there. Over the
/// beach's 1:20 slope, from 0.2 m to 8 m of water, the shoaled waves and
/// shore trains the GPU draws (read back, `Renderer::sea_heights`) are the
/// tick's (og_core's `Ocean::surface_near`, what og.Buoyancy floats bodies
/// on over og.Water.seabed) to 2 cm, at several times.
#[test]
fn shallow_floaters_sit_in_the_drawn_shoaled_sea() {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut failures = Vec::new();
    for (k, extra) in [0u64, 37, 95].into_iter().enumerate() {
        let mut scene = ocean_scene("water-shore-storm");
        scene.tick += extra;
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let water = scene.water[0].clone();
        let shore = water.shore.clone().expect("a seabed");
        let sea =
            og_core::ocean::Ocean::new(water.wind, water.swell, water.choppiness).expect("a sea");
        let coast = og_core::ocean::Coast {
            shore: &shore.field,
            trains: shore.trains,
            zero: glam::DVec2::ZERO,
        };
        let extent = water.size.max_element();
        let even = extent / og_core::ocean::grid_segments(water.segments, extent) as f32;
        let line = BEACH_LINE + 15.0;
        let points: Vec<Vec2> = [0.2f32, 0.5, 1.0, 2.0, 4.0, 8.0]
            .iter()
            .map(|d| Vec2::new(3.0 * k as f32, line + 20.0 * d))
            .collect();
        let drawn = renderer.sea_heights(&points).expect("the sea's heights");
        let seconds = scene.tick as f64 / 60.0;
        for (p, [_, swell, _]) in points.iter().zip(&drawn) {
            let cell = og_core::ocean::grid_cell(*p, water.size * 0.5, even);
            let at = glam::DVec2::new(f64::from(p.x), f64::from(p.y));
            let tick = sea.surface_near(at, seconds, cell, Some(&coast)).height;
            println!("{seconds} s at {p}: drawn {swell:+.3} m, the tick's {tick:+.3} m");
            if (swell - tick).abs() >= 0.02 {
                failures.push(format!(
                    "{seconds} s at {p}: drawn {swell} m, the tick's {tick} m"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// W8: the sea runs at the display's rate. Two frames of one tick drawn at
/// different interpolation alphas draw different seas (the code before drew
/// both at the tick, identically), and at either alpha the tick's waves are
/// drawn where og.physics.waterHeight puts them at that alpha's time, to
/// 2 cm, under the buoys floating there, so floaters blended between ticks
/// stay on them.
#[test]
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
        scene.water.push(og_render::Water {
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

/// How much more `d` (a luma difference sampled at world points of the
/// sea, by `at`) holds together along `dir` than across it: its correlation
/// at `lag` metres along the unit `dir` over that across it, over the
/// points `ps` where both neighbours are in the frame. Streaks along `dir`
/// give well above 1, round blotches about 1.
fn streakiness(ps: &[Vec2], dir: Vec2, lag: f32, at: &dyn Fn(Vec2) -> Option<f32>) -> f32 {
    let across = Vec2::new(-dir.y, dir.x);
    let corr = |off: Vec2| {
        let (mut sxy, mut sxx, mut syy) = (0.0f64, 0.0f64, 0.0f64);
        for &p in ps {
            let (Some(a), Some(b)) = (at(p), at(p + off)) else {
                continue;
            };
            sxy += f64::from(a * b);
            sxx += f64::from(a * a);
            syy += f64::from(b * b);
        }
        (sxy / (sxx * syy).sqrt().max(1e-12)) as f32
    };
    corr(dir * lag) / corr(across * lag).max(1e-3)
}

/// W4: a gale's old foam streaks along the wind. The foam's lace, gathered
/// by the windrows and stretched with age, is what the near sea gains over
/// the same sea with foamStreaks 0, and what it gains holds together along
/// the wind far more than across it (on the sea itself, at 3 m), as
/// Beaufort's force 8 has it: foam blown in well-marked streaks along the
/// wind (the code before drew round, ageless blotches whatever its setting:
/// it had none). And the spray its crests throw changes the frame (spray 0
/// against 1), none of it under a breeze too light to carry spray.
#[test]
fn a_gales_foam_streaks_along_the_wind_and_its_crests_throw_spray() {
    let frame_of = |streaks: f32, spray: f32| {
        let mut s = ocean_scene("water-ocean-storm");
        for w in &mut s.water {
            w.foam_streaks = streaks;
            w.spray = spray;
        }
        frame(&s)
    };
    let streaked = frame_of(1.0, 0.0);
    let plain = frame_of(0.0, 0.0);
    let sprayed = frame_of(1.0, 1.0);
    let scene = ocean_scene("water-ocean-storm");
    let (w, h) = (streaked.width, streaked.height);
    let gained = |p: Vec2| {
        let (x, y) = pixel_of(&scene, Vec3::new(p.x, 0.0, p.y), w, h)?;
        Some(luma(px(&streaked, x, y)) - luma(px(&plain, x, y)))
    };
    // The near sea, 30 to 90 m out, a point every metre.
    let ps: Vec<Vec2> = (-30..=30)
        .flat_map(|x| (30..=90).map(move |z| Vec2::new(x as f32, -(z as f32))))
        .collect();
    let gain = ps
        .iter()
        .filter_map(|&p| gained(p))
        .map(f32::abs)
        .sum::<f32>()
        / ps.len() as f32;
    let wind = scene.water[0].wind.normalize();
    let along = streakiness(&ps, wind, 3.0, &gained);
    let d_spray = difference(&streaked, &sprayed);
    println!(
        "gale: the streaks add {gain} of luma, {along} times as coherent along the wind; spray changes the frame by {d_spray}"
    );
    // (The code before: 0.13, its lace's cut all but never reached; spray 0.)
    assert!(gain > 0.25, "the gale's foam has no streaks ({gain})");
    assert!(
        along > 1.5,
        "the foam does not streak along the wind ({along})"
    );
    assert!(d_spray > 0.005, "no spray ({d_spray})");
    // A breeze (9 m/s), under spray's threshold, throws none.
    let breeze = |spray: f32| {
        let mut s = ocean_scene("water-ocean");
        for w in &mut s.water {
            w.spray = spray;
        }
        frame(&s)
    };
    let d_breeze = difference(&breeze(0.0), &breeze(1.0));
    assert!(
        d_breeze < 0.1 * d_spray,
        "a breeze threw spray ({d_breeze} against the gale's {d_spray})"
    );
}

/// W2: clear water's colour is its optics over the light's path. Across
/// water-shallows' white slope (0 m deep a sixth in from the left to 20 m a
/// sixth from the right),
/// the seabed's blue over green rises band by band as the green is absorbed
/// faster than the blue on the longer path, up to the bluest band; deeper
/// than that the seabed is gone and the body's own colour, its scattering's,
/// holds it within 5 % (the seabed's light, which the depth turns ever
/// bluer, can pass it on the way); and its red falls away first:
/// the shallowest band keeps more of its red against green than the deepest.
#[test]
fn clear_water_turns_from_turquoise_to_blue_with_depth() {
    let img = frame(&ocean_scene("water-shallows"));
    let (w, h) = (img.width, img.height);
    let bands = 6u32;
    let mut ratios = Vec::new();
    let mut reds = Vec::new();
    for b in 0..bands {
        let x0 = w / 6 + b * (w * 2 / 3) / bands;
        let x1 = x0 + (w * 2 / 3) / bands;
        // The water's body: the darker half of the band's pixels (the sun's glitter on the waves is the brighter).
        let mut band: Vec<[f32; 3]> = (h * 2 / 5..h * 3 / 5)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .map(|(x, y)| px(&img, x, y))
            .collect();
        band.sort_by(|a, b| luma(*a).total_cmp(&luma(*b)));
        let (mut r, mut g, mut bl) = (0.0f32, 0.0f32, 0.0f32);
        for p in &band[..band.len() / 2] {
            r += p[0];
            g += p[1];
            bl += p[2];
        }
        ratios.push(bl / g.max(1.0));
        reds.push(r / g.max(1.0));
    }
    println!("shallows: blue/green {ratios:?}, red/green {reds:?}");
    let peak = ratios
        .iter()
        .copied()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("bands")
        .0;
    assert!(
        ratios[..=peak].windows(2).all(|p| p[1] > p[0])
            && ratios[peak..].iter().all(|&r| r > 0.95 * ratios[peak]),
        "blue over green does not rise with depth: {ratios:?}"
    );
    assert!(
        ratios[bands as usize - 1] > 1.25 * ratios[0],
        "the deep water is no bluer than the shallows: {ratios:?}"
    );
    assert!(
        reds[0] > reds[bands as usize - 1],
        "red does not fall with depth: {reds:?}"
    );
}

/// W2: a sea's caustics are its waves'. On water-shallows' slope the
/// caustics (og.Water.caustics) add light and dark to the shallow seabed
/// that the same sea without them lacks, and that pattern moves with the
/// waves: half a second later it has changed far more than the seabed
/// without caustics has (the code before drew cells unrelated to the sea).
#[test]
fn a_seas_caustics_are_its_waves_and_move_with_them() {
    let shot = |caustics: f32, tick: u64| {
        let mut s = ocean_scene("water-shallows");
        s.tick = tick;
        for w in &mut s.water {
            w.caustics = caustics;
        }
        frame(&s)
    };
    let t0 = 5400;
    let (on, off) = (shot(1.0, t0), shot(0.0, t0));
    let (w, h) = (on.width, on.height);
    // The shallows (2 to 7 m deep, inside the caustics' grid about the camera), the frame's middle rows.
    let (x0, x1, y0, y1) = (w / 4, w * 2 / 5, h * 2 / 5, h * 3 / 5);
    let (t_on, t_off) = (texture(&on, x0, y0, x1, y1), texture(&off, x0, y0, x1, y1));
    let later_on = shot(1.0, t0 + 30);
    let later_off = shot(0.0, t0 + 30);
    let crop = |img: &RgbaImage| lumas(img, x0, y0, x1, y1);
    let moved = |a: &RgbaImage, b: &RgbaImage| {
        crop(a)
            .iter()
            .zip(crop(b))
            .map(|(p, q)| (p - q).abs())
            .sum::<f32>()
            / crop(a).len() as f32
    };
    let (m_on, m_off) = (moved(&on, &later_on), moved(&off, &later_off));
    println!(
        "shallows: texture {t_on} with caustics, {t_off} without; moved {m_on} with, {m_off} without"
    );
    assert!(
        t_on > 1.5 * t_off,
        "no caustics on the seabed ({t_on} against {t_off})"
    );
    assert!(
        m_on > 2.0 * m_off,
        "the caustics do not move with the waves ({m_on} against {m_off})"
    );
}

/// `water-wake` run to its last frame with its sea's dynamicWaves at `scale`.
fn wake_run(scale: f32) -> (RenderScene, RgbaImage) {
    use og_tests::ocean_scenes::{WAKE_FRAMES, wake_update};
    let mut scene = ocean_scene("water-wake");
    scene.water[0].dynamic_waves = scale;
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    for frame in 0..WAKE_FRAMES {
        wake_update(&mut scene, frame);
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (scene, renderer.read_offscreen().expect("capture"))
}

/// The pixel (x, y) of world point `p` through `scene`'s camera in an image of `w` x `h`.
fn pixel_of(scene: &RenderScene, p: Vec3, w: u32, h: u32) -> Option<(u32, u32)> {
    let cam = &scene.camera;
    let fov = match cam.projection {
        og_render::Projection::Perspective { fov_y_degrees } => fov_y_degrees.to_radians(),
        _ => panic!("a perspective camera"),
    };
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let proj =
        glam::camera::rh::proj::directx::perspective(fov, w as f32 / h as f32, cam.near, cam.far);
    let c = proj * view * p.extend(1.0);
    if c.w <= 0.0 {
        return None;
    }
    let (x, y) = (c.x / c.w, c.y / c.w);
    let (px, py) = ((0.5 + 0.5 * x) * w as f32, (0.5 - 0.5 * y) * h as f32);
    (px >= 3.0 && py >= 3.0 && px < w as f32 - 3.0 && py < h as f32 - 3.0)
        .then_some((px as u32, py as u32))
}

/// W3: a boat running at 5 m/s for 5 s over a light breeze's sea leaves a
/// Kelvin wake: the frame with the hulls' dynamic waves differs from the
/// same run without them (dynamicWaves 0, the sea as before) along the two
/// arms of the wedge behind it, at 19.5° either side of its track, more
/// than three times as much as ahead of its bow and twice as much as
/// outside the wedge at 45° (where deep water's dispersion leaves no
/// waves; a plain wave equation's cone at this speed spreads to some 60°).
#[test]
fn a_moving_hull_leaves_a_kelvin_wake() {
    use og_tests::ocean_scenes::{WAKE_FRAMES, wake_boat};
    let (scene, on) = wake_run(1.0);
    let (_, off) = wake_run(0.0);
    let (w, h) = (on.width, on.height);
    let bow = wake_boat(WAKE_FRAMES - 1) - Vec2::new(0.0, 3.0);
    let mean_diff = |points: &[Vec2]| {
        let mut sum = 0.0;
        let mut n = 0;
        for p in points {
            let Some((x, y)) = pixel_of(&scene, Vec3::new(p.x, 0.0, p.y), w, h) else {
                continue;
            };
            let d = lumas(&on, x - 2, y - 2, x + 3, y + 3)
                .iter()
                .zip(lumas(&off, x - 2, y - 2, x + 3, y + 3))
                .map(|(a, b)| (a - b).abs())
                .sum::<f32>()
                / 25.0;
            sum += d;
            n += 1;
        }
        assert!(n > 4, "too few points in the frame: {n}");
        sum / n as f32
    };
    let along = |degrees: f32, behind: bool| -> Vec<Vec2> {
        let t = degrees.to_radians().tan();
        (0..12)
            .flat_map(|k| {
                let d = 6.0 + 1.5 * k as f32;
                let z = if behind { bow.y + d } else { bow.y - d };
                [Vec2::new(d * t, z), Vec2::new(-d * t, z)]
            })
            .collect()
    };
    let arms = mean_diff(&along(19.5, true));
    let outside = mean_diff(&along(45.0, true));
    let ahead = mean_diff(&along(19.5, false));
    println!("wake: {arms} along the arms, {outside} outside the wedge, {ahead} ahead of the bow");
    assert!(arms > 0.02, "no wake drawn ({arms})");
    assert!(
        arms > 3.0 * ahead,
        "the wake is not behind the boat ({arms} against {ahead} ahead)"
    );
    assert!(
        arms > 2.0 * outside,
        "the wake is not a Kelvin wedge ({arms} against {outside} outside)"
    );
}

/// W5: `water-river`'s river falls off its terrace white: the sheet
/// where the course drops 4 m is brighter than the river's glassy reach
/// above it (seen at a grazing 15°, a mirror of the bright sky) and far
/// brighter than its body some 15 m on down the plain (seen steeply, its own
/// colour), and its pool churns white just below the fall, more than there; its current moves the surface between two
/// frames half a second apart. The code before drew no river at all (the
/// og.Water's grid, flat across the valley).
#[test]
fn a_river_falls_white_off_its_terrace_and_churns_below() {
    use og_tests::ocean_scenes::RIVER_COURSE;
    let scene = ocean_scene("water-river");
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let img = renderer.read_offscreen().expect("capture");
    let mut later = ocean_scene("water-river");
    later.tick += 30;
    renderer
        .render(&later, &RenderTarget::Offscreen)
        .expect("render");
    let moved = renderer.read_offscreen().expect("capture");
    let (w, h) = (img.width, img.height);
    let luma_at = |img: &RgbaImage, p: Vec3| {
        let (x, y) = pixel_of(&scene, p, w, h).expect("in the frame");
        mean(&lumas(img, x - 2, y - 2, x + 3, y + 3))
    };
    let lerp = |a: usize, t: f32| RIVER_COURSE[a].lerp(RIVER_COURSE[a + 1], t);
    let fall = luma_at(&img, lerp(2, 0.5));
    let above = luma_at(&img, lerp(1, 0.5));
    let pool = luma_at(&img, lerp(3, 0.15));
    let downstream = luma_at(&img, lerp(3, 0.8));
    let flowing = difference(&img, &moved);
    println!(
        "river: fall {fall}, above it {above}, its pool {pool}, downstream {downstream}; moved {flowing}"
    );
    assert!(
        fall > above && fall > 1.4 * downstream,
        "the fall is not white water ({fall} against {above} above it, {downstream} downstream)"
    );
    assert!(
        pool > 1.2 * downstream,
        "the pool does not churn ({pool} against {downstream} downstream)"
    );
    assert!(flowing > 0.2, "the river does not flow ({flowing})");
}

/// W4: the first frame of a sea five minutes into a session warms its foam
/// up over the lace's three τ (some 170 steps) and still submits the
/// water's work ahead of the frame in at most 4 command buffers; the next
/// tick's frame steps only the one tick. The code before submitted once per
/// warm-up step, some 140 submits on that frame.
#[test]
fn the_foam_warms_up_in_a_few_submits() {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut stats = Vec::new();
    for tick in [18_000u64, 18_001] {
        let mut scene = with_sea("water-ocean", None, None);
        scene.tick = tick;
        scene.tick_rate = 60.0;
        stats.push(
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render"),
        );
    }
    let (first, next) = (stats[0], stats[1]);
    println!(
        "foam warm-up: {} steps in {} submits, then {} steps in {}",
        first.foam_steps, first.water_submits, next.foam_steps, next.water_submits
    );
    assert!(
        first.foam_steps >= 150,
        "the foam was not warmed up ({} steps)",
        first.foam_steps
    );
    assert!(
        first.water_submits <= 4,
        "the warm-up took {} submits",
        first.water_submits
    );
    assert!(
        next.foam_steps <= 2 && next.water_submits <= 4,
        "the next tick: {next:?}"
    );
}

/// W6: a camera bobbing at the surface of a swell draws the waterline
/// where its near plane meets the water: the frame's rows change from the
/// air above to the water's fog below within 3 px of the row where the near
/// plane's point lies on the drawn surface (the tick's waves and the chop
/// over them), the eye 2 cm above it (some 120 px under the horizon). The
/// meniscus (2 px, the default) then lays a dark band along that row,
/// darker than the water's fog below. (Located without it: the band, darker
/// than both sides, moves where the rows' mean crosses.) The code before drew
/// the whole frame from the air, its one change of light the horizon.
#[test]
fn the_waterline_splits_the_frame_where_the_near_plane_meets_the_sea() {
    let water = og_render::Water {
        size: Vec2::splat(6000.0),
        segments: 256,
        wind: Vec2::new(2.0, -6.0),
        choppiness: 0.5,
        foam_width: 0.0,
        depth_fade: 30.0,
        wave_strength: 0.0,
        underwater_density: 0.25,
        meniscus: 0.0,
        ..Default::default()
    };
    let mut scene = RenderScene::new();
    scene.tick = 600;
    scene.tick_rate = 60.0;
    scene.water.push(water);
    let (near, fov) = (0.1f32, 50.0f32);
    let at = |y: f32| og_render::Camera {
        position: Vec3::new(0.0, y, 0.0),
        target: Vec3::new(0.0, y, -50.0),
        near,
        far: 3000.0,
        projection: og_render::Projection::Perspective { fov_y_degrees: fov },
        ..Default::default()
    };
    // The drawn surface's mean height along the near plane's middle row (its chop with the tick's waves, `near`
    // ahead of the eye, across the plane's width: a millimetre there is 8 rows), from a frame above it.
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    scene.camera = at(5.0);
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let across: Vec<Vec2> = (-4..=4)
        .map(|i| Vec2::new(i as f32 * 0.02, -near))
        .collect();
    let heights = renderer.sea_heights(&across).expect("heights");
    let h = heights.iter().map(|p| p[0]).sum::<f32>() / heights.len() as f32;
    let lift = 0.02;
    scene.camera = at(h + lift);
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let img = renderer.read_offscreen().expect("capture");
    let rows: Vec<f32> = (0..img.height)
        .map(|y| mean(&lumas(&img, 0, y, img.width, y + 1)))
        .collect();
    let half = img.height as f32 * 0.5;
    let expected = half + lift / (near * (fov.to_radians() * 0.5).tan()) * half;
    let above = mean(&rows[(expected as usize).saturating_sub(40)..expected as usize - 10]);
    let below = mean(&rows[expected as usize + 10..(expected as usize + 40).min(rows.len())]);
    let cut = 0.5 * (above + below);
    let seen = (expected as usize - 60..(expected as usize + 60).min(rows.len()))
        .find(|&y| (rows[y] - cut) * (above - cut) < 0.0)
        .map_or(f32::MAX, |y| y as f32);
    println!("waterline: row {seen}, expected {expected}; luma {above} above, {below} below");
    assert!(
        (above - below).abs() > 20.0,
        "no change from air to water ({above} against {below})"
    );
    assert!(
        (seen - expected).abs() <= 3.0,
        "the waterline at row {seen}, the near plane meets the sea at {expected}"
    );
    scene.water[0].meniscus = 2.0;
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let img = renderer.read_offscreen().expect("capture");
    let band = (seen as u32 - 6..seen as u32 + 7)
        .map(|y| mean(&lumas(&img, 0, y, img.width, y + 1)))
        .fold(f32::MAX, f32::min);
    println!("meniscus: darkest row {band} at the line, the fog below {below}");
    assert!(
        band < below - 10.0,
        "no meniscus along the line ({band} against {below})"
    );
}
