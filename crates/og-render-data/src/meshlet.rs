//! Meshlets (docs/design.md §2.6, the realism wave's R1, after
//! three.js#35191's `MeshletDrawingPipeline`): a mesh cut into clusters of
//! at most [`CLUSTER_TRIANGLES`] triangles, each level of detail its own
//! clusters, culled one by one on the GPU.
//!
//! The cook (og-cook's `model`) simplifies each primitive into levels and
//! measures each level's error, as before (`lod.rs`), then cuts every level
//! into clusters with meshoptimizer (`meshopt_buildMeshlets`, at most
//! [`CLUSTER_VERTICES`] distinct vertices) and reorders the level's indices
//! so each cluster is a contiguous range of them. A cluster carries its
//! bounding sphere and its normal cone (`meshopt_computeMeshletBounds`).
//!
//! Per frame and per instance the level is the coarsest whose error, seen
//! at the instance's distance, stays under a pixel ([`crate::lod::select`]
//! without hysteresis: the GPU keeps no level from frame to frame); then
//! each of the level's clusters is tested on its own:
//!
//! - **frustum**: its sphere against the view's planes ([`outside`]);
//! - **normal cone**: every triangle faces away from the eye
//!   ([`backfacing`]), skipped for two-sided materials, for transforms
//!   that mirror or scale unevenly (the cone does not survive them) and in
//!   shadow passes (which draw back faces);
//! - **occlusion**, in two phases: the clusters visible last frame are
//!   drawn first into a depth of their own, a pyramid of its farthest
//!   depths is built ([`pyramid_level`] picks the level), and every other
//!   cluster is tested against it ([`ScreenRect`]); what passes is drawn
//!   too and is visible next frame. So a cluster that comes into view is
//!   drawn the frame it does, and occlusion never hides a visible pixel.
//!
//! The functions here are the CPU statement of what the WGSL computes, and
//! the tests check them; the GPU path mirrors them line for line.

use glam::{Mat4, Vec2, Vec3, Vec4};

/// The most triangles a cluster holds (three.js's `MESHLET_SIZE`).
pub const CLUSTER_TRIANGLES: usize = 64;

/// The most distinct vertices a cluster references: three a triangle, so
/// a faceted mesh (few shared vertices) still fills its clusters; the
/// clusters index the mesh's vertex buffer, not a local one.
pub const CLUSTER_VERTICES: usize = CLUSTER_TRIANGLES * 3;

/// One cluster of a level: a range of its mesh's indices and its bounds,
/// in the mesh's local space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cluster {
    /// Its first index in the mesh's indices (every level's, level 0 first).
    pub first: u32,
    /// Its triangles (1 to [`CLUSTER_TRIANGLES`]).
    pub triangles: u32,
    pub center: Vec3,
    pub radius: f32,
    /// The normal cone: the triangles' mean facing and the cutoff
    /// ([`backfacing`]); a cutoff of 1 or more never culls.
    pub cone_axis: Vec3,
    pub cone_cutoff: f32,
    /// Selection per cluster ([`selected`]): its error and its parents',
    /// in the mesh's units, both projected from the sphere
    /// `error_center`/`error_radius`, and the group it was simplified with.
    /// The cook's per-level chain gives every cluster of a level the same
    /// values (the level's error, the next level's, the mesh's sphere, the
    /// level as the group); a cluster DAG gives each its own.
    pub error: f32,
    pub parent_error: f32,
    pub error_center: Vec3,
    pub error_radius: f32,
    pub group: u32,
}

