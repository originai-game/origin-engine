// The two scan stages of a cull that keeps what it keeps in order (the
// instances', cull.wgsl, and the crowds', crowd_cull.wgsl), written once.
// The visibility stage leaves each item's count and each block's sum in
// `scan`; these turn both into exclusive prefixes, and the module's own
// scatter stage reads them for where each kept item goes. The including
// module declares `Sum` (a count, or a vector of counts), `scan`, an
// array of them, and `shared_sum`, a workgroup array of WG of them
// (cull_scan.wgsl).

// The block sums at scan[at .. at + count] into their exclusive prefix:
// one workgroup, a block of WG at a time, the carry running on.
fn scan_row_blocks(li: u32, at: u32, count: u32) {
  var carry: Sum;
  for (var base = 0u; base < count; base += WG) {
    let i = base + li;
    var v: Sum;
    if (i < count) { v = scan[at + i]; }
    shared_sum[li] = v;
    workgroupBarrier();
    scan_shared(li);
    if (i < count) { scan[at + i] = carry + shared_sum[li] - v; }
    carry += shared_sum[WG - 1u];
    workgroupBarrier();
  }
}

// Item `k` of `n`'s count at scan[at + k] into its exclusive prefix (its
// workgroup's block starts at scan[before_at]), and the row's total into
// scan[at + n].
fn scan_row_prefix(li: u32, k: u32, n: u32, at: u32, before_at: u32) {
  var v: Sum;
  if (k < n) { v = scan[at + k]; }
  shared_sum[li] = v;
  workgroupBarrier();
  scan_shared(li);
  let before = scan[before_at];
  if (k < n) {
    scan[at + k] = before + shared_sum[li] - v;
    if (k == n - 1u) { scan[at + n] = before + shared_sum[li]; }
  }
}
