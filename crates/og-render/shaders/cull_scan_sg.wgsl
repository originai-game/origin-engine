// Subgroup-accelerated inclusive scan (`Tier::Full`, `Caps::SUBGROUP`, WG = 256):
// replaces 16 Blelloch barriers with 2 workgroup barriers using hardware
// `subgroupInclusiveAdd` / `subgroupExclusiveAdd` across any power-of-2
// subgroup size >= 4. The host prepends `enable subgroups;` and populates
// `cull_sg_size` and `cull_sg_lane` from entry point builtins.
var<private> cull_sg_size: u32;
var<private> cull_sg_lane: u32;
var<workgroup> scan_sg_totals: array<Sum, 64>;

fn sg_scan_shared(li: u32) {
  let val = shared_sum[li];
  let local_inc = subgroupInclusiveAdd(val);
  let sg_id = li / cull_sg_size;
  let num_sgs = (WG + cull_sg_size - 1u) / cull_sg_size;
  if (cull_sg_lane == cull_sg_size - 1u) {
    scan_sg_totals[sg_id] = local_inc;
  }
  workgroupBarrier();
  if (sg_id == 0u) {
    var carry: Sum;
    for (var base = 0u; base < num_sgs; base += cull_sg_size) {
      let idx = base + cull_sg_lane;
      var x: Sum;
      if (idx < num_sgs) { x = scan_sg_totals[idx]; }
      let excl = subgroupExclusiveAdd(x);
      let tot = subgroupAdd(x);
      if (idx < num_sgs) { scan_sg_totals[idx] = carry + excl; }
      carry += tot;
    }
  }
  workgroupBarrier();
  shared_sum[li] = local_inc + scan_sg_totals[sg_id];
  workgroupBarrier();
}
