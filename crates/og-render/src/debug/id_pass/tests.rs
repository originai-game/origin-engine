//! Tests of the id pass.

use super::*;
use crate::scene::{Material, Sprite};
use crate::shapes;

pub(super) fn scene_with_boxes() -> (RenderScene, IdSources) {
    scene_in_order(false)
}

/// The slab and the cube, submitted cube first when `swapped`.
pub(super) fn scene_in_order(swapped: bool) -> (RenderScene, IdSources) {
    let mut scene = RenderScene::new();
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let m = scene.add_material(Material::default());
    scene.frame.camera.position = Vec3::new(0.0, 0.0, 10.0);
    scene.frame.camera.target = Vec3::ZERO;
    // A wide floor-like slab behind, a box in front of its left half.
    scene.add_instance(
        cube,
        m,
        Mat4::from_scale_rotation_translation(
            Vec3::new(6.0, 2.0, 0.2),
            glam::Quat::IDENTITY,
            Vec3::new(0.0, 0.0, -2.0),
        ),
    );
    scene.add_instance(cube, m, Mat4::from_translation(Vec3::new(-1.0, 0.0, 0.0)));
    let mut ids = IdSources {
        instances: vec![7, 9],
        ..IdSources::default()
    };
    if swapped {
        scene.objects.instances.swap(0, 1);
        ids.instances.swap(0, 1);
    }
    (scene, ids)
}

#[test]
pub(super) fn nearer_object_wins_and_hides_part_of_the_farther() {
    let (scene, ids) = scene_with_boxes();
    let f = render_ids(&scene, &ids, &|_, _, _| None, 200, 100);
    let slab = f.object(7).expect("the slab shows");
    let cube = f.object(9).expect("the cube shows");
    assert_eq!(cube.occlusion(), 0.0, "nothing is in front of the cube");
    assert!(
        slab.occlusion() > 0.05 && slab.occlusion() < 0.5,
        "the cube hides part of the slab: {}",
        slab.occlusion()
    );
    assert!(cube.depth < slab.depth, "the cube is nearer");
    // The cube is left of the centre; the slab spans both sides.
    assert!(cube.bounds[2] <= 100, "{:?}", cube.bounds);
    assert!(
        slab.bounds[0] < cube.bounds[0] && slab.bounds[2] > 120,
        "{:?}",
        slab.bounds
    );
    // The point it names shows it.
    assert_eq!(f.id_at(cube.point[0], cube.point[1]), 9);
    assert_eq!(f.id_at(slab.point[0], slab.point[1]), 7);
    // The hidden part is inside its drawn bounds, not its visible ones.
    assert_eq!(slab.drawn_bounds, slab.bounds);
    let (id, at) = f
        .at(cube.point[0] as f32 + 0.5, cube.point[1] as f32 + 0.5)
        .expect("a hit");
    assert_eq!(id, 9);
    assert!(
        (at.z - 0.5).abs() < 0.05,
        "the cube's front face is z 0.5: {at}"
    );
}

/// The cube on layer 1 alone: the main camera (layer 0) does not see
/// it, a view of layers 0 and 1 does, from its own eye (from the right),
/// and a view of layer 0 does not.
#[test]
pub(super) fn a_view_sees_what_its_layers_admit_from_its_own_eye() {
    let (mut scene, ids) = scene_with_boxes();
    scene.objects.instance_layers = vec![1, 2];
    let eye = |layers| crate::View {
        camera: Camera {
            position: Vec3::new(8.0, 0.0, 4.0),
            target: Vec3::new(-1.0, 0.0, 0.0),
            ..Camera::default()
        },
        target: crate::ViewTarget::Viewport([0.0, 0.0, 0.5, 1.0]),
        layers,
        ..Default::default()
    };
    scene.frame.views = vec![eye(3), eye(1)];
    assert!(
        render_ids(&scene, &ids, &|_, _, _| None, 200, 100)
            .object(9)
            .is_none()
    );
    let cams = view_id_cameras(&scene, (400, 200));
    assert_eq!(cams.len(), 2);
    assert_eq!(cams[0].2, (200, 200), "the viewport's pixels");
    let seen = render_ids_from(&scene, &ids, &|_, _, _| None, 100, 100, &cams[0].1);
    let cube = seen.object(9).expect("the layer-1 view sees the cube");
    let slab = seen.object(7).expect("and the slab");
    // From its eye the slab's right end is the nearest point (about
    // 7 m along its view; the main camera has the slab 11.9 m away).
    assert!(slab.depth < cube.depth, "{} {}", slab.depth, cube.depth);
    assert!(slab.depth < 9.0, "{} from the view's eye", slab.depth);
    let unseen = render_ids_from(&scene, &ids, &|_, _, _| None, 100, 100, &cams[1].1);
    assert!(unseen.object(9).is_none() && unseen.object(7).is_some());
}

