//! Tests of the 2D batch.

use super::*;
use crate::SpriteShadow;
use glam::Vec3;

/// An eye at the origin with no frustum, sorting by distance.
pub(super) const NO_EYE: SpriteEye<'static> = SpriteEye {
    position: Vec3::ZERO,
    forward: None,
    frustum: None,
};

#[test]
pub(super) fn faced_sprites_sort_back_to_front_cull_to_the_frustum_and_cast_sun_facing_shadows() {
    let mut scene = RenderScene::new();
    scene.lighting.light.direction_to_light = Vec3::new(1.0, 1.0, 0.0);
    let at = |z: f32, layer: i32| Sprite {
        position: Vec3::new(0.0, 0.0, z),
        layer,
        ..Default::default()
    };
    // Submitted near to far, a flat one between, one far out to the side.
    scene.sprite.sprites = vec![
        at(-2.0, 0),
        at(-6.0, 0),
        at(-4.0, 0),
        at(-9.0, 0),
        at(-3.0, 0),
    ];
    scene.sprite.sprites[3].position.x = 500.0;
    scene.sprite.sprite_facing = vec![
        SpriteFacing::CameraY,
        SpriteFacing::Camera,
        SpriteFacing::None,
        SpriteFacing::Camera,
        SpriteFacing::Camera,
    ];
    // Its own key wins over the depth.
    scene.sprite.sprite_order = vec![crate::SpriteOrder {
        sprite: 4,
        key: 5.0,
    }];
    scene.sprite.sprite_shadows = vec![crate::SpriteShadow {
        sprite: 0,
        opacity: 1.0,
        ..Default::default()
    }];
    let camera = crate::frame::camera_frame(&crate::Camera::default(), 16.0 / 9.0, Vec2::ZERO);
    let frustum = crate::frame::Frustum::from_view_proj(camera.view_proj);
    let eye = SpriteEye {
        position: crate::Camera::default().position,
        forward: Some(camera.forward),
        frustum: Some(&frustum),
    };
    let f = build(
        &scene,
        |_| None,
        0,
        |_| None,
        &[],
        (&[], &mut GpuSprites::default()),
        |_, _| 0.0,
        None,
        &[],
        1,
        &eye,
    )
    .expect("a frame");
    assert_eq!(
        (f.sprites, f.culled),
        (4, 1),
        "the one off to the side is culled"
    );
    // In the layer order the key-0 flat sprite, then the keyed
    // billboard (its key keeps it there); the others sort with the
    // transparent meshes: their shadow, then they far to near, each
    // with its view depth.
    let zs = |runs: &mut dyn Iterator<Item = &SpriteRun>| -> Vec<(f32, u32)> {
        runs.flat_map(|r| &f.instances[r.first as usize..(r.first + r.count) as usize])
            .map(|i| (i.position_rotation[2], i.facing))
            .collect()
    };
    assert_eq!(
        zs(&mut f.runs.iter()),
        [(-4.0, FACING_NONE), (-3.0, FACING_CAMERA)]
    );
    assert_eq!(zs(&mut f.billboard_shadows.iter()), [(-2.0, FACING_SUN)]);
    assert_eq!(
        zs(&mut f.billboards.iter().map(|b| &b.1)),
        [(-6.0, FACING_CAMERA), (-2.0, FACING_CAMERA_Y)]
    );
    let depth = |z: f32| (Vec3::new(0.0, 0.0, z) - eye.position).dot(camera.forward);
    let depths: Vec<f32> = f.billboards.iter().map(|b| b.0).collect();
    assert_eq!(depths, [depth(-6.0), depth(-2.0)], "view depths, far first");
    // By distance without a forward (a probe's faces), still far first.
    let f = build(
        &scene,
        |_| None,
        0,
        |_| None,
        &[],
        (&[], &mut GpuSprites::default()),
        |_, _| 0.0,
        None,
        &[],
        1,
        &NO_EYE,
    )
    .expect("a frame");
    assert_eq!(f.culled, 0, "no frustum, nothing culled");
    let first = f.billboards[0].1.first as usize;
    assert_eq!(f.instances[first].position_rotation[0], 500.0);
    // The sun-facing card: the sun as a camera's right.
    assert!((sun_card_right(Vec3::new(1.0, 1.0, 0.0)) - Vec3::NEG_Z).length() < 1e-6);
    assert_eq!(sun_card_right(Vec3::Y), Vec3::X, "overhead: +x");
    // A camera looking straight down keeps the upright card's x level.
    let (x, y) = face_axes(SpriteFacing::CameraY, Vec3::new(0.6, 0.8, 0.0), Vec3::Z);
    assert!((x - Vec3::X).length() < 1e-6 && y == Vec3::Y);
}

