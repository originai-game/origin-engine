// Stable stream compaction, using cull.wgsl's blocked Hillis-Steele scan.
// Scan (visible, soft-eligible) counts; the first soft-cap eligible slots
// go to software and the rest to hardware. Both lists therefore have a
// stable order even at overflow and after the late cull adds survivors.
// Separate dispatches supply the device-wide barriers; no spin waiting.
@group(0) @binding(0) var<uniform> params: vec4<u32>; // slots, soft first, cap, rows
// BatchRow's two vec4s, as in meshlet_cull.wgsl.
@group(0) @binding(1) var<storage, read> batches: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read> meshes: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read> selected: array<vec2<u32>>;
// Exclusive per-slot prefixes; block totals live separately so a full
// input binding does not need extra room for the scan's bookkeeping.
@group(0) @binding(4) var<storage, read_write> sums: array<vec2<u32>>;
@group(0) @binding(5) var<storage, read_write> items: array<vec2<u32>>;
@group(0) @binding(6) var<storage, read_write> draws: array<u32>;
@group(0) @binding(7) var<storage, read_write> block_sums: array<vec2<u32>>;
const WG: u32 = 256u;
var<workgroup> shared_sum: array<vec2<u32>, 256>;

fn block(wid: vec3<u32>) -> u32 { return wid.x + wid.y * 65535u; }
fn blocks() -> u32 { return (params.x + WG - 1u) / WG; }
fn value(k: u32) -> vec2<u32> {
  if (k >= params.x) { return vec2<u32>(0u); }
  let s = selected[k];
  return vec2<u32>(u32(s.x != 0u), s.y);
}
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
fn reduce(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
  let b = block(wid);
  if (b >= blocks()) { return; }
  shared_sum[li] = value(b * WG + li);
  workgroupBarrier();
  for (var s = WG / 2u; s > 0u; s >>= 1u) {
    if (li < s) { shared_sum[li] += shared_sum[li + s]; }
    workgroupBarrier();
  }
  if (li == 0u) { block_sums[b] = shared_sum[0]; }
}

@compute @workgroup_size(256)
fn scan_blocks(@builtin(local_invocation_index) li: u32) {
  var carry = vec2<u32>(0u);
  for (var base = 0u; base < blocks(); base += WG) {
    let i = base + li;
    var v = vec2<u32>(0u);
    if (i < blocks()) { v = block_sums[i]; }
    shared_sum[li] = v;
    workgroupBarrier();
    scan_shared(li);
    if (i < blocks()) { block_sums[i] = carry + shared_sum[li] - v; }
    carry += shared_sum[WG - 1u];
    workgroupBarrier();
  }
  if (li == 0u) {
    block_sums[blocks()] = carry;
    draws[params.w * 4u] = min(carry.y, params.z);
  }
}

@compute @workgroup_size(256)
fn prefix(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
  let b = block(wid);
  if (b >= blocks()) { return; }
  let k = b * WG + li;
  let v = value(k);
  shared_sum[li] = v;
  workgroupBarrier();
  scan_shared(li);
  if (k < params.x) { sums[k] = block_sums[b] + shared_sum[li] - v; }
}

fn hardware(p: vec2<u32>) -> u32 { return p.x - min(p.y, params.z); }
fn before(k: u32) -> vec2<u32> {
  if (k == params.x) { return block_sums[blocks()]; }
  return sums[k];
}

@compute @workgroup_size(256)
fn scatter(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
  let k = block(wid) * WG + li;
  if (k >= params.x) { return; }
  var lo = 0u;
  var hi = params.w - 1u;
  loop {
    if (lo >= hi) { break; }
    let mid = (lo + hi + 1u) / 2u;
    if (batches[mid * 2u].w <= k) { lo = mid; } else { hi = mid - 1u; }
  }
  let a = batches[lo * 2u];
  let b = batches[lo * 2u + 1u];
  let start = hardware(sums[a.w]);
  if (k == a.w) {
    draws[lo * 4u + 1u] = hardware(before(a.w + a.z * b.w)) - start;
  }
  let s = selected[k];
  if (s.x == 0u) { return; }
  let instance = a.y + (k - a.w) / b.w;
  let item = vec2<u32>(instance, s.x - 1u);
  let p = sums[k];
  if (s.y != 0u && p.y < params.z) {
    items[params.y + p.y * 2u] = item;
    items[params.y + p.y * 2u + 1u] = vec2<u32>(lo | (((b.y >> 2u) & 3u) << 28u), meshes[a.x * 10u + 1u].y);
  } else {
    items[a.w + hardware(p) - start] = item;
  }
}