#[test]
pub(super) fn deterministic_and_order_free() {
    let (scene, ids) = scene_with_boxes();
    let a = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let (swapped, ids2) = scene_in_order(true);
    let b = render_ids(&swapped, &ids2, &|_, _, _| None, 160, 90);
    assert_eq!(a.ids, b.ids);
    assert_eq!(a.objects, b.objects);
}

#[test]
pub(super) fn sprites_draw_over_by_layer_and_behind_nearer_meshes() {
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    let quad = |x: f32, layer: i32| Sprite {
        position: Vec3::new(x, 0.0, 0.0),
        size: Vec2::new(4.0, 4.0),
        anchor: Vec2::new(0.5, 0.5),
        layer,
        ..Sprite::default()
    };
    scene.add_sprite(quad(-1.0, 2));
    scene.add_sprite(quad(1.0, 1));
    let ids = IdSources {
        sprites: vec![3, 4],
        ..IdSources::default()
    };
    let f = render_ids(&scene, &ids, &|_, _, _| None, 100, 100);
    // At the middle both overlap: the higher layer (3) is on top.
    assert_eq!(f.id_at(50, 50), 3);
    assert!(f.object(4).expect("shows").occlusion() > 0.2);
    assert_eq!(f.id_at(2, 2), 0, "the corner shows nothing");
}

/// A terrain 64 m across on a half-metre grid, its heights `h(x, z)`.
pub(super) fn field(h: impl Fn(f32, f32) -> f32) -> crate::TerrainData {
    let n = 129u32;
    let mut d = crate::TerrainData::flat(Vec2::splat(64.0), n, n);
    d.heights = (0..n * n)
        .map(|k| h((k % n) as f32 * 0.5 - 32.0, (k / n) as f32 * 0.5 - 32.0))
        .collect::<Vec<_>>()
        .into();
    d
}

/// A scene drawing `data` placed by `transform`, every level of it at
/// the finest spacing (ring 256), seen from `eye` towards `target`.
pub(super) fn terrain_scene(
    data: crate::TerrainData,
    transform: Mat4,
    eye: Vec3,
    target: Vec3,
) -> (RenderScene, IdSources) {
    let mut scene = RenderScene::new();
    let handle = scene.add_terrain(data);
    let mut t = crate::Terrain::new(handle);
    t.transform = transform;
    t.lod.ring = 256;
    scene.world.terrains.push(t);
    scene.frame.camera.position = eye;
    scene.frame.camera.target = target;
    scene.frame.camera.far = 500.0;
    let ids = IdSources {
        terrains: vec![5],
        ..IdSources::default()
    };
    (scene, ids)
}

pub(super) fn hill(x: f32, z: f32) -> f32 {
    6.0 * (-(x * x + z * z) / 64.0).exp() + 0.1 * x
}

