// The scan a cull compacts with, written once for the instances'
// (cull.wgsl), the crowds' (crowd_cull.wgsl) and the clusters'
// (meshlet_compact.wgsl). The including module declares `shared_sum`, a
// workgroup array of WG counts.

// Work-efficient Blelloch inclusive scan of `shared_sum` (WG is a power of 2).
fn scan_shared(li: u32) {
  let orig = shared_sum[li];
  for (var offset = 1u; offset < WG; offset <<= 1u) {
    let idx = (li + 1u) * (offset << 1u) - 1u;
    if (idx < WG) {
      shared_sum[idx] += shared_sum[idx - offset];
    }
    workgroupBarrier();
  }
  if (li == 0u) {
    var zero: Sum;
    shared_sum[WG - 1u] = zero;
  }
  workgroupBarrier();
  for (var offset = WG >> 1u; offset > 0u; offset >>= 1u) {
    let idx = (li + 1u) * (offset << 1u) - 1u;
    if (idx < WG) {
      let t = shared_sum[idx - offset];
      shared_sum[idx - offset] = shared_sum[idx];
      shared_sum[idx] += t;
    }
    workgroupBarrier();
  }
  shared_sum[li] += orig;
  workgroupBarrier();
}