/// Whether a cluster is drawn at its detail: its error, projected at
/// `pixels_per_unit` ([`crate::lod::pixels_per_unit`] of its error sphere,
/// the instance's scale folded in) times `bias`, stays under
/// [`crate::lod::LOD_PIXEL_ERROR`] and its parents' does not. In the
/// per-level chain exactly one level passes, the one [`crate::lod::select`]
/// picks (without hysteresis). An error of 0 projects to 0 at any distance.
pub fn selected(error: f32, parent_error: f32, pixels_per_unit: f32, bias: f32) -> bool {
    let screen = |e: f32| {
        if e <= 0.0 {
            0.0
        } else if bias <= 0.0 || !pixels_per_unit.is_finite() {
            f32::INFINITY
        } else {
            e * pixels_per_unit * bias
        }
    };
    let t = crate::lod::LOD_PIXEL_ERROR;
    screen(error) < t && screen(parent_error) >= t
}

/// One level of detail of a clustered mesh.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClusterLevel {
    /// Its range of the mesh's indices.
    pub first: u32,
    pub count: u32,
    /// Its largest distance from level 0, both ways, in the mesh's units
    /// (0 for level 0): `lod.rs`.
    pub error: f32,
    /// Its range of [`MeshClusters::clusters`], covering its indices in order.
    pub clusters: std::ops::Range<u32>,
}

/// A mesh's levels and their clusters, as the cook made them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshClusters {
    pub levels: Vec<ClusterLevel>,
    pub clusters: Vec<Cluster>,
    /// Every level's triangles, level 0 first, indexing the mesh's
    /// vertices (the mesh's own indices are level 0's alone: every pass
    /// but the meshlet path draws them).
    pub indices: Vec<u32>,
}

impl MeshClusters {
    /// Whether the levels and clusters describe `index_count` indices: each
    /// level's clusters lie in order inside it and cover all of it, each
    /// within the triangle limit. A mesh that fails is drawn without them.
    pub fn validate(&self, index_count: u32) -> Result<(), String> {
        if self.levels.is_empty() {
            return Err("no levels".into());
        }
        for (l, level) in self.levels.iter().enumerate() {
            let end = level.first as u64 + level.count as u64;
            if end > index_count as u64 || level.count % 3 != 0 {
                return Err(format!("level {l}: indices out of range"));
            }
            let clusters = self
                .clusters
                .get(level.clusters.start as usize..level.clusters.end as usize)
                .ok_or_else(|| format!("level {l}: clusters out of range"))?;
            let mut at = level.first as u64;
            for (c, cluster) in clusters.iter().enumerate() {
                if cluster.first as u64 != at
                    || cluster.triangles == 0
                    || cluster.triangles as usize > CLUSTER_TRIANGLES
                {
                    return Err(format!("level {l}, cluster {c}: not the next range"));
                }
                at += cluster.triangles as u64 * 3;
            }
            if at != end {
                return Err(format!("level {l}: its clusters cover {at} of {end}"));
            }
        }
        Ok(())
    }

    /// Every level's error, level 0 first ([`crate::lod::select`]'s chain).
    pub fn errors(&self) -> Vec<f32> {
        self.levels.iter().map(|l| l.error).collect()
    }
}

/// A foliage mesh's deformation data (the vegetation's trees: `foliage.rs`
/// states the deformation): per vertex its branch pivot and leaf anchor
/// ([`crate::foliage::FoliageVertex`]), in the mesh's vertex order, and
/// what the whole model shares, so every primitive of one tree (bark and
/// leaves) bends as one. On the meshlet path every pull of the mesh's
/// vertices moves them by `foliage_offset` (foliage.wgsl): the lit
/// pass, the prepass (and its previous position, from the last drawn
/// frame's wind), the occluders' depth, the shadow casters and the
/// visibility buffer's rasteriser; the cull widens the mesh's and its
/// clusters' spheres by the deformation's bound
/// ([`crate::foliage::FoliageBounds::deform_margin`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshFoliage {
    pub vertices: Vec<crate::foliage::FoliageVertex>,
    /// The model's top in its own units (its highest point over its
    /// default scene): where the trunk's bend is full.
    pub top: f32,
    /// The model's canopy centre in its own units (the leaf anchors' box
    /// centre over the whole model), for the leaves' dome normals.
    pub canopy_center: Vec3,
    /// Its triangles are leaf cards (glTF `extras.ogFoliage.leaves`).
    pub leaves: bool,
    /// How its leaves follow the year: its material's
    /// ([`crate::foliage::LeafSeason`]; evergreen: never). Every pass reads
    /// it here, the ones that bind no material too, so a leaf's shadow
    /// grows and falls with it.
    pub season: crate::foliage::LeafSeason,
    /// The crown its leaves fill, for the sky they hide from each other
    /// (foliage.wgsl `meshlet_crown`): the half-extents of an
    /// ellipsoid about [`Self::canopy_center`] (zero: no crown), and its
    /// leaves' extinction, half their one-sided area per unit of the
    /// ellipsoid's volume (randomly oriented leaves, Ross's G of 1/2), both
    /// in the model's units.
    pub crown: Vec3,
    pub extinction: f32,
    /// Per level of the mesh (its [`MeshClusters`] levels, in order), how
    /// much larger that level draws each leaf about its anchor: a thinned
    /// level of leaf cards keeps fewer leaves, each larger, over the full
    /// mesh's own vertices. At most [`FOLIAGE_LEVEL_SCALES`], each 1 or
    /// more; a level past the list draws at 1.
    pub level_scales: Vec<f32>,
}