/// Every terrain pixel's world point lies on the field's surface (the
/// drawn triangles, through the terrain's placement), not on a box.
pub(super) fn on_the_surface(f: &IdFrame, data: &crate::TerrainData, transform: Mat4) -> u32 {
    let to_local = transform.inverse();
    let mut n = 0;
    for y in 0..f.height {
        for x in 0..f.width {
            let Some((5, p)) = f.at(x as f32 + 0.5, y as f32 + 0.5) else {
                continue;
            };
            let l = to_local.transform_point3(p);
            let h = data.height(Vec2::new(l.x, l.z));
            assert!(
                (l.y - h).abs() < 0.05,
                "pixel ({x}, {y}) sees {l} on the terrain, whose surface there is at {h}"
            );
            n += 1;
        }
    }
    n
}

#[test]
pub(super) fn a_terrain_is_drawn_under_its_id_on_its_surface() {
    let data = field(hill);
    let eye = Vec3::new(0.0, 14.0, 30.0);
    let (mut scene, ids) = terrain_scene(data.clone(), Mat4::IDENTITY, eye, Vec3::ZERO);
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let o = f.object(5).expect("the terrain shows");
    assert!(o.visible > 160 * 90 / 3, "{} pixels", o.visible);
    assert!(on_the_surface(&f, &data, Mat4::IDENTITY) == o.visible);
    // Hidden, it is not drawn.
    scene.world.terrains[0].visible = false;
    assert!(
        render_ids(&scene, &ids, &|_, _, _| None, 160, 90)
            .object(5)
            .is_none()
    );
}

/// Drawn finer than its samples (`lod.spacing` a quarter of a cell),
/// the ground near the eye lies on the field's split triangles, the
/// surface collision and og.terrain.sample answer, not on a bilinear
/// patch: on a checkerboard of 20 cm steps a cell's middle is 10 cm
/// off between the two.
#[test]
pub(super) fn a_terrain_drawn_finer_than_its_samples_lies_on_its_triangles() {
    let n = 129u32;
    let mut data = crate::TerrainData::flat(Vec2::splat(64.0), n, n);
    data.heights = (0..n * n)
        .map(|k| 0.2 * ((k % n + k / n) % 2) as f32)
        .collect::<Vec<_>>()
        .into();
    let eye = Vec3::new(0.3, 3.0, 2.2);
    let (mut scene, ids) =
        terrain_scene(data.clone(), Mat4::IDENTITY, eye, Vec3::new(0.3, 0.0, -1.0));
    scene.world.terrains[0].lod.spacing = 0.125;
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let mut near = 0;
    for y in 0..f.height {
        for x in 0..f.width {
            let Some((5, p)) = f.at(x as f32 + 0.5, y as f32 + 0.5) else {
                continue;
            };
            // Inside the finest level, short of its morph band.
            if (p - eye).length() > 8.0 {
                continue;
            }
            let h = data.height(Vec2::new(p.x, p.z));
            assert!(
                (p.y - h).abs() < 0.02,
                "pixel ({x}, {y}) sees {p}, the field's triangles {h}"
            );
            near += 1;
        }
    }
    assert!(near > 160 * 90 / 4, "{near} pixels near the eye");
}

#[test]
pub(super) fn a_turned_and_moved_terrain_is_drawn_where_it_stands() {
    let data = field(hill);
    let transform = Mat4::from_scale_rotation_translation(
        Vec3::new(1.5, 2.0, 1.5),
        glam::Quat::from_rotation_y(1.1),
        Vec3::new(300.0, -20.0, -140.0),
    );
    let target = transform.transform_point3(Vec3::ZERO);
    let eye = target + Vec3::new(-25.0, 30.0, 35.0);
    let (scene, ids) = terrain_scene(data.clone(), transform, eye, target);
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let n = on_the_surface(&f, &data, transform);
    assert!(n > 160 * 90 / 3, "{n} pixels");
    // The peak, turned and lifted with it, is where the frame shows it.
    let peak = transform.transform_point3(Vec3::new(0.0, hill(0.0, 0.0), 0.0));
    let c = f.view_proj * peak.extend(1.0);
    let (x, y) = (
        (c.x / c.w * 0.5 + 0.5) * 160.0,
        (0.5 - c.y / c.w * 0.5) * 90.0,
    );
    let (id, p) = f.at(x, y + 1.0).expect("the peak shows");
    assert_eq!(id, 5);
    assert!(p.distance(peak) < 2.5, "{p} by the peak {peak}");
}

