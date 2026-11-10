//! Forward+ clustered lighting (`docs/design.md` §2.6).
//!
//! The view is cut into [`CLUSTER_GRID`] clusters: 16 × 9 screen tiles,
//! each split into 24 slices of view depth (logarithmic between the
//! camera's near and far planes for a perspective camera, even for an
//! orthographic one). Every frame the CPU assigns each point and spot light
//! to the clusters its sphere of influence (`range`) can touch: the sphere's
//! view-space box, clipped to the near plane, projected to a screen
//! rectangle (widened by a pixel for the TAA jitter) and a depth range. The
//! lit shader finds its fragment's cluster from the pixel and the view depth
//! and shades only that cluster's lights, in scene order, so a light it
//! skips is one that adds exactly zero there and frames with the old
//! per-fragment loop's lights are unchanged.
//!
//! The CPU build was measured against a compute pass and kept: at 256
//! lights it takes tens of microseconds (`the former test sample`), less
//! than a compute dispatch's own overhead. The tables reach the shader as
//! read-only storage buffers.
//!
//! Limits, and how they degrade:
//! - Lights whose sphere misses the view are culled (`lights_culled`).
//! - At most [`MAX_LIGHTS`] visible lights are drawn: past that the ones
//!   with the most radiance at the camera are kept (intensity × the
//!   brightest channel of the colour over the squared distance from the
//!   camera, at least 1 m), the rest counted in `lights_dropped`.
//! - A cluster holds at most [`MAX_CLUSTER_LIGHTS`]. Past that cap a
//!   cluster keeps its most important lights (the same score) and loses the
//!   rest there only (`cluster_lights_dropped`).

use crate::frame::{CameraFrame, Frustum};
use crate::scene::{Light, LightKind, MAX_LIGHTS};
use glam::{Vec3, Vec4};

/// Clusters across, down and in depth.
pub const CLUSTER_GRID: [u32; 3] = [16, 9, 24];
/// The number of clusters.
pub(crate) const CLUSTERS: usize = (CLUSTER_GRID[0] * CLUSTER_GRID[1] * CLUSTER_GRID[2]) as usize;
/// The most lights one cluster shades.
pub const MAX_CLUSTER_LIGHTS: usize = 64;
/// The light references all clusters hold together (one byte each, four to
/// a word, in a storage buffer): every cluster full.
pub(crate) const LIGHT_REFS: usize = CLUSTERS * MAX_CLUSTER_LIGHTS;

/// Byte sizes of the three buffers: lights, the cluster table, the refs.
fn sizes() -> [u64; 3] {
    [
        (MAX_LIGHTS * std::mem::size_of::<crate::frame::LightGpu>()) as u64,
        (CLUSTERS * 4) as u64,
        LIGHT_REFS as u64,
    ]
}

