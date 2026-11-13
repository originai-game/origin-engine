// GPU-driven culling (cull_gpu.rs): per pass, every sorted
// instance's world sphere is tested against the pass's planes (and, for
// the main pass, the occluders' depth pyramid); the visible ones are
// compacted, batch by batch and in their sorted order, into the pass's
// region of `instances`, and each batch gets an indexed indirect draw.
// No atomics decide an order: a deterministic prefix sum does.
//
// One workgroup row (workgroup_id.y) per pass of the stage:
//   visibility  writes 0/1 per instance and each workgroup's sum
//   scan_blocks scans the workgroup sums (one workgroup per pass)
//   prefix      turns the 0/1 into the exclusive prefix (N + 1 entries)
//   scatter     copies the visible transforms and writes the draws (the
//               camera pass's previous transforms too, with object motion)

const WG: u32 = 256u;
const MAX_PLANES: u32 = 12u;

// Pass kinds.
const FRUSTUM_ALL: u32 = 0u;     // every batch (shadow passes)
const FRUSTUM_OPAQUE: u32 = 1u;  // opaque batches (the planar reflection)
const OCCLUDERS: u32 = 2u;       // last frame's visible occluders
const MAIN: u32 = 3u;            // opaque, frustum and depth pyramid

// Batch flags.
const TRANSPARENT: u32 = 1u;
// Writes the depth it is drawn with (opaque, no discard, no vertex hook).
const OCCLUDER: u32 = 2u;
// Draws inside its bounds (no material vertex hook): may be hidden.
const OCCLUDABLE: u32 = 4u;
const FOLIAGE: u32 = 8u;
// A skinned batch (cull_gpu.rs `SkinnedCull`): an instance's matrix takes
// the unit box to its posed box, and its record is in `skinned`.
const SKINNED: u32 = 16u;
// A skinned batch that casts no shadow: left out of the shadow passes.
const NO_CAST: u32 = 32u;
// The floats of a skinned instance's record (skin.rs SkinnedInstanceGpu).
const SKINNED_FLOATS: u32 = 28u;

struct PassGpu {
  planes: array<vec4<f32>, 12>,
  // x: kind, y: planes, z: the pass's global index, w: its layer mask
  info: vec4<u32>,
};

struct Params {
  // x: instances (N), y: batches, z: passes of this stage, w: passes in all
  counts: vec4<u32>,
  // x: workgroups per pass (blocks), y: pyramid levels, zw: screen size
  sizes: vec4<u32>,
  // x: 1 when `previous` holds the sorted instances' previous transforms
  // (TAA's object motion): the camera pass copies them beside its own.
  // y: this stage's first pass in `passes`. z: the first skinned box
  // among the instances (after the sorted ones), w: the skinned instances.
  motion: vec4<u32>,
  view_proj: mat4x4<f32>,
};