#[test]
pub(super) fn a_terrain_hides_what_its_hill_stands_before_and_shows_it_through_holes() {
    // A ridge 10 m high across x at z in (-3, 3); a box behind it.
    let ridge = |_: f32, z: f32| if z.abs() < 3.0 { 10.0 } else { 0.0 };
    let mut data = field(ridge);
    let (mut scene, mut ids) = terrain_scene(
        data.clone(),
        Mat4::IDENTITY,
        Vec3::new(0.0, 3.0, 24.0),
        Vec3::new(0.0, 2.0, -10.0),
    );
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(2.0)));
    let m = scene.add_material(Material::default());
    scene.add_instance(cube, m, Mat4::from_translation(Vec3::new(0.0, 2.0, -10.0)));
    ids.instances = vec![9];
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    assert!(f.object(5).is_some());
    let behind = f.object(9).expect("the box is drawn");
    assert_eq!(behind.visible, 0, "hidden behind the ridge");
    assert!(behind.drawn > 20 && behind.occlusion() == 1.0);
    // Cut the ridge out under a window 12 m wide: the box shows through.
    let (cx, cz) = data.cells();
    let mut holes = vec![0u8; (cx * cz) as usize];
    for r in 0..cz {
        for c in 0..cx {
            let (x, z) = (c as f32 * 0.5 - 31.75, r as f32 * 0.5 - 31.75);
            holes[(r * cx + c) as usize] = u8::from(x.abs() < 6.0 && z.abs() < 4.0);
        }
    }
    data.holes = Some(holes.into());
    scene.set_terrain(scene.world.terrains[0].terrain, data.clone());
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let seen = f.object(9).expect("the box is drawn");
    assert!(seen.visible > 20, "through the hole: {}", seen.visible);
    // No terrain pixel lies over a hole.
    for y in 0..f.height {
        for x in 0..f.width {
            if let Some((5, p)) = f.at(x as f32 + 0.5, y as f32 + 0.5) {
                assert!(!data.hole(Vec2::new(p.x, p.z)), "({x}, {y}) at {p}");
            }
        }
    }
}

#[test]
pub(super) fn a_streamed_terrain_draws_as_its_field_in_memory_does() {
    let mut data = field(hill);
    let (cx, cz) = data.cells();
    // A hole in the middle, whose flag the samples carry when streamed.
    let mut holes = vec![0u8; (cx * cz) as usize];
    for r in 60..68 {
        for c in 60..68 {
            holes[(r * cx + c) as usize] = 1;
        }
    }
    data.holes = Some(holes.into());
    let eye = Vec3::new(4.0, 18.0, 26.0);
    let (mut scene, ids) = terrain_scene(data.clone(), Mat4::IDENTITY, eye, Vec3::ZERO);
    let layout = scene
        .terrain_layout(scene.world.terrains[0].terrain)
        .unwrap();
    assert!(layout.holes);
    // The tiles' samples: the height with the hole flag of the cell at
    // it in its lowest bit.
    let raw = |_: usize, c: u32, r: u32| {
        let h = data.heights[(r * 129 + c) as usize];
        let hole = data.hole(Vec2::new(c as f32 * 0.5 - 31.75, r as f32 * 0.5 - 31.75));
        Some(f32::from_bits((h.to_bits() & !1) | u32::from(hole)))
    };
    let a = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    scene.world.terrains[0].terrain = scene.add_terrain_stream(layout, data.range());
    let b = render_ids(&scene, &ids, &raw, 160, 90);
    let differ = a.ids.iter().zip(&b.ids).filter(|(p, q)| p != q).count();
    assert!(differ <= 2, "{differ} pixels differ");
    assert!(a.object(5).unwrap().visible > 1000);
    // Unknown heights leave it out.
    assert!(
        render_ids(&scene, &ids, &|_, _, _| None, 160, 90)
            .object(5)
            .is_none()
    );
}

