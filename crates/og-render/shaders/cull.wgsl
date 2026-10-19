// GPU-driven culling (full tier, cull_gpu.rs): per pass, every sorted
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
//   scatter     copies the visible transforms and writes the draws

const WG: u32 = 256u;
const MAX_PLANES: u32 = 12u;
const MAX_PASSES: u32 = 18u;

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

struct PassGpu {
  planes: array<vec4<f32>, 12>,
  // x: kind, y: planes, z: the pass's global index, w: unused
  info: vec4<u32>,
};

struct Params {
  // x: instances (N), y: batches, z: passes of this stage, w: passes in all
  counts: vec4<u32>,
  // x: workgroups per pass (blocks), y: pyramid levels, zw: screen size
  sizes: vec4<u32>,
  view_proj: mat4x4<f32>,
  passes: array<PassGpu, 18>,
};

struct BatchGpu {
  first: u32,
  count: u32,
  index_count: u32,
  flags: u32,
  // xyz: the mesh's local bounds centre, w: its half extent's length
  bounds: vec4<f32>,
  // xyz: the local half extent
  extent: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;
// Region 0: the sorted transforms; region g + 1: pass g's visible ones.
@group(0) @binding(1) var<storage, read_write> instances: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> batches: array<BatchGpu>;
// [0, N): last frame's main visibility; then per pass N + 1 prefix
// entries; then per pass `blocks` workgroup sums.
@group(0) @binding(3) var<storage, read_write> scan: array<u32>;
// Per (pass, batch) five words of DrawIndexedIndirect, then per pass the
// instances inside its planes (statistics).
@group(0) @binding(4) var<storage, read_write> args: array<atomic<u32>>;
@group(0) @binding(5) var pyramid: texture_2d<f32>;

var<workgroup> shared_sum: array<u32, 256>;
var<workgroup> shared_inside: atomic<u32>;

fn n_instances() -> u32 { return params.counts.x; }
fn prefix_at(g: u32) -> u32 { return n_instances() + g * (n_instances() + 1u); }
fn blocks_at(g: u32) -> u32 { return n_instances() + params.counts.w * (n_instances() + 1u) + g * params.sizes.x; }

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

// An instance's box (its local bounds through its transform) on screen:
// its pixel rectangle and nearest depth; `ok` is false when a corner is
// behind the camera.
struct ScreenBox {
  lo: vec2<f32>,
  hi: vec2<f32>,
  near: f32,
  ok: bool,
};

fn screen_box(m: mat4x4<f32>, center: vec3<f32>, extent: vec3<f32>) -> ScreenBox {
  let size = vec2<f32>(params.sizes.zw);
  var r = ScreenBox(vec2<f32>(1e30), vec2<f32>(-1e30), 1e30, true);
  for (var i = 0u; i < 8u; i++) {
    let corner = center + extent * vec3<f32>(
      select(-1.0, 1.0, (i & 1u) != 0u),
      select(-1.0, 1.0, (i & 2u) != 0u),
      select(-1.0, 1.0, (i & 4u) != 0u));
    let clip = params.view_proj * (m * vec4<f32>(corner, 1.0));
    if (clip.w <= 1e-6) { r.ok = false; return r; }
    let ndc = clip.xyz / clip.w;
    let px = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * size;
    r.lo = min(r.lo, px);
    r.hi = max(r.hi, px);
    r.near = min(r.near, ndc.z);
  }
  return r;
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

// Whether the instance's box lies behind the occluders' depth everywhere
// it covers (conservative: anything uncertain is visible).
fn occluded(m: mat4x4<f32>, center: vec3<f32>, extent: vec3<f32>) -> bool {
  let size = vec2<f32>(params.sizes.zw);
  let r = screen_box(m, center, extent);
  if (!r.ok) { return false; }
  let lo = r.lo;
  let hi = r.hi;
  let near = r.near;
  if (near <= 0.0) { return false; }
  // One pixel of margin, clamped to the screen.
  let x0 = u32(clamp(floor(lo.x) - 1.0, 0.0, size.x - 1.0));
  let y0 = u32(clamp(floor(lo.y) - 1.0, 0.0, size.y - 1.0));
  let x1 = u32(clamp(floor(hi.x) + 1.0, 0.0, size.x - 1.0));
  let y1 = u32(clamp(floor(hi.y) + 1.0, 0.0, size.y - 1.0));
  let texels = max(x1 - x0, y1 - y0) + 1u;
  // The level whose texels are at least the rectangle's size: at most
  // 2 x 2 of them cover it.
  var level = 0u;
  while ((1u << level) < texels) { level++; }
  if (level >= params.sizes.y) { return false; }
  // The level's size from the screen's (as wgpu sizes mips), not
  // textureDimensions(pyramid, level): llvmpipe answers level 0's size for
  // every level, so a box at the right edge read a texel past the last and
  // got zero (nearest) depth.
  let dims = max(vec2<u32>(1u), params.sizes.zw >> vec2<u32>(level));
  let t0 = min(vec2<u32>(x0, y0) >> vec2<u32>(level), dims - 1u);
  let t1 = min(vec2<u32>(x1, y1) >> vec2<u32>(level), dims - 1u);
  var farthest = 0.0;
  for (var y = t0.y; y <= t1.y; y++) {
    for (var x = t0.x; x <= t1.x; x++) {
      farthest = max(farthest, textureLoad(pyramid, vec2<u32>(x, y), level).r);
    }
  }
  // A margin over the rasterized depth's rounding (16 ulps near 1).
  return near > farthest + 1e-6;
}

@compute @workgroup_size(256)
fn visibility(@builtin(global_invocation_id) gid: vec3<u32>,
              @builtin(local_invocation_index) li: u32,
              @builtin(workgroup_id) wid: vec3<u32>) {
  let pass_info = params.passes[wid.y].info;
  let kind = pass_info.x;
  let g = pass_info.z;
  let k = gid.x;
  if (li == 0u) { atomicStore(&shared_inside, 0u); }
  workgroupBarrier();
  var visible = 0u;
  if (k < n_instances()) {
    let b = batches[batch_of(k)];
    let m = instances[k];
    let center = (m * vec4<f32>(b.bounds.xyz, 1.0)).xyz;
    let scale = max(length(m[0].xyz), max(length(m[1].xyz), length(m[2].xyz)));
    // The CPU path's sphere, widened by its rounding.
    let radius = b.bounds.w * scale * (1.0 + 1e-5);
    var inside = kind == FRUSTUM_ALL || (b.flags & TRANSPARENT) == 0u;
    for (var p = 0u; p < pass_info.y && inside; p++) {
      let plane = params.passes[wid.y].planes[p];
      inside = dot(plane.xyz, center) + plane.w >= -radius;
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
}

// An inclusive scan of `shared_sum` (Hillis-Steele).
fn scan_shared(li: u32) {
  for (var s = 1u; s < WG; s <<= 1u) {
    var v = shared_sum[li];
    if (li >= s) { v += shared_sum[li - s]; }
    workgroupBarrier();
    shared_sum[li] = v;
    workgroupBarrier();
  }
}

@compute @workgroup_size(256)
fn scan_blocks(@builtin(local_invocation_index) li: u32,
               @builtin(workgroup_id) wid: vec3<u32>) {
  let g = params.passes[wid.y].info.z;
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
  let g = params.passes[wid.y].info.z;
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
  let g = params.passes[wid.y].info.z;
  let n = n_instances();
  let k = gid.x;
  if (k >= n) { return; }
  let bi = batch_of(k);
  let b = batches[bi];
  let at = prefix_at(g);
  let start = scan[at + b.first];
  if (scan[at + k + 1u] != scan[at + k]) {
    instances[(g + 1u) * n + b.first + scan[at + k] - start] = instances[k];
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
