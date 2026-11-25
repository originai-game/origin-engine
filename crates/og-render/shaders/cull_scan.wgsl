// The scan a cull compacts with, written once for the instances'
// (cull.wgsl), the crowds' (crowd_cull.wgsl) and the clusters'
// (meshlet_compact.wgsl). The including module declares `shared_sum`, a
// workgroup array of WG counts.

// Bank-conflict-free index for 32-bank shared memory.
fn scan_pad(i: u32) -> u32 { return i + (i >> 5u); }
var<workgroup> scan_buf: array<Sum, WG + (WG >> 5u)>;

// Work-efficient bank-conflict-free Blelloch inclusive scan of `shared_sum`
// (WG is a power of 2).
fn scan_shared(li: u32) {
  let orig = shared_sum[li];
  scan_buf[scan_pad(li)] = orig;
  workgroupBarrier();
  for (var offset = 1u; offset < WG; offset <<= 1u) {
    let idx = (li + 1u) * (offset << 1u) - 1u;
    if (idx < WG) {
      scan_buf[scan_pad(idx)] += scan_buf[scan_pad(idx - offset)];
    }
    workgroupBarrier();
  }
  if (li == 0u) {
    var zero: Sum;
    scan_buf[scan_pad(WG - 1u)] = zero;
  }
  workgroupBarrier();
  for (var offset = WG >> 1u; offset > 0u; offset >>= 1u) {
    let idx = (li + 1u) * (offset << 1u) - 1u;
    if (idx < WG) {
      let pa = scan_pad(idx - offset);
      let pb = scan_pad(idx);
      let t = scan_buf[pa];
      scan_buf[pa] = scan_buf[pb];
      scan_buf[pb] += t;
    }
    workgroupBarrier();
  }
  shared_sum[li] = scan_buf[scan_pad(li)] + orig;
  workgroupBarrier();
}
