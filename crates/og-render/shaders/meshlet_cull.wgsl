// Balanced 2-stage GPU meshlet culling (meshlet_gpu.rs):
//
// Stage 1 (`cull_instances`, 1 thread per instance, @workgroup_size(64)):
//   Tests each instance's bounding sphere against its view's frustum (and Hi-Z
//   in phase 2), picks its LOD level, and emits 64-cluster work chunks into
//   `chunks`, accumulating the total chunk count in `draws[params.chunks.x + 3]`.
//   Culled instances emit 0 chunks; large multi-thousand-cluster instances fan
//   out across parallel 64-cluster chunks instead of serializing on 1 workgroup.
//
// Stage 1b (`finalize_chunks`, 1 thread):
//   Converts the total chunk count into DispatchWorkgroupsIndirect args at
//   `draws[params.chunks.x .. + 3]`.
//
// Stage 2 (`cull_clusters`, indirect dispatch, 1 workgroup per 64-cluster chunk):
//   Each thread tests one cluster (frustum, backfacing normal cone, camera Hi-Z
//   and phase visibility), counts camera triangles, and marks visible slots in
//   `selected` for stable compaction (`meshlet_compact.wgsl`).
//
// Each entry point stays <= 7 storage buffers (WebGPU Compat compatible).

struct Params {
  // The camera's (the pyramid's projection).
  view_proj: mat4x4<f32>,
  // x: the scene's LOD bias; y: the first of `views`' rows holding the
  // levels callers picked, four sorted slots a row (255: none), when z
  // is 1; w: the camera's near plane.
  lod: vec4<f32>,
  // xy: the screen's size, z: the pyramid's levels (0: no occlusion
  // test), w: the batches.
  sizes: vec4<u32>,
  // x: the phase (0 the only one, 1 early, 2 late), y: last frame's half
  // of `vis` (its first word), z: this frame's, w: total instances.
  phase: vec4<u32>,
  // The soft list (meshlet_gpu.rs `plan`): x its first item, y its
  // capacity in clusters (0: none), z its count's word in `draws`; w the
  // word that sums the camera's drawn triangles (FrameStats::triangles).
  soft: vec4<u32>,
  // Balanced 2-stage chunk dispatch: x first word of DispatchWorkgroupsIndirect
  // in `draws` ([dx, dy, dz, total_chunks]), y capacity of `chunks`.
  chunks: vec4<u32>,
};

// a: mesh slot, first instance (its sorted slot), instances, first work
// item. b: first instance offset across all batches, flags (1: the cone may
// cull; 2: soft, its small clusters go to the soft list; from bit 2 its side),
// view, the mesh's most clusters at any level (a cluster slot's stride).
struct BatchRow {
  a: vec4<u32>,
  b: vec4<u32>,
};

