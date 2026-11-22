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

/// A light's record in the light tables: five vec4s (frame.wgsl
/// `LocalLight`).
pub const LIGHT_VEC4S: usize = 5;
/// Where the light tables' parts start, in bytes: the lights, then the
/// cluster table (one word per cluster), then the references (one byte
/// each, four to a word); and their end.
pub const LIGHT_TABLES_AT: [u64; 4] = {
    let table = (MAX_LIGHTS * LIGHT_VEC4S * 16) as u64;
    let refs = table + CLUSTERS as u64 * 4;
    [0, table, refs, refs + LIGHT_REFS as u64]
};
const _: () = assert!(LIGHT_TABLES_AT[1] % 16 == 0 && LIGHT_TABLES_AT[2] % 16 == 0);

/// The WGSL declaring the light tables, one read-only storage buffer
/// (group 0, binding 7: [`LIGHT_TABLES_AT`]), and their accessors, placed
/// after frame.wgsl in the lit shader: the lights as [`MAX_LIGHTS`] records
/// of [`LIGHT_VEC4S`] vec4s, the table as one word per cluster (the first
/// reference, 24 bits, and the count, 8 bits), the references one byte
/// each, four to a word. One buffer, as WebGPU's compatibility mode allows
/// a fragment stage four storage buffers.
pub fn wgsl() -> String {
    let decls = format!(
        "@group(0) @binding(7) var<storage, read> light_tables: array<vec4<u32>, {vec4s}>;

fn light_at(i: u32) -> LocalLight {{
  let b = i * {per}u;
  return LocalLight(
    bitcast<vec4<f32>>(light_tables[b]),
    bitcast<vec4<f32>>(light_tables[b + 1u]),
    bitcast<vec4<f32>>(light_tables[b + 2u]),
    bitcast<vec4<f32>>(light_tables[b + 3u]),
    bitcast<vec4<f32>>(light_tables[b + 4u]),
  );
}}

fn light_word(w: u32) -> u32 {{
  return light_tables[w >> 2u][w & 3u];
}}

fn cluster_word(c: u32) -> u32 {{
  return light_word({table}u + c);
}}

fn cluster_ref(j: u32) -> u32 {{
  return (light_word({refs}u + (j >> 2u)) >> ((j & 3u) * 8u)) & 255u;
}}
",
        vec4s = LIGHT_TABLES_AT[3] / 16,
        per = LIGHT_VEC4S,
        table = LIGHT_TABLES_AT[1] / 4,
        refs = LIGHT_TABLES_AT[2] / 4,
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