/// The WGSL declaring the light tables' read-only storage buffers (group 0,
/// bindings 7 to 9) and their accessors, placed after frame.wgsl in the lit
/// shader: the lights as [`MAX_LIGHTS`] records of four vec4s, the table as
/// one word per cluster (the first reference, 24 bits, and the count, 8
/// bits), the references one byte each, four to a word.
pub(crate) fn wgsl() -> String {
    let decls = format!(
        "@group(0) @binding(7) var<storage, read> light_list: array<LocalLight, {lights}>;
@group(0) @binding(8) var<storage, read> cluster_table: array<u32, {table}>;
@group(0) @binding(9) var<storage, read> cluster_refs: array<u32, {refs}>;

fn light_at(i: u32) -> LocalLight {{
  return light_list[i];
}}

fn cluster_word(c: u32) -> u32 {{
  return cluster_table[c];
}}

fn cluster_ref(j: u32) -> u32 {{
  return (cluster_refs[j >> 2u] >> ((j & 3u) * 8u)) & 255u;
}}
",
        lights = MAX_LIGHTS,
        table = CLUSTERS,
        refs = LIGHT_REFS / 4,
    );
    format!(
        "// The local lights and their clusters (clusters.rs).
const CLUSTER_X: u32 = {gx}u;
const CLUSTER_Y: u32 = {gy}u;
const CLUSTER_Z: u32 = {gz}u;
{decls}
// The lights a fragment at pixel `frag` and world position `p` shades: x
// its cluster's first reference and y their count, or, in a frame without
// clusters (a reflection probe's capture, probes.rs), x NO_CLUSTER and y
// every light of the frame. Read the k-th with `frame_light`.
const NO_CLUSTER: u32 = 0xffffffffu;
// Whether a pipeline has the local lights' code (pipeline_features.rs).
override OG_LOCAL_LIGHTS: bool = true;
fn frame_lights(frag: vec2<f32>, p: vec3<f32>) -> vec2<u32> {{
  if (!OG_LOCAL_LIGHTS) {{ return vec2<u32>(0u); }}
  if (frame.params.z < 0.5) {{ return vec2<u32>(0u); }}
  if (frame.cluster.x > 0.0) {{
    let word = cluster_word(cluster_at(frag, p));
    return vec2<u32>(word & 0xffffffu, word >> 24u);
  }}
  return vec2<u32>(NO_CLUSTER, u32(frame.params.z));
}}

fn frame_light(lights: vec2<u32>, k: u32) -> LocalLight {{
  var i = k;
  if (lights.x != NO_CLUSTER) {{ i = cluster_ref(lights.x + k); }}
  return light_at(i);
}}

// The cluster of a fragment at pixel `frag` and world position `p`.
fn cluster_at(frag: vec2<f32>, p: vec3<f32>) -> u32 {{
  let x = min(u32(max(frag.x * frame.cluster.x, 0.0)), CLUSTER_X - 1u);
  let y = min(u32(max(frag.y * frame.cluster.y, 0.0)), CLUSTER_Y - 1u);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  var s = depth * frame.cluster.z + frame.cluster.w;
  if (frame.view_pos.w < 0.5) {{ s = log2(max(depth, 1e-6)) * frame.cluster.z + frame.cluster.w; }}
  let z = u32(clamp(s, 0.0, f32(CLUSTER_Z - 1u)));
  return (z * CLUSTER_Y + y) * CLUSTER_X + x;
}}
",
        gx = CLUSTER_GRID[0],
        gy = CLUSTER_GRID[1],
        gz = CLUSTER_GRID[2],
    )
}

/// The frame layout's entries for the three buffers.
pub(crate) fn layout_entries() -> [wgpu::BindGroupLayoutEntry; 3] {
    [7, 8, 9].map(|binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    })
}

/// The three buffers, at their fixed sizes.
pub(crate) fn buffers(device: &wgpu::Device) -> [wgpu::Buffer; 3] {
    let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
    let labels = ["lights", "cluster table", "cluster lights"];
    let sizes = sizes();
    [0, 1, 2].map(|i| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(labels[i]),
            size: sizes[i],
            usage,
            mapped_at_creation: false,
        })
    })
}

/// What the camera looks like to the cluster build.
pub(crate) struct ClusterView<'a> {
    pub camera: &'a CameraFrame,
    pub frustum: &'a Frustum,
    pub position: Vec3,
    pub near: f32,
    pub far: f32,
    pub width: u32,
    pub height: u32,
}

/// One frame's clusters.
#[derive(Default)]
pub(crate) struct ClusterStats {
    /// Lights outside the view.
    pub culled: u32,
    /// Visible lights past [`MAX_LIGHTS`].
    pub dropped: u32,
    /// Light references dropped from clusters over their cap.
    pub refs_dropped: u32,
    /// Light references stored.
    pub refs: u32,
}

