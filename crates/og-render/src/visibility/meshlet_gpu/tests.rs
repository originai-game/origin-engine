//! Tests of the meshlet plan and pools.

use super::*;
use crate::meshlet::{Cluster, ClusterLevel};
use glam::Vec3;

pub(super) fn batch(mesh: usize, first: u32, count: u32) -> Batch {
    Batch {
        mesh,
        material: 0,
        first,
        count,
        side: Side::Front,
        transparent: false,
        blend: false,
        physical: false,
        transmissive: false,
        dissolve: false,
        effects: false,
        shader: None,
        lean: false,
        derivatives: false,
        compact: false,
        array: false,
        foliage: false,
        simulated: false,
        cast: true,
        layers: 1,
        probe: 0,
        crowd: None,
    }
}

#[test]
pub(super) fn each_batch_gets_aligned_items_for_its_worst_level_and_a_workgroup_per_instance() {
    let batches = [
        batch(0, 0, 3),
        batch(5, 3, 0),
        batch(1, 3, 10),
        batch(9, 13, 2),
    ];
    // Mesh 9 is not in the pools: skipped. Batch 1 has no instances.
    let slot = |m: usize| match m {
        0 => Some((0, 7)),
        1 => Some((1, 40)),
        5 => Some((2, 1)),
        _ => None,
    };
    let (rows, draws, items, workgroups, _) = plan(
        &batches,
        [(&[0, 1, 2, 3][..], true)],
        slot,
        |_| true,
        |_| false,
    );
    assert_eq!(rows.len(), 2);
    assert_eq!(
        draws
            .iter()
            .map(|d| (d.batch, d.row, d.first_item))
            .collect::<Vec<_>>(),
        [(0, 0, 0), (2, 1, 32)]
    );
    // Batch 0: 3 x 7 = 21 items, then batch 2 from 32: 10 x 40.
    assert_eq!(items, 32 + 400);
    assert_eq!(workgroups, 13);
    assert_eq!(rows[1].first_workgroup, 3);
    assert_eq!((rows[1].first_instance, rows[1].instances), (3, 10));
    assert_eq!((rows[1].mesh_slot, rows[1].flags), (1, 1));
    // An instance's cluster slots are its mesh's most clusters apart.
    assert_eq!((rows[0].most, rows[1].most), (7, 40));
}

#[test]
pub(super) fn each_view_gets_rows_of_its_own_and_only_lit_views_cull_by_the_cone() {
    let batches = [batch(0, 0, 3), batch(1, 3, 10)];
    let slot = |m: usize| Some((m as u32, if m == 0 { 7 } else { 40 }));
    // The camera takes both; a reflection (lit) and a shadow view take
    // batch 1 only.
    let views: [(&[u32], bool); 3] = [(&[0, 1], true), (&[1], true), (&[1], false)];
    let (rows, draws, items, workgroups, _) = plan(&batches, views, slot, |_| true, |_| false);
    assert_eq!(
        rows.iter()
            .map(|r| (r.view, r.flags, r.first_instance))
            .collect::<Vec<_>>(),
        [(0, 1, 0), (0, 1, 3), (1, 1, 3), (2, 0, 3)]
    );
    assert_eq!(
        draws.iter().map(|d| d.lit).collect::<Vec<_>>(),
        [true, true, true, false]
    );
    // Each view's batch has its own items and draw, each from a
    // multiple of 32: 21 from 0, then 400 from 32, 448 and 864.
    assert_eq!(
        draws
            .iter()
            .map(|d| (d.batch, d.view, d.row, d.first_item))
            .collect::<Vec<_>>(),
        [(0, 0, 0, 0), (1, 0, 1, 32), (1, 1, 2, 448), (1, 2, 3, 864)]
    );
    assert_eq!(items, 1264);
    assert_eq!(workgroups, 3 + 10 * 3);
    assert_eq!(rows[3].first_workgroup, 23);
}