/// The levels whose leaf scale a foliage mesh's header holds
/// ([`MeshFoliage::level_scales`]): two rows of four.
pub const FOLIAGE_LEVEL_SCALES: usize = 8;

/// vec4 rows of a foliage mesh's header in the vertex pool (meshlet_gpu.rs
/// `pool_foliage`), before its two rows per vertex: (canopy centre, top),
/// (flags: 1 leaves, its season, its season shift, 0), (autumn colour, 0),
/// (the second autumn colour, 0), (the crown's half-extents, its
/// extinction), then the leaf scales of levels 0 to 3 and of 4 to 7.
pub const FOLIAGE_HEADER_ROWS: u32 = 7;

impl MeshFoliage {
    /// Whether it describes a mesh of `vertex_count` vertices: one entry a
    /// vertex, every value finite, stiffness and flutter within 0..1 and a
    /// positive top. A mesh that fails keeps still.
    pub fn validate(&self, vertex_count: usize) -> Result<(), String> {
        if self.vertices.len() != vertex_count {
            return Err(format!(
                "{} foliage vertices for {vertex_count} vertices",
                self.vertices.len()
            ));
        }
        if !(self.top.is_finite() && self.top > 0.0) || !self.canopy_center.is_finite() {
            return Err("its top and canopy centre must be finite, the top positive".into());
        }
        let s = &self.season;
        let colours = s.autumn_color.iter().chain(&s.autumn_color2);
        if !(s.season.is_finite()
            && s.shift.is_finite()
            && colours.into_iter().all(|x| x.is_finite()))
        {
            return Err(format!("its season must be finite: {s:?}"));
        }
        if !(self.crown.is_finite() && self.crown.min_element() >= 0.0)
            || !(self.extinction.is_finite() && self.extinction >= 0.0)
        {
            return Err("its crown must be finite and not negative".into());
        }
        if self.level_scales.len() > FOLIAGE_LEVEL_SCALES
            || !self.level_scales.iter().all(|s| s.is_finite() && *s >= 1.0)
        {
            return Err(format!(
                "its level scales must be at most {FOLIAGE_LEVEL_SCALES}, each finite and 1 or more: {:?}",
                self.level_scales
            ));
        }
        for (i, v) in self.vertices.iter().enumerate() {
            let finite = v.branch.iter().chain(&v.leaf).all(|x| x.is_finite());
            let unit = |x: f32| (0.0..=1.0).contains(&x);
            if !finite || !unit(v.branch[3]) || !unit(v.leaf[3]) {
                return Err(format!("vertex {i}: {v:?}"));
            }
        }
        Ok(())
    }

