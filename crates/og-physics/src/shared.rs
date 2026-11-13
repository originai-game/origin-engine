//! What the system keeps between ticks, and the keys and chunks of the
//! shapes it builds.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct ShapeKey {
    pub shape: u32,
    pub mesh: u64,
    pub scale: [u32; 3],
}

/// The key of chunk `chunk` of the height field `key` names
/// ([`FIELD_CHUNK`]): the chunk plus 1 in the shape code's bits from 8
/// (a save's [`Hole`] names a chunk so); a code with none there is the
/// whole field.
pub(crate) fn chunk_key(key: ShapeKey, chunk: u32) -> ShapeKey {
    ShapeKey {
        shape: SHAPE_HEIGHTFIELD | ((chunk + 1) << 8),
        ..key
    }
}

/// The chunk a shape code names ([`chunk_key`]), if it is a chunk's.
pub(crate) fn key_chunk(shape: u32) -> Option<u32> {
    (shape & 0xff == SHAPE_HEIGHTFIELD && shape >> 8 != 0).then(|| (shape >> 8) - 1)
}

/// Where a saved simulation's bytes lack an asset shape: at byte `at` of
/// the cut bytes, the shape `key` names (rebuilt from the registered mesh
/// or height field when the save is decoded; `sim::save`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Hole {
    pub at: u64,
    pub key: ShapeKey,
}

/// What a [`Push`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PushKind {
    /// A force (N) over the next step.
    Force,
    /// An impulse (N·s) at once.
    Impulse,
    /// A torque (N·m) over the next step.
    Torque,
    /// An angular impulse (N·m·s) at once.
    TorqueImpulse,
}

/// A force, impulse or torque game code gave a body, applied by the next
/// step (world state: the `og.physics` state block saves those not yet
/// applied).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Push {
    pub entity: u64,
    pub kind: PushKind,
    /// The force, impulse, torque or angular impulse, in world axes.
    pub vector: Vec3,
    /// Where a force or impulse acts (world space); `None` at the centre of mass.
    pub point: Option<Vec3>,
}

/// The host's geometry by handle, for [`PhysicsSystem::set_mesh_source`].
pub type MeshSource = Arc<dyn Fn(u64) -> Option<MeshDesc> + Send + Sync>;

/// What the system keeps between ticks: the simulation of the active
/// profile (world state, saved by [`PhysicsState`]), registered meshes and
/// the shapes built from them (assets and caches).
#[derive(Default)]
pub(crate) struct Shared {
    #[cfg(feature = "physics3d")]
    pub sim3: Option<Box<dim3::Sim>>,
    #[cfg(feature = "physics2d")]
    pub sim2: Option<Box<dim2::Sim>>,
    /// Meshes registered with new geometry since the last step; colliders
    /// built from them are rebuilt.
    pub stale_meshes: BTreeSet<u64>,
    pub meshes: IdMap<u64, Arc<MeshDesc>>,
    /// Where a collider's mesh nobody registered comes from
    /// ([`PhysicsSystem::set_mesh_source`]).
    pub mesh_source: Option<MeshSource>,
    /// Height fields by handle (assets, like meshes).
    pub heightfields: IdMap<u64, Arc<HeightfieldDesc>>,
    /// Per stale height field whose every change since the last step was a
    /// region edit ([`PhysicsSystem::edit_heightfield`]): the chunks those
    /// edits reached, which are all its static colliders rebuild. A stale
    /// field without an entry is rebuilt whole.
    pub field_edits: IdMap<u64, BTreeSet<u32>>,
    /// The chunk shapes of static height-field colliders of more than one
    /// chunk ([`FIELD_CHUNK`]), keyed like the shape cache with the chunk in
    /// the key's shape code ([`chunk_key`]); outside the cache's limit, so
    /// a large field's chunks are never cleared together.
    #[cfg(feature = "physics3d")]
    pub chunks3: BTreeMap<ShapeKey, rapier3d::prelude::SharedShape>,
    /// Never filled (height fields are 3D only); the step is written once.
    #[cfg(feature = "physics2d")]
    pub chunks2: BTreeMap<ShapeKey, rapier2d::prelude::SharedShape>,
    /// The bytes of the simulation a restore applied, until the next step
    /// ([`Restored`]).
    #[cfg(feature = "physics3d")]
    pub restored: Option<Restored>,
    /// Why no height field can be registered in this build
    /// ([`PhysicsSystem::refuse_heightfields`]): every SHAPE_HEIGHTFIELD
    /// collider is refused with it.
    pub heightfields_absent: Option<&'static str>,
    #[cfg(feature = "physics3d")]
    pub shapes3: BTreeMap<ShapeKey, rapier3d::prelude::SharedShape>,
    #[cfg(feature = "physics2d")]
    pub shapes2: BTreeMap<ShapeKey, rapier2d::prelude::SharedShape>,
    /// Triangle-mesh shapes the cook baked over a mesh at scale 1, by mesh
    /// handle ([`PhysicsSystem::set_baked_trimesh`]): taken instead of a
    /// build when the mesh is exactly the one baked.
    #[cfg(feature = "physics3d")]
    pub baked3: IdMap<u64, rapier3d::prelude::SharedShape>,
    /// The colliders [`PhysicsSystem::query_system`] recorded this tick, and
    /// the query view built from them on the first query that asked.
    pub recorded: Option<(Physics, Scene)>,
    pub recorded_query: Option<Arc<QueryInner>>,
    /// Ragdoll rigs by entity (assets, like meshes) and the animation's
    /// bone poses the host gave for this tick.
    pub rigs: IdMap<u64, Arc<RagdollRig>>,
    pub ragdoll_targets: IdMap<u64, Vec<(Vec3, Quat)>>,
    /// Garment rigs by entity, with the key the host gave them (assets,
    /// like ragdoll rigs), and the skin's pose of each for this tick
    /// ([`PhysicsSystem::set_garment_pose`]), taken by the step.
    pub garment_rigs: IdMap<u64, (u64, Arc<GarmentRig>)>,
    pub garment_poses: IdMap<u64, GarmentPose>,
    /// What the last step cost ([`PhysicsSystem::stats`]; not world state).
    pub stats: PhysicsStats,
    /// Forces and impulses waiting for the next step, in the order given
    /// (world state, saved with the simulation).
    pub pushes: Vec<Push>,
    /// The og.Water bodies og.Buoyancy floats on, as the host read them
    /// as the tick started ([`PhysicsSystem::set_waters`]; derived from the
    /// world's components, not world state).
    pub waters: Vec<WaterBody>,
    /// The world's air the next steps move things through: og.Wind and
    /// the og.WindMotors as the host read them when the tick started
    /// ([`PhysicsSystem::set_air`]; derived from the world's components,
    /// not world state here).
    pub air: og_env::air::Air,
}