/// Every batch's draw group window holds its items and lies inside the
/// items buffer at its offset (the bounce of 427a61745: a window as wide
/// as the buffer, at a nonzero offset).
#[test]
pub(super) fn each_batch_s_window_holds_its_items_and_fits_the_buffer_at_its_offset() {
    let batches = [batch(0, 0, 3), batch(1, 3, 10), batch(0, 13, 1)];
    let slot = |m: usize| Some((m as u32, if m == 0 { 7 } else { 40 }));
    let views: [(&[u32], bool); 2] = [(&[0, 1, 2], true), (&[2, 0], false)];
    let (rows, _, items, _, _) = plan(&batches, views, slot, |_| true, |_| false);
    let (window, buffer) = item_window(&rows, items);
    assert_eq!(window, 416 * 8, "batch 1's 400 items, aligned");
    for r in &rows {
        let offset = r.first_item as u64 * 8;
        assert_eq!(offset % 256, 0, "a storage offset's alignment");
        assert!(r.instances as u64 * r.most as u64 * 8 <= window);
        assert!(
            offset + window <= buffer,
            "row at {offset}: {window} past {buffer}"
        );
    }
    // The last batch (3 x 7 items from its first) still gets the
    // whole window.
    let last = rows.iter().map(|r| r.first_item).max().unwrap() as u64 * 8;
    assert_eq!(buffer, last + window);
    assert!(buffer > items as u64 * 8);
}

/// Only the camera's rows of soft batches send clusters to the
/// visibility buffer; the soft list sits after every row's items with
/// two items for each cluster slot of those rows (capped), and each
/// row's flags carry its side for the rasteriser's face culling.
#[test]
pub(super) fn only_the_camera_s_soft_rows_feed_the_soft_list_after_every_row() {
    let mut back = batch(1, 3, 10);
    back.side = Side::Back;
    let batches = [batch(0, 0, 3), back];
    let slot = |m: usize| Some((m as u32, if m == 0 { 7 } else { 40 }));
    // The camera, a reflection (lit) and a shadow view all take batch 1.
    let views: [(&[u32], bool); 3] = [(&[0, 1], true), (&[1], true), (&[1], false)];
    let (rows, draws, items, _, list) = plan(&batches, views, slot, |_| false, |b| b.mesh == 1);
    assert_eq!(
        draws.iter().map(|d| d.soft).collect::<Vec<_>>(),
        [false, true, false, false]
    );
    assert_eq!(rows[1].flags, FLAG_SOFT | 1 << FLAG_SIDE_SHIFT);
    assert_eq!(rows[0].flags, 0);
    // The rows end at 864 + 400; the list starts aligned after them,
    // 416 clusters (batch 1's 10 x 40 slots, aligned to 32) of two
    // items each.
    let (first, cap) = list.expect("a soft row");
    assert_eq!((first, cap), (1280, 416));
    assert_eq!(items, first + 2 * cap);
    // No soft row: no list, the same rows.
    let (_, _, plain, _, none) = plan(&batches, views, slot, |_| false, |_| false);
    assert!(none.is_none());
    assert_eq!(plain, 1264);
}

/// The picked levels pack four sorted slots a row, slot `s` at row
/// `s / 4` lane `s % 4` (the shader's read), the last row's unused
/// lanes left unpicked.
#[test]
pub(super) fn picked_levels_pack_four_slots_a_row() {
    let rows = level_rows(&[0, 1, 2, u8::MAX, 3, 7]);
    assert_eq!(rows, vec![[0.0, 1.0, 2.0, 255.0], [3.0, 7.0, 255.0, 255.0]]);
    assert!(level_rows(&[]).is_empty());
}