#[test]
pub(super) fn a_coarse_ring_covers_the_terrain_without_cracks() {
    // Seen from straight above with 16 quads a level: many levels,
    // morphing into each other, each leaving its finer level's square.
    let data = field(hill);
    let (mut scene, ids) = terrain_scene(
        data.clone(),
        Mat4::IDENTITY,
        Vec3::new(3.3, 70.0, 2.1),
        Vec3::new(3.3, 0.0, 2.0),
    );
    scene.world.terrains[0].lod.ring = 16;
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 160);
    let corner = |x: f32, z: f32| {
        let c = f.view_proj * Vec3::new(x, hill(x, z), z).extend(1.0);
        (
            ((c.x / c.w * 0.5 + 0.5) * 160.0) as u32,
            ((0.5 - c.y / c.w * 0.5) * 160.0) as u32,
        )
    };
    let (x0, y0) = corner(-29.0, -29.0);
    let (x1, y1) = corner(29.0, 29.0);
    let mut cracks = 0;
    for y in y0.min(y1)..y0.max(y1) {
        for x in x0.min(x1)..x0.max(x1) {
            cracks += u32::from(f.id_at(x, y) != 5);
        }
    }
    assert_eq!(cracks, 0, "pixels inside the terrain that show nothing");
}

/// As the camera glides, the clipmap's vertices keep their places and
/// rise or sink a little each step: the morph moves a corner only in
/// height, towards the coarser level's surface, and a cell's diagonal
/// moves between its own and its coarser cell's through its centre's
/// height, never flipping. (Collapsing odd corners onto their
/// neighbours slid them sideways, and a split chosen from morphed
/// corners flipped mid-morph: a whole triangle jumped.)
#[test]
pub(super) fn the_morph_moves_vertices_only_a_little_up_or_down() {
    // Rough ground: no two neighbouring cells are planar.
    let data =
        field(|x, z| 2.0 * (1.3 * x).sin() * (0.9 * z).cos() + 0.7 * (3.1 * x + 1.7 * z).sin());
    let layout = crate::TerrainLayout::new(data.size, data.columns, data.rows, 32, None).unwrap();
    let mut t = crate::Terrain::new(RenderScene::new().add_terrain(data.clone()));
    t.lod.ring = 16;
    let raw = |c: u32, r: u32| data.heights.get((r * data.columns + c) as usize).copied();
    let all = [Vec4::new(0.0, 0.0, 0.0, 1.0); 6];
    let mesh = |x: f32| {
        let eye = Vec3::new(x, 10.0, 1.3);
        let view = crate::TerrainView::perspective(eye, 500.0, 60.0, 720);
        let clipmap = t.clipmap(layout, &view, &[]);
        (
            clipmap.levels.clone(),
            terrain_mesh(&t, layout, &raw, &clipmap, 0.0, &all).unwrap(),
        )
    };
    let (mut compared, mut most) = (0, 0.0f32);
    let mut before = mesh(0.05);
    for i in 1..400 {
        let now = mesh(0.05 + i as f32 * 0.01);
        // A level that snapped to its next centre is a new grid.
        if now.0 == before.0 {
            let (a, b) = (&before.1.0, &now.1.0);
            assert_eq!(a.len(), b.len());
            for (p, q) in a.iter().zip(b) {
                assert!(
                    Vec2::new(p.x - q.x, p.z - q.z).length() < 1e-4,
                    "a vertex slid from {p} to {q}"
                );
                most = most.max((p.y - q.y).abs());
            }
            compared += 1;
        }
        before = now;
    }
    assert!(compared > 300, "{compared} steps compared");
    assert!(most < 0.1, "a vertex jumped {most} m in a 1 cm step");
}

#[test]
pub(super) fn an_origin_shift_draws_the_same_terrain() {
    let data = field(hill);
    let at = Vec3::new(4096.0 * 3.0 + 70.0, 12.0, -4096.0 * 2.0 + 30.0);
    let (mut scene, ids) = terrain_scene(
        data,
        Mat4::from_translation(at),
        at + Vec3::new(0.0, 14.0, 30.0),
        at,
    );
    let a = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    scene.rebase(Vec3::new(4096.0 * 3.0, 0.0, -4096.0 * 2.0));
    let b = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let differ = a.ids.iter().zip(&b.ids).filter(|(p, q)| p != q).count();
    assert!(differ <= 2, "{differ} pixels differ");
    assert!(a.object(5).unwrap().visible > 1000);
}

