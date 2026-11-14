//! Fog of war over the lit image: what a team sees is drawn as it is, what
//! it has only explored shows through dimmed, the rest is the fog's
//! colour, and nothing outside the map is covered; on the ground from each
//! pixel's surface, in a 2D world's plane from its ray.
//!
//! Ways this could fail: the map laid mirrored or off by the frame's
//! origin, the levels' texture not uploaded (or stale after a change), the
//! pass reading the sky's depth as ground, or covering what lies outside.

use super::*;
use og_render::{FogOfWar, SightPlane, Terrain, TerrainData, TerrainLayer};
use std::sync::Arc;

const UNSEEN: u8 = 0;
const EXPLORED: u8 = 1;
const VISIBLE: u8 = 2;

fn desc() -> RenderDesc {
    RenderDesc {
        width: 256,
        height: 256,
        ..scenes::base_desc()
    }
}

/// A 32 m map of 2 m cells about the origin: seen where x < 0, explored
/// where x > 0 and the other axis is negative, unseen in the last quarter.
fn fog(plane: SightPlane) -> FogOfWar {
    let levels: Vec<u8> = (0..16 * 16)
        .map(|i| match (i % 16 < 8, i / 16 < 8) {
            (true, _) => VISIBLE,
            (false, true) => EXPLORED,
            (false, false) => UNSEEN,
        })
        .collect();
    FogOfWar {
        min: Vec2::splat(-16.0),
        cell: 2.0,
        columns: 16,
        rows: 16,
        levels: Arc::from(levels),
        plane,
        color: Vec3::new(0.5, 0.0, 0.0),
        explored: 0.4,
    }
}

fn px(img: &RgbaImage, x: i32, y: i32) -> [f32; 3] {
    let i = ((y as u32 * img.width + x as u32) * 4) as usize;
    [
        f32::from(img.pixels[i]),
        f32::from(img.pixels[i + 1]),
        f32::from(img.pixels[i + 2]),
    ]
}

/// Checks a frame under [`fog`] against the same frame without it, `scale`
/// pixels a metre about the centre, the second axis growing by `down` per
/// row.
fn check(clear: &RgbaImage, fogged: &RgbaImage, scale: f32, down: f32) {
    let at = |img: &RgbaImage, x: f32, v: f32| {
        px(
            img,
            128 + (x * scale) as i32,
            128 + (v * scale * down) as i32,
        )
    };
    let (c, f) = (at(clear, -8.0, -8.0), at(fogged, -8.0, -8.0));
    assert!(c[1] > 40.0, "the scene is drawn: {c:?}");
    assert_eq!(c, f, "what is seen is drawn as it is");
    assert_eq!(at(clear, -8.0, 8.0), at(fogged, -8.0, 8.0), "seen");
    let e = at(fogged, 8.0, -8.0);
    let c = at(clear, 8.0, -8.0);
    assert!(
        e[1] < 0.9 * c[1] && e[1] > 0.2 * c[1] && e[0] > e[1],
        "explored ground shows through the fog: {e:?} of {c:?}"
    );
    let u = at(fogged, 8.0, 8.0);
    assert!(
        u[0] > 60.0 && u[1] < 8.0 && u[2] < 8.0,
        "the unseen is the fog's colour: {u:?}"
    );
    for (x, v) in [(19.0, 8.0), (8.0, 19.0), (19.0, 19.0)] {
        assert_eq!(
            at(clear, x, v),
            at(fogged, x, v),
            "outside the map at {x}, {v}"
        );
    }
}

#[test]
#[ignore = "requires GPU"]
fn fog_of_war_covers_the_ground_a_team_does_not_see() {
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(TerrainData::flat(Vec2::splat(96.0), 97, 97));
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.4),
            ..Default::default()
        }],
        cast_shadows: false,
        fade_in: false,
        ..Terrain::new(h)
    });
    scene.atmosphere.fog = None;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.light.cast_shadows = false;
    // Straight down from 40 m: 23.1 m to each edge, +x right, +z down.
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 40.0, 0.0),
        target: Vec3::ZERO,
        up: Vec3::NEG_Z,
        near: 0.5,
        far: 100.0,
        projection: Projection::Perspective {
            fov_y_degrees: 60.0,
        },
        ..Default::default()
    };
    let mut r = Renderer::new_blocking(&desc()).expect("renderer");
    let mut draw = |scene: &RenderScene| {
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    let clear = draw(&scene);
    scene.look.fog_of_war = Some(fog(SightPlane::Ground));
    let fogged = draw(&scene);
    let _ = gfx::save_png(&out_dir().join("fog-of-war.png"), &fogged);
    let scale = 128.0 / (40.0 * 30f32.to_radians().tan());
    check(&clear, &fogged, scale, 1.0);

    // The team walks on: the cells it now sees are drawn, at once.
    let mut moved = fog(SightPlane::Ground);
    moved.levels = Arc::from(vec![VISIBLE; 256]);
    scene.look.fog_of_war = Some(moved);
    let seen = draw(&scene);
    assert_eq!(px(&seen, 180, 180), px(&clear, 180, 180), "now seen");

    // The world's zero moved 8 m along x: the map's corner is given from
    // the new origin, and the same ground is covered.
    let mut shifted = fog(SightPlane::Ground);
    shifted.min.x -= 8.0;
    scene.look.fog_of_war = Some(shifted);
    scene.frame.camera.position.x -= 8.0;
    scene.frame.camera.target.x -= 8.0;
    scene.frame.origin.x += 8.0;
    let again = draw(&scene);
    check(&clear, &again, scale, 1.0);
}

#[test]
#[ignore = "requires GPU"]
fn fog_of_war_covers_a_2d_worlds_plane() {
    let mut scene = RenderScene::new();
    scene.lighting.environment.background = Background::Color(Vec3::splat(0.4));
    scene.atmosphere.fog = None;
    // 48 units high: 5.33 px a unit, +y up.
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 48.0);
    let mut r = Renderer::new_blocking(&desc()).expect("renderer");
    let mut draw = |scene: &RenderScene| {
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    let clear = draw(&scene);
    scene.look.fog_of_war = Some(fog(SightPlane::Flat));
    let fogged = draw(&scene);
    let _ = gfx::save_png(&out_dir().join("fog-of-war-2d.png"), &fogged);
    check(&clear, &fogged, 256.0 / 48.0, -1.0);
}