/// A secondary view's pixels per unit, from its view-projection
/// alone, are the camera's from its projection: perspective and
/// orthographic, seen directly or in a mirror (a reflected view).
#[test]
pub(super) fn a_view_seen_through_its_matrix_picks_levels_as_the_camera_would() {
    let mirror = Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0));
    for projection in [
        crate::Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        crate::Projection::Orthographic { height: 12.0 },
    ] {
        let cam = crate::Camera {
            position: Vec3::new(3.0, 2.0, 9.0),
            target: Vec3::new(-1.0, 0.5, -4.0),
            projection,
            ..Default::default()
        };
        let frame = crate::frame::camera_frame(&cam, 16.0 / 9.0, glam::Vec2::ZERO);
        let camera = MeshletView::camera(&cam, &[], Vec::new(), 720);
        for view_proj in [frame.view_proj, frame.view_proj * mirror] {
            let seen = MeshletView::seen(&[], Vec::new(), cam.position, view_proj, 720);
            assert_eq!(seen.perspective, camera.perspective, "{projection:?}");
            assert!(
                (seen.per_unit - camera.per_unit).abs() <= 1e-3 * camera.per_unit,
                "{projection:?}: {} against {}",
                seen.per_unit,
                camera.per_unit
            );
        }
    }
    // A shadow view keeps the camera's eye and levels, drawn unlit.
    let cam = crate::Camera::default();
    let camera = MeshletView::camera(&cam, &[], vec![1], 720);
    let shadow = MeshletView::shadow(&camera, &[Vec4::X], vec![2]);
    assert_eq!(
        (shadow.eye, shadow.per_unit, shadow.lit),
        (camera.eye, camera.per_unit, false)
    );
    assert_eq!((shadow.planes, shadow.taken), (vec![Vec4::X], vec![2]));
}

#[test]
pub(super) fn pool_rows_offset_clusters_and_indices_into_the_pools() {
    let cl = |first, triangles| Cluster {
        first,
        triangles,
        center: Vec3::new(1.0, 2.0, 3.0),
        radius: 0.5,
        cone_axis: Vec3::Z,
        cone_cutoff: 0.25,
        error_center: Vec3::new(0.0, 1.0, 0.0),
        error_radius: 4.0,
        ..Default::default()
    };
    let c = MeshClusters {
        levels: vec![
            ClusterLevel {
                first: 0,
                count: 300,
                error: 0.0,
                clusters: 0..2,
            },
            ClusterLevel {
                first: 300,
                count: 30,
                error: 0.2,
                clusters: 2..3,
            },
        ],
        clusters: vec![cl(0, 64), cl(192, 36), cl(300, 10)],
        indices: Vec::new(),
    };
    let (mesh, rows) = pool_rows(&c, 1000, 50, 7).unwrap();
    // A foliage mesh's clusters name its region (plus one) in their
    // first row's w, and only there (the sphere and cone rows keep theirs).
    let mut pointed = rows.clone();
    pool_foliage(&mut pointed, 40);
    for (k, (a, b)) in pointed.iter().zip(&rows).enumerate() {
        if k % CLUSTER_ROWS == 0 {
            assert_eq!(a[..3], b[..3]);
            assert_eq!(a[3], 41);
        } else {
            assert_eq!(a, b);
        }
    }
    assert_eq!(mesh[0], [0.0f32, 1.0, 0.0, 4.0].map(f32::to_bits));
    assert_eq!(mesh[1], [2, 7, 0, 0], "levels, the first vertex");
    assert_eq!(mesh[2], [50, 2, 0, 0]);
    assert_eq!(mesh[3], [52, 1, 0.2f32.to_bits(), 0]);
    assert_eq!(rows.len(), 9);
    // The third cluster: level 1, its first index past the pool's base.
    assert_eq!(rows[6], [1300, 10, 1, 0]);
    assert_eq!(rows[7], [1.0f32, 2.0, 3.0, 0.5].map(f32::to_bits));
    assert_eq!(rows[8], [0.0f32, 0.0, 1.0, 0.25].map(f32::to_bits));
    // Past eight levels: refused.
    let mut many = c.clone();
    many.levels = vec![c.levels[0].clone(); MAX_LEVELS + 1];
    assert!(pool_rows(&many, 0, 0, 0).is_none());
}