#[test]
pub(super) fn a_sprite_anchored_elsewhere_keeps_its_corners() {
    // A nine-slice piece off the panel's anchor, turned with the panel.
    let piece = Sprite {
        position: Vec3::new(2.0, 1.0, -3.0),
        size: Vec2::new(0.5, 2.0),
        anchor: Vec2::new(0.0, 1.0),
        rotation: 0.7,
        ..Default::default()
    };
    let corners = |s: &Sprite| {
        let (sin, cos) = s.rotation.sin_cos();
        [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(u, v)| {
            let l = (Vec2::new(u, v) - s.anchor) * s.size;
            s.position.truncate() + Vec2::new(l.x * cos - l.y * sin, l.x * sin + l.y * cos)
        })
    };
    let moved = piece.anchored_at(Vec3::new(-1.0, 4.0, 9.0));
    assert_eq!(moved.position, Vec3::new(-1.0, 4.0, -3.0), "its z is kept");
    for (a, b) in corners(&piece).iter().zip(corners(&moved)) {
        assert!((*a - b).length() < 1e-5, "{a} moved to {b}");
    }
}

pub(super) fn frame(scene: &RenderScene) -> SpriteFrame {
    build(
        scene,
        |_| None,
        0,
        |_| None,
        &[],
        (&[], &mut GpuSprites::default()),
        |_, _| 0.0,
        None,
        &[],
        1,
        &NO_EYE,
    )
    .expect("a frame")
}

#[test]
pub(super) fn a_drop_shadow_draws_first_in_its_layer_projected_along_the_sun() {
    let mut scene = RenderScene::new();
    scene.lighting.light.direction_to_light = Vec3::new(1.0, 2.0, -0.5);
    let sprite = |x: f32, layer: i32| Sprite {
        position: Vec3::new(x, 0.0, 0.0),
        size: glam::Vec2::new(1.0, 2.0),
        anchor: glam::Vec2::new(0.5, 0.0),
        color: [1.0, 1.0, 1.0, 0.5],
        layer,
        ..Default::default()
    };
    scene.sprite.sprites = vec![sprite(0.0, 1), sprite(2.0, 1), sprite(4.0, 0)];
    scene.sprite.sprite_shadows = vec![SpriteShadow {
        sprite: 1,
        opacity: 0.8,
        ..Default::default()
    }];
    let f = frame(&scene);
    assert_eq!(f.sprites, 3, "a shadow is not a sprite of the frame");
    // Layer 0's sprite, then layer 1: the shadow before both sprites.
    let xs: Vec<f32> = f.instances.iter().map(|i| i.position_rotation[0]).collect();
    assert_eq!(xs, [4.0, 2.0, 0.0, 2.0]);
    let shadow = &f.instances[1];
    assert_eq!(shadow.color, [0.0, 0.0, 0.0, 0.4]);
    // Per metre of height: -x/y and -z/y of the direction to the sun.
    assert_eq!(shadow.ground, [1.0, -0.5, 0.25, crate::SPRITE_SHADOW_LIFT]);
    assert!(f.instances.iter().filter(|i| i.ground[0] > 0.0).count() == 1);
    // No sun above the horizon, no shadow; an index past the sprites is refused.
    scene.lighting.light.direction_to_light = Vec3::new(0.0, -1.0, 0.0);
    assert_eq!(frame(&scene).instances.len(), 3);
    scene.sprite.sprite_shadows[0].sprite = 3;
    assert!(
        build(
            &scene,
            |_| None,
            0,
            |_| None,
            &[],
            (&[], &mut GpuSprites::default()),
            |_, _| 0.0,
            None,
            &[],
            1,
            &NO_EYE
        )
        .is_err()
    );
}

