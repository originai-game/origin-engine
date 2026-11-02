// The meshlet path's cluster culling (meshlet.rs states each test on the
// CPU, meshlet_gpu.rs drives it). One workgroup takes one instance of a
// clustered batch, from the sorted instances GPU culling uploaded (region
// 0 of its buffer, so a work item keeps the instance's sorted slot, where
// its previous transform also is): it tests the instance's sphere against
// the frustum and GPU culling's depth pyramid, picks its level by
// screen-space error (lod.rs `select` without hysteresis), then its 64
// threads test the level's clusters (frustum, normal cone, the pyramid)
// and append the visible ones as work items (instance, cluster) to the
// batch's draw, whose instance count is the batch's visible clusters.

struct Params {
  view_proj: mat4x4<f32>,
  planes: array<vec4<f32>, 6>,
  // xyz: the eye; w: 1 for a perspective camera, 0 orthographic.
  eye: vec4<f32>,
  // x: pixels per unit of length at distance 1 (perspective: the view's
  // height over 2 tan(fov / 2); orthographic: its height over the view's
  // height in units), y: the scene's LOD bias.
  lod: vec4<f32>,
  // xy: the screen's size, z: the pyramid's levels (0: no occlusion
  // test), w: the batches.
  sizes: vec4<u32>,
};

// a: mesh slot, first instance (its sorted slot), instances, first work
// item. b: first workgroup, flags (1: the cone may cull).
struct BatchRow {
  a: vec4<u32>,
  b: vec4<u32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> instances: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> batches: array<BatchRow>;
// Per mesh slot 10 x vec4: its sphere, (levels, 0, 0, 0), then per level
// (first cluster, clusters, error as bits, 0).
@group(0) @binding(4) var<storage, read> meshes: array<vec4<u32>>;
// Per cluster 3 x vec4: (first index, triangles, level, 0), sphere, cone.
@group(0) @binding(5) var<storage, read> clusters: array<vec4<u32>>;
@group(0) @binding(6) var<storage, read_write> items: array<vec2<u32>>;
// Per batch a draw: (vertices, instances, first vertex, first instance).
@group(0) @binding(7) var<storage, read_write> draws: array<atomic<u32>>;
@group(0) @binding(8) var pyramid: texture_2d<f32>;

const WG: u32 = 64u;
const MESH_ROWS: u32 = 10u;
// lod.rs LOD_PIXEL_ERROR.
const PIXEL_ERROR: f32 = 1.0;

// meshlet.rs `outside`.
fn outside(c: vec3<f32>, r: f32) -> bool {
  for (var i = 0u; i < 6u; i++) {
    let p = params.planes[i];
    if (dot(p.xyz, c) + p.w < -r * length(p.xyz)) { return true; }
  }
  return false;
}

// meshlet.rs `ScreenRect::of_sphere` and `occluded`: the box about the
// sphere behind the pyramid everywhere it covers.
fn occluded(c: vec3<f32>, r: f32) -> bool {
  let size = vec2<f32>(params.sizes.xy);
  var lo = vec2<f32>(1e30);
  var hi = vec2<f32>(-1e30);
  var near = 1e30;
  for (var i = 0u; i < 8u; i++) {
    let corner = c + r * vec3<f32>(
      select(-1.0, 1.0, (i & 1u) != 0u),
      select(-1.0, 1.0, (i & 2u) != 0u),
      select(-1.0, 1.0, (i & 4u) != 0u));
    let clip = params.view_proj * vec4<f32>(corner, 1.0);
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
  if (level >= params.sizes.z) { return false; }
  // The level's size from the screen's (cull.wgsl's note on llvmpipe).
  let dims = max(vec2<u32>(1u), params.sizes.xy >> vec2<u32>(level));
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

@compute @workgroup_size(64)
fn cull_clusters(@builtin(workgroup_id) wid: vec3<u32>,
                 @builtin(num_workgroups) nw: vec3<u32>,
                 @builtin(local_invocation_index) li: u32) {
  let count = params.sizes.w;
  if (count == 0u) { return; }
  let w = wid.x + wid.y * nw.x;
  // The batch whose workgroups hold `w` (their firsts ascend).
  var lo = 0u;
  var hi = count - 1u;
  loop {
    if (lo >= hi) { break; }
    let mid = (lo + hi + 1u) / 2u;
    if (batches[mid].b.x <= w) { lo = mid; } else { hi = mid - 1u; }
  }
  let row = batches[lo];
  let local = w - row.b.x;
  if (local >= row.a.z) { return; }
  let instance = row.a.y + local;
  let t0 = instances[instance * 4u];
  let t1 = instances[instance * 4u + 1u];
  let t2 = instances[instance * 4u + 2u];
  let c0 = t0.xyz;
  let c1 = vec3<f32>(t0.w, t1.xy);
  let c2 = vec3<f32>(t1.zw, t2.x);
  let m = mat4x4<f32>(vec4<f32>(c0, 0.0), vec4<f32>(c1, 0.0), vec4<f32>(c2, 0.0), vec4<f32>(t2.yzw, 1.0));
  // meshlet.rs `transform_scale`.
  let la = length(c0);
  let lb = length(c1);
  let lc = length(c2);
  let scale = max(la, max(lb, lc));
  let even = abs(la - lb) <= 1e-4 * scale && abs(la - lc) <= 1e-4 * scale
    && dot(c0, cross(c1, c2)) >= 0.0;
  let mesh = row.a.x * MESH_ROWS;
  let sphere = bitcast<vec4<f32>>(meshes[mesh]);
  let center = (m * vec4<f32>(sphere.xyz, 1.0)).xyz;
  let radius = sphere.w * scale;
  // lod.rs `pixels_per_unit` at the sphere's near side, then `select`.
  let perspective = params.eye.w > 0.5;
  var per_unit = params.lod.x;
  var inside = false;
  if (perspective) {
    let d = length(center - params.eye.xyz) - radius;
    if (d <= 1e-6) { inside = true; } else { per_unit = params.lod.x / d; }
  }
  let levels = meshes[mesh + 1u].x;
  let bias = params.lod.y;
  var level = 0u;
  if (!inside && bias > 0.0) {
    var error = 0.0;
    for (var k = 1u; k < levels; k++) {
      error = max(error, bitcast<f32>(meshes[mesh + 2u + k].z));
      if (error * per_unit * scale * bias < PIXEL_ERROR) { level = k; } else { break; }
    }
  }
  if (outside(center, radius) || (params.sizes.z > 0u && occluded(center, radius))) { return; }
  let lv = meshes[mesh + 2u + level];
  let cone = (row.b.y & 1u) != 0u && even && perspective;
  for (var i = li; i < lv.y; i += WG) {
    let ci = lv.x + i;
    let s = bitcast<vec4<f32>>(clusters[ci * 3u + 1u]);
    let wc = (m * vec4<f32>(s.xyz, 1.0)).xyz;
    let wr = s.w * scale;
    if (outside(wc, wr)) { continue; }
    if (cone) {
      // meshlet.rs `backfacing`.
      let k = bitcast<vec4<f32>>(clusters[ci * 3u + 2u]);
      if (k.w < 1.0) {
        let axis = normalize((m * vec4<f32>(k.xyz, 0.0)).xyz);
        let to = wc - params.eye.xyz;
        if (dot(to, axis) >= k.w * length(to) + wr) { continue; }
      }
    }
    if (params.sizes.z > 0u && occluded(wc, wr)) { continue; }
    let n = atomicAdd(&draws[lo * 4u + 1u], 1u);
    items[row.a.w + n] = vec2<u32>(instance, ci);
  }
}
