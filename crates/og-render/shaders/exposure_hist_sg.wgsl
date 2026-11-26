// Tier::Full subgroup + WG=256 luminance histogram auto-exposure
// (`post/exposure_hist.rs`, `Caps::SUBGROUP`): exposure_hist.wgsl's meter
// with a doubled histogram tile (32x32 pixels per 16x16 workgroup = 2x
// fewer global atomics, 1 thread per bin) and `cs_average`'s 256-iteration
// serial loops as parallel 256-wide subgroup reductions. Prepended by
// `enable subgroups;` and `common.wgsl`.

struct HistParams {
  flags: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: HistParams;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> histogram: array<atomic<u32>, 256>;
@group(0) @binding(3) var metered_out: texture_storage_2d<rgba16float, write>;

// exposure_hist.wgsl's range, the fragment meter's floor: the three meters
// take the same mean, so a scene exposes alike on every host.
const MIN_LOG_LUM: f32 = -16.609640474;
const MAX_LOG_LUM: f32 = 16.0;
const LOG_RANGE: f32 = MAX_LOG_LUM - MIN_LOG_LUM;
const INV_LOG_RANGE: f32 = 1.0 / LOG_RANGE;

var<workgroup> local_hist: array<atomic<u32>, 256>;
var<workgroup> sg_u32: array<u32, 64>;
var<workgroup> sg_f32: array<f32, 64>;

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

// Single-workgroup reduction: the weighted mean log2 luminance over every
// bin, as exposure_hist.wgsl's `cs_average` (no trimming: a meter that
// dropped the darkest tenth and the brightest twentieth exposed a scene
// otherwise here than on a host without subgroups), written as (mean, 1,
// 0, 1) for `metered_log` in fullscreen.wgsl; zeros `histogram` in place
// for the next frame. Each subgroup sums its bins' weights and weighted
// logs, then the first sums the subgroups'.
@compute @workgroup_size(256, 1, 1)
fn cs_average(
  @builtin(local_invocation_index) li: u32,
  @builtin(subgroup_size) sg_size: u32,
  @builtin(subgroup_invocation_id) sg_lane: u32,
) {
  let count = atomicExchange(&histogram[li], 0u);
  let sg_id = li / sg_size;
  let num_sgs = (256u + sg_size - 1u) / sg_size;

  let local_total = subgroupAdd(count);
  let local_weighted = subgroupAdd(f32(count) * (MIN_LOG_LUM + (f32(li) / 255.0) * LOG_RANGE));
  if (sg_lane == 0u) {
    sg_u32[sg_id] = local_total;
    sg_f32[sg_id] = local_weighted;
  }
  workgroupBarrier();
  if (sg_id == 0u) {
    var total = 0u;
    var weighted = 0.0;
    for (var base = 0u; base < num_sgs; base += sg_size) {
      let idx = base + sg_lane;
      total += subgroupAdd(select(0u, sg_u32[idx], idx < num_sgs));
      weighted += subgroupAdd(select(0.0, sg_f32[idx], idx < num_sgs));
    }
    if (sg_lane == 0u) {
      let avg_log = select(MIN_LOG_LUM, weighted / f32(max(total, 1u)), total > 0u);
      textureStore(metered_out, vec2<i32>(0, 0), vec4<f32>(avg_log, 1.0, 0.0, 1.0));
    }
  }
}