#[test]
pub(super) fn a_drop_shadow_lies_on_its_ground_at_its_height_and_slope() {
    use crate::ShadowGround;
    let mut scene = RenderScene::new();
    scene.lighting.light.direction_to_light = Vec3::new(1.0, 2.0, -0.5);
    scene.sprite.sprites = vec![
        Sprite {
            position: Vec3::new(0.0, 2.0, 0.0),
            size: glam::Vec2::new(1.0, 2.0),
            ..Default::default()
        };
        2
    ];
    scene.sprite.sprite_shadows = vec![
        SpriteShadow {
            sprite: 0,
            opacity: 1.0,
            ground: ShadowGround::level(2.0),
        },
        SpriteShadow {
            sprite: 1,
            opacity: 1.0,
            ground: ShadowGround {
                height: 1.5,
                normal: Vec3::new(-0.5, 1.0, 0.25).normalize(),
                bend: None,
            },
        },
    ];
    let f = frame(&scene);
    let shadows: Vec<_> = f.instances.iter().filter(|i| i.ground[0] > 0.0).collect();
    assert_eq!(shadows.len(), 2);
    // A level ground at 2 m: no slope, the lift over it.
    assert_eq!(shadows[0].ground[3], 2.0 + crate::SPRITE_SHADOW_LIFT);
    assert_eq!(shadows[0].fx, [0.0, 0.0, 0.0]);
    // A slope: the rise per metre along x and z, carried in fx.yz.
    assert_eq!(shadows[1].ground[3], 1.5 + crate::SPRITE_SHADOW_LIFT);
    let [_, rx, rz] = shadows[1].fx;
    assert!(
        (rx - 0.5).abs() < 1e-5 && (rz + 0.25).abs() < 1e-5,
        "{rx} {rz}"
    );
    // A cliff is taken at about 70 degrees; a normal pointing down is level.
    let cliff = ShadowGround {
        height: 0.0,
        normal: Vec3::X,
        bend: None,
    };
    assert_eq!(cliff.slope(), [0.0, 0.0]);
    let steep = ShadowGround {
        height: 0.0,
        normal: Vec3::new(-1.0, 0.1, 0.0),
        bend: None,
    };
    assert!(
        (2.5..3.0).contains(&steep.slope()[0]),
        "{:?}",
        steep.slope()
    );
}

#[test]
pub(super) fn a_fitted_ground_follows_the_terrain_under_the_shadow() {
    use crate::ShadowGround;
    let sun = Vec3::new(0.0, 1.0, -1.0);
    let sprite = Sprite {
        position: Vec3::new(1.0, 0.0, 2.0),
        size: glam::Vec2::new(1.0, 2.0),
        ..Default::default()
    };
    // A plane rising 0.5 per metre along x and falling 0.25 along z.
    let plane = |x: f32, z: f32| 1.0 + 0.5 * x - 0.25 * z;
    let g = ShadowGround::fit(plane, &sprite, sun);
    assert!((g.height - plane(1.0, 2.0)).abs() < 1e-5, "{g:?}");
    let [rx, rz] = g.slope();
    assert!(
        (rx - 0.5).abs() < 1e-4 && (rz + 0.25).abs() < 1e-4,
        "{rx} {rz}"
    );
    // A hill under the middle of the shadow lifts the plane over it.
    let hill = |_x: f32, z: f32| if (z - 3.0).abs() < 0.5 { 0.3 } else { 0.0 };
    let g = ShadowGround::fit(hill, &sprite, sun);
    assert!((g.height - 0.3).abs() < 1e-5, "{g:?}");
    // No sun above the horizon: level at the anchor's ground.
    let g = ShadowGround::fit(plane, &sprite, Vec3::NEG_Y);
    assert_eq!(g, ShadowGround::level(plane(1.0, 2.0)));
}