#[test]
pub(super) fn a_stamp_is_drawn_under_its_own_id() {
    let data = field(|_, _| 0.0);
    let (mut scene, mut ids) =
        terrain_scene(data, Mat4::IDENTITY, Vec3::new(0.0, 6.0, 20.0), Vec3::ZERO);
    let rock = scene.add_mesh(shapes::cuboid(Vec3::new(6.0, 8.0, 6.0)));
    scene.world.terrain_stamps.push(crate::TerrainStampDraw {
        terrain: 0,
        mesh: rock,
        transform: Mat4::from_translation(Vec3::new(0.0, 4.0, 0.0)),
        blend: 1.0,
        cast_shadows: true,
    });
    ids.stamps = vec![8];
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let c = f.view_proj * Vec3::new(0.0, 3.0, 3.0).extend(1.0);
    let (x, y) = (
        (c.x / c.w * 0.5 + 0.5) * 160.0,
        (0.5 - c.y / c.w * 0.5) * 90.0,
    );
    assert_eq!(f.at(x, y).map(|a| a.0), Some(8), "the rock's face");
    assert!(
        f.object(5).unwrap().occlusion() > 0.0,
        "it hides some ground"
    );
    // Its terrain hidden, it is not drawn either.
    scene.world.terrains[0].visible = false;
    assert!(
        render_ids(&scene, &ids, &|_, _, _| None, 160, 90)
            .object(8)
            .is_none()
    );
}

/// A flat terrain whose left half (x < 0) is layer 1 and right half
/// layer 0, with a grass field (id 7) on layer 1, 0.6 m tall, to 20 m
/// from the camera; seen from above its middle.
pub(super) fn grass_scene() -> (RenderScene, IdSources) {
    let mut data = field(|_, _| 0.0);
    let side = 64u32;
    let rgba: Vec<u8> = (0..side * side)
        .flat_map(|k| {
            if k % side < side / 2 {
                [0, 255, 0, 0]
            } else {
                [255, 0, 0, 0]
            }
        })
        .collect();
    data.control = Some(crate::TerrainControl {
        width: side,
        height: side,
        rgba: rgba.into(),
    });
    let (mut scene, mut ids) = terrain_scene(
        data,
        Mat4::IDENTITY,
        Vec3::new(0.0, 8.0, 18.0),
        Vec3::new(0.0, 0.0, 0.0),
    );
    scene.world.terrains[0].layers = vec![crate::TerrainLayer::default(); 2];
    scene.world.grass.push(crate::GrassDraw {
        terrain: 0,
        grass: crate::Grass {
            layer: 1,
            height: 0.6,
            distance: 20.0,
            ..Default::default()
        },
    });
    ids.grass = vec![7];
    (scene, ids)
}

