//! Tracks in what lies on the ground: an og.GrassPusher walking through
//! lying snow or sand packs it darker and leaves a track whose sides face
//! in, which fills in again; bare ground, and a pusher passing above the
//! cover, keep none.
//!
//! Ways this could fail: the terrain never reads the trample map (the map
//! not stepped without vegetation, its grid not written, the wrong cells),
//! the sides lean the wrong way (lit from the far side), the track never
//! fills, or the map presses ground nothing lies on.

use super::*;
use og_render::{GrassPusher, SurfaceWeather};

/// Flat grey ground seen straight down from 8 m (27.7 px a metre, +x to
/// the right), the sun low in the +x, one tick a second.
fn track_scene() -> RenderScene {
    let data = TerrainData::flat(Vec2::splat(64.0), 129, 129);
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(data);
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.3),
            ..Default::default()
        }],
        cast_shadows: false,
        fade_in: false,
        ..Terrain::new(h)
    });
    scene.atmosphere.fog = None;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::new(1.0, 0.5, 0.0).normalize(),
        intensity: 1.5,
        cast_shadows: false,
        ..scene.lighting.light
    };
    scene.frame.tick_rate = 1.0;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 8.0, 0.0),
        target: Vec3::ZERO,
        up: Vec3::NEG_Z,
        near: 0.1,
        far: 50.0,
        projection: Projection::Perspective {
            fov_y_degrees: 60.0,
        },
        ..Default::default()
    };
    scene
}

fn snow() -> Option<SurfaceWeather> {
    Some(SurfaceWeather {
        snow: 1.0,
        snow_depth: 0.1,
        shelter: 0.0,
        ..Default::default()
    })
}

/// A pusher 1.5 m across its reach, standing at the frame's centre.
fn pusher(y: f32) -> Vec<GrassPusher> {
    vec![GrassPusher {
        position: Vec3::new(0.0, y, 0.0),
        radius: 1.5,
        height: 1.8,
        strength: 1.0,
    }]
}

/// The mean luma of the 9 px square at (`dx`, `dz`) pixels from the centre.
fn at(img: &RgbaImage, dx: i32, dz: i32) -> f32 {
    let (cx, cy) = (img.width as i32 / 2 + dx, img.height as i32 / 2 + dz);
    mean(
        img,
        (cx - 4) as u32,
        (cy - 4) as u32,
        (cx + 5) as u32,
        (cy + 5) as u32,
    )
}

/// The track's floor, its side towards the sun's far side (-x, facing the
/// sun), its side under the sun (+x, facing away), and untrodden snow.
fn read(img: &RgbaImage) -> [f32; 4] {
    [
        at(img, 0, 0),
        at(img, -28, 0),
        at(img, 28, 0),
        at(img, 90, 90),
    ]
}

#[test]
#[ignore = "requires GPU"]
fn lying_snow_keeps_a_track_that_fills_in_again() {
    let desc = og_render::RenderDesc {
        width: 256,
        height: 256,
        ..scenes::base_desc()
    };
    let mut r = Renderer::new_blocking(&desc).expect("renderer");
    let mut scene = track_scene();
    // Every pipeline made first: a feature turning on (the weather) is
    // otherwise not drawn until its full variant has compiled.
    r.render(&scene, &RenderTarget::Offscreen).expect("render");
    while r.warm() {}
    let mut draw = |scene: &mut RenderScene, tick: u64| {
        scene.frame.tick = tick;
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };

    // Bare ground keeps nothing.
    let bare = draw(&mut scene, 0);
    scene.atmosphere.grass_pushers = pusher(0.0);
    let mut bare_pressed = draw(&mut scene, 1);
    for tick in 2..=4 {
        bare_pressed = draw(&mut scene, tick);
    }
    let (b, bp) = (read(&bare), read(&bare_pressed));
    println!("bare {b:?}, pressed {bp:?}");
    for k in 0..4 {
        assert!(
            (b[k] - bp[k]).abs() < 1.5,
            "bare ground changed: {b:?} to {bp:?}"
        );
    }

    // Snow: untrodden, then a pusher above it, then one standing in it.
    scene.atmosphere.grass_pushers.clear();
    scene.atmosphere.surface_weather = snow();
    let clean = draw(&mut scene, 20);
    keep("track-clean", &clean);
    let c = read(&clean);
    scene.atmosphere.grass_pushers = pusher(1.0);
    let mut above = draw(&mut scene, 21);
    for tick in 22..=24 {
        above = draw(&mut scene, tick);
    }
    let a = read(&above);
    println!("clean {c:?}, passed above {a:?}");
    for k in 0..4 {
        assert!(
            (c[k] - a[k]).abs() < 1.5,
            "a pusher above the snow marked it: {c:?} to {a:?}"
        );
    }
    scene.atmosphere.grass_pushers = pusher(0.0);
    let mut pressed = draw(&mut scene, 25);
    for tick in 26..=28 {
        pressed = draw(&mut scene, tick);
    }
    keep("track-pressed", &pressed);
    let p = read(&pressed);
    println!("pressed {p:?}");
    assert!(c[0] > 60.0, "the snow is drawn: {c:?}");
    assert!(
        p[0] < c[0] - 8.0,
        "the floor is packed darker: {} of {}",
        p[0],
        c[0]
    );
    assert!(
        p[1] > 1.1 * p[2],
        "the side facing the sun is the brighter: {} against {}",
        p[1],
        p[2]
    );
    assert!(
        (p[3] - c[3]).abs() < 1.5,
        "untrodden snow changed: {} to {}",
        c[3],
        p[3]
    );

    // It fills in: half way after 120 s, gone after 240 s.
    scene.atmosphere.grass_pushers.clear();
    let mut later = pressed.clone();
    let mut half = None;
    for step in 1..=32 {
        later = draw(&mut scene, 28 + 8 * step);
        if step == 15 {
            half = Some(read(&later));
        }
    }
    keep("track-filled", &later);
    let (h, f) = (half.expect("drawn"), read(&later));
    println!("half filled {h:?}, filled {f:?}");
    let left = (c[0] - h[0]) / (c[0] - p[0]);
    assert!(
        (0.2..0.85).contains(&left),
        "after 120 s {left} of the track is left"
    );
    for k in 0..4 {
        assert!(
            (c[k] - f[k]).abs() < 1.5,
            "after 256 s the track is still there: {c:?}, {f:?}"
        );
    }

    // Lying sand keeps one too.
    scene.atmosphere.surface_weather = Some(SurfaceWeather {
        dust: 1.0,
        shelter: 0.0,
        ..Default::default()
    });
    let sand = read(&draw(&mut scene, 300));
    scene.atmosphere.grass_pushers = pusher(0.0);
    let mut trodden = draw(&mut scene, 301);
    for tick in 302..=304 {
        trodden = draw(&mut scene, tick);
    }
    keep("track-sand", &trodden);
    let t = read(&trodden);
    println!("sand {sand:?}, trodden {t:?}");
    assert!(
        t[0] < sand[0] - 8.0,
        "the sand is packed darker: {} of {}",
        t[0],
        sand[0]
    );
}