/// The CPU cluster build. Its vectors are kept between frames.
#[derive(Default)]
pub(crate) struct ClusterBuilder {
    /// Scene indices of the lights drawn, in scene order (their records).
    pub kept: Vec<usize>,
    /// Per cluster: first reference | count << 24.
    pub table: Vec<u32>,
    /// One byte per reference.
    pub refs: Vec<u8>,
    /// The depth slices' near and far planes of the last build (what
    /// [`ClusterBuilder::bin_decals`] slices by).
    planes: (f32, f32),
    /// x: clusters across per pixel, y: down per pixel, z and w: the depth
    /// slice's scale and bias (of log2 depth for a perspective camera).
    pub params: [f32; 4],
    pub stats: ClusterStats,
    boxes: Vec<[u32; 6]>,
    order: Vec<usize>,
    total: Vec<u32>,
    count: Vec<u32>,
    fill: Vec<u32>,
    scores: Vec<f32>,
    /// A renderer made without rect lights (`LitFeatures::AREA_LIGHTS`)
    /// keeps none: no record, no shadow, no light anywhere.
    pub skip_rects: bool,
}

/// The light's weight when lights must be dropped.
pub(crate) fn importance(light: &Light, eye: Vec3) -> f32 {
    let d2 = (light.position - eye).length_squared().max(1.0);
    light.color.max_element().max(0.0) * light.intensity.max(0.0) / d2
}

impl ClusterBuilder {
    /// Assigns the lights to clusters.
    pub fn build(&mut self, lights: &[Light], view: &ClusterView) {
        self.stats = ClusterStats::default();
        self.select(lights, view);
        self.assign(lights, view);
    }

    /// Assigns `kept` (another view's [`ClusterBuilder::kept`], whose light
    /// records are shared) to this view's clusters: the planar reflection's
    /// mirrored camera shades the main view's lights.
    pub fn build_kept(&mut self, lights: &[Light], kept: &[usize], view: &ClusterView) {
        self.stats = ClusterStats::default();
        self.kept.clear();
        self.kept.extend_from_slice(kept);
        self.assign(lights, view);
    }

    /// The visible lights, then the most important [`MAX_LIGHTS`] of them.
    fn select(&mut self, lights: &[Light], view: &ClusterView) {
        self.kept.clear();
        for (i, l) in lights.iter().enumerate() {
            if self.skip_rects && matches!(l.kind, LightKind::Rect { .. }) {
                continue;
            }
            if view
                .frustum
                .intersects_sphere(l.position, l.range.max(1e-3))
            {
                self.kept.push(i);
            } else {
                self.stats.culled += 1;
            }
        }
        if self.kept.len() > MAX_LIGHTS {
            self.scores.clear();
            self.scores.extend(
                self.kept
                    .iter()
                    .map(|&i| importance(&lights[i], view.position)),
            );
            let mut by_score: Vec<usize> = (0..self.kept.len()).collect();
            og_core::sort::sort_by(&mut by_score, |&a, &b| {
                self.scores[b].total_cmp(&self.scores[a]).then(a.cmp(&b))
            });
            let mut keep = vec![false; self.kept.len()];
            for &k in &by_score[..MAX_LIGHTS] {
                keep[k] = true;
            }
            self.stats.dropped = (self.kept.len() - MAX_LIGHTS) as u32;
            let mut k = 0;
            self.kept.retain(|_| {
                k += 1;
                keep[k - 1]
            });
        }
    }

