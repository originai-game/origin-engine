// The scan a cull compacts with, written once for the instances'
// (cull.wgsl), the crowds' (crowd_cull.wgsl) and the clusters'
// (meshlet_compact.wgsl). The including module declares `shared_sum`, a
// workgroup array of WG counts.

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