#[test]
pub(super) fn a_shadow_bends_over_a_hollow_narrower_than_itself() {
    use crate::{SHADOW_BEND_COLUMNS, SHADOW_BEND_ROWS, ShadowGround};
    // The sun behind, the shadow falling 1 m along +z per metre of
    // height: a 2 m sprite's shadow runs from z 2 to z 4.
    let sun = Vec3::new(0.0, 1.0, -1.0);
    let sprite = Sprite {
        position: Vec3::new(1.0, 0.0, 2.0),
        size: glam::Vec2::new(1.0, 2.0),
        anchor: glam::Vec2::new(0.5, 0.0),
        ..Default::default()
    };
    // Level ground: the plane, no bend.
    let g = ShadowGround::fit(|_, _| 0.0, &sprite, sun);
    assert_eq!(g, ShadowGround::level(0.0));
    // A pit 0.5 m deep under the middle of the shadow, 0.6 m across.
    let pit = |_x: f32, z: f32| if (z - 3.0).abs() < 0.3 { -0.5 } else { 0.0 };
    let g = ShadowGround::fit(pit, &sprite, sun);
    let bend = g.bend.expect("a pit narrower than the shadow bends it");
    // Row 2 is the quad's middle, 1 m up: its ray enters the pit at
    // z 3 and meets the far wall 0.3 m down (z 3.3); the base and the
    // tip land on the level ground.
    for column in 0..SHADOW_BEND_COLUMNS {
        assert_eq!(bend.heights[0][column], 0.0);
        assert!((bend.heights[2][column] + 0.3).abs() < 1e-3, "{bend:?}");
        assert!(bend.heights[SHADOW_BEND_ROWS - 1][column].abs() < 1e-3);
    }
    // Drawn as eight cells of the quad, each corner at its height.
    let mut scene = RenderScene::new();
    scene.lighting.light.direction_to_light = sun;
    scene.sprite.sprites = vec![sprite];
    scene.sprite.sprite_shadows = vec![SpriteShadow {
        sprite: 0,
        opacity: 0.5,
        ground: g,
    }];
    let f = frame(&scene);
    assert_eq!(f.sprites, 1);
    let cells: Vec<_> = f.instances.iter().filter(|i| i.ground[0] > 1.5).collect();
    assert_eq!(cells.len(), 8, "4 x 2 cells");
    let lift = crate::SPRITE_SHADOW_LIFT;
    // The cell in row 1, column 0: its top corners are row 2's.
    let cell = cells
        .iter()
        .find(|c| c.flash[..2] == [0, 1])
        .expect("row 1, column 0");
    assert_eq!(cell.ground[3], bend.heights[1][0] + lift);
    assert_eq!(
        cell.fx,
        [
            bend.heights[1][1] + lift,
            bend.heights[2][1] + lift,
            bend.heights[2][0] + lift,
        ]
    );
    // A ridge is bent over too, not only a pit.
    let ridge = |_x: f32, z: f32| if (z - 3.5).abs() < 0.2 { 0.4 } else { 0.0 };
    assert!(ShadowGround::fit(ridge, &sprite, sun).bend.is_some());
}

#[test]
pub(super) fn a_sprite_mesh_is_one_run_in_its_layers_sprite_order() {
    use crate::{SpriteMesh, SpriteMeshVertex};
    let mut scene = RenderScene::new();
    let at = |x: f32, layer: i32| Sprite {
        position: Vec3::new(x, 0.0, 0.0),
        layer,
        ..Default::default()
    };
    scene.sprite.sprites = vec![at(0.0, 2), at(1.0, 0), at(2.0, 1)];
    let corner = SpriteMeshVertex {
        color: [1.0; 4],
        ..Default::default()
    };
    // Layer 1, after the layer's sprites (key 1 > their 0); two triangles.
    scene.sprite.sprite_meshes = vec![SpriteMesh {
        vertices: vec![corner; 4],
        indices: vec![0, 1, 2, 2, 3, 0],
        layer: 1,
        key: 1.0,
        ..Default::default()
    }];
    let f = frame(&scene);
    assert_eq!(f.sprites, 3, "a mesh is not a sprite");
    assert_eq!(f.instances.len(), 3, "nor an instance");
    // Layers 0 and 1's sprites share a texture and batch into one run;
    // the mesh follows them and breaks the run before layer 2's sprite.
    let kinds: Vec<_> = f.runs.iter().map(|r| r.source).collect();
    assert_eq!(kinds.len(), 3, "{kinds:?}");
    assert_eq!((f.runs[0].first, f.runs[0].count), (0, 2));
    let xs: Vec<f32> = f.instances.iter().map(|i| i.position_rotation[0]).collect();
    assert_eq!(xs, [1.0, 2.0, 0.0]);
    assert_eq!(
        kinds[1],
        RunSource::Mesh(0),
        "between layer 1's sprite and layer 2's"
    );
    assert_eq!((f.runs[1].first, f.runs[1].count), (0, 6));
    assert_eq!((f.runs[2].first, f.runs[2].count), (2, 1));
    // An index past its vertices is refused.
    scene.sprite.sprite_meshes[0].indices[5] = 9;
    assert!(
        build(
            &scene,
            |_| None,
            0,
            |_| None,
            &[],
            (&[], &mut GpuSprites::default()),
            |_, _| 0.0,
            None,
            &[],
            1,
            &NO_EYE
        )
        .is_err()
    );
}