    /// The kept lights' clusters, capped, in the tables.
    fn assign(&mut self, lights: &[Light], view: &ClusterView) {
        let near = view.near.max(1e-4);
        let far = view.far.max(near * 1.0001);
        self.planes = (near, far);
        let gz = CLUSTER_GRID[2] as f32;
        let (scale, bias) = if view.camera.orthographic {
            let scale = gz / (far - near);
            (scale, -near * scale)
        } else {
            let scale = gz / (far / near).log2();
            (scale, -near.log2() * scale)
        };
        self.params = [
            CLUSTER_GRID[0] as f32 / view.width.max(1) as f32,
            CLUSTER_GRID[1] as f32 / view.height.max(1) as f32,
            scale,
            bias,
        ];

        // Each kept light's cluster box.
        self.boxes.clear();
        for &i in &self.kept {
            let l = &lights[i];
            let b = sphere_box(l.position, l.range, view, near, scale, bias);
            self.boxes.push(b);
        }

        // Count, cap, then fill in importance order.
        self.total.clear();
        self.total.resize(CLUSTERS, 0);
        for b in &self.boxes {
            for_each_cluster(b, |c| self.total[c] += 1);
        }
        // Every cluster at its cap fits the references' buffer (LIGHT_REFS).
        let cap = MAX_CLUSTER_LIGHTS as u32;
        self.count.clear();
        self.count.extend(self.total.iter().map(|&t| t.min(cap)));
        let overflow = self.total.iter().any(|&t| t > cap);
        self.table.clear();
        let mut offset = 0u32;
        for &n in &self.count {
            self.table.push(offset | (n << 24));
            offset += n;
        }
        self.stats.refs = offset;
        self.stats.refs_dropped = self.total.iter().map(|&t| t - t.min(cap)).sum();
        self.order.clear();
        self.order.extend(0..self.kept.len());
        if overflow {
            self.scores.clear();
            self.scores.extend(
                self.kept
                    .iter()
                    .map(|&i| importance(&lights[i], view.position)),
            );
            let scores = &self.scores;
            og_core::sort::sort_by(&mut self.order, |&a, &b| {
                scores[b].total_cmp(&scores[a]).then(a.cmp(&b))
            });
        }
        self.refs.clear();
        self.refs.resize(offset as usize, 0);
        self.fill.clear();
        self.fill.resize(CLUSTERS, 0);
        for &r in &self.order {
            let (refs, fill, count, table) =
                (&mut self.refs, &mut self.fill, &self.count, &self.table);
            for_each_cluster(&self.boxes[r], |c| {
                if fill[c] < count[c] {
                    refs[((table[c] & 0x00ff_ffff) + fill[c]) as usize] = r as u8;
                    fill[c] += 1;
                }
            });
        }
        if overflow {
            // Shade in scene order, as without the cap.
            for (c, &n) in self.count.iter().enumerate() {
                let first = (self.table[c] & 0x00ff_ffff) as usize;
                self.refs[first..first + n as usize].sort_unstable();
            }
        }
    }

    /// The references as little-endian words, padded to whole vec4s.
    pub fn ref_words(&self) -> Vec<u32> {
        let mut words: Vec<u32> = self
            .refs
            .chunks(4)
            .map(|c| {
                let mut b = [0u8; 4];
                b[..c.len()].copy_from_slice(c);
                u32::from_le_bytes(b)
            })
            .collect();
        words.resize(words.len().next_multiple_of(4).max(4), 0);
        words
    }
}

/// One frame's decals binned into the main view's clusters: `cells` is the
/// table (one word per cluster: the first reference, 24 bits, and the
/// count, 8 bits) followed by the references (one decal record index a
/// word, ascending within a cluster, so later decals blend over earlier).
#[derive(Default)]
pub(crate) struct DecalBins {
    pub cells: Vec<u32>,
    /// Decals outside the view.
    pub culled: u32,
    /// References dropped from clusters over [`crate::MAX_CLUSTER_DECALS`].
    pub refs_dropped: u32,
    boxes: Vec<(u32, [u32; 6])>,
    count: Vec<u32>,
    fill: Vec<u32>,
}