    /// Its foliage for its mesh drawn once at each of `placements`, as one
    /// mesh (og-runtime's placed scatter mesh): each vertex's pivot and
    /// anchor placed as the vertex is, the vertices in placement order;
    /// the canopy, top and crown where the first placement puts them (its
    /// largest scale stretching the crown and thinning its extinction).
    pub fn placed(&self, placements: &[Mat4]) -> MeshFoliage {
        let first = placements.first().copied().unwrap_or(Mat4::IDENTITY);
        let scale = first.to_scale_rotation_translation().0.abs().max_element();
        let place = |t: &Mat4, r: [f32; 4]| {
            let p = t.transform_point3(Vec3::new(r[0], r[1], r[2]));
            [p.x, p.y, p.z, r[3]]
        };
        let c = self.canopy_center;
        MeshFoliage {
            vertices: placements
                .iter()
                .flat_map(|t| {
                    self.vertices.iter().map(|v| crate::foliage::FoliageVertex {
                        branch: place(t, v.branch),
                        leaf: place(t, v.leaf),
                    })
                })
                .collect(),
            top: first.transform_point3(Vec3::new(c.x, self.top, c.z)).y,
            canopy_center: first.transform_point3(c),
            crown: self.crown * scale,
            extinction: if scale > 0.0 {
                self.extinction / scale
            } else {
                0.0
            },
            ..self.clone()
        }
    }

    /// Its rows in the vertex pool: the header, then two rows a vertex.
    pub fn pool_rows(&self) -> Vec<[f32; 4]> {
        let c = self.canopy_center;
        let mut rows = Vec::with_capacity(FOLIAGE_HEADER_ROWS as usize + self.vertices.len() * 2);
        rows.push([c.x, c.y, c.z, self.top]);
        let s = &self.season;
        rows.push([
            self.leaves as u32 as f32,
            s.season.clamp(0.0, 1.0),
            s.shift,
            0.0,
        ]);
        let [r, g, b] = s.autumn_color;
        rows.push([r, g, b, 0.0]);
        let [r, g, b] = s.autumn_color2;
        rows.push([r, g, b, 0.0]);
        let k = self.crown;
        rows.push([k.x, k.y, k.z, self.extinction]);
        let scale = |l: usize| self.level_scales.get(l).copied().unwrap_or(1.0);
        rows.push(std::array::from_fn(scale));
        rows.push(std::array::from_fn(|l| scale(l + 4)));
        for v in &self.vertices {
            rows.push(v.branch);
            rows.push(v.leaf);
        }
        rows
    }

    /// The box (min and max corner) of where its levels draw `positions`
    /// (the mesh's) at rest: each vertex where it is and as far from its
    /// anchor as its largest level draws it. `None` for no vertex.
    pub fn rest_box(&self, positions: impl Iterator<Item = Vec3>) -> Option<(Vec3, Vec3)> {
        let largest = self.level_scales.iter().fold(1.0f32, |a, &s| a.max(s));
        let mut out: Option<(Vec3, Vec3)> = None;
        for (p, v) in positions.zip(&self.vertices) {
            let anchor = Vec3::new(v.leaf[0], v.leaf[1], v.leaf[2]);
            for q in [p, anchor + (p - anchor) * largest] {
                out = Some(out.map_or((q, q), |(lo, hi)| (lo.min(q), hi.max(q))));
            }
        }
        out
    }

    /// The bounds of its deformation over `positions` (the mesh's), each
    /// leaf also as large as its largest level draws it (a radius about a
    /// point is largest at one end of the way a vertex grows).
    pub fn bounds(&self, positions: impl Iterator<Item = Vec3>) -> crate::foliage::FoliageBounds {
        let mut b = crate::foliage::FoliageBounds::default();
        let largest = self.level_scales.iter().fold(1.0f32, |a, &s| a.max(s));
        for (p, v) in positions.zip(&self.vertices) {
            b.include(p, *v);
            if largest > 1.0 {
                let anchor = Vec3::new(v.leaf[0], v.leaf[1], v.leaf[2]);
                b.include(anchor + (p - anchor) * largest, *v);
            }
        }
        b
    }
}

