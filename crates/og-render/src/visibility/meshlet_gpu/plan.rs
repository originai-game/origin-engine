//! What a frame's meshlet views cull: each batch's rows and its window of items.

use super::*;

/// Whether `b` draws through the meshlet path, given the scene and what
/// only the renderer knows about its mesh (`lightmapped`).
pub(crate) fn takes(scene: &RenderScene, b: &Batch, clustered: bool, lightmapped: bool) -> bool {
    scene.objects.meshlets
        && clustered
        && !b.transparent
        && b.shader.is_none()
        && !b.physical
        && !b.array
        && !b.compact
        && !lightmapped
}

/// One view of the frame's dispatch: its planes, the batches it takes
/// (indices into the frame's batches) and the eye its levels are picked
/// from. The first is the camera's; a shadow view (not `lit`) carries the
/// camera's eye and never culls by the normal cone.
#[derive(Clone, Debug)]
pub(crate) struct MeshletView {
    pub planes: Vec<Vec4>,
    pub taken: Vec<u32>,
    pub eye: Vec3,
    /// Pixels per unit of length at distance 1 (perspective) or anywhere
    /// (orthographic).
    pub per_unit: f32,
    pub perspective: bool,
    /// Drawn lit (the camera, a reflection, a view, a probe face), not a
    /// shadow pass.
    pub lit: bool,
}

impl MeshletView {
    /// The camera's view: its pixels per unit from its projection and
    /// the frame's `height` in pixels (lod.rs `pixels_per_unit`).
    pub fn camera(camera: &crate::Camera, planes: &[Vec4], taken: Vec<u32>, height: u32) -> Self {
        let (perspective, per_unit) = match camera.projection {
            crate::Projection::Perspective { fov_y_degrees } => (
                true,
                height as f32 / (2.0 * (fov_y_degrees.to_radians() * 0.5).tan().max(1e-6)),
            ),
            crate::Projection::Orthographic { height: h } => (false, height as f32 / h.max(1e-6)),
        };
        MeshletView {
            planes: planes.to_vec(),
            taken,
            eye: camera.position,
            per_unit,
            perspective,
            lit: true,
        }
    }

    /// A secondary view seen from `eye` through `view_proj` (a rigid view
    /// times a projection, a mirror's or an oblique one included) into a
    /// target `height` pixels high: half the height times the length of
    /// its clip y row is the pixels per unit at distance 1 for a
    /// perspective projection, and anywhere for an orthographic one.
    pub fn seen(planes: &[Vec4], taken: Vec<u32>, eye: Vec3, view_proj: Mat4, height: u32) -> Self {
        MeshletView {
            planes: planes.to_vec(),
            taken,
            eye,
            per_unit: 0.5 * height as f32 * view_proj.row(1).truncate().length(),
            perspective: view_proj.row(3).truncate().length() > 1e-6,
            lit: true,
        }
    }

    /// A shadow pass's view of `taken` within `planes`, its levels the
    /// camera's (`camera` the camera's view).
    pub fn shadow(camera: &MeshletView, planes: &[Vec4], taken: Vec<u32>) -> Self {
        MeshletView {
            planes: planes.to_vec(),
            taken,
            lit: false,
            ..camera.clone()
        }
    }
}

/// The levels callers picked per sorted slot (`u8::MAX`: none), four
/// slots a row of the views buffer (meshlet_cull.wgsl `params.lod`).
pub(crate) fn level_rows(levels: &[u8]) -> Vec<[f32; 4]> {
    levels
        .chunks(4)
        .map(|c| {
            let mut row = [u8::MAX as f32; 4];
            for (r, &l) in row.iter_mut().zip(c) {
                *r = l as f32;
            }
            row
        })
        .collect()
}

/// A batch row's capacity in work items, every cluster slot of its
/// instances, rounded up to the item alignment.
pub(super) fn capacity(r: &BatchRow) -> u32 {
    r.instances.saturating_mul(r.most).div_ceil(ITEM_ALIGN) * ITEM_ALIGN
}

/// [`plan`]'s (rows, draws, items, workgroups, the soft list).
pub(crate) type Plan = (
    Vec<BatchRow>,
    Vec<MeshletDraw>,
    u32,
    u32,
    Option<(u32, u32)>,
);

/// The frame's batch table and the work its dispatch covers, from each
/// view's clustered batches' instances (their range of the sorted
/// instances) and their meshes' most clusters at any level: (rows, draws,
/// items, workgroups, the soft list). Only the rows of lit views (`true`
/// beside their batches) may cull by `cone`; only the camera's (view 0,
/// lit) may send their small clusters to the visibility buffer (`soft`).
/// The soft list, after every row's items, holds two items per cluster
/// rasterised in compute ((instance, cluster), (row with its side from
/// bit 28, the mesh's first vertex), written by the cull): its first item
/// and its capacity in clusters, `None` without a soft row.
pub(crate) fn plan<'a>(
    batches: &[Batch],
    views: impl IntoIterator<Item = (&'a [u32], bool)>,
    slot: impl Fn(usize) -> Option<(u32, u32)>,
    cone: impl Fn(&Batch) -> bool,
    soft: impl Fn(&Batch) -> bool,
) -> Plan {
    let mut rows = Vec::new();
    let mut draws = Vec::new();
    let (mut items, mut workgroups, mut listed) = (0u32, 0u32, 0u32);
    for (view, lit, &b) in views
        .into_iter()
        .enumerate()
        .flat_map(|(v, (taken, lit))| taken.iter().map(move |b| (v as u32, lit, b)))
    {
        let batch = &batches[b as usize];
        let Some((mesh_slot, most)) = slot(batch.mesh) else {
            continue;
        };
        if batch.count == 0 {
            continue;
        }
        let first_item = items.div_ceil(ITEM_ALIGN) * ITEM_ALIGN;
        let is_soft = view == 0 && lit && soft(batch);
        let side = match batch.side {
            Side::Front => 0,
            Side::Back => 1,
            Side::Double => 2,
        };
        let row = BatchRow {
            mesh_slot,
            first_instance: batch.first,
            instances: batch.count,
            first_item,
            first_workgroup: workgroups,
            flags: if lit && cone(batch) { FLAG_CONE } else { 0 }
                | if is_soft { FLAG_SOFT } else { 0 }
                | side << FLAG_SIDE_SHIFT,
            view,
            most,
        };
        if is_soft {
            listed = listed.saturating_add(capacity(&row));
        }
        rows.push(row);
        draws.push(MeshletDraw {
            batch: b,
            view,
            lit,
            row: rows.len() as u32 - 1,
            first_item,
            soft: is_soft,
        });
        items = first_item + batch.count * most;
        workgroups += batch.count;
    }
    let list = (listed > 0).then(|| {
        let first = items.div_ceil(ITEM_ALIGN) * ITEM_ALIGN;
        let cap = listed.min(SOFT_CAP);
        items = first + 2 * cap;
        (first, cap)
    });
    (rows, draws, items, workgroups, list)
}

/// The bytes a draw group binds of the work items from a batch's first
/// (its dynamic offset): the largest batch's items, rounded up to the
/// item alignment; and the bytes the items buffer needs so every batch's
/// window lies inside it (wgpu refuses an offset whose window would pass
/// the buffer's end). `items` is the plan's total.
pub(crate) fn item_window(rows: &[BatchRow], items: u32) -> (u64, u64) {
    let window = rows.iter().map(capacity).max().unwrap_or(0).max(ITEM_ALIGN) as u64 * 8;
    let last = rows.iter().map(|r| r.first_item).max().unwrap_or(0) as u64 * 8;
    (window, (items as u64 * 8).max(last + window))
}