// One 64-cluster work chunk emitted by `cull_instances` for `cull_clusters`.
struct CullChunk {
  // x: instance sorted slot, y: first cluster index, z: first selected slot,
  // w: count (0..7) | view (8..23) | cone (24) | soft_eligible (25).
  a: vec4<u32>,
  // x: world sway grow margin, y: instance max axis scale, zw: unused.
  b: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> instances: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> views: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> batches: array<BatchRow>;
@group(0) @binding(4) var<storage, read> meshes: array<vec4<u32>>;
@group(0) @binding(5) var<storage, read> clusters: array<vec4<u32>>;
@group(0) @binding(6) var<storage, read_write> selected: array<vec2<u32>>;
@group(0) @binding(7) var<storage, read_write> draws: array<atomic<u32>>;
@group(0) @binding(8) var pyramid: texture_2d<f32>;
@group(0) @binding(9) var<storage, read_write> vis: array<u32>;
@group(0) @binding(10) var<storage, read_write> chunks: array<CullChunk>;

const WG: u32 = 64u;
const WORKGROUPS_PER_ROW: u32 = 65535u;
// meshlet_gpu.rs VIEW_PLANES (cull_gpu.rs MAX_PLANES).
const VIEW_PLANES: u32 = 12u;
const VIEW_ROWS: u32 = VIEW_PLANES + 2u;
const MESH_ROWS: u32 = 10u;
// meshlet.rs SOFT_PIXELS.
const SOFT_PIXELS: f32 = 24.0;

// meshlet.rs `outside`.
fn outside(view: u32, c: vec3<f32>, r: f32) -> bool {
  for (var i = 0u; i < VIEW_PLANES; i++) {
    let p = views[view * VIEW_ROWS + i];
    if (!plane_keeps(p, c, r * length(p.xyz))) { return true; }
  }
  return false;
}

// meshlet.rs `occluded`: the box about the sphere behind the pyramid
// everywhere it covers (cull_hiz.wgsl).
fn occluded(c: vec3<f32>, r: f32) -> bool {
  let box = hiz_sphere(params.view_proj, c, r, vec2<f32>(params.sizes.xy));
  return hiz_hidden(box, params.sizes.xy, params.sizes.z);
}

fn load_instance_matrix(instance: u32) -> mat4x4<f32> {
  let t0 = instances[instance * 4u];
  let t1 = instances[instance * 4u + 1u];
  let t2 = instances[instance * 4u + 2u];
  let c0 = t0.xyz;
  let c1 = vec3<f32>(t0.w, t1.xy);
  let c2 = vec3<f32>(t1.zw, t2.x);
  return mat4x4<f32>(vec4<f32>(c0, 0.0), vec4<f32>(c1, 0.0), vec4<f32>(c2, 0.0), vec4<f32>(t2.yzw, 1.0));
}

@compute @workgroup_size(64)
fn cull_instances(@builtin(workgroup_id) wid: vec3<u32>,
                  @builtin(num_workgroups) nw: vec3<u32>,
                  @builtin(local_invocation_index) li: u32) {
  let count = params.sizes.w;
  let total_instances = params.phase.w;
  if (count == 0u) { return; }
  let w = (wid.x + wid.y * nw.x) * WG + li;
  if (w >= total_instances) { return; }
  // The batch whose instances hold `w` (their firsts ascend).
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
  let view = row.b.z;
  let phase = params.phase.x;
  // The late phase is the camera's.
  if (phase == 2u && view != 0u) { return; }

  let instance = row.a.y + local;
  let m = load_instance_matrix(instance);
  let c0 = m[0].xyz;
  let c1 = m[1].xyz;
  let c2 = m[2].xyz;
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
  let sway = instances[instance * 4u + 3u].x;
  let grow = bitcast<f32>(meshes[mesh + 1u].z) * max(1.0, max(select(-sway - 1.0, sway, sway >= 0.0), 0.0)) * scale;

  let occlusion = phase == 2u;
  if (outside(view, center, radius + grow) || (occlusion && occluded(center, radius + grow))) { return; }

  let seen = views[view * VIEW_ROWS + VIEW_PLANES];
  let eye = seen.xyz;
  let perspective = views[view * VIEW_ROWS + VIEW_PLANES + 1u].x > 0.5;
  var per_unit = seen.w;
  var inside = false;
  if (perspective) {
    let d = length(center - eye) - radius;
    if (d <= 1e-6) { inside = true; } else { per_unit = seen.w / d; }
  }
  let levels = meshes[mesh + 1u].x;
  let bias = params.lod.x;
  var level = 0u;
  var picked = 255u;
  if (params.lod.z > 0.5) {
    picked = u32(views[u32(params.lod.y) + instance / 4u][instance % 4u]);
  }
  if (picked < 255u) {
    level = min(picked, levels - 1u);
  } else if (!inside && bias > 0.0) {
    var error = 0.0;
    for (var k = 1u; k < levels; k++) {
      if (lod_holds(&error, bitcast<f32>(meshes[mesh + 2u + k].z), per_unit, scale, bias)) { level = k; } else { break; }
    }
  }
  let lv = meshes[mesh + 2u + level];
  if (lv.y == 0u) { return; }

  let n_chunks = (lv.y + WG - 1u) / WG;
  let base = atomicAdd(&draws[params.chunks.x + 3u], n_chunks);
  let slots = row.a.w + local * row.b.w;
  let cone_bit = u32((row.b.y & 1u) != 0u && even && perspective && grow <= 0.0);
  let soft_bit = u32((row.b.y & 2u) != 0u && perspective);
  let view_flags = ((view & 0xffffu) << 8u) | (cone_bit << 24u) | (soft_bit << 25u);
  for (var k = 0u; k < n_chunks; k++) {
    let dst = base + k;
    if (dst < params.chunks.y) {
      let off = k * WG;
      let cnt = min(WG, lv.y - off);
      chunks[dst] = CullChunk(
        vec4<u32>(instance, lv.x + off, slots + off, cnt | view_flags),
        vec4<f32>(grow, scale, 0.0, 0.0),
      );
    }
  }
}

@compute @workgroup_size(1)
fn finalize_chunks() {
  let base = params.chunks.x;
  let total = min(atomicLoad(&draws[base + 3u]), params.chunks.y);
  atomicStore(&draws[base + 0u], min(total, WORKGROUPS_PER_ROW));
  atomicStore(&draws[base + 1u], select(0u, (total + WORKGROUPS_PER_ROW - 1u) / WORKGROUPS_PER_ROW, total > 0u));
  atomicStore(&draws[base + 2u], select(0u, 1u, total > 0u));
  atomicStore(&draws[base + 3u], total);
}

@compute @workgroup_size(64)
fn cull_clusters(@builtin(workgroup_id) wid: vec3<u32>,
                 @builtin(num_workgroups) nw: vec3<u32>,
                 @builtin(local_invocation_index) li: u32) {
  let idx = wid.x + wid.y * nw.x;
  let total = atomicLoad(&draws[params.chunks.x + 3u]);
  if (idx >= total) { return; }
  let ch = chunks[idx];
  let cnt = ch.a.w & 0xffu;
  if (li >= cnt) { return; }

  let instance = ch.a.x;
  let ci = ch.a.y + li;
  let slot = ch.a.z + li;
  let view = (ch.a.w >> 8u) & 0xffffu;
  let cone = (ch.a.w & (1u << 24u)) != 0u;
  let soft_eligible = (ch.a.w & (1u << 25u)) != 0u;
  let grow = ch.b.x;
  let scale = ch.b.y;
  let phase = params.phase.x;

  let m = load_instance_matrix(instance);
  let seen = views[view * VIEW_ROWS + VIEW_PLANES];
  let eye = seen.xyz;
  let s = bitcast<vec4<f32>>(clusters[ci * 3u + 1u]);
  let wc = (m * vec4<f32>(s.xyz, 1.0)).xyz;
  let wr = s.w * scale + grow;
  if (outside(view, wc, wr)) { return; }
  if (cone) {
    // meshlet.rs `backfacing`.
    let k = bitcast<vec4<f32>>(clusters[ci * 3u + 2u]);
    if (k.w < 1.0) {
      let axis = normalize((m * vec4<f32>(k.xyz, 0.0)).xyz);
      let to = wc - eye;
      if (dot(to, axis) >= k.w * length(to) + wr) { return; }
    }
  }
  if (view == 0u) {
    let before = vis[params.phase.y + slot] != 0u;
    if (phase == 1u && !before) { return; }
    if (phase == 2u) {
      if (occluded(wc, wr)) { return; }
      vis[params.phase.z + slot] = 1u;
      if (before) { return; }
    }
    if (phase == 0u) { vis[params.phase.z + slot] = 1u; }
    // Drawn by the camera, here or in the visibility buffer: counted once.
    atomicAdd(&draws[params.soft.w], clusters[ci * 3u].y);
  }
  // meshlet.rs `soft`: a small cluster past the near plane goes to the
  // visibility buffer. Compaction assigns its capacity by stable rank.
  var soft = 0u;
  if (soft_eligible) {
    let depth = (params.view_proj * vec4<f32>(wc, 1.0)).w;
    let d = depth - wr;
    if (d > params.lod.w && 2.0 * wr * seen.w <= SOFT_PIXELS * d) {
      soft = 1u;
    }
  }
  selected[slot] = vec2<u32>(ci + 1u, soft);
}