/// How an instance's transform carries a cluster's bounds: the largest
/// axis scale (a sphere's radius grows by it), and whether the cone
/// survives (an even scale and no mirroring).
pub fn transform_scale(m: Mat4) -> (f32, bool) {
    let (a, b, c) = (
        m.x_axis.truncate(),
        m.y_axis.truncate(),
        m.z_axis.truncate(),
    );
    let (la, lb, lc) = (a.length(), b.length(), c.length());
    let max = la.max(lb).max(lc);
    let even = (la - lb).abs() <= 1e-4 * max && (la - lc).abs() <= 1e-4 * max;
    let mirrored = a.dot(b.cross(c)) < 0.0;
    (max, even && !mirrored)
}

/// Whether a sphere lies wholly outside one of `planes` (`n·p + w >= 0`
/// inside, normals unnormalised: the distance is scaled by their length).
pub fn outside(planes: &[Vec4], center: Vec3, radius: f32) -> bool {
    planes
        .iter()
        .any(|p| p.truncate().dot(center) + p.w < -radius * p.truncate().length())
}

/// Whether every triangle of a cluster at `center` (world) with `radius`
/// and its cone (`axis` world, normalised) faces away from `eye`
/// (meshoptimizer's test: the apex-free form, conservative by the radius).
pub fn backfacing(center: Vec3, radius: f32, axis: Vec3, cutoff: f32, eye: Vec3) -> bool {
    if cutoff >= 1.0 {
        return false;
    }
    let to = center - eye;
    to.dot(axis) >= cutoff * to.length() + radius
}

/// A sphere on the screen: its pixel rectangle (y down) and its nearest
/// depth (0 near, 1 far), as the GPU's occlusion test reads the pyramid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenRect {
    pub lo: Vec2,
    pub hi: Vec2,
    pub near: f32,
}