/// At most this many mesh-derived shapes are cached (a scale animated every
/// tick would otherwise grow the cache without bound).
pub(crate) const SHAPE_CACHE_LIMIT: usize = 1024;

/// Cells per side of one chunk of a static height-field collider. A field
/// of more cells along x or z is built as one rapier collider per chunk
/// (each a height field over its square, sharing its border samples with
/// its neighbours), so a region edit ([`PhysicsSystem::edit_heightfield`])
/// rebuilds only the chunks under it: parry's height field cannot change
/// its heights in place. A field of one chunk is one collider, as before.
pub(crate) const FIELD_CHUNK: u32 = 256;

/// The chunks along x and z of a height field of `columns` × `rows`
/// heights ([`FIELD_CHUNK`]).
pub(crate) fn field_chunks(columns: u32, rows: u32) -> (u32, u32) {
    (
        (columns.max(2) - 1).div_ceil(FIELD_CHUNK),
        (rows.max(2) - 1).div_ceil(FIELD_CHUNK),
    )
}

/// The heights chunk `k` covers (row by row, [`field_chunks`]'s x first):
/// its first and last column and row, inclusive.
pub(crate) fn chunk_samples(f: &HeightfieldDesc, k: u32) -> ([u32; 2], [u32; 2]) {
    let (cx, _) = field_chunks(f.columns, f.rows);
    let (i, j) = (k % cx, k / cx);
    let c0 = i * FIELD_CHUNK;
    let r0 = j * FIELD_CHUNK;
    (
        [c0, (c0 + FIELD_CHUNK).min(f.columns - 1)],
        [r0, (r0 + FIELD_CHUNK).min(f.rows - 1)],
    )
}

/// A height field's spacing between samples along x and z.
pub(crate) fn field_cell(f: &HeightfieldDesc) -> Vec2 {
    Vec2::new(
        f.size.x / (f.columns - 1) as f32,
        f.size.y / (f.rows - 1) as f32,
    )
}

/// Where chunk `k`'s centre lies from its field's, in the frame of a
/// collider at `scale`.
pub(crate) fn chunk_offset(f: &HeightfieldDesc, scale: Vec3, k: u32) -> Vec3 {
    let ([c0, c1], [r0, r1]) = chunk_samples(f, k);
    let centre = field_cell(f) * Vec2::new((c0 + c1) as f32, (r0 + r1) as f32) * 0.5 - f.size * 0.5;
    Vec3::new(centre.x, 0.0, centre.y) * scale.abs()
}

/// The chunks whose heights a change inside `lo`..`hi` (the field's own
/// space: metres, centred on it) can reach, ascending. A sample on a
/// chunk's edge belongs to both chunks beside it.
pub(crate) fn chunks_under(f: &HeightfieldDesc, lo: Vec2, hi: Vec2) -> Vec<u32> {
    let (cx, cz) = field_chunks(f.columns, f.rows);
    let span = |a: f32, b: f32, size: f32, n: u32, chunks: u32| {
        let cell = size / (n - 1) as f32;
        let at = |x: f32| ((x + size * 0.5) / cell).clamp(0.0, (n - 1) as f32);
        let (s0, s1) = (at(a).floor() as u32, at(b).ceil() as u32);
        (
            s0.saturating_sub(1) / FIELD_CHUNK,
            (s1 / FIELD_CHUNK).min(chunks - 1),
        )
    };
    let (i0, i1) = span(lo.x, hi.x, f.size.x, f.columns, cx);
    let (j0, j1) = span(lo.y, hi.y, f.size.y, f.rows, cz);
    (j0..=j1)
        .flat_map(|j| (i0..=i1).map(move |i| j * cx + i))
        .collect()
}
