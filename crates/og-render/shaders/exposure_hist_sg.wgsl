// Tier::Full subgroup + WG=256 trimmed luminance histogram auto-exposure
// (`post/exposure_hist.rs`, `Caps::SUBGROUP`): doubles histogram tile size
// (32x32 pixels per 16x16 workgroup = 2x fewer global atomics, 1 thread per
// bin) and replaces `cs_average`'s 256-iteration serial loop with a parallel
// 256-wide subgroup prefix scan and reduction. Prepended by
// `enable subgroups;` and `common.wgsl`.

struct HistParams {
  flags: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: HistParams;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> histogram: array<atomic<u32>, 256>;
@group(0) @binding(3) var metered_out: texture_storage_2d<rgba16float, write>;

const MIN_LOG_LUM: f32 = -10.0;
const MAX_LOG_LUM: f32 = 12.0;
const LOG_RANGE: f32 = 22.0;
const INV_LOG_RANGE: f32 = 1.0 / 22.0;

var<workgroup> local_hist: array<atomic<u32>, 256>;
var<workgroup> sg_u32: array<u32, 64>;
var<workgroup> sg_f32: array<vec2<f32>, 64>;
var<workgroup> hist_total: u32;

fn accumulate_pixel(px: vec2<i32>, dims: vec2<i32>, untonemapped: bool) {
  if (px.x >= dims.x || px.y >= dims.y) { return; }
  let c = textureLoad(source, px, 0);
  let w = select(4u, u32(round(clamp(c.a, 0.0, 1.0) * 4.0)), untonemapped);
  if (w == 0u) { return; }
  let lum = dot(max(c.rgb, vec3<f32>(0.0)), LUMA);
  let t = clamp((log2(max(lum, 1e-5)) - MIN_LOG_LUM) * INV_LOG_RANGE, 0.0, 1.0);
  let bin = min(u32(t * 255.0 + 0.5), 255u);
  atomicAdd(&local_hist[bin], w);
}

@compute @workgroup_size(16, 16, 1)
fn cs_histogram(
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(local_invocation_id) lid: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
) {
  atomicStore(&local_hist[li], 0u);
  workgroupBarrier();

  let dims = vec2<i32>(textureDimensions(source));
  let base = vec2<i32>(wg.xy * vec2<u32>(32u, 32u) + lid.xy * 2u);
  let untonemapped = params.flags.x > 0.5;
  accumulate_pixel(base, dims, untonemapped);
  accumulate_pixel(base + vec2<i32>(1, 0), dims, untonemapped);
  accumulate_pixel(base + vec2<i32>(0, 1), dims, untonemapped);
  accumulate_pixel(base + vec2<i32>(1, 1), dims, untonemapped);
  workgroupBarrier();

  let c = atomicLoad(&local_hist[li]);
  if (c > 0u) { atomicAdd(&histogram[li], c); }
}

@compute @workgroup_size(256, 1, 1)
fn cs_average(
  @builtin(local_invocation_index) li: u32,
  @builtin(subgroup_size) sg_size: u32,
  @builtin(subgroup_invocation_id) sg_lane: u32,
) {
  let count = atomicExchange(&histogram[li], 0u);
  let sg_id = li / sg_size;
  let num_sgs = (256u + sg_size - 1u) / sg_size;

  // Parallel exclusive prefix sum of bin counts + total across all 256 bins.
  let local_excl = subgroupExclusiveAdd(count);
  let local_tot = subgroupAdd(count);
  if (sg_lane == 0u) {
    sg_u32[sg_id] = local_tot;
  }
  workgroupBarrier();
  if (sg_id == 0u) {
    var carry = 0u;
    for (var base = 0u; base < num_sgs; base += sg_size) {
      let idx = base + sg_lane;
      let v = select(0u, sg_u32[idx], idx < num_sgs);
      let ex = subgroupExclusiveAdd(v);
      let tot = subgroupAdd(v);
      if (idx < num_sgs) { sg_u32[idx] = carry + ex; }
      carry += tot;
    }
    if (sg_lane == 0u) { hist_total = carry; }
  }
  workgroupBarrier();

  let total = hist_total;
  if (total == 0u) {
    if (li == 0u) {
      textureStore(metered_out, vec2<i32>(0, 0), vec4<f32>(MIN_LOG_LUM, 1.0, 0.0, 1.0));
    }
    return;
  }

  let accum = f32(sg_u32[sg_id] + local_excl);
  let next = accum + f32(count);
  let total_f = f32(total);
  let low_cut = total_f * 0.10;
  let high_cut = total_f * 0.95;
  let take = max(min(next, high_cut) - max(accum, low_cut), 0.0);
  let log_lum = MIN_LOG_LUM + (f32(li) / 255.0) * LOG_RANGE;
  let w_pair = vec2<f32>(take * log_lum, take);

  let sg_w = subgroupAdd(w_pair);
  if (sg_lane == 0u) {
    sg_f32[sg_id] = sg_w;
  }
  workgroupBarrier();
  if (sg_id == 0u) {
    var sum = vec2<f32>(0.0);
    for (var base = 0u; base < num_sgs; base += sg_size) {
      let idx = base + sg_lane;
      let v = select(vec2<f32>(0.0), sg_f32[idx], idx < num_sgs);
      sum += subgroupAdd(v);
    }
    if (sg_lane == 0u) {
      let avg_log = select(MIN_LOG_LUM, sum.x / max(sum.y, 1e-6), sum.y > 0.0);
      textureStore(metered_out, vec2<i32>(0, 0), vec4<f32>(avg_log, 1.0, 0.0, 1.0));
    }
  }
}