impl ScreenRect {
    /// The rectangle of the box about a sphere (`center` world, `radius`)
    /// through `view_proj` on a `size` screen; `None` when a corner is
    /// behind the eye (never occluded).
    pub fn of_sphere(view_proj: Mat4, center: Vec3, radius: f32, size: Vec2) -> Option<Self> {
        let mut r = ScreenRect {
            lo: Vec2::splat(f32::MAX),
            hi: Vec2::splat(f32::MIN),
            near: f32::MAX,
        };
        for i in 0..8 {
            let corner = center
                + Vec3::new(
                    if i & 1 != 0 { radius } else { -radius },
                    if i & 2 != 0 { radius } else { -radius },
                    if i & 4 != 0 { radius } else { -radius },
                );
            let clip = view_proj * corner.extend(1.0);
            if clip.w <= 1e-6 {
                return None;
            }
            let ndc = clip.truncate() / clip.w;
            let px = Vec2::new(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * size;
            r.lo = r.lo.min(px);
            r.hi = r.hi.max(px);
            r.near = r.near.min(ndc.z);
        }
        Some(r)
    }

    /// The pixels it covers with one of margin, clamped to the screen:
    /// (x0, y0, x1, y1), inclusive.
    pub fn texels(&self, size: Vec2) -> [u32; 4] {
        let clamp = |v: f32, s: f32| v.clamp(0.0, s - 1.0) as u32;
        [
            clamp((self.lo.x).floor() - 1.0, size.x),
            clamp((self.lo.y).floor() - 1.0, size.y),
            clamp((self.hi.x).floor() + 1.0, size.x),
            clamp((self.hi.y).floor() + 1.0, size.y),
        ]
    }
}

/// The pyramid level whose texels are at least `texels` pixels wide, so at
/// most 2 x 2 of them cover a rectangle that wide (`cull.wgsl`'s
/// `occluded`); `None` when the pyramid has no such level.
pub fn pyramid_level(texels: u32, levels: u32) -> Option<u32> {
    let level = texels.max(1).next_power_of_two().trailing_zeros();
    (level < levels).then_some(level)
}

/// Whether a rectangle is behind the pyramid everywhere it covers:
/// `farthest(level, x, y)` reads a texel. Conservative: a rectangle
/// reaching behind the eye or with no level is visible.
pub fn occluded(
    rect: Option<ScreenRect>,
    size: (u32, u32),
    levels: u32,
    farthest: impl Fn(u32, u32, u32) -> f32,
) -> bool {
    let Some(r) = rect else { return false };
    if r.near <= 0.0 {
        return false;
    }
    let [x0, y0, x1, y1] = r.texels(Vec2::new(size.0 as f32, size.1 as f32));
    let Some(level) = pyramid_level((x1 - x0).max(y1 - y0) + 1, levels) else {
        return false;
    };
    let dims = ((size.0 >> level).max(1), (size.1 >> level).max(1));
    let mut far = 0.0f32;
    for y in (y0 >> level).min(dims.1 - 1)..=(y1 >> level).min(dims.1 - 1) {
        for x in (x0 >> level).min(dims.0 - 1)..=(x1 >> level).min(dims.0 - 1) {
            far = far.max(farthest(level, x, y));
        }
    }
    r.near > far + 1e-6
}

// The visibility buffer (R1 batch 4, three.js#35191's
// `VisibilityBufferDrawingPipeline`): the camera's visible clusters of
// opaque materials whose clusters are small on the screen ([`soft`]) are
// rasterised in compute, one thread a triangle ([`raster_class`],
// [`covers`]), into two words per pixel, the depth and the id of the
// triangle seen there; a resolve per batch shades each of its pixels once
// from them, writing its depth, so the hardware's clusters (the larger
// ones, drawn as before) and the resolved pixels meet by the depth test.
// The rasteriser first takes the nearest depth ([`vis_depth`], an atomic
// max), then, in a second pass, writes its id where its depth is the
// pixel's ([`vis_id`], an atomic max again, so equal depths keep one id):
// WebGPU has no 64-bit atomics to take both at once. three.js decides per
// triangle (up to 8 pixels in compute); deciding per cluster keeps the
// hardware's clusters one indirect draw each, as before.

/// The most pixels across a cluster's sphere may cover to be rasterised
/// in compute ([`soft`]): 64 triangles over it are a few pixels each.
pub const SOFT_PIXELS: f32 = 24.0;

/// The widest and tallest triangle (its box, in pixels) the compute
/// rasteriser covers; a soft cluster's triangles stay well inside it
/// (its sphere's projection, stretched at most a little off the axis).
pub const RASTER_MAX: f32 = 64.0;

/// Whether a cluster is rasterised in compute: its sphere (radius
/// `radius`, its centre `depth` in front of the eye along the view axis,
/// a perspective view's clip w) lies wholly past the `near` plane, and
/// covers at most [`SOFT_PIXELS`] across at `pixels_per_unit` (pixels per
/// unit of length at distance 1).
pub fn soft(depth: f32, radius: f32, pixels_per_unit: f32, near: f32) -> bool {
    let d = depth - radius;
    d > near && 2.0 * radius * pixels_per_unit <= SOFT_PIXELS * d
}

/// Subpixel steps per pixel a rasterised vertex snaps to (a hardware
/// rasteriser's 8 bits): with vertices taken relative to the triangle's
/// box, the edge functions of a triangle up to [`RASTER_MAX`] across are
/// exact in 32-bit integers (64 x 256 squared is under 2^31).
pub const SUBPIXEL: i32 = 256;

/// A depth's word in the visibility buffer: nearer is larger (an atomic
/// max keeps the nearest), and 0 is a pixel nothing covered.
pub fn vis_depth(z: f32) -> u32 {
    !z.max(0.0).to_bits()
}

/// The depth a [`vis_depth`] word holds.
pub fn vis_depth_value(word: u32) -> f32 {
    f32::from_bits(!word)
}

/// A pixel's id word: a work item (an instance's cluster) and the
/// triangle in its cluster, 0 for none.
pub fn vis_id(item: u32, triangle: u32) -> u32 {
    debug_assert!(item < 1 << 26 && (triangle as usize) < CLUSTER_TRIANGLES);
    ((item << 6) | triangle) + 1
}

/// The (item, triangle) a [`vis_id`] word names.
pub fn vis_item(word: u32) -> Option<(u32, u32)> {
    (word != 0).then(|| ((word - 1) >> 6, (word - 1) & 63))
}

/// A clip position's pixel position (x right, y down, in pixels) and
/// depth, for a `size` pixels screen; `None` behind the eye or before the
/// near plane (the hardware clips those triangles).
pub fn to_screen(clip: Vec4, size: Vec2) -> Option<Vec3> {
    if clip.w <= 1e-6 || clip.z < 0.0 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    Some(Vec3::new(
        (ndc.x * 0.5 + 0.5) * size.x,
        (0.5 - ndc.y * 0.5) * size.y,
        ndc.z,
    ))
}

/// What becomes of a triangle in the visibility pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Raster {
    /// Covers no pixel centre, or faces the way its side culls.
    Skip,
    /// Too large, or crossing the near plane: not the compute
    /// rasteriser's ([`soft`] keeps those clusters on the hardware).
    Hardware,
    /// Rasterised in compute over these pixels (inclusive).
    Pixels { x0: i32, y0: i32, x1: i32, y1: i32 },
}