impl ClusterBuilder {
    /// Bins the decals' bounding spheres (centre, radius, in record order)
    /// into the clusters of the view this builder last built: a cluster
    /// over [`crate::MAX_CLUSTER_DECALS`] keeps its newest (highest index).
    pub fn bin_decals(&self, spheres: &[(Vec3, f32)], view: &ClusterView, out: &mut DecalBins) {
        let cap = crate::MAX_CLUSTER_DECALS as u32;
        out.culled = 0;
        out.refs_dropped = 0;
        out.boxes.clear();
        let (scale, bias) = (self.params[2], self.params[3]);
        for (i, &(c, r)) in spheres.iter().enumerate() {
            if view.frustum.intersects_sphere(c, r.max(1e-3)) {
                let b = sphere_box(c, r, view, self.planes.0.max(1e-4), scale, bias);
                out.boxes.push((i as u32, b));
            } else {
                out.culled += 1;
            }
        }
        out.count.clear();
        out.count.resize(CLUSTERS, 0);
        for (_, b) in &out.boxes {
            for_each_cluster(b, |c| out.count[c] += 1);
        }
        out.cells.clear();
        let mut offset = 0u32;
        for n in out.count.iter_mut() {
            out.refs_dropped += n.saturating_sub(cap);
            *n = (*n).min(cap);
            out.cells.push(offset | (*n << 24));
            offset += *n;
        }
        out.cells.resize(CLUSTERS + offset as usize, 0);
        // Newest first, each written from its cluster's end backwards.
        out.fill.clear();
        out.fill.resize(CLUSTERS, 0);
        let (cells, fill, count) = (&mut out.cells, &mut out.fill, &out.count);
        for (i, b) in out.boxes.iter().rev() {
            for_each_cluster(b, |c| {
                if fill[c] < count[c] {
                    fill[c] += 1;
                    let at = (cells[c] & 0x00ff_ffff) + count[c] - fill[c];
                    cells[CLUSTERS + at as usize] = *i;
                }
            });
        }
    }
}

/// Calls `f` with every cluster index in the box (x0, x1, y0, y1, z0, z1, inclusive).
fn for_each_cluster(b: &[u32; 6], mut f: impl FnMut(usize)) {
    let (gx, gy) = (CLUSTER_GRID[0], CLUSTER_GRID[1]);
    for z in b[4]..=b[5] {
        for y in b[2]..=b[3] {
            let row = (z * gy + y) * gx;
            for x in b[0]..=b[1] {
                f((row + x) as usize);
            }
        }
    }
}

