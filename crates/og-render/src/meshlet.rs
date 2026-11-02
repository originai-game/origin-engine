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

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster(first: u32, triangles: u32) -> Cluster {
        Cluster {
            first,
            triangles,
            cone_cutoff: 1.0,
            ..Default::default()
        }
    }

    #[test]
    fn levels_must_be_covered_by_their_clusters_in_order() {
        let mut m = MeshClusters {
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
                    error: 0.1,
                    clusters: 2..3,
                },
            ],
            clusters: vec![cluster(0, 64), cluster(192, 36), cluster(300, 10)],
        };
        assert_eq!(m.validate(330), Ok(()));
        // An index count short of the last level.
        assert!(m.validate(329).is_err());
        // A gap between two clusters.
        m.clusters[1].first = 195;
        assert!(m.validate(330).is_err());
        m.clusters[1].first = 192;
        // A cluster past the limit.
        m.clusters[0].triangles = 65;
        m.clusters[1] = cluster(195, 35);
        assert!(m.validate(330).is_err());
    }

    #[test]
    fn spheres_outside_a_plane_by_more_than_their_radius_are_culled() {
        // x >= 0, with an unnormalised normal (length 2).
        let planes = [Vec4::new(2.0, 0.0, 0.0, 0.0)];
        assert!(!outside(&planes, Vec3::new(-0.9, 0.0, 0.0), 1.0));
        assert!(outside(&planes, Vec3::new(-1.1, 0.0, 0.0), 1.0));
        assert!(!outside(&planes, Vec3::new(5.0, 0.0, 0.0), 1.0));
    }

    #[test]
    fn a_cone_culls_only_when_every_facing_is_away() {
        // A cluster at the origin facing +z within 30° (cutoff sin 30°: the
        // triangles face away from any eye within 60° of -z).
        let axis = Vec3::Z;
        let cutoff = 0.5;
        // An eye straight behind it (on -z) sees only back faces.
        assert!(backfacing(
            Vec3::ZERO,
            0.1,
            axis,
            cutoff,
            Vec3::new(0.0, 0.0, -10.0)
        ));
        // In front of it: front faces.
        assert!(!backfacing(
            Vec3::ZERO,
            0.1,
            axis,
            cutoff,
            Vec3::new(0.0, 0.0, 10.0)
        ));
        // From the side (90°): some triangles may face it.
        assert!(!backfacing(
            Vec3::ZERO,
            0.1,
            axis,
            cutoff,
            Vec3::new(10.0, 0.0, 0.0)
        ));
        // 55° off -z is inside 60°, but a large radius close by keeps it.
        let eye = Vec3::new(55f32.to_radians().sin(), 0.0, -55f32.to_radians().cos()) * 10.0;
        assert!(backfacing(Vec3::ZERO, 0.1, axis, cutoff, eye));
        assert!(!backfacing(Vec3::ZERO, 2.0, axis, cutoff, eye));
        // A cone too wide to cull.
        assert!(!backfacing(
            Vec3::ZERO,
            0.1,
            axis,
            1.0,
            Vec3::new(0.0, 0.0, -10.0)
        ));
    }

    #[test]
    fn only_even_unmirrored_scales_keep_the_cone() {
        let (s, keep) = transform_scale(Mat4::from_scale(Vec3::splat(2.0)));
        assert_eq!((s, keep), (2.0, true));
        let (s, keep) = transform_scale(Mat4::from_scale(Vec3::new(1.0, 3.0, 1.0)));
        assert_eq!((s, keep), (3.0, false));
        let (_, keep) = transform_scale(Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0)));
        assert!(!keep);
    }

    #[test]
    fn the_pyramid_level_holds_the_rectangle_in_two_by_two_texels() {
        assert_eq!(pyramid_level(1, 10), Some(0));
        assert_eq!(pyramid_level(2, 10), Some(1));
        assert_eq!(pyramid_level(3, 10), Some(2));
        assert_eq!(pyramid_level(64, 10), Some(6));
        assert_eq!(pyramid_level(65, 10), Some(7));
        assert_eq!(pyramid_level(1024, 10), None);
    }

    #[test]
    fn a_sphere_behind_a_near_wall_is_occluded_and_one_in_front_is_not() {
        let size = (256u32, 256u32);
        // A 90° perspective with depth 0 to 1 (wgpu's), the eye at the
        // origin looking down -z.
        let (n, f) = (0.1f32, 100.0f32);
        let vp = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, f / (n - f), -1.0),
            Vec4::new(0.0, 0.0, n * f / (n - f), 0.0),
        );
        let s = Vec2::new(256.0, 256.0);
        // A wall at 5 m covering the screen: its depth everywhere.
        let wall = (vp * Vec4::new(0.0, 0.0, -5.0, 1.0)).z / 5.0;
        let behind = ScreenRect::of_sphere(vp, Vec3::new(0.0, 0.0, -20.0), 1.0, s);
        let front = ScreenRect::of_sphere(vp, Vec3::new(0.0, 0.0, -3.0), 0.5, s);
        assert!(occluded(behind, size, 9, |_, _, _| wall));
        assert!(!occluded(front, size, 9, |_, _, _| wall));
        // The wall with a hole the sphere's size where it is: visible. A
        // single far texel inside the rectangle is enough.
        let r = behind.unwrap();
        let [x0, y0, x1, y1] = r.texels(s);
        let level = pyramid_level((x1 - x0).max(y1 - y0) + 1, 9).unwrap();
        let hole = (((x0 + x1) / 2) >> level, ((y0 + y1) / 2) >> level);
        assert!(!occluded(behind, size, 9, |l, x, y| {
            if l == level && (x, y) == hole {
                1.0
            } else {
                wall
            }
        }));
        // Reaching behind the eye: never occluded.
        assert_eq!(
            ScreenRect::of_sphere(vp, Vec3::new(0.0, 0.0, 0.5), 1.0, s),
            None
        );
        assert!(!occluded(None, size, 9, |_, _, _| 0.0));
    }
}
