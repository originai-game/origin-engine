/// The most local lights one frame draws, after those outside the view are
/// culled: past it the least important are dropped (the radiance at the
/// camera; `clusters.rs`) and counted in [`crate::FrameStats::lights_dropped`].
pub const MAX_LIGHTS: usize = 256;

/// Clusters across, down and in depth.
pub const CLUSTER_GRID: [u32; 3] = [16, 9, 24];
/// The number of clusters.
pub const CLUSTERS: usize = (CLUSTER_GRID[0] * CLUSTER_GRID[1] * CLUSTER_GRID[2]) as usize;
/// The most lights one cluster shades.
pub const MAX_CLUSTER_LIGHTS: usize = 64;
/// The light references all clusters hold together (one byte each, four to
/// a word, in a storage buffer): every cluster full.
pub const LIGHT_REFS: usize = CLUSTERS * MAX_CLUSTER_LIGHTS;

/// The WGSL declaring the light tables' read-only storage buffers (group 0,
/// bindings 7 to 9) and their accessors, placed after frame.wgsl in the lit
/// shader: the lights as [`MAX_LIGHTS`] records of four vec4s, the table as
/// one word per cluster (the first reference, 24 bits, and the count, 8
/// bits), the references one byte each, four to a word.
pub fn wgsl() -> String {
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