/// The clusters a sphere (a light's range, a decal's box) can touch.
fn sphere_box(
    center: Vec3,
    radius: f32,
    view: &ClusterView,
    near: f32,
    scale: f32,
    bias: f32,
) -> [u32; 6] {
    let [gx, gy, gz] = CLUSTER_GRID;
    let r = radius.max(1e-3);
    let cam = view.camera;
    // Depth slices from the view depth of the sphere's extent.
    let d = (center - view.position).dot(cam.forward);
    let slice = |depth: f32| {
        let s = if cam.orthographic {
            depth * scale + bias
        } else {
            depth.max(1e-6).log2() * scale + bias
        };
        s.clamp(0.0, (gz - 1) as f32)
    };
    // A hair of margin: the shader's log2 may round the other way.
    let z0 = (slice(d - r) - 0.01).max(0.0) as u32;
    let z1 = ((slice(d + r) + 0.01) as u32).min(gz - 1);
    // The view-space box, clipped to the near plane, projected.
    let c = cam.view.transform_point3(center);
    let zmax = if cam.orthographic {
        c.z + r
    } else {
        (c.z + r).min(-near)
    };
    let zmin = c.z - r;
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    if zmin > zmax {
        return [0, gx - 1, 0, gy - 1, z0, z1];
    }
    for k in 0..8 {
        let p = Vec4::new(
            if k & 1 == 0 { c.x - r } else { c.x + r },
            if k & 2 == 0 { c.y - r } else { c.y + r },
            if k & 4 == 0 { zmin } else { zmax },
            1.0,
        );
        let q = cam.proj * p;
        let ndc = q.truncate() / q.w.max(1e-6);
        lo = lo.min(ndc);
        hi = hi.max(ndc);
    }
    // NDC to clusters, a pixel wider on each side (the TAA jitter).
    let (px, py) = (
        2.0 / view.width.max(1) as f32,
        2.0 / view.height.max(1) as f32,
    );
    let tile = |v: f32, n: u32| ((v * 0.5 + 0.5) * n as f32).clamp(0.0, (n - 1) as f32) as u32;
    let x0 = tile(lo.x - px, gx);
    let x1 = tile(hi.x + px, gx);
    // Pixel rows run down: NDC +y is row 0.
    let y0 = tile(-(hi.y + py), gy);
    let y1 = tile(-(lo.y - py), gy);
    [x0, x1, y0, y1, z0, z1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::camera_frame;
    use glam::Vec2;

    /// The cluster the lit shader's `cluster_at` finds for world point `p`.
    fn cluster_of(
        b: &ClusterBuilder,
        cam: &CameraFrame,
        eye: Vec3,
        size: (u32, u32),
        p: Vec3,
    ) -> usize {
        let q = cam.view_proj.project_point3(p);
        let frag = Vec2::new(
            (q.x * 0.5 + 0.5) * size.0 as f32,
            (0.5 - q.y * 0.5) * size.1 as f32,
        );
        let x = ((frag.x * b.params[0]) as u32).min(CLUSTER_GRID[0] - 1);
        let y = ((frag.y * b.params[1]) as u32).min(CLUSTER_GRID[1] - 1);
        let depth = (p - eye).dot(cam.forward);
        let s = depth.log2() * b.params[2] + b.params[3];
        let z = (s.clamp(0.0, (CLUSTER_GRID[2] - 1) as f32)) as u32;
        ((z * CLUSTER_GRID[1] + y) * CLUSTER_GRID[0] + x) as usize
    }

    /// Decals bin by their bounding spheres: one behind the camera is
    /// culled, one in view is found in the cluster the shader looks up for
    /// its centre, and a cluster past the cap keeps its newest decals, in
    /// ascending order (later ones blend over earlier ones).
    #[test]
    fn decals_bin_into_the_shaders_cluster_and_keep_the_newest() {
        let camera = crate::Camera::default();
        let size = (1280, 720);
        let cam = camera_frame(&camera, size.0 as f32 / size.1 as f32, Vec2::ZERO);
        let frustum = Frustum::from_view_proj(cam.view_proj);
        let view = ClusterView {
            camera: &cam,
            frustum: &frustum,
            position: camera.position,
            near: camera.near,
            far: camera.far,
            width: size.0,
            height: size.1,
        };
        let mut b = ClusterBuilder::default();
        b.build(&[], &view);
        let n = crate::MAX_CLUSTER_DECALS + 45;
        let mut spheres = vec![(Vec3::new(0.3, 0.2, 0.1), 0.05); n];
        spheres.push((Vec3::new(0.0, 2.0, 30.0), 1.0));
        spheres.push((Vec3::new(-2.0, 0.0, -1.0), 0.05));
        let mut bins = DecalBins::default();
        b.bin_decals(&spheres, &view, &mut bins);
        assert_eq!(bins.culled, 1);
        let refs = |c: usize| {
            let w = bins.cells[c];
            let first = CLUSTERS + (w & 0x00ff_ffff) as usize;
            bins.cells[first..first + (w >> 24) as usize].to_vec()
        };
        let here = refs(cluster_of(&b, &cam, camera.position, size, spheres[0].0));
        let kept: Vec<u32> = (45..n as u32).collect();
        assert_eq!(here, kept);
        let there = refs(cluster_of(
            &b,
            &cam,
            camera.position,
            size,
            spheres[n + 1].0,
        ));
        assert_eq!(there, vec![n as u32 + 1]);
        // Every cluster the crowd reaches dropped the same 45 oldest.
        let crowded = (0..CLUSTERS)
            .filter(|&c| refs(c).len() == kept.len())
            .count() as u32;
        assert!(crowded >= 1);
        assert_eq!(bins.refs_dropped, crowded * 45);
    }
}