#[test]
pub(super) fn a_low_sun_casts_at_most_five_heights() {
    let g = shadow_ground(Vec3::new(1.0, 0.01, 0.0)).unwrap();
    assert!((g[1].abs() - 5.0).abs() < 0.01, "{g:?}");
    assert_eq!(shadow_ground(Vec3::new(1.0, 0.0, 0.0)), None);
}

#[test]
pub(super) fn particles_simulated_in_compute_draw_their_range_in_layer_order_and_share_it_across_builds()
 {
    // particle_sprites.wgsl writes an instance as 31 words.
    assert_eq!(SPRITE_STRIDE, 31 * 4);
    let mut scene = RenderScene::new();
    let sprite = |x: f32, layer: i32| Sprite {
        position: Vec3::new(x, 0.0, 0.0),
        layer,
        ..Default::default()
    };
    scene.sprite.sprites = vec![sprite(0.0, 0), sprite(1.0, 2)];
    let system = |layer: i32, max: u32, glow: f32| crate::particles_gpu::SpriteSystem {
        id: 1,
        draw_entry: 0,
        layer,
        layers: 1,
        blend: SpriteBlend::Additive,
        glow,
        soft: true,
        texture: None,
        array: None,
        frames: 1,
        columns: 1,
        rows: 1,
        max,
        z: 0.0,
    };
    let systems = [system(1, 100, 0.0), system(2, 40, 1.0)];
    let mut gpu = GpuSprites::default();
    let build_once = |gpu: &mut GpuSprites| {
        build(
            &scene,
            |_| None,
            0,
            |_| None,
            &[],
            (&systems, gpu),
            |_, _| 0.0,
            None,
            &[],
            1,
            &NO_EYE,
        )
        .expect("a frame")
    };
    let f = build_once(&mut gpu);
    // Layer 0's sprite, layer 1's particles, then layer 2's sprite and particles.
    let sources: Vec<RunSource> = f.runs.iter().map(|r| r.source).collect();
    assert_eq!(
        sources,
        [
            RunSource::Sprite(SpriteBlend::Alpha, None),
            RunSource::Particles(0),
            RunSource::Sprite(SpriteBlend::Alpha, None),
            RunSource::Particles(1),
        ]
    );
    assert_eq!((f.runs[1].first, f.runs[1].count), (0, 100));
    assert_eq!((f.runs[3].first, f.runs[3].count), (100, 40));
    assert_eq!(f.sprites, 2, "a particle range is not a sprite");
    // The glowing system glows from its range; the instance buffer holds no particle.
    assert_eq!(f.glow_runs.len(), 1);
    assert_eq!(f.glow_runs[0].source, RunSource::Particles(1));
    assert_eq!(f.instances.len(), 2);
    // A second build (a view) lit alike shares the ranges; a mask without the system's layer leaves it out.
    let g = build_once(&mut gpu);
    assert_eq!(g.runs[1].source, RunSource::Particles(0));
    assert_eq!((gpu.ranges.len(), gpu.total), (2, 140));
    let masked = build(
        &scene,
        |_| None,
        0,
        |_| None,
        &[],
        (&systems, &mut gpu),
        |_, _| 0.0,
        None,
        &[],
        2,
        &NO_EYE,
    )
    .expect("a frame");
    assert!(
        !masked
            .runs
            .iter()
            .any(|r| matches!(r.source, RunSource::Particles(_)))
    );
}