/// A triangle's fate ([`Raster`]) from its corners' clip positions, the
/// screen's `size` and the side its material draws.
pub fn raster_class(clip: [Vec4; 3], size: Vec2, side: crate::Side) -> Raster {
    let (Some(a), Some(b), Some(c)) = (
        to_screen(clip[0], size),
        to_screen(clip[1], size),
        to_screen(clip[2], size),
    ) else {
        return Raster::Hardware;
    };
    // Counter-clockwise as seen is the front (wgpu's default): a negative
    // signed area with the screen's y down.
    let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let front = area < 0.0;
    let culled = match side {
        crate::Side::Front => !front,
        crate::Side::Back => front,
        crate::Side::Double => false,
    };
    if area == 0.0 || culled {
        return Raster::Skip;
    }
    let lo = a.min(b).min(c);
    let hi = a.max(b).max(c);
    // Entirely before the screen or past the far plane.
    if lo.z > 1.0 || hi.x < 0.0 || hi.y < 0.0 || lo.x > size.x || lo.y > size.y {
        return Raster::Skip;
    }
    if hi.x - lo.x > RASTER_MAX || hi.y - lo.y > RASTER_MAX {
        return Raster::Hardware;
    }
    // The pixel centres in the box, on the screen.
    let x0 = ((lo.x - 0.5).ceil() as i32).max(0);
    let y0 = ((lo.y - 0.5).ceil() as i32).max(0);
    let x1 = ((hi.x - 0.5).floor() as i32).min(size.x as i32 - 1);
    let y1 = ((hi.y - 0.5).floor() as i32).min(size.y as i32 - 1);
    if x0 > x1 || y0 > y1 {
        return Raster::Skip;
    }
    Raster::Pixels { x0, y0, x1, y1 }
}

/// Whether the edge from `a` to `b` of a positively wound triangle owns
/// the pixel centres on it: of two triangles sharing an edge (walked in
/// opposite directions), exactly one does.
fn owns(a: [i32; 2], b: [i32; 2]) -> bool {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    dy < 0 || (dy == 0 && dx > 0)
}

/// `p`'s side of the edge from `a` to `b` (positive inside a positively
/// wound triangle).
fn edge(a: [i32; 2], b: [i32; 2], p: [i32; 2]) -> i32 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}