/// Ways this could fail: no grass in the pass (the old one drew none,
/// so describe never listed a field), the field drawn on its terrain's
/// surface (tied with the ground) or at its blades' tips instead of
/// the half height its fidelity states, over the whole terrain instead
/// of where its layer grows, past its distance, or still drawn with
/// its terrain hidden.
#[test]
pub(super) fn a_grass_field_is_drawn_under_its_owner_id_where_it_grows() {
    let (mut scene, ids) = grass_scene();
    let eye = scene.frame.camera.position;
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    let grass = f.object(7).expect("the field shows");
    assert!(grass.visible > 400, "{} pixels", grass.visible);
    // The ground shows where the field does not grow.
    assert!(f.object(5).is_some_and(|t| t.visible > 400));
    let mut far = 0.0f32;
    for y in 0..f.height {
        for x in 0..f.width {
            let Some((id, p)) = f.at(x as f32 + 0.5, y as f32 + 0.5) else {
                continue;
            };
            if id == 7 {
                assert!((p.y - 0.3).abs() < 0.02, "({x}, {y}) at {p}: half of 0.6 m");
                assert!(p.x < 0.5, "({x}, {y}) at {p}: on layer 1's half");
                far = far.max(p.distance(eye));
            } else if id == 5 && p.x < -0.5 && p.distance(eye) < 19.0 {
                panic!("({x}, {y}) at {p}: bare ground where the field grows");
            }
        }
    }
    assert!(
        far <= 20.0 && far > 16.0,
        "the field reaches {far} m of its 20"
    );
    // A pebble lower than the half height in the meadow is hidden by
    // it; a post taller than the blades still shows.
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.0)));
    let m = scene.add_material(Material::default());
    let low = Mat4::from_scale_rotation_translation(
        Vec3::new(1.0, 0.2, 1.0),
        Default::default(),
        Vec3::new(-4.0, 0.1, 4.0),
    );
    let tall = Mat4::from_scale_rotation_translation(
        Vec3::new(0.4, 3.0, 0.4),
        Default::default(),
        Vec3::new(-6.0, 1.5, 2.0),
    );
    scene.add_instance(cube, m, low);
    scene.add_instance(cube, m, tall);
    let mut ids2 = ids.clone();
    ids2.instances = vec![8, 9];
    let f = render_ids(&scene, &ids2, &|_, _, _| None, 160, 90);
    assert!(
        f.object(8).is_none_or(|o| o.visible == 0),
        "the pebble is in the grass"
    );
    assert!(
        f.object(9).is_some_and(|o| o.visible > 10),
        "the post stands above it"
    );
    // Hidden with its terrain.
    scene.world.terrains[0].visible = false;
    assert!(
        render_ids(&scene, &ids, &|_, _, _| None, 160, 90)
            .object(7)
            .is_none()
    );
}

/// Ways this could fail: the density map ignored (a meadow painted
/// out still answering pick), read from the wrong axis (the cleared
/// half mirrored), or a hole in the ground keeping its grass.
#[test]
pub(super) fn a_painted_out_meadow_and_a_hole_have_no_grass_in_the_pass() {
    let (mut scene, ids) = grass_scene();
    // Black where z > 8 (nearest the camera, inside the field's
    // distance), white elsewhere.
    let side = 16u32;
    let rgba: Vec<u8> = (0..side * side)
        .flat_map(|k| {
            if k / side >= 10 {
                [0, 0, 0, 255]
            } else {
                [255, 255, 255, 255]
            }
        })
        .collect();
    let map = scene.add_texture(crate::TextureData {
        width: side,
        height: side,
        rgba,
        color_space: crate::ColorSpace::Linear,
        mips: crate::Mips::None,
    });
    scene.world.grass[0].grass.density_map = Some(map);
    // A hole near the camera, in the meadow.
    let handle = scene.world.terrains[0].terrain;
    let mut data = scene.terrain(handle).unwrap().clone();
    let (cx, cz) = data.cells();
    let mut holes = vec![0u8; (cx * cz) as usize];
    for r in 0..cz {
        for c in 0..cx {
            let (x, z) = (c as f32 * 0.5 - 31.75, r as f32 * 0.5 - 31.75);
            holes[(r * cx + c) as usize] = u8::from((x + 4.0).abs() < 1.5 && (z - 6.0).abs() < 1.5);
        }
    }
    data.holes = Some(holes.into());
    scene.set_terrain(handle, data.clone());
    let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
    assert!(f.object(7).is_some_and(|o| o.visible > 100));
    for y in 0..f.height {
        for x in 0..f.width {
            if let Some((7, p)) = f.at(x as f32 + 0.5, y as f32 + 0.5) {
                assert!(p.z < 8.5, "({x}, {y}) at {p}: painted out");
                assert!(
                    !data.hole(Vec2::new(p.x, p.z)),
                    "({x}, {y}) at {p}: over the hole"
                );
            }
        }
    }
}
