// Scatter culling (scatter_cull.rs): each visible chunk's copies tested one
// by one against the main camera before its prepass and lit draws, the
// survivors compacted, in order, into the frame's copy list and counted
// into the chunk's two indirect draws (the mesh's, the impostor's).
//
// A copy goes when it stands past the scatter's distance along x and z,
// when its bounding sphere (the mesh's radius times its scale, grown by
// the furthest the wind and the trample move it) is outside the view, when
// the frame's depth pyramid (cull_gpu.rs, its occluders' farthest depth)
// covers it everywhere it would draw, or when the scatter's density map
// is below the copy's own value at its place. Shadow passes draw the
// chunks' whole lists: a copy the camera does not see may still cast.
//
// One workgroup per chunk of at most 1024 copies (scatter.rs CHUNK_CELLS
// squared): each lane takes four consecutive copies, a prefix sum over the
// lanes gives each its place, so the list keeps the chunk's order on
// every run.

// scatter_cull.rs ViewGpu.
struct CullView {
  view_proj: mat4x4<f32>,       // the jittered view the pyramid was drawn with
  planes: array<vec4<f32>, 6>,  // inside: dot(xyz, p) + w >= 0
  camera: vec4<f32>,            // xyz: the camera (frame coordinates)
  sizes: vec4<u32>,             // x, y: the pyramid's base (the frame's pixels), z: its levels (0: no occlusion)
};

// scatter_cull.rs ItemGpu: one chunk.
struct CullItem {
  to_terrain: mat4x4<f32>,  // the terrain's space from the frame's (its density map's uv)
  item: vec4<u32>,          // x: the chunk's first place in the list, y: its copies, z: its draws' row, w: 1 with a density map
  mesh: vec4<u32>,          // x: the mesh's index count
  shape: vec4<f32>,         // x: the mesh's radius about its origin, y: how far a copy moves at most (m), z: the scatter's distance (m)
  extent: vec4<f32>,        // xy: the terrain's size (its space)
};

@group(0) @binding(0) var<uniform> view: CullView;
@group(0) @binding(1) var<uniform> item: CullItem;
@group(0) @binding(2) var<storage, read_write> list: array<vec4<f32>>;
// Per chunk 10 words: an indexed indirect draw (index count, instances,
// first index, base vertex, first instance), then a plain one (vertices,
// instances, first vertex, first instance), then a pad.
@group(0) @binding(3) var<storage, read_write> draws: array<u32>;
@group(0) @binding(4) var pyramid: texture_2d<f32>;
// The chunk's copies (instances.rs GpuTransform, four vectors each).
@group(1) @binding(0) var<storage, read> copies: array<vec4<f32>>;
@group(2) @binding(0) var density_map: texture_2d<f32>;
@group(2) @binding(1) var density_sampler: sampler;

const CULL_LANES: u32 = 256u;
const CULL_PER_LANE: u32 = 4u;

fn cull_hash(x: u32) -> u32 {
  var h = x;
  h ^= h >> 16u;
  h *= 0x7feb352du;
  h ^= h >> 15u;
  h *= 0x846ca68bu;
  return h ^ (h >> 16u);
}

fn cull_outside(c: vec3<f32>, r: f32) -> bool {
  for (var i = 0; i < 6; i++) {
    if (dot(view.planes[i].xyz, c) + view.planes[i].w < -r) { return true; }
  }
  return false;
}