struct BatchGpu {
  first: u32,
  count: u32,
  index_count: u32,
  flags: u32,
  // xyz: the mesh's local bounds centre, w: its half extent's length
  bounds: vec4<f32>,
  // The local half extent.
  extent: vec3<f32>,
  // Its three.js layer mask: a pass draws it when they share a bit.
  layers: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
// Region 0: the sorted transforms; region g + 1: pass g's visible ones.
// An instance is 16 floats: its columns' xyz (instance.wgsl), its
// texture-array layer and its look (instances.rs `GpuTransform`).
@group(0) @binding(1) var<storage, read_write> instances: array<f32>;
@group(0) @binding(2) var<storage, read> batches: array<BatchGpu>;
// [0, N): last frame's main visibility; then per pass N + 1 prefix
// entries; then per pass `blocks` workgroup sums.
@group(0) @binding(3) var<storage, read_write> scan: array<u32>;
// Per (pass, batch) five words of DrawIndexedIndirect, then per pass the
// instances inside its planes, then per batch the camera pass's instances
// the occluders hid (statistics).
@group(0) @binding(4) var<storage, read_write> args: array<atomic<u32>>;
@group(0) @binding(5) var pyramid: texture_2d<f32>;
// Laid out as `instances`: region 0 the sorted instances' previous
// transforms and sway, region g + 1 those of the visible ones (camera,
// and every pass for foliage whose current sway shares this stream).
@group(0) @binding(6) var<storage, read_write> previous: array<f32>;
// Every pass of the frame (cull_gpu.rs), the other passes' stage first and
// the camera's after; the stage's row wid.y is passes[pass_at(wid.y)].
@group(0) @binding(7) var<storage, read> passes: array<PassGpu>;
// The skinned instances' records: region 0 the frame's, region g + 1 pass
// g's visible ones (each region `motion.w` records long).
@group(0) @binding(8) var<storage, read_write> skinned: array<f32>;

fn pass_at(row: u32) -> u32 { return params.motion.y + row; }

var<workgroup> shared_sum: array<u32, 256>;
var<workgroup> shared_inside: atomic<u32>;
// The camera pass's hidden instances of the workgroup's first two batches
// (the instances are sorted by batch), and the first one's index.
var<workgroup> shared_hidden: array<atomic<u32>, 2>;
var<workgroup> first_batch: u32;

fn n_instances() -> u32 { return params.counts.x; }
fn prefix_at(g: u32) -> u32 { return n_instances() + g * (n_instances() + 1u); }
fn blocks_at(g: u32) -> u32 { return n_instances() + params.counts.w * (n_instances() + 1u) + g * params.sizes.x; }
fn hidden_at(bi: u32) -> u32 { return params.counts.w * params.counts.y * 5u + params.counts.w + bi; }

// Sorted instance k's transform.
fn instance_at(k: u32) -> mat4x4<f32> {
  let a = k * 16u;
  return instance_matrix(
    vec3<f32>(instances[a], instances[a + 1u], instances[a + 2u]),
    vec3<f32>(instances[a + 3u], instances[a + 4u], instances[a + 5u]),
    vec3<f32>(instances[a + 6u], instances[a + 7u], instances[a + 8u]),
    vec3<f32>(instances[a + 9u], instances[a + 10u], instances[a + 11u]));
}

// The batch holding sorted instance k (batches cover 0..N in order).
fn batch_of(k: u32) -> u32 {
  var lo = 0u;
  var hi = params.counts.y - 1u;
  loop {
    if (lo >= hi) { break; }
    let mid = (lo + hi + 1u) / 2u;
    if (batches[mid].first <= k) { lo = mid; } else { hi = mid - 1u; }
  }
  return lo;
}

// Whether an occluder is worth drawing into the pyramid: its box covers at
// least a sixteenth of the screen's larger side (or reaches behind the
// camera). Small ones cost their triangles and hide little.
fn large_occluder(m: mat4x4<f32>, center: vec3<f32>, extent: vec3<f32>) -> bool {
  let r = screen_box(m, center, extent);
  if (!r.ok) { return true; }
  let size = vec2<f32>(params.sizes.zw);
  let covered = clamp(r.hi, vec2<f32>(0.0), size) - clamp(r.lo, vec2<f32>(0.0), size);
  return max(covered.x, covered.y) * 16.0 >= max(size.x, size.y);
}

@compute @workgroup_size(256)
fn visibility(@builtin(global_invocation_id) gid: vec3<u32>,
              @builtin(local_invocation_index) li: u32,
              @builtin(workgroup_id) wid: vec3<u32>) {
  let pass_info = passes[pass_at(wid.y)].info;
  let kind = pass_info.x;
  let g = pass_info.z;
  let k = gid.x;
  if (li == 0u) {
    atomicStore(&shared_inside, 0u);
    atomicStore(&shared_hidden[0], 0u);
    atomicStore(&shared_hidden[1], 0u);
    first_batch = batch_of(min(wid.x * WG, n_instances() - 1u));
  }
  workgroupBarrier();
  var visible = 0u;
  if (k < n_instances()) {
    let bi = batch_of(k);
    let b = batches[bi];
    let m = instance_at(k);
    let center = (m * vec4<f32>(b.bounds.xyz, 1.0)).xyz;
    let scale = max(length(m[0].xyz), max(length(m[1].xyz), length(m[2].xyz)));
    // The CPU path's sphere, widened by its rounding.
    let radius = b.bounds.w * scale * (1.0 + 1e-5);
    let boxed = (b.flags & SKINNED) != 0u;
    var inside = (kind == FRUSTUM_ALL || (b.flags & TRANSPARENT) == 0u)
      && (kind != FRUSTUM_ALL || (b.flags & NO_CAST) == 0u)
      && (b.layers & pass_info.w) != 0u;
    for (var p = 0u; p < pass_info.y && inside; p++) {
      let plane = passes[pass_at(wid.y)].planes[p];
      // A box (a skinned instance's): each plane against its corner
      // farthest inside it, as the CPU's `Frustum::intersects_box`.
      var reach = radius;
      if (boxed) {
        reach = (abs(dot(plane.xyz, m[0].xyz)) * b.extent.x
          + abs(dot(plane.xyz, m[1].xyz)) * b.extent.y
          + abs(dot(plane.xyz, m[2].xyz)) * b.extent.z) * (1.0 + 1e-5);
      }
      inside = dot(plane.xyz, center) + plane.w >= -reach;
    }
    if (inside) { atomicAdd(&shared_inside, 1u); }
    let occluder = (b.flags & OCCLUDER) != 0u && scan[k] != 0u;
    switch kind {
      case OCCLUDERS: {
        visible = u32(inside && occluder && large_occluder(m, b.bounds.xyz, b.extent.xyz));
      }
      case MAIN: {
        // The occluders' pyramid holds real depth at this frame's camera,
        // so a box behind it everywhere draws nothing; the frontmost
        // occluder at a pixel is never behind its own depth.
        // The box widened by the rounding of its transform.
        let hidden = inside && (b.flags & OCCLUDABLE) != 0u
          && occluded(m, b.bounds.xyz, b.extent.xyz * (1.0 + 1e-5) + vec3<f32>(1e-6));
        visible = u32(inside && !hidden);
        scan[k] = visible;
        if (hidden) {
          let slot = bi - first_batch;
          if (slot < 2u) {
            atomicAdd(&shared_hidden[slot], 1u);
          } else {
            atomicAdd(&args[hidden_at(bi)], 1u);
          }
        }
      }
      default: { visible = u32(inside); }
    }
    scan[prefix_at(g) + k] = visible;
  }
  shared_sum[li] = visible;
  workgroupBarrier();
  for (var s = WG / 2u; s > 0u; s >>= 1u) {
    if (li < s) { shared_sum[li] += shared_sum[li + s]; }
    workgroupBarrier();
  }
  if (li == 0u) {
    scan[blocks_at(g) + wid.x] = shared_sum[0];
    let inside = atomicLoad(&shared_inside);
    if (inside > 0u) {
      atomicAdd(&args[params.counts.w * params.counts.y * 5u + g], inside);
    }
  }
  if (li < 2u && first_batch + li < params.counts.y) {
    let hidden = atomicLoad(&shared_hidden[li]);
    if (hidden > 0u) { atomicAdd(&args[hidden_at(first_batch + li)], hidden); }
  }
}

@compute @workgroup_size(256)
fn scan_blocks(@builtin(local_invocation_index) li: u32,
               @builtin(workgroup_id) wid: vec3<u32>) {
  let g = passes[pass_at(wid.y)].info.z;
  let at = blocks_at(g);
  var carry = 0u;
  for (var base = 0u; base < params.sizes.x; base += WG) {
    let i = base + li;
    var v = 0u;
    if (i < params.sizes.x) { v = scan[at + i]; }
    shared_sum[li] = v;
    workgroupBarrier();
    scan_shared(li);
    if (i < params.sizes.x) { scan[at + i] = carry + shared_sum[li] - v; }
    carry += shared_sum[WG - 1u];
    workgroupBarrier();
  }
}

@compute @workgroup_size(256)
fn prefix(@builtin(global_invocation_id) gid: vec3<u32>,
          @builtin(local_invocation_index) li: u32,
          @builtin(workgroup_id) wid: vec3<u32>) {
  let g = passes[pass_at(wid.y)].info.z;
  let n = n_instances();
  let k = gid.x;
  var v = 0u;
  if (k < n) { v = scan[prefix_at(g) + k]; }
  shared_sum[li] = v;
  workgroupBarrier();
  scan_shared(li);
  let before = scan[blocks_at(g) + wid.x];
  if (k < n) {
    scan[prefix_at(g) + k] = before + shared_sum[li] - v;
    if (k == n - 1u) { scan[prefix_at(g) + n] = before + shared_sum[li]; }
  }
}

@compute @workgroup_size(256)
fn scatter(@builtin(global_invocation_id) gid: vec3<u32>,
           @builtin(workgroup_id) wid: vec3<u32>) {
  let g = passes[pass_at(wid.y)].info.z;
  let n = n_instances();
  let k = gid.x;
  if (k >= n) { return; }
  let bi = batch_of(k);
  let b = batches[bi];
  let at = prefix_at(g);
  let start = scan[at + b.first];
  if (scan[at + k + 1u] != scan[at + k] && (b.flags & SKINNED) != 0u) {
    // A skinned instance: its record into the pass's skinned region.
    let base = params.motion.z;
    let src = (k - base) * SKINNED_FLOATS;
    let dst = ((g + 1u) * params.motion.w + b.first - base + scan[at + k] - start) * SKINNED_FLOATS;
    for (var j = 0u; j < SKINNED_FLOATS; j++) { skinned[dst + j] = skinned[src + j]; }
  } else if (scan[at + k + 1u] != scan[at + k]) {
    let to = ((g + 1u) * n + b.first + scan[at + k] - start) * 16u;
    for (var j = 0u; j < 16u; j++) { instances[to + j] = instances[k * 16u + j]; }
    if (params.motion.x != 0u && ((b.flags & FOLIAGE) != 0u || passes[pass_at(wid.y)].info.x == MAIN)) {
      for (var j = 0u; j < 16u; j++) { previous[to + j] = previous[k * 16u + j]; }
    }
  }
  if (k == b.first) {
    let draw = (g * params.counts.y + bi) * 5u;
    atomicStore(&args[draw], b.index_count);
    atomicStore(&args[draw + 1u], scan[at + b.first + b.count] - start);
    atomicStore(&args[draw + 2u], 0u);
    atomicStore(&args[draw + 3u], 0u);
    atomicStore(&args[draw + 4u], 0u);
  }
}