/// The depth at pixel `(x, y)` of a triangle given in screen space ([`to_screen`]:
/// pixels and depth), or `None` when the pixel's centre is outside it.
/// Its vertices snap to [`SUBPIXEL`] steps relative to the box's whole
/// pixel corner, so its edge functions are exact and two triangles sharing
/// an edge cover each centre on it once ([`owns`]).
pub fn covers(p: [Vec3; 3], x: i32, y: i32) -> Option<f32> {
    let ox = p[0].x.min(p[1].x).min(p[2].x).floor();
    let oy = p[0].y.min(p[1].y).min(p[2].y).floor();
    let snap = |v: Vec3| {
        [
            ((v.x - ox) * SUBPIXEL as f32).round() as i32,
            ((v.y - oy) * SUBPIXEL as f32).round() as i32,
        ]
    };
    let mut v = [snap(p[0]), snap(p[1]), snap(p[2])];
    let mut z = [p[0].z, p[1].z, p[2].z];
    let mut area = edge(v[0], v[1], v[2]);
    if area == 0 {
        return None;
    }
    if area < 0 {
        v.swap(1, 2);
        z.swap(1, 2);
        area = -area;
    }
    let half = SUBPIXEL / 2;
    let c = [
        (x - ox as i32) * SUBPIXEL + half,
        (y - oy as i32) * SUBPIXEL + half,
    ];
    let e = [
        edge(v[1], v[2], c),
        edge(v[2], v[0], c),
        edge(v[0], v[1], c),
    ];
    let inside = |e: i32, a: [i32; 2], b: [i32; 2]| e > 0 || (e == 0 && owns(a, b));
    if !(inside(e[0], v[1], v[2]) && inside(e[1], v[2], v[0]) && inside(e[2], v[0], v[1])) {
        return None;
    }
    let depth = (e[0] as f32 * z[0] + e[1] as f32 * z[1] + e[2] as f32 * z[2]) / area as f32;
    (0.0..=1.0).contains(&depth).then_some(depth)
}

/// The perspective-correct barycentric weights of a pixel at `ndc` on the
/// triangle of clip positions `clip`: what the hardware interpolates a
/// varying with (the resolve rebuilds a pixel's attributes with them).
pub fn barycentrics(clip: [Vec4; 3], ndc: Vec2) -> Vec3 {
    // The screen weights, then each over its corner's w.
    let q = screen_weights(clip, ndc) / Vec3::new(clip[0].w, clip[1].w, clip[2].w);
    q / (q.x + q.y + q.z)
}

/// The screen-linear barycentric weights of a pixel at `ndc` on the
/// triangle of clip positions `clip`: what affine texturing (retro 3D,
/// pbr.wgsl `stylised_uv`) interpolates uv with, the hardware's
/// perspective-correct ratio of `uv w` over `w` being exactly these.
pub fn screen_weights(clip: [Vec4; 3], ndc: Vec2) -> Vec3 {
    let p = clip.map(|c| c.truncate().truncate() / c.w);
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let area = cross(p[1] - p[0], p[2] - p[0]);
    Vec3::new(
        cross(p[1] - ndc, p[2] - ndc),
        cross(p[2] - ndc, p[0] - ndc),
        cross(p[0] - ndc, p[1] - ndc),
    ) / area
}

/// Retro 3D's vertex (pbr.wgsl `retro_clip`, which the compute
/// rasteriser repeats): the clip position snapped to the frame's grid of
/// `stylise.xy` cells and its depth quantised to `stylise.w` bits
/// ([`crate::Retro3d::uniform`]); the identity when both are 0.
pub fn retro_clip(clip: Vec4, stylise: [f32; 4]) -> Vec4 {
    let mut c = clip;
    if c.w <= 0.0 {
        return c;
    }
    if stylise[0] > 0.0 {
        let half = Vec2::new(stylise[0], stylise[1]) * 0.5;
        let ndc = Vec2::new(c.x, c.y) / c.w;
        let snapped = (ndc * half).round() / half * c.w;
        c = Vec4::new(snapped.x, snapped.y, c.z, c.w);
    }
    if stylise[3] > 0.0 {
        let steps = stylise[3].exp2();
        c.z = (c.z / c.w * steps).floor() / steps * c.w;
    }
    c
}