// meshlet_cull.wgsl `occluded`: the box about the sphere behind the
// pyramid everywhere it covers.
fn cull_occluded(c: vec3<f32>, r: f32) -> bool {
  if (view.sizes.z == 0u) { return false; }
  let size = vec2<f32>(view.sizes.xy);
  var lo = vec2<f32>(1e30);
  var hi = vec2<f32>(-1e30);
  var near = 1e30;
  for (var i = 0u; i < 8u; i++) {
    let corner = c + r * vec3<f32>(
      select(-1.0, 1.0, (i & 1u) != 0u),
      select(-1.0, 1.0, (i & 2u) != 0u),
      select(-1.0, 1.0, (i & 4u) != 0u));
    let clip = view.view_proj * vec4<f32>(corner, 1.0);
    if (clip.w <= 1e-6) { return false; }
    let ndc = clip.xyz / clip.w;
    let px = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * size;
    lo = min(lo, px);
    hi = max(hi, px);
    near = min(near, ndc.z);
  }
  if (near <= 0.0) { return false; }
  let x0 = u32(clamp(floor(lo.x) - 1.0, 0.0, size.x - 1.0));
  let y0 = u32(clamp(floor(lo.y) - 1.0, 0.0, size.y - 1.0));
  let x1 = u32(clamp(floor(hi.x) + 1.0, 0.0, size.x - 1.0));
  let y1 = u32(clamp(floor(hi.y) + 1.0, 0.0, size.y - 1.0));
  let texels = max(x1 - x0, y1 - y0) + 1u;
  var level = 0u;
  while ((1u << level) < texels) { level++; }
  if (level >= view.sizes.z) { return false; }
  let dims = max(vec2<u32>(1u), view.sizes.xy >> vec2<u32>(level));
  let t0 = min(vec2<u32>(x0, y0) >> vec2<u32>(level), dims - 1u);
  let t1 = min(vec2<u32>(x1, y1) >> vec2<u32>(level), dims - 1u);
  var farthest = 0.0;
  for (var y = t0.y; y <= t1.y; y++) {
    for (var x = t0.x; x <= t1.x; x++) {
      farthest = max(farthest, textureLoad(pyramid, vec2<u32>(x, y), level).r);
    }
  }
  return near > farthest + 1e-6;
}

// Whether copy `k` of the chunk draws for the camera.
fn cull_keep(k: u32) -> bool {
  let v0 = copies[k * 4u];
  let v2 = copies[k * 4u + 2u];
  let at = v2.yzw;
  let scale = length(v0.xyz);
  let flat = at.xz - view.camera.xz;
  if (dot(flat, flat) > item.shape.z * item.shape.z) { return false; }
  let r = item.shape.x * scale + item.shape.y;
  if (cull_outside(at, r)) { return false; }
  if (item.item.w == 1u) {
    let local = (item.to_terrain * vec4<f32>(at, 1.0)).xyz;
    let uv = local.xz / item.extent.xy + 0.5;
    let q = vec2<i32>(round(local.xz * 64.0));
    let own = f32(cull_hash(bitcast<u32>(q.x) ^ cull_hash(bitcast<u32>(q.y) + 0x9e3779b9u)) >> 8u) / 16777216.0;
    if (own >= textureSampleLevel(density_map, density_sampler, uv, 0.0).r) { return false; }
  }
  return !cull_occluded(at, r);
}

var<workgroup> lane_counts: array<u32, CULL_LANES>;

@compute @workgroup_size(256)
fn cull(@builtin(local_invocation_index) lane: u32) {
  let n = item.item.y;
  var kept = array<bool, CULL_PER_LANE>(false, false, false, false);
  var mine = 0u;
  for (var j = 0u; j < CULL_PER_LANE; j++) {
    let k = lane * CULL_PER_LANE + j;
    if (k < n && cull_keep(k)) {
      kept[j] = true;
      mine += 1u;
    }
  }
  lane_counts[lane] = mine;
  workgroupBarrier();
  // An inclusive scan over the lanes (Hillis and Steele).
  for (var step = 1u; step < CULL_LANES; step <<= 1u) {
    var add = 0u;
    if (lane >= step) { add = lane_counts[lane - step]; }
    workgroupBarrier();
    lane_counts[lane] += add;
    workgroupBarrier();
  }
  var at = item.item.x + lane_counts[lane] - mine;
  for (var j = 0u; j < CULL_PER_LANE; j++) {
    if (kept[j]) {
      let k = lane * CULL_PER_LANE + j;
      for (var c = 0u; c < 4u; c++) {
        list[at * 4u + c] = copies[k * 4u + c];
      }
      at += 1u;
    }
  }
  if (lane == CULL_LANES - 1u) {
    let total = lane_counts[lane];
    let row = item.item.z * 10u;
    draws[row] = item.mesh.x;
    draws[row + 1u] = total;
    draws[row + 2u] = 0u;
    draws[row + 3u] = 0u;
    draws[row + 4u] = 0u;
    draws[row + 5u] = 6u;
    draws[row + 6u] = total;
    draws[row + 7u] = 0u;
    draws[row + 8u] = 0u;
    draws[row + 9u] = 0u;
  }
}
